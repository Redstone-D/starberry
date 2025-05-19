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
        use starberry_lib::random_string;
        // Generate a random statement name
        let stmt_name = format!("stmt_{}", random_string(8));
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use crate::sql::connection::{DbConnectionBuilder, SslMode};

    #[test]
    fn validate_params_allows_safe_strings() {
        let params = vec!["hello".to_string(), "world".to_string()];
        assert!(validate_params(&params).is_ok());
    }

    #[test]
    fn validate_params_rejects_null_byte() {
        let params = vec!["bad\0".to_string()];
        assert!(validate_params(&params).is_err());
    }

    #[test]
    fn query_result_row_count_and_first_row() {
        let mut row = HashMap::new();
        row.insert("key".to_string(), "value".to_string());
        let qr = QueryResult::Rows(vec![row.clone()]);
        assert_eq!(qr.row_count(), 1);
        assert_eq!(qr.first_row(), Some(&row));
        let qr2 = QueryResult::Count(5);
        assert_eq!(qr2.row_count(), 5);
        let qr3 = QueryResult::Empty;
        assert_eq!(qr3.row_count(), 0);
    }

    #[test]
    fn test_substitute_placeholders() {
        let query = "SELECT $1, $2";
        let params = vec!["foo".to_string(), "bar".to_string()];
        let mut sql = query.to_string();
        for (i, p) in params.iter().enumerate() {
            let placeholder = format!("${}", i + 1);
            let escaped = p.replace("'", "''");
            let quoted = format!("'{}'", escaped);
            let pos = sql.find(&placeholder).expect("placeholder not found");
            sql.replace_range(pos..pos + placeholder.len(), &quoted);
        }
        assert_eq!(sql, "SELECT 'foo', 'bar'");
    }

    #[tokio::test]
    async fn integration_test_postgres_local() {
        // Requires a local Postgres at 127.0.0.1:5432 with user 'postgres' and password 'JerrySu5379'
        let mut conn = DbConnectionBuilder::new("127.0.0.1", 5432)
            .ssl_mode(SslMode::Disable)
            .database("postgres")
            .username("postgres")
            .password("JerrySu5379")
            .connect().await.expect("Failed to connect to Postgres");
        let result = conn.execute_query("SELECT 1 AS v", vec![]).await.expect("Query failed");
        assert_eq!(result.row_count(), 1);
        let row = result.first_row().expect("No row returned");
        assert_eq!(row.get("v"), Some(&"1".to_string()));
        conn.close().await.expect("Failed to close connection");
    }

    #[tokio::test]
    async fn integration_test_crud_operations() {
        // Requires a local Postgres at 127.0.0.1:5432 with user 'postgres' and password 'JerrySu5379'
        let mut conn = DbConnectionBuilder::new("127.0.0.1", 5432)
            .ssl_mode(SslMode::Disable)
            .database("postgres")
            .username("postgres")
            .password("JerrySu5379")
            .connect().await.expect("Failed to connect to Postgres");

        // Clean up any existing test table
        let _ = conn.execute_query("DROP TABLE IF EXISTS test_tbl", vec![]).await;

        // Create table
        conn.execute_query(
            "CREATE TABLE test_tbl (id SERIAL PRIMARY KEY, name TEXT)",
            vec![],
        ).await.expect("Create table failed");

        // Insert rows
        let insert_res = conn.execute_query(
            "INSERT INTO test_tbl (name) VALUES ($1), ($2), ($3)",
            vec!["Alice".to_string(), "Bob".to_string(), "Carol".to_string()],
        ).await.expect("Insert failed");
        assert_eq!(insert_res.row_count(), 3);

        // Select all rows
        let select_res = conn.execute_query(
            "SELECT name FROM test_tbl ORDER BY id",
            vec![],
        ).await.expect("Select failed");
        assert_eq!(select_res.row_count(), 3);
        if let QueryResult::Rows(rows) = select_res {
            assert_eq!(rows[0].get("name"), Some(&"Alice".to_string()));
            assert_eq!(rows[1].get("name"), Some(&"Bob".to_string()));
            assert_eq!(rows[2].get("name"), Some(&"Carol".to_string()));
        } else {
            panic!("Expected rows");
        }

        // Delete one row
        let delete_res = conn.execute_query(
            "DELETE FROM test_tbl WHERE name=$1",
            vec!["Bob".to_string()],
        ).await.expect("Delete failed");
        assert_eq!(delete_res.row_count(), 1);

        // Select remaining rows
        let select_res2 = conn.execute_query(
            "SELECT name FROM test_tbl ORDER BY id",
            vec![],
        ).await.expect("Select failed");
        assert_eq!(select_res2.row_count(), 2);
        if let QueryResult::Rows(rows2) = select_res2 {
            assert_eq!(rows2[0].get("name"), Some(&"Alice".to_string()));
            assert_eq!(rows2[1].get("name"), Some(&"Carol".to_string()));
        } else {
            panic!("Expected rows");
        }

        // Drop test table
        conn.execute_query("DROP TABLE test_tbl", vec![]).await.expect("Drop test table failed");
    }

    #[tokio::test]
    async fn integration_test_create_database_and_tables() {
        // Connect to default postgres database
        let mut conn = DbConnectionBuilder::new("127.0.0.1", 5432)
            .ssl_mode(SslMode::Disable)
            .database("postgres")
            .username("postgres")
            .password("JerrySu5379")
            .connect().await.expect("Failed to connect to Postgres");

        // Drop database if it exists
        let _ = conn.execute_query("DROP DATABASE IF EXISTS test_db", vec![]).await;

        // Create new database
        let create_db = conn.execute_query("CREATE DATABASE test_db", vec![]).await.expect("CREATE DATABASE failed");
        assert!(matches!(create_db, QueryResult::Empty));

        // Connect to the new database
        let mut db_conn = DbConnectionBuilder::new("127.0.0.1", 5432)
            .ssl_mode(SslMode::Disable)
            .database("test_db")
            .username("postgres")
            .password("JerrySu5379")
            .connect().await.expect("Failed to connect to test_db");

        // Create tables
        let res1 = db_conn.execute_query("CREATE TABLE t1 (i INT)", vec![]).await.expect("CREATE t1 failed");
        assert!(matches!(res1, QueryResult::Empty));
        let res2 = db_conn.execute_query("CREATE TABLE t2 (s TEXT)", vec![]).await.expect("CREATE t2 failed");
        assert!(matches!(res2, QueryResult::Empty));

        // Verify tables exist
        let sel = db_conn.execute_query(
            "SELECT tablename FROM pg_tables WHERE schemaname='public' AND tablename IN ('t1','t2') ORDER BY tablename",
            vec![],
        ).await.expect("SELECT tables failed");
        assert_eq!(sel.row_count(), 2);
        if let QueryResult::Rows(rows) = sel {
            let names: Vec<String> = rows.iter().filter_map(|r| r.get("tablename").cloned()).collect();
            assert!(names.contains(&"t1".to_string()));
            assert!(names.contains(&"t2".to_string()));
        } else {
            panic!("Expected rows");
        }

        // Cleanup: drop tables and database
        db_conn.execute_query("DROP TABLE t1", vec![]).await.expect("Drop t1 failed");
        db_conn.execute_query("DROP TABLE t2", vec![]).await.expect("Drop t2 failed");
        db_conn.close().await.expect("Close test_db connection failed");
        let drop_db = conn.execute_query("DROP DATABASE test_db", vec![]).await.expect("Drop database failed");
        assert!(matches!(drop_db, QueryResult::Empty));
    }
}