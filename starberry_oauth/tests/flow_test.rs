use std::sync::Arc;
use starberry_oauth::{AuthorizationCodePkceFlow, ClientCredentialsFlow, RefreshTokenFlow, InMemoryTokenStorage, InMemoryHttpClient, OAuthHttpClient};
use starberry_oauth::oauth_core::types::OAuthError;
use starberry_oauth::oauth_core::http_client::HttpResponse;
use serde_json::json;

#[tokio::test]
async fn test_pkce_flow_in_memory() {
    let storage = Arc::new(InMemoryTokenStorage::new());
    let flow = AuthorizationCodePkceFlow::new::<String, String>(
        "client1".to_string(), Some("secret".to_string()),
        "https://auth.local/authorize".to_string(), "https://auth.local/token".to_string(),
        vec!["scope1".to_string()], storage.clone()
    );
    let redirect_uri = "https://app.local/callback";
    // Initiate will store state & verifier
    let auth_url = flow.initiate(redirect_uri, 3600).await.unwrap();
    assert!(auth_url.starts_with("https://auth.local/authorize?"));

    // Simulate token endpoint response
    let token_json = json!({
        "access_token": "ACCESS123",
        "refresh_token": "REFRESH456",
        "expires_in": 3600
    });
    let resp = HttpResponse { status: 200, headers: vec![], body: serde_json::to_vec(&token_json).unwrap() };
    let client = InMemoryHttpClient::with_default(resp);

    // Exchange code
    let code = "CODEXYZ";
    let token = flow.exchange(&client, code, redirect_uri).await.unwrap();
    assert_eq!(token.access_token, "ACCESS123");
    assert_eq!(token.refresh_token.unwrap(), "REFRESH456");
}

#[tokio::test]
async fn test_client_credentials_flow_in_memory() {
    let flow = ClientCredentialsFlow::new(
        "cid", "csecret", "https://auth.local/token", vec!["scopeA".to_string()]
    );
    let token_json = json!({
        "access_token": "TOKENABC",
        "refresh_token": null,
        "expires_in": 1800
    });
    let resp = HttpResponse { status: 200, headers: vec![], body: serde_json::to_vec(&token_json).unwrap() };
    let client = InMemoryHttpClient::with_default(resp);
    let token = flow.execute(&client).await.unwrap();
    assert_eq!(token.access_token, "TOKENABC");
    assert!(token.refresh_token.is_none());
}

#[tokio::test]
async fn test_refresh_token_flow_in_memory() {
    let flow = RefreshTokenFlow::new(
        "cid", Some("csecret"), "https://auth.local/token"
    );
    let token_json = json!({
        "access_token": "NEW123",
    "refresh_token": null,
        "expires_in": 7200
    });
    let resp = HttpResponse { status: 200, headers: vec![], body: serde_json::to_vec(&token_json).unwrap() };
    let client = InMemoryHttpClient::with_default(resp);
    let new_token = flow.execute(&client, "OLDREF").await.unwrap();
    assert_eq!(new_token.access_token, "NEW123");
    assert!(new_token.refresh_token.is_none());
}

#[tokio::test]
async fn test_pkce_flow_exchange_failure() {
    // Setup PKCE flow and storage
    let storage = Arc::new(InMemoryTokenStorage::new());
    let flow = AuthorizationCodePkceFlow::new::<String, String>(
        "client1".to_string(), Some("secret".to_string()),
        "https://auth.local/authorize".to_string(), "https://auth.local/token".to_string(),
        vec!["scope1".to_string()], storage.clone()
    );
    let redirect_uri = "https://app.local/callback";
    flow.initiate(redirect_uri, 3600).await.unwrap();

    // Simulate an error response from the token endpoint
    let error_json = json!({
        "error": "invalid_grant",
        "error_description": "The provided authorization code is invalid or expired."
    });
    let resp = HttpResponse { status: 400, headers: vec![], body: serde_json::to_vec(&error_json).unwrap() };
    let client = InMemoryHttpClient::with_default(resp);

    // Attempt exchange and assert InvalidGrant error
    let result = flow.exchange(&client, "EXPIRED_CODE", redirect_uri).await;
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), OAuthError::InvalidGrant));
} 