// Integration tests for plugin module compilation

use starberry_oauth::{
    oauth_core::types::{Token, TokenModel, JWTAlgorithm, OAuthContext},
};

#[test]
fn compile_core() {
    let token = Token {
        model: TokenModel::BearerOpaque,
        access_token: String::new(),
        refresh_token: None,
        expires_in: 3600,
        scope: None,
        id_token: None,
    };
    let _ctx = OAuthContext {
        client_id: String::new(),
        scopes: Vec::new(),
        token,
        user: None,
    };
}

#[cfg(feature = "openid")]
#[test]
fn compile_openid() {
    use starberry_oauth::openid::discovery::OIDCDiscovery;
    let _disc = OIDCDiscovery {
        issuer: String::new(),
        authorization_endpoint: String::new(),
        token_endpoint: String::new(),
        userinfo_endpoint: String::new(),
        jwks_uri: String::new(),
    };
}

#[cfg(feature = "social")]
#[test]
fn compile_social() {
    use starberry_oauth::social::provider::ExternalLoginProvider;
    let providers: Vec<Box<dyn ExternalLoginProvider>> = Vec::new();
    let _ = providers;
}
