use std::error::Error as StdError;
use std::fmt;
use crate::connection::error::ConnectionError;

/// Represents errors that can occur during database operations.
#[derive(Debug, Clone)]
pub enum DbError {
    ConnectionError(String),
    QueryError(String),
    TimeoutError(String),
    ProtocolError(String),
    OtherError(String),
}

impl fmt::Display for DbError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DbError::ConnectionError(msg) => write!(f, "Connection error: {}", msg),
            DbError::QueryError(msg) => write!(f, "Query error: {}", msg),
            DbError::TimeoutError(msg) => write!(f, "Timeout error: {}", msg),
            DbError::ProtocolError(msg) => write!(f, "Protocol error: {}", msg),
            DbError::OtherError(msg) => write!(f, "Other database error: {}", msg),
        }
    }
}

impl StdError for DbError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        None
    }
}

impl From<std::io::Error> for DbError {
    fn from(error: std::io::Error) -> Self {
        DbError::ConnectionError(error.to_string())
    }
}

impl From<&str> for DbError {
    fn from(error: &str) -> Self {
        DbError::OtherError(error.to_string())
    }
}

impl From<String> for DbError {
    fn from(error: String) -> Self {
        DbError::OtherError(error)
    }
}

impl From<ConnectionError> for DbError {
    fn from(err: ConnectionError) -> Self {
        match err {
            ConnectionError::ConnectionTimeout =>
                DbError::TimeoutError(err.to_string()),
            ConnectionError::ProtocolError(msg) |
            ConnectionError::TlsError(msg) =>
                DbError::ProtocolError(msg),
            ConnectionError::IoError(_) |
            ConnectionError::ConnectionRefused |
            ConnectionError::AuthenticationFailed |
            ConnectionError::HostResolutionFailed(_) |
            ConnectionError::ConnectionClosed =>
                DbError::ConnectionError(err.to_string()),
            ConnectionError::PoolExhausted =>
                DbError::OtherError(err.to_string()),
        }
    }
}
