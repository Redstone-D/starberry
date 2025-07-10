#![allow(unused_imports)]
//! Minimal OAuth2 server example
//! Run with: `cargo run --example minimal`

use std::sync::Arc;
use tokio::net::TcpListener;
use starberry_core::app::application::{App, AppBuilder};
use starberry_core::app::protocol::ProtocolHandlerBuilder;
use starberry_core::http::context::HttpReqCtx;
use starberry_oauth::OAuthLayer;

#[tokio::main]
async fn main() {
    // Build the application with OAuth2 middleware
    let app = App::new()
        .single_protocol(
            ProtocolHandlerBuilder::<HttpReqCtx>::new()
                .append_middleware::<OAuthLayer>()
        )
        .build();

    // Run the server on 127.0.0.1:8080
    app.run().await;
}