//! OAuth2 core module for sbmstd.
//!
//! Place OAuth2 primitives and traits here.

use std::pin::Pin;
use std::future::Future;
use super::types::{Client, Grant, Token, OAuthError};
use async_trait::async_trait;

/// Trait for retrieving OAuth2 clients.
#[async_trait]
pub trait ClientStore: Send + Sync + 'static {
    /// Retrieves a client by its identifier asynchronously.
    async fn get_client(&self, id: &str) -> Result<Client, OAuthError>;
}

/// Trait for managing OAuth2 tokens.
#[async_trait]
pub trait TokenManager: Send + Sync + 'static {
    /// Generates a new token based on the provided grant asynchronously.
    async fn generate_token(&self, grant: Grant) -> Result<Token, OAuthError>;

    /// Revokes the provided token asynchronously.
    async fn revoke_token(&self, token: &str) -> Result<(), OAuthError>;

    /// Validates the provided token asynchronously, returning the associated Token if valid.
    async fn validate_token(&self, token: &str) -> Result<Token, OAuthError>;
}

/// Trait for managing OAuth2 authorization (consent and scope checks).
#[async_trait]
pub trait Authorizer: Send + Sync + 'static {
    /// Records consent for the given client and user with specified scopes asynchronously.
    async fn record_consent(&self, client_id: &str, user_id: &str, scopes: &[String]) -> Result<(), OAuthError>;

    /// Checks if the given client and user have consent for the specified scopes asynchronously.
    async fn check_consent(&self, client_id: &str, user_id: &str, scopes: &[String]) -> Result<bool, OAuthError>;
}

/// Trait to abstract storage operations for access/refresh tokens, PKCE verifiers, and CSRF states.
#[async_trait]
pub trait TokenStorage: Send + Sync + 'static {
    /// Store an access token string with its associated Token data and expiry.
    async fn store_access_token(
        &self,
        token: &str,
        data: Token,
        expires_in: u64,
    ) -> Result<(), OAuthError>;

    /// Retrieve an access token if it exists.
    async fn get_access_token(&self, token: &str) -> Result<Option<Token>, OAuthError>;

    /// Delete an access token.
    async fn delete_access_token(&self, token: &str) -> Result<(), OAuthError>;

    /// Store a refresh token mapping to an access token.
    async fn store_refresh_token(&self, refresh_token: &str, access_token: &str, expires_in: u64) -> Result<(), OAuthError>;

    /// Retrieve the access token associated with a refresh token.
    async fn get_refresh_token(&self, refresh_token: &str) -> Result<Option<String>, OAuthError>;

    /// Delete a refresh token.
    async fn delete_refresh_token(&self, refresh_token: &str) -> Result<(), OAuthError>;

    /// Store a PKCE code verifier keyed by its code challenge.
    async fn store_pkce_verifier(&self, code_challenge: &str, code_verifier: &str) -> Result<(), OAuthError>;

    /// Retrieve a PKCE code verifier for a given code challenge.
    async fn get_pkce_verifier(&self, code_challenge: &str) -> Result<Option<String>, OAuthError>;

    /// Delete a PKCE code verifier.
    async fn delete_pkce_verifier(&self, code_challenge: &str) -> Result<(), OAuthError>;

    /// Store a CSRF state value with expiry.
    async fn store_csrf_state(&self, state: &str, expires_in: u64) -> Result<(), OAuthError>;

    /// Check if a CSRF state exists.
    async fn get_csrf_state(&self, state: &str) -> Result<bool, OAuthError>;

    /// Delete a CSRF state.
    async fn delete_csrf_state(&self, state: &str) -> Result<(), OAuthError>;
}

// TODO: Implement OAuth2 core functionality. 