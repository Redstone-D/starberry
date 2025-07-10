//! OAuth2 server with social login plugin example
//! Run with: `cargo run --example social --features social`

use starberry_core::app::application::{App, AppBuilder};
use starberry_core::app::protocol::ProtocolHandlerBuilder;
use starberry_core::http::context::HttpReqCtx;
use starberry_oauth::OAuthLayer;
use std::sync::Arc;
use async_trait::async_trait;

// Social login provider stub enabled only when 'social' feature is active
#[cfg(feature = "social")]
use starberry_oauth::social::provider::ExternalLoginProvider;

// Dummy provider stub implementation
#[cfg(feature = "social")]
struct DummyProvider;

#[cfg(feature = "social")]
#[async_trait]
impl ExternalLoginProvider for DummyProvider {
    fn scheme(&self) -> &str {
        "foo"
    }
    fn auth_redirect(&self, state: &str) -> String {
        // Build redirect URL to external OAuth2 provider
        format!("https://example.com/authorize?state={}", state)
    }
    async fn handle_callback(&self, code: &str, state: &str) -> Result<starberry_oauth::oauth_core::types::UserContext, starberry_oauth::oauth_core::types::OAuthError> {
        // Exchange code for user info here
        Ok(starberry_oauth::oauth_core::types::UserContext {
            subject: code.to_string(),
            email: None,
            email_verified: None,
            name: None,
            picture: None,
        })
    }
}

#[tokio::main]
async fn main() {
    #[cfg(feature = "social")]
    {
        let providers: Vec<Arc<dyn ExternalLoginProvider>> = vec![Arc::new(DummyProvider)];
        // Build the application with social login plugin enabled
        let app = App::new()
            .single_protocol(
                ProtocolHandlerBuilder::<HttpReqCtx>::new()
                    .append_middleware::<OAuthLayer>()
            )
            .build();
        // The /login/foo and /login/foo/cb endpoints are served automatically
        app.run().await;
    }
    #[cfg(not(feature = "social"))]
    {
        eprintln!("Enable the 'social' feature to run this example");
    }
}