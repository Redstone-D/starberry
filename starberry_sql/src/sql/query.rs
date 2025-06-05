use super::connection::DbConnection;
use super::error::DbError;
use std::collections::HashMap;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use starberry_core::connection::Connection as GenericConnection;

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

/// Reads server messages and collects rows and optional affected row count.
async fn read_response(stream: &mut GenericConnection) -> Result<(Vec<HashMap<String, String>>, Option<usize>), DbError> {
    let mut rows = Vec::new();
    let mut columns = Vec::new();
    let mut count: Option<usize> = None;
    loop {
        let mut tag = [0u8];
        stream.read_exact(&mut tag).await.map_err(|e| DbError::ProtocolError(e.to_string()))?;
        let mut len_buf = [0u8; 4];
        stream.read_exact(&mut len_buf).await.map_err(|e| DbError::ProtocolError(e.to_string()))?;
        let payload_len = u32::from_be_bytes(len_buf) as usize - 4;
        let mut payload = vec![0u8; payload_len];
        stream.read_exact(&mut payload).await.map_err(|e| DbError::ProtocolError(e.to_string()))?;

        match tag[0] {
            b'T' => {
                // RowDescription: parse column names
                let mut off = 0;
                let fcnt = u16::from_be_bytes([payload[off], payload[off+1]]) as usize;
                off += 2;
                for _ in 0..fcnt {
                    let end = payload[off..].iter().position(|b| *b == 0).unwrap();
                    let name = String::from_utf8_lossy(&payload[off..off+end]).to_string();
                    columns.push(name);
                    off += end + 1 + 18;
                }
            }
            b'D' => {
                // DataRow: parse each row
                let mut off = 0;
                let c = u16::from_be_bytes([payload[off], payload[off+1]]) as usize;
                off += 2;
                let mut row_map = HashMap::new();
                for i in 0..c {
                    let l = i32::from_be_bytes([payload[off], payload[off+1], payload[off+2], payload[off+3]]);
                    off += 4;
                    let val = if l < 0 {
                        String::new()
                    } else {
                        let s = String::from_utf8_lossy(&payload[off..off + l as usize]).to_string();
                        off += l as usize;
                        s
                    };
                    row_map.insert(columns[i].clone(), val);
                }
                rows.push(row_map);
            }
            b'C' => {
                // CommandComplete: affected row count
                let tag = String::from_utf8_lossy(&payload[..payload.len()-1]).to_string();
                if let Some(n) = tag.split_whitespace().last().and_then(|s| s.parse().ok()) {
                    count = Some(n);
                }
            }
            b'E' => {
                // ErrorResponse
                let msg = String::from_utf8_lossy(&payload[..payload.len()-1]).to_string();
                return Err(DbError::QueryError(msg));
            }
            b'Z' => {
                // ReadyForQuery: end of request
                break;
            }
            _ => {
                // ignore other messages
            }
        }
    }
    Ok((rows, count))
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
        // 1. Basic validation: disallow NULL bytes
        validate_params(&params)?;

        // 2. Ensure underlying stream is available
        let stream = self
            .stream
            .as_mut()
            .ok_or_else(|| DbError::ConnectionError("No active connection".into()))?;

        // ---- 3. Parse message ----
        // Format: 'P' | Int32(len) | statement_name\0 | query\0 | param_type_count(0)
        let mut buf = Vec::new();
        buf.push(b'P');
        let mut body = Vec::new();
        // unnamed statement
        body.extend_from_slice(b""); body.push(0);
        // SQL text
        body.extend_from_slice(query.as_bytes()); body.push(0);
        // 0 means do not specify parameter types explicitly; server will infer via context or casts
        body.extend_from_slice(&0u16.to_be_bytes());

        let len = (body.len() + 4) as u32;
        buf.extend_from_slice(&len.to_be_bytes());
        buf.extend_from_slice(&body);

        stream
            .write_all(&buf)
            .await
            .map_err(|e| DbError::ProtocolError(e.to_string()))?;

        // ---- 4. Bind message ----
        // 'B' | Int32(len) | portal_name\0 | statement_name\0
        //             | format_code_count(0=All text)
        //             | param_count | [ param_len | param_bytes ]*
        //             | result_format_count(0=All text)
        let mut buf = Vec::new();
        buf.push(b'B');
        let mut body = Vec::new();
        // portal name (empty = unnamed portal)
        body.extend_from_slice(b""); body.push(0);
        // statement name (same as in Parse; empty = unnamed)
        body.extend_from_slice(b""); body.push(0);

        // use text format for all parameters
        body.extend_from_slice(&0u16.to_be_bytes());

        // number of parameters
        body.extend_from_slice(&(params.len() as u16).to_be_bytes());
        for p in &params {
            let v = p.as_bytes();
            // 每个参数：int32(len) + bytes
            body.extend_from_slice(&(v.len() as i32).to_be_bytes());
            body.extend_from_slice(v);
        }

        // use text format for all results
        body.extend_from_slice(&0u16.to_be_bytes());

        let len = (body.len() + 4) as u32;
        buf.extend_from_slice(&len.to_be_bytes());
        buf.extend_from_slice(&body);

        stream
            .write_all(&buf)
            .await
            .map_err(|e| DbError::ProtocolError(e.to_string()))?;

        // ---- 5. Execute message ----
        // 'E' | Int32(len) | portal_name\0 | max_rows(0=Unlimited)
        let mut buf = Vec::new();
        buf.push(b'E');
        let mut body = Vec::new();
        // portal_name
        body.extend_from_slice(b""); body.push(0);
        // max rows = 0 (fetch all at once)
        body.extend_from_slice(&0u32.to_be_bytes());

        let len = (body.len() + 4) as u32;
        buf.extend_from_slice(&len.to_be_bytes());
        buf.extend_from_slice(&body);

        stream
            .write_all(&buf)
            .await
            .map_err(|e| DbError::ProtocolError(e.to_string()))?;

        // ---- 6. Sync message ----
        stream
            .write_all(&[b'S', 0, 0, 0, 4])
            .await
            .map_err(|e| DbError::ProtocolError(e.to_string()))?;

        // ---- 7. Read server responses ----
        let (rows, count) = read_response(stream).await?;

        // ---- 8. Return result ----
        if query.trim_start().to_uppercase().starts_with("SELECT") {
            Ok(QueryResult::Rows(rows))
        } else if let Some(n) = count {
            Ok(QueryResult::Count(n))
        } else {
            Ok(QueryResult::Empty)
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
        // 1. Validate parameters
        validate_params(&params)?;
        // 2. Ensure underlying stream is available
        let stream = self
            .stream
            .as_mut()
            .ok_or_else(|| DbError::ConnectionError("No active connection".into()))?;
        // 3. Bind message: portal="", statement=statement_id, params in text format
        let mut buf = Vec::new();
        buf.push(b'B');
        let mut body = Vec::new();
        // portal name (empty = unnamed portal)
        body.extend_from_slice(b""); body.push(0);
        // statement name
        body.extend_from_slice(statement_id.as_bytes()); body.push(0);
        // all parameters in text format
        body.extend_from_slice(&0u16.to_be_bytes());
        // parameter count
        body.extend_from_slice(&(params.len() as u16).to_be_bytes());
        for p in &params {
            let bytes = p.as_bytes();
            // int32 length + bytes
            body.extend_from_slice(&(bytes.len() as i32).to_be_bytes());
            body.extend_from_slice(bytes);
        }
        // all results in text format
        body.extend_from_slice(&0u16.to_be_bytes());
        // prepend length and send
        let len = (body.len() + 4) as u32;
        buf.extend_from_slice(&len.to_be_bytes());
        buf.extend_from_slice(&body);
        stream.write_all(&buf).await.map_err(|e| DbError::ProtocolError(e.to_string()))?;
        // 4. Execute message: portal="", max_rows=0 (fetch all)
        let mut buf = Vec::new();
        buf.push(b'E');
        let mut body = Vec::new();
        body.extend_from_slice(b""); body.push(0);
        body.extend_from_slice(&0u32.to_be_bytes());
        let len = (body.len() + 4) as u32;
        buf.extend_from_slice(&len.to_be_bytes());
        buf.extend_from_slice(&body);
        stream.write_all(&buf).await.map_err(|e| DbError::ProtocolError(e.to_string()))?;
        // 5. Sync message
        stream.write_all(&[b'S', 0, 0, 0, 4]).await.map_err(|e| DbError::ProtocolError(e.to_string()))?;
        // 6. Read server responses
        let (rows, count) = read_response(stream).await?;
        // 7. Return result for prepared execution
        if !rows.is_empty() {
            Ok(QueryResult::Rows(rows))
        } else if let Some(n) = count {
            Ok(QueryResult::Count(n))
        } else {
            Ok(QueryResult::Empty)
        }
    }
}
