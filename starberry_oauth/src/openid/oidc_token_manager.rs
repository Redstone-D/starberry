use async_trait::async_trait;
use crate::oauth_core::types::Grant;
use crate::oauth_core::types::{Client, UserContext, Token, OAuthError};

/// OIDC-server extension of TokenManager
#[async_trait]
pub trait OidcTokenManager {
    /// Generate access/refresh *and* an id_token when "openid" scope is present.
    async fn generate_oidc_token(
        &self,
        grant: Grant,
        user: &UserContext,
        client: &Client,
        nonce: Option<String>,
    ) -> Result<Token, OAuthError>;
}

// Blanket impl hint (devs can override if they want custom behavior)
#[async_trait]
impl<T> OidcTokenManager for T
where
    T: crate::oauth_core::oauth_provider::TokenManager + Send + Sync + 'static,
{
    async fn generate_oidc_token(
        &self,
        grant: Grant,
        user: &UserContext,
        client: &Client,
        nonce: Option<String>,
    ) -> Result<Token, OAuthError> {
        // calls core TokenManager + adds id_token
        unimplemented!()
    }
}
