use super::connection::DbConnection;
use super::error::DbError;
use std::collections::HashMap;

/// Represents a database query result with PostgreSQL specifics.
#[derive(Debug, Clone)]
pub enum QueryResult {
    Rows(Vec<HashMap<String, String>>),  // Rows with column names
    Count(usize),  // Number of affected rows
    Empty,
    Error(DbError),  // Use DbError for better error handling
}

/// Ensures no null bytes in parameters to avoid protocol injection
fn validate_params(params: &Vec<String>) -> Result<(), DbError> {
    if params.iter().any(|p| p.contains('\0')) {
        return Err(DbError::QueryError("Null byte detected in parameter".to_string()));
    }
    Ok(())
}

impl QueryResult {
    /// Returns the number of rows in the result.
    pub fn row_count(&self) -> usize {
        match self {
            QueryResult::Rows(rows) => rows.len(),
            QueryResult::Count(count) => *count,
            QueryResult::Empty => 0,
            QueryResult::Error(_) => 0,
        }
    }

    /// Returns the first row if available.
    pub fn first_row(&self) -> Option<&HashMap<String, String>> {
        match self {
            QueryResult::Rows(rows) if !rows.is_empty() => Some(&rows[0]),
            _ => None,
        }
    }
}

impl DbConnection {
    /// Executes a general SQL query.
    pub async fn execute_query(&mut self, query: &str, params: Vec<String>) -> Result<QueryResult, DbError> {
        // Validate params to prevent injection
        if let Err(e) = validate_params(&params) {
            return Err(e);
        }
        if let Some(stream) = &mut self.stream {
            use tokio::io::{AsyncWriteExt, AsyncReadExt};
            // Substitute placeholders $1, $2, ... with safely quoted parameters
            let mut sql = query.to_string();
            for (i, p) in params.iter().enumerate() {
                let placeholder = format!("${}", i + 1);
                let escaped = p.replace("'", "''");
                let quoted = format!("'{}'", escaped);
                if let Some(pos) = sql.find(&placeholder) {
                    sql.replace_range(pos..pos + placeholder.len(), &quoted);
                } else {
                    return Err(DbError::QueryError(format!("Missing parameter {} in query", placeholder)));
                }
            }
            // Build simple query message ('Q')
            let mut msg = Vec::new();
            msg.push(b'Q');
            let mut body = sql.into_bytes();
            body.push(0);
            let len = (body.len() + 4) as u32;
            msg.extend_from_slice(&len.to_be_bytes());
            msg.extend_from_slice(&body);
            // Send Query
            stream.write_all(&msg).await.map_err(|e| DbError::QueryError(e.to_string()))?;
            stream.flush().await.map_err(|e| DbError::QueryError(e.to_string()))?;
            // Read responses
            let mut rows = Vec::new();
            let mut columns = Vec::new();
            let mut count: Option<usize> = None;
            loop {
                let mut tag = [0u8];
                stream.read_exact(&mut tag).await.map_err(|e| DbError::ProtocolError(e.to_string()))?;
                let mut len_buf = [0u8; 4];
                stream.read_exact(&mut len_buf).await.map_err(|e| DbError::ProtocolError(e.to_string()))?;
                let payload_len = u32::from_be_bytes(len_buf);
                let mut payload = vec![0u8; (payload_len - 4) as usize];
                stream.read_exact(&mut payload).await.map_err(|e| DbError::ProtocolError(e.to_string()))?;
                match tag[0] {
                    b'T' => { // RowDescription
                        let mut off = 0;
                        let field_count = u16::from_be_bytes([payload[off], payload[off+1]]) as usize;
                        off += 2;
                        for _ in 0..field_count {
                            let end = payload[off..].iter().position(|&b| b==0).unwrap_or(0);
                            let name = String::from_utf8_lossy(&payload[off..off+end]).to_string();
                            columns.push(name);
                            off += end + 1 + 18; // skip name + descriptor
                        }
                    }
                    b'D' => { // DataRow
                        let mut off = 0;
                        let col_count = u16::from_be_bytes([payload[off], payload[off+1]]) as usize;
                        off += 2;
                        let mut row = HashMap::new();
                        for i in 0..col_count {
                            let col_len = i32::from_be_bytes([
                                payload[off], payload[off+1], payload[off+2], payload[off+3]
                            ]);
                            off += 4;
                            let val = if col_len < 0 {
                                String::new()
                            } else {
                                let v = String::from_utf8_lossy(&payload[off..off + col_len as usize]).to_string();
                                off += col_len as usize;
                                v
                            };
                            row.insert(columns[i].clone(), val);
                        }
                        rows.push(row);
                    }
                    b'C' => { // CommandComplete
                        let tag_str = String::from_utf8_lossy(&payload[..payload.len()-1]).to_string();
                        if let Some(n) = tag_str.split_whitespace().last().and_then(|s| s.parse::<usize>().ok()) {
                            count = Some(n);
                        }
                    }
                    b'E' => { // ErrorResponse
                        let msg = String::from_utf8_lossy(&payload[..payload.len()-1]).to_string();
                        return Err(DbError::QueryError(msg));
                    }
                    b'Z' => { // ReadyForQuery
                        break;
                    }
                    _ => {} // ignore others
                }
            }
            if query.trim().to_uppercase().starts_with("SELECT") {
                Ok(QueryResult::Rows(rows))
            } else if let Some(n) = count {
                Ok(QueryResult::Count(n))
            } else {
                Ok(QueryResult::Empty)
            }
        } else {
            Err(DbError::ConnectionError("No active connection stream".to_string()))
        }
    }

    /// Executes a batch of queries.
    pub async fn batch_execute(&mut self, queries: Vec<(&str, Vec<String>)>) -> Result<QueryResult, DbError> {
        let mut total = 0;
        for (q, params) in queries {
            let res = self.execute_query(q, params).await?;
            total += res.row_count();
        }
        Ok(QueryResult::Count(total))
    }

    /// Begins a transaction.
    pub async fn begin_transaction(&mut self) -> Result<(), DbError> {
        self.execute_query("BEGIN", vec![]).await.map(|_| ())
    }

    /// Commits a transaction.
    pub async fn commit_transaction(&mut self) -> Result<(), DbError> {
        self.execute_query("COMMIT", vec![]).await.map(|_| ())
    }

    /// Rolls back a transaction.
    pub async fn rollback_transaction(&mut self) -> Result<(), DbError> {
        self.execute_query("ROLLBACK", vec![]).await.map(|_| ())
    }

    /// Prepares a statement for repeated execution.
    pub async fn prepare_statement(&mut self, query: &str) -> Result<String, DbError> {
        use starberry_lib::random_alphanumeric_string;
        // Generate a random statement name
        let stmt_name = format!("stmt_{}", random_alphanumeric_string(8));
        let prep = format!("PREPARE {} AS {}", stmt_name, query);
        self.execute_query(&prep, vec![]).await?;
        Ok(stmt_name)
    }

    /// Executes a prepared statement.
    pub async fn execute_prepared(&mut self, statement_id: &str, params: Vec<String>) -> Result<QueryResult, DbError> {
        // Validate params
        validate_params(&params)?;
        // Build EXECUTE statement
        let mut exec = format!("EXECUTE {}", statement_id);
        if !params.is_empty() {
            let args: Vec<String> = params.into_iter().map(|p| format!("'{}'", p.replace("'", "''"))).collect();
            exec.push_str(&format!(" ({})", args.join(", ")));
        }
        self.execute_query(&exec, vec![]).await
    }
}
