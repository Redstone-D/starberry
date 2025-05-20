use super::error::DbError;

/// Trait for encoding Rust types into SQL-safe parameter strings
pub trait Encode {
    /// Encode self into a SQL parameter string
    fn encode(&self) -> Result<String, DbError>;
}

impl Encode for i32 {
    fn encode(&self) -> Result<String, DbError> {
        Ok(self.to_string())
    }
}

impl Encode for i64 {
    fn encode(&self) -> Result<String, DbError> {
        Ok(self.to_string())
    }
}

impl Encode for &str {
    fn encode(&self) -> Result<String, DbError> {
        // Escape single quotes and wrap in single quotes
        let escaped = self.replace("'", "''");
        Ok(format!("'{}'", escaped))
    }
}

impl Encode for String {
    fn encode(&self) -> Result<String, DbError> {
        // Escape single quotes and wrap in single quotes
        let escaped = self.replace("'", "''");
        Ok(format!("'{}'", escaped))
    }
}

// Additional Encode implementations for common types
impl Encode for bool {
    fn encode(&self) -> Result<String, DbError> {
        Ok(self.to_string().to_uppercase())
    }
}

impl Encode for f32 {
    fn encode(&self) -> Result<String, DbError> {
        Ok(self.to_string())
    }
}

impl Encode for f64 {
    fn encode(&self) -> Result<String, DbError> {
        Ok(self.to_string())
    }
}

impl<T: Encode> Encode for Option<T> {
    fn encode(&self) -> Result<String, DbError> {
        match self {
            Some(v) => v.encode(),
            None => Ok("NULL".to_string()),
        }
    }
} 