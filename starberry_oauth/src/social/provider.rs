use async_trait::async_trait;
use crate::oauth_core::types::{UserContext, OAuthError};

/// Implement this to support "Login via X" (Google, GitHub, another OAuth2 server…)
#[async_trait]
pub trait ExternalLoginProvider: Send + Sync + 'static {
    /// e.g. "google", "github", "my-oauth2-server"
    fn scheme(&self) -> &str;

    /// Build the redirect URL to the upstream /authorize endpoint
    fn auth_redirect(&self, state: &str) -> String;

    /// After callback: exchange code → token → userinfo/id_token → UserContext
    async fn handle_callback(
        &self,
        code: &str,
        state: &str,
    ) -> Result<UserContext, OAuthError>;
}
