use std::fmt;
use std::io;

#[derive(Debug)]
pub enum ConnectionError {
    IoError(io::Error),
    TlsError(String),
    ConnectionTimeout,
    HostResolutionFailed(String),
    AuthenticationFailed,
    ConnectionRefused,
    ConnectionClosed,
    ProtocolError(String),
    PoolExhausted,
}

impl fmt::Display for ConnectionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::IoError(err) => write!(f, "I/O error: {}", err),
            Self::TlsError(err) => write!(f, "TLS error: {}", err),
            Self::ConnectionTimeout => write!(f, "Connection timed out"),
            Self::HostResolutionFailed(h) => write!(f, "Failed to resolve host: {}", h),
            Self::AuthenticationFailed => write!(f, "Authentication failed"),
            Self::ConnectionRefused => write!(f, "Connection refused"),
            Self::ConnectionClosed => write!(f, "Connection closed unexpectedly"),
            Self::ProtocolError(err) => write!(f, "Protocol error: {}", err),
            Self::PoolExhausted => write!(f, "Connection pool exhausted"),
        }
    }
}

impl std::error::Error for ConnectionError {}

impl From<io::Error> for ConnectionError {
    fn from(err: io::Error) -> Self {
        Self::IoError(err)
    }
}

impl From<tokio::time::error::Elapsed> for ConnectionError {
    fn from(_err: tokio::time::error::Elapsed) -> Self {
        Self::ConnectionTimeout 
    }
}

pub type Result<T> = std::result::Result<T, ConnectionError>; 
