//! In-memory default implementations for OAuth core traits.

use std::{sync::Arc, pin::Pin, future::Future};
use dashmap::DashMap;
use uuid::Uuid;
use super::types::{Client, Grant, Token, TokenModel, OAuthError};
use super::oauth_provider::{ClientStore, TokenManager, Authorizer, TokenStorage};
use tokio::sync::RwLock;
use std::collections::{HashMap, HashSet};
use async_trait::async_trait;

#[derive(Clone)]
pub struct InMemoryClientStore {
    clients: Arc<DashMap<String, Client>>,
}

impl InMemoryClientStore {
    /// Creates a new in-memory client store with an initial set of clients.
    pub fn new(initial_clients: Vec<Client>) -> Self {
        let map = DashMap::new();
        for client in initial_clients {
            map.insert(client.id.clone(), client);
        }
        Self { clients: Arc::new(map) }
    }
}

#[async_trait]
impl ClientStore for InMemoryClientStore {
    async fn get_client(&self, id: &str) -> Result<Client, OAuthError> {
        self.clients.get(id)
            .map(|entry| entry.value().clone())
            .ok_or(OAuthError::InvalidClient)
    }
}

#[derive(Clone)]
pub struct InMemoryTokenManager {
    tokens: Arc<DashMap<String, Token>>,
}

impl InMemoryTokenManager {
    /// Creates a new in-memory token manager.
    pub fn new() -> Self {
        Self { tokens: Arc::new(DashMap::new()) }
    }
}

#[async_trait]
impl TokenManager for InMemoryTokenManager {
    async fn generate_token(&self, _grant: Grant) -> Result<Token, OAuthError> {
        let access_token = Uuid::new_v4().to_string();
        let refresh_token = Some(Uuid::new_v4().to_string());
        let token = Token {
            model: TokenModel::BearerOpaque,
            access_token: access_token.clone(),
            refresh_token: refresh_token.clone(),
            expires_in: 3600,
            scope: None,
        };
        self.tokens.insert(access_token.clone(), token.clone());
        Ok(token)
    }

    async fn revoke_token(&self, token: &str) -> Result<(), OAuthError> {
        self.tokens.remove(token);
        Ok(())
    }

    async fn validate_token(&self, token: &str) -> Result<Token, OAuthError> {
        self.tokens.get(token)
            .map(|entry| entry.value().clone())
            .ok_or(OAuthError::InvalidToken)
    }
}

#[derive(Clone)]
pub struct InMemoryAuthorizer {
    consents: Arc<DashMap<(String, String), Vec<String>>>,
}

impl InMemoryAuthorizer {
    /// Creates a new in-memory authorizer.
    pub fn new() -> Self {
        Self { consents: Arc::new(DashMap::new()) }
    }
}

#[async_trait]
impl Authorizer for InMemoryAuthorizer {
    async fn record_consent(&self, client_id: &str, user_id: &str, scopes: &[String]) -> Result<(), OAuthError> {
        self.consents.insert((client_id.to_owned(), user_id.to_owned()), scopes.to_vec());
        Ok(())
    }

    async fn check_consent(&self, client_id: &str, user_id: &str, scopes: &[String]) -> Result<bool, OAuthError> {
        if let Some(entry) = self.consents.get(&(client_id.to_owned(), user_id.to_owned())) {
            let granted = entry.value();
            Ok(scopes.iter().all(|s| granted.contains(s)))
        } else {
            Ok(false)
        }
    }
}

/// In-memory storage backend for OAuth tokens, PKCE verifiers, and CSRF states.
#[derive(Clone)]
pub struct InMemoryTokenStorage {
    access_tokens: Arc<RwLock<HashMap<String, Token>>>,
    refresh_tokens: Arc<RwLock<HashMap<String, String>>>,
    pkce_store: Arc<RwLock<HashMap<String, String>>>,
    csrf_store: Arc<RwLock<HashSet<String>>>,
}

impl InMemoryTokenStorage {
    /// Creates a new in-memory token storage.
    pub fn new() -> Self {
        Self {
            access_tokens: Arc::new(RwLock::new(HashMap::new())),
            refresh_tokens: Arc::new(RwLock::new(HashMap::new())),
            pkce_store: Arc::new(RwLock::new(HashMap::new())),
            csrf_store: Arc::new(RwLock::new(HashSet::new())),
        }
    }
}

#[async_trait]
impl TokenStorage for InMemoryTokenStorage {
    async fn store_access_token(&self, token: &str, data: Token, _expires_in: u64) -> Result<(), OAuthError> {
        let mut guard = self.access_tokens.write().await;
        guard.insert(token.to_string(), data);
        Ok(())
    }

    async fn get_access_token(&self, token: &str) -> Result<Option<Token>, OAuthError> {
        let guard = self.access_tokens.read().await;
        Ok(guard.get(token).cloned())
    }

    async fn delete_access_token(&self, token: &str) -> Result<(), OAuthError> {
        let mut guard = self.access_tokens.write().await;
        guard.remove(token);
        Ok(())
    }

    async fn store_refresh_token(&self, refresh_token: &str, access_token: &str, _expires_in: u64) -> Result<(), OAuthError> {
        let mut guard = self.refresh_tokens.write().await;
        guard.insert(refresh_token.to_string(), access_token.to_string());
        Ok(())
    }

    async fn get_refresh_token(&self, refresh_token: &str) -> Result<Option<String>, OAuthError> {
        let guard = self.refresh_tokens.read().await;
        Ok(guard.get(refresh_token).cloned())
    }

    async fn delete_refresh_token(&self, refresh_token: &str) -> Result<(), OAuthError> {
        let mut guard = self.refresh_tokens.write().await;
        guard.remove(refresh_token);
        Ok(())
    }

    async fn store_pkce_verifier(&self, code_challenge: &str, code_verifier: &str) -> Result<(), OAuthError> {
        let mut guard = self.pkce_store.write().await;
        guard.insert(code_challenge.to_string(), code_verifier.to_string());
        Ok(())
    }

    async fn get_pkce_verifier(&self, code_challenge: &str) -> Result<Option<String>, OAuthError> {
        let guard = self.pkce_store.read().await;
        Ok(guard.get(code_challenge).cloned())
    }

    async fn delete_pkce_verifier(&self, code_challenge: &str) -> Result<(), OAuthError> {
        let mut guard = self.pkce_store.write().await;
        guard.remove(code_challenge);
        Ok(())
    }

    async fn store_csrf_state(&self, state: &str, _expires_in: u64) -> Result<(), OAuthError> {
        let mut guard = self.csrf_store.write().await;
        guard.insert(state.to_string());
        Ok(())
    }

    async fn get_csrf_state(&self, state: &str) -> Result<bool, OAuthError> {
        let guard = self.csrf_store.read().await;
        Ok(guard.contains(state))
    }

    async fn delete_csrf_state(&self, state: &str) -> Result<(), OAuthError> {
        let mut guard = self.csrf_store.write().await;
        guard.remove(state);
        Ok(())
    }
} 