use std::collections::VecDeque;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::{Mutex, Semaphore, OwnedSemaphorePermit};
use async_trait::async_trait;
use starberry_core::connection::transmit::Pool;

use super::connection::{DbConnectionBuilder, DbConnection};
use super::error::DbError;

/// Async connection pool for database connections.
#[derive(Clone)]
pub struct SqlPool {
    builder: DbConnectionBuilder,
    connections: Arc<Mutex<VecDeque<DbConnection>>>,
    semaphore: Arc<Semaphore>,
    max_size: usize,
}

impl SqlPool {
    /// Create a new pool with the given `DbConnectionBuilder` and maximum pool size.
    pub fn new(builder: DbConnectionBuilder, max_size: usize) -> Self {
        Self {
            builder,
            connections: Arc::new(Mutex::new(VecDeque::with_capacity(max_size))),
            semaphore: Arc::new(Semaphore::new(max_size)),
            max_size,
        }
    }

    /// Acquire a pooled connection, establishing a new one if necessary.
    pub async fn get(&self) -> Result<PooledSqlConnection, DbError> {
        // Acquire a permit to ensure we don't exceed max_size
        let permit = self.semaphore.clone().acquire_owned()
            .await
            .map_err(|_| DbError::OtherError("Failed to acquire pool permit".into()))?;
        // Try to reuse an existing connection
        let mut conns = self.connections.lock().await;
        if let Some(conn) = conns.pop_front() {
            Ok(PooledSqlConnection { pool: self.clone(), conn: Some(conn), _permit: permit })
        } else {
            drop(conns);
            // No idle connection, create a new one
            let conn = self.builder.connect().await?;
            Ok(PooledSqlConnection { pool: self.clone(), conn: Some(conn), _permit: permit })
        }
    }

    /// Return a connection to the pool.
    async fn release(&self, conn: DbConnection) {
        let mut conns = self.connections.lock().await;
        if conns.len() < self.max_size {
            conns.push_back(conn);
        }
        // Permit is released when `_permit` is dropped
    }
}

/// Wrapper for a pooled connection that returns it to the pool on drop.
pub struct PooledSqlConnection {
    pool: SqlPool,
    conn: Option<DbConnection>,
    _permit: OwnedSemaphorePermit,
}

impl PooledSqlConnection {
    /// Get a mutable reference to the underlying `DbConnection`.
    pub fn connection(&mut self) -> &mut DbConnection {
        self.conn.as_mut().unwrap()
    }
}

impl Drop for PooledSqlConnection {
    fn drop(&mut self) {
        if let Some(conn) = self.conn.take() {
            let pool = self.pool.clone();
            // Spawn a task to release the connection without blocking.
            tokio::spawn(async move {
                pool.release(conn).await;
            });
        }
    }
}

#[async_trait]
impl Pool for SqlPool {
    type Item = PooledSqlConnection;
    type Error = DbError;

    async fn get(&self) -> std::result::Result<Self::Item, Self::Error> {
        SqlPool::get(self).await
    }

    async fn release(&self, item: Self::Item) {
        // Dropping the item returns its connection to the pool.
        drop(item);
    }
} 