//! OAuth2 core module for sbmstd.
//!
//! Place OAuth2 primitives and traits here.

use std::pin::Pin;
use std::future::Future;
use super::types::{Client, Grant, Token, OAuthError};

/// Trait for retrieving OAuth2 clients.
pub trait ClientStore: Send + Sync + 'static {
    /// Retrieves a client by its identifier asynchronously.
    fn get_client(&self, id: &str) -> Pin<Box<dyn Future<Output = Result<Client, OAuthError>> + Send + 'static>>;
}

/// Trait for managing OAuth2 tokens.
pub trait TokenManager: Send + Sync + 'static {
    /// Generates a new token based on the provided grant asynchronously.
    fn generate_token(&self, grant: Grant) -> Pin<Box<dyn Future<Output = Result<Token, OAuthError>> + Send + 'static>>;

    /// Revokes the provided token asynchronously.
    fn revoke_token(&self, token: &str) -> Pin<Box<dyn Future<Output = Result<(), OAuthError>> + Send + 'static>>;

    /// Validates the provided token asynchronously, returning the associated Token if valid.
    fn validate_token(&self, token: &str) -> Pin<Box<dyn Future<Output = Result<Token, OAuthError>> + Send + 'static>>;
}

/// Trait for managing OAuth2 authorization (consent and scope checks).
pub trait Authorizer: Send + Sync + 'static {
    /// Records consent for the given client and user with specified scopes asynchronously.
    fn record_consent(&self, client_id: &str, user_id: &str, scopes: &[String]) -> Pin<Box<dyn Future<Output = Result<(), OAuthError>> + Send + 'static>>;

    /// Checks if the given client and user have consent for the specified scopes asynchronously.
    fn check_consent(&self, client_id: &str, user_id: &str, scopes: &[String]) -> Pin<Box<dyn Future<Output = Result<bool, OAuthError>> + Send + 'static>>;
}

// TODO: Implement OAuth2 core functionality. 