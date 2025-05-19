use super::error::DbError;
use std::collections::HashMap;

/// Trait for constructing a type from a database row
pub trait FromRow: Sized {
    /// Build an instance of the implementing type from a row map
    fn from_row(row: &HashMap<String, String>) -> Result<Self, DbError>;
} 