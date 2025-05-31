//! Rate limiting abstraction for OAuth middleware.

use async_trait::async_trait;
use super::types::OAuthError;

/// Trait for rate limiting by a given key (e.g., client ID or IP address).
#[async_trait]
pub trait RateLimiter: Send + Sync + 'static {
    /// Attempts to consume one token for the specified key.
    /// Returns Ok(true) if allowed, Ok(false) if rate-limited, or Err on internal error.
    async fn consume(&self, key: &str) -> Result<bool, OAuthError>;
} 