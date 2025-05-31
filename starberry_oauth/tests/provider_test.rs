use starberry_oauth::oauth_core::types::{Client, Grant, Token, OAuthError, TokenModel};
use starberry_oauth::oauth_core::oauth_provider::{ClientStore, TokenManager, Authorizer, TokenStorage};
use starberry_oauth::oauth_core::memory::{InMemoryClientStore, InMemoryTokenManager, InMemoryAuthorizer, InMemoryTokenStorage};

#[tokio::test]
async fn test_in_memory_client_store() {
    let client = Client {
        id: "client1".to_string(),
        secret: Some("secret".to_string()),
        redirect_uris: vec!["https://app.local/callback".to_string()],
    };
    let store = InMemoryClientStore::new(vec![client.clone()]);
    // Existing client
    let fetched = store.get_client("client1").await.unwrap();
    assert_eq!(fetched.id, client.id);
    assert_eq!(fetched.secret, client.secret);
    assert_eq!(fetched.redirect_uris, client.redirect_uris);
    // Missing client
    let err = store.get_client("missing").await.unwrap_err();
    assert!(matches!(err, OAuthError::InvalidClient));
}

#[tokio::test]
async fn test_in_memory_token_manager() {
    let mgr = InMemoryTokenManager::new();
    let grant = Grant::ClientCredentials;
    // Generate token
    let token = mgr.generate_token(grant).await.unwrap();
    assert!(matches!(token.model, TokenModel::BearerOpaque));
    // Validate existing token
    let validated = mgr.validate_token(&token.access_token).await.unwrap();
    assert_eq!(validated.access_token, token.access_token);
    // Revoke token
    mgr.revoke_token(&token.access_token).await.unwrap();
    let err = mgr.validate_token(&token.access_token).await.unwrap_err();
    assert!(matches!(err, OAuthError::InvalidToken));
}

#[tokio::test]
async fn test_in_memory_authorizer() {
    let auth = InMemoryAuthorizer::new();
    let scopes_full = vec!["read".to_string(), "write".to_string()];
    // No consent yet
    assert!(!auth.check_consent("cid", "uid", &scopes_full).await.unwrap());
    // Record consent for a subset
    auth.record_consent("cid", "uid", &["read".to_string()]).await.unwrap();
    // Check subset consent
    assert!(auth.check_consent("cid", "uid", &["read".to_string()]).await.unwrap());
    // Check full scopes - should fail
    assert!(!auth.check_consent("cid", "uid", &scopes_full).await.unwrap());
}

#[tokio::test]
async fn test_in_memory_token_storage() {
    let storage = InMemoryTokenStorage::new();
    let token = Token {
        model: TokenModel::BearerOpaque,
        access_token: "atoken".to_string(),
        refresh_token: Some("rtoken".to_string()),
        expires_in: 600,
        scope: Some("scope1".to_string()),
    };
    // Access token operations
    storage.store_access_token(&token.access_token, token.clone(), token.expires_in).await.unwrap();
    let fetched_at = storage.get_access_token(&token.access_token).await.unwrap().unwrap();
    assert_eq!(fetched_at.access_token, token.access_token);
    storage.delete_access_token(&token.access_token).await.unwrap();
    assert!(storage.get_access_token(&token.access_token).await.unwrap().is_none());
    // Refresh token operations
    let rt = token.refresh_token.as_ref().unwrap();
    storage.store_refresh_token(rt, &token.access_token, token.expires_in).await.unwrap();
    let fetched_rt = storage.get_refresh_token(rt).await.unwrap().unwrap();
    assert_eq!(fetched_rt, token.access_token);
    storage.delete_refresh_token(rt).await.unwrap();
    assert!(storage.get_refresh_token(rt).await.unwrap().is_none());
    // PKCE verifier operations
    storage.store_pkce_verifier("challenge", "verifier").await.unwrap();
    assert_eq!(storage.get_pkce_verifier("challenge").await.unwrap().unwrap(), "verifier".to_string());
    storage.delete_pkce_verifier("challenge").await.unwrap();
    assert!(storage.get_pkce_verifier("challenge").await.unwrap().is_none());
    // CSRF state operations
    storage.store_csrf_state("state123", 0).await.unwrap();
    assert!(storage.get_csrf_state("state123").await.unwrap());
    storage.delete_csrf_state("state123").await.unwrap();
    assert!(!storage.get_csrf_state("state123").await.unwrap());
} 