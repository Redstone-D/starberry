use async_trait::async_trait;
use starberry_core::connection::Tx;
use super::connection::{DbConnectionBuilder, DbConnection};
use super::error::DbError;
use super::query::QueryResult;

/// Context for managing a single SQL connection.
///
/// Holds a connection builder and lazily establishes
/// a live `DbConnection` on first use.
pub struct SqlContext {
    /// Builder for creating a database connection.
    pub builder: DbConnectionBuilder,
    /// The established connection, once `connect` has been called.
    pub connection: Option<DbConnection>,
    /// The most recent query result.
    pub last_result: Option<QueryResult>,
}

impl SqlContext {
    /// Create a new context with the given `DbConnectionBuilder`.
    pub fn new(builder: DbConnectionBuilder) -> Self {
        SqlContext { builder, connection: None, last_result: None }
    }

    /// Get a mutable reference to the live connection,
    /// connecting if not already connected.
    pub async fn get_connection(&mut self) -> Result<&mut DbConnection, DbError> {
        if self.connection.is_none() {
            let conn = self.builder.connect().await?;
            self.connection = Some(conn);
        }
        Ok(self.connection.as_mut().unwrap())
    }
}

#[async_trait]  
impl Tx for SqlContext { 
    type Request = (String, Vec<String>);
    type Response = QueryResult;
    type Config = DbConnectionBuilder;
    type Error = DbError;

    async fn process(&mut self, request: Self::Request) -> Result<&mut Self::Response, Self::Error> {
        let (sql, params) = request;
        // Ensure a live connection
        let conn = self.get_connection().await?;
        // Execute the query and store the result
        let result = conn.execute_query(&sql, params).await?;
        self.last_result = Some(result);
        Ok(self.last_result.as_mut().unwrap())
    }

    async fn shutdown(&mut self) -> Result<(), Self::Error> {
        if let Some(mut conn) = self.connection.take() {
            conn.close().await?;
        }
        Ok(())
    }

    async fn fetch<T: Into<String> + Send + Sync>(_: T, request: Self::Request, config: Self::Config) -> Self::Response {
        let mut ctx = SqlContext::new(config);
        match ctx.process(request).await {
            Ok(res) => res.clone(),
            Err(e) => QueryResult::Error(e),
        }
    }
}