//! OAuth2 server with OpenID Connect support example
//! Run with: `cargo run --example openid --features openid`

use starberry_core::app::application::{App, AppBuilder};
use starberry_core::app::protocol::ProtocolHandlerBuilder;
use starberry_core::http::context::HttpReqCtx;
use starberry_oauth::OAuthLayer;

#[tokio::main]
async fn main() {
    // Build the application with OpenID Connect plugin enabled
    let app = App::new()
        .single_protocol(
            ProtocolHandlerBuilder::<HttpReqCtx>::new()
                .append_middleware::<OAuthLayer>()
        )
        .build();

    // The /.well-known/openid-configuration and /jwks.json endpoints are served automatically
    app.run().await;
}