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

/// Trait to abstract storage operations for access/refresh tokens, PKCE verifiers, and CSRF states.
pub trait TokenStorage: Send + Sync + 'static {
    /// Store an access token string with its associated Token data and expiry.
    fn store_access_token(
        &self,
        token: &str,
        data: Token,
        expires_in: u64,
    ) -> Pin<Box<dyn Future<Output = Result<(), OAuthError>> + Send + 'static>>;

    /// Retrieve an access token if it exists.
    fn get_access_token(
        &self,
        token: &str,
    ) -> Pin<Box<dyn Future<Output = Result<Option<Token>, OAuthError>> + Send + 'static>>;

    /// Delete an access token.
    fn delete_access_token(
        &self,
        token: &str,
    ) -> Pin<Box<dyn Future<Output = Result<(), OAuthError>> + Send + 'static>>;

    /// Store a refresh token mapping to an access token.
    fn store_refresh_token(
        &self,
        refresh_token: &str,
        access_token: &str,
        expires_in: u64,
    ) -> Pin<Box<dyn Future<Output = Result<(), OAuthError>> + Send + 'static>>;

    /// Retrieve the access token associated with a refresh token.
    fn get_refresh_token(
        &self,
        refresh_token: &str,
    ) -> Pin<Box<dyn Future<Output = Result<Option<String>, OAuthError>> + Send + 'static>>;

    /// Delete a refresh token.
    fn delete_refresh_token(
        &self,
        refresh_token: &str,
    ) -> Pin<Box<dyn Future<Output = Result<(), OAuthError>> + Send + 'static>>;

    /// Store a PKCE code verifier keyed by its code challenge.
    fn store_pkce_verifier(
        &self,
        code_challenge: &str,
        code_verifier: &str,
    ) -> Pin<Box<dyn Future<Output = Result<(), OAuthError>> + Send + 'static>>;

    /// Retrieve a PKCE code verifier for a given code challenge.
    fn get_pkce_verifier(
        &self,
        code_challenge: &str,
    ) -> Pin<Box<dyn Future<Output = Result<Option<String>, OAuthError>> + Send + 'static>>;

    /// Delete a PKCE code verifier.
    fn delete_pkce_verifier(
        &self,
        code_challenge: &str,
    ) -> Pin<Box<dyn Future<Output = Result<(), OAuthError>> + Send + 'static>>;

    /// Store a CSRF state value with expiry.
    fn store_csrf_state(
        &self,
        state: &str,
        expires_in: u64,
    ) -> Pin<Box<dyn Future<Output = Result<(), OAuthError>> + Send + 'static>>;

    /// Check if a CSRF state exists.
    fn get_csrf_state(
        &self,
        state: &str,
    ) -> Pin<Box<dyn Future<Output = Result<bool, OAuthError>> + Send + 'static>>;

    /// Delete a CSRF state.
    fn delete_csrf_state(
        &self,
        state: &str,
    ) -> Pin<Box<dyn Future<Output = Result<(), OAuthError>> + Send + 'static>>;
}

// TODO: Implement OAuth2 core functionality. 