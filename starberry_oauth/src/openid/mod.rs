#![cfg(feature = "openid")]
//! Optional OpenID Connect server support (discovery, JWKS, id_token, userinfo).

pub mod discovery;
pub mod oidc_token_manager;
