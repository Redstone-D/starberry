use std::collections::VecDeque;
use std::sync::Arc;
use tokio::sync::{Mutex, Semaphore};
use std::time::{Duration, Instant};

use crate::connection::builder::ConnectionBuilder;
use crate::connection::error::{ConnectionError, Result};
use crate::connection::connection::Connection; 

/// Connection pool for managing connections 
struct PooledConnection {
    connection: Connection,
    last_used: Instant,
} 

pub struct ConnectionPool {
    builder: ConnectionBuilder,
    connections: Arc<Mutex<VecDeque<PooledConnection>>>,
    semaphore: Arc<Semaphore>,
    max_size: usize,
    min_size: usize,
    max_lifetime: Duration,
    max_idle_time: Duration,
} 

impl ConnectionPool { 
    pub fn new(builder: ConnectionBuilder, max_size: usize) -> Self {
        Self {
            builder,
            connections: Arc::new(Mutex::new(VecDeque::with_capacity(max_size))),
            semaphore: Arc::new(Semaphore::new(max_size)),
            max_size,
            min_size: 1,
            max_lifetime: Duration::from_secs(30 * 60), // 30 minutes
            max_idle_time: Duration::from_secs(5 * 60), // 5 minutes
        }
    } 

    /// Set the minimum pool size
    pub fn min_size(mut self, size: usize) -> Self {
        self.min_size = size;
        self
    }
    
    /// Set the maximum lifetime of a connection
    pub fn max_lifetime(mut self, duration: Duration) -> Self {
        self.max_lifetime = duration;
        self
    }
    
    /// Set the maximum idle time before a connection is removed from the pool
    pub fn max_idle_time(mut self, duration: Duration) -> Self {
        self.max_idle_time = duration;
        self
    }
    
    /// Initialize the connection pool
    pub async fn initialize(&self) -> Result<()> {
        let mut connections = self.connections.lock().await;
        
        // Create minimum connections
        for _ in 0..self.min_size {
            let conn = self.builder.connect().await?;
            connections.push_back(PooledConnection {
                connection: conn,
                last_used: Instant::now(),
            });
        }
        
        Ok(())
    }
    
    /// Get a connection from the pool
    pub async fn get(&self) -> Result<PooledConnection> {
        // Try to acquire a permit from the semaphore
        let semaphore = self.semaphore.clone();
        let permit = match semaphore.try_acquire() {
            Ok(permit) => permit,
            Err(_) => {
                // No available connections, wait for one
                semaphore.acquire().await
                    .map_err(|_| ConnectionError::PoolExhausted)?
            }
        };
        
        let mut connections = self.connections.lock().await;
        
        if let Some(mut pooled) = connections.pop_front() {
            // Check if the connection is too old or has been idle for too long
            let now = Instant::now();
            if now - pooled.last_used > self.max_idle_time {
                // Connection has been idle for too long, create a new one
                drop(pooled); // Explicitly drop to close the connection
                let conn = self.builder.connect().await?;
                return Ok(PooledConnection {
                    connection: conn,
                    last_used: now,
                });
            }
            
            // Update last used time
            pooled.last_used = now;
            return Ok(pooled);
        }
        
        // No available connection in the pool, create a new one
        let conn = self.builder.connect().await?;
        Ok(PooledConnection {
            connection: conn,
            last_used: Instant::now(),
        }) 
    } 

    /// Return a connection to the pool
    pub async fn release(&self, conn: PooledConnection) {
        let mut connections = self.connections.lock().await;
        
        // If we're below max size, return the connection to the pool
        if connections.len() < self.max_size {
            connections.push_back(conn);
        }
        
        // Release the semaphore permit
        self.semaphore.add_permits(1);
    }
    
    /// Close all connections in the pool
    pub async fn close(&self) {
        let mut connections = self.connections.lock().await;
        connections.clear();
    } 

    /// Maintenance routine to clean up idle connections
    pub async fn maintain(&self) {
        let mut connections = self.connections.lock().await;
        let now = Instant::now();
        
        // Filter out connections that have been idle for too long
        let active_count = connections.len();
        connections.retain(|conn| now - conn.last_used <= self.max_idle_time);
        
        // If we dropped below min_size, create new connections
        if connections.len() < self.min_size {
            for _ in connections.len()..self.min_size {
                if let Ok(conn) = self.builder.connect().await {
                    connections.push_back(PooledConnection {
                        connection: conn,
                        last_used: Instant::now(),
                    });
                }
            }
        }
        
        // Update semaphore if needed
        let diff = active_count as i32 - connections.len() as i32;
        if diff > 0 {
            // We removed some connections, add permits to the semaphore
            self.semaphore.add_permits(diff as usize);
        }
    } 
} 