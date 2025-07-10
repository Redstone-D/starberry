// Integration test for OpenID Connect structs

#[cfg(feature = "openid")]
use starberry_oauth::openid::discovery::OIDCDiscovery;

#[cfg(feature = "openid")]
#[test]
fn test_openid_discovery_struct() {
    let disc = OIDCDiscovery {
        issuer: "issuer".to_string(),
        authorization_endpoint: "auth".to_string(),
        token_endpoint: "token".to_string(),
        userinfo_endpoint: "userinfo".to_string(),
        jwks_uri: "jwks".to_string(),
    };
    assert_eq!(disc.issuer, "issuer");
    assert_eq!(disc.jwks_uri, "jwks");
} 