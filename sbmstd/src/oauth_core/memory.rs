//! In-memory default implementations for OAuth core traits.

use std::{sync::Arc, pin::Pin, future::Future};
use dashmap::DashMap;
use uuid::Uuid;
use super::types::{Client, Grant, Token, TokenModel, OAuthError};
use super::oauth_provider::{ClientStore, TokenManager, Authorizer, TokenStorage};
use tokio::sync::RwLock;
use std::collections::{HashMap, HashSet};

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

impl ClientStore for InMemoryClientStore {
    fn get_client(&self, id: &str) -> Pin<Box<dyn Future<Output = Result<Client, OAuthError>> + Send + 'static>> {
        let clients = self.clients.clone();
        let id_owned = id.to_owned();
        Box::pin(async move {
            clients.get(&id_owned)
                .map(|entry| entry.value().clone())
                .ok_or(OAuthError::InvalidClient)
        })
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

impl TokenManager for InMemoryTokenManager {
    fn generate_token(&self, _grant: Grant) -> Pin<Box<dyn Future<Output = Result<Token, OAuthError>> + Send + 'static>> {
        let tokens = self.tokens.clone();
        Box::pin(async move {
            // Generate a new opaque token pair
            let access_token = Uuid::new_v4().to_string();
            let refresh_token = Some(Uuid::new_v4().to_string());
            let token = Token {
                model: TokenModel::BearerOpaque,
                access_token: access_token.clone(),
                refresh_token: refresh_token.clone(),
                expires_in: 3600,
                scope: None,
            };
            tokens.insert(access_token.clone(), token.clone());
            Ok(token)
        })
    }

    fn revoke_token(&self, token: &str) -> Pin<Box<dyn Future<Output = Result<(), OAuthError>> + Send + 'static>> {
        let tokens = self.tokens.clone();
        let token_owned = token.to_owned();
        Box::pin(async move {
            tokens.remove(&token_owned);
            Ok(())
        })
    }

    fn validate_token(&self, token: &str) -> Pin<Box<dyn Future<Output = Result<Token, OAuthError>> + Send + 'static>> {
        let tokens = self.tokens.clone();
        let token_owned = token.to_owned();
        Box::pin(async move {
            tokens.get(&token_owned)
                .map(|entry| entry.value().clone())
                .ok_or(OAuthError::InvalidToken)
        })
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

impl Authorizer for InMemoryAuthorizer {
    fn record_consent(&self, client_id: &str, user_id: &str, scopes: &[String]) -> Pin<Box<dyn Future<Output = Result<(), OAuthError>> + Send + 'static>> {
        let consents = self.consents.clone();
        let key = (client_id.to_owned(), user_id.to_owned());
        let scopes_vec = scopes.to_vec();
        Box::pin(async move {
            consents.insert(key, scopes_vec);
            Ok(())
        })
    }

    fn check_consent(&self, client_id: &str, user_id: &str, scopes: &[String]) -> Pin<Box<dyn Future<Output = Result<bool, OAuthError>> + Send + 'static>> {
        let consents = self.consents.clone();
        let key = (client_id.to_owned(), user_id.to_owned());
        let required = scopes.to_vec();
        Box::pin(async move {
            if let Some(entry) = consents.get(&key) {
                let granted = entry.value();
                Ok(required.iter().all(|s| granted.contains(s)))
            } else {
                Ok(false)
            }
        })
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

impl TokenStorage for InMemoryTokenStorage {
    fn store_access_token(
        &self,
        token: &str,
        data: Token,
        _expires_in: u64,
    ) -> Pin<Box<dyn Future<Output = Result<(), OAuthError>> + Send + 'static>> {
        let map = self.access_tokens.clone();
        let key = token.to_string();
        Box::pin(async move {
            let mut guard = map.write().await;
            guard.insert(key, data);
            Ok(())
        })
    }

    fn get_access_token(
        &self,
        token: &str,
    ) -> Pin<Box<dyn Future<Output = Result<Option<Token>, OAuthError>> + Send + 'static>> {
        let map = self.access_tokens.clone();
        let key = token.to_string();
        Box::pin(async move {
            let guard = map.read().await;
            Ok(guard.get(&key).cloned())
        })
    }

    fn delete_access_token(
        &self,
        token: &str,
    ) -> Pin<Box<dyn Future<Output = Result<(), OAuthError>> + Send + 'static>> {
        let map = self.access_tokens.clone();
        let key = token.to_string();
        Box::pin(async move {
            let mut guard = map.write().await;
            guard.remove(&key);
            Ok(())
        })
    }

    fn store_refresh_token(
        &self,
        refresh_token: &str,
        access_token: &str,
        _expires_in: u64,
    ) -> Pin<Box<dyn Future<Output = Result<(), OAuthError>> + Send + 'static>> {
        let map = self.refresh_tokens.clone();
        let rkey = refresh_token.to_string();
        let akey = access_token.to_string();
        Box::pin(async move {
            let mut guard = map.write().await;
            guard.insert(rkey, akey);
            Ok(())
        })
    }

    fn get_refresh_token(
        &self,
        refresh_token: &str,
    ) -> Pin<Box<dyn Future<Output = Result<Option<String>, OAuthError>> + Send + 'static>> {
        let map = self.refresh_tokens.clone();
        let key = refresh_token.to_string();
        Box::pin(async move {
            let guard = map.read().await;
            Ok(guard.get(&key).cloned())
        })
    }

    fn delete_refresh_token(
        &self,
        refresh_token: &str,
    ) -> Pin<Box<dyn Future<Output = Result<(), OAuthError>> + Send + 'static>> {
        let map = self.refresh_tokens.clone();
        let key = refresh_token.to_string();
        Box::pin(async move {
            let mut guard = map.write().await;
            guard.remove(&key);
            Ok(())
        })
    }

    fn store_pkce_verifier(
        &self,
        code_challenge: &str,
        code_verifier: &str,
    ) -> Pin<Box<dyn Future<Output = Result<(), OAuthError>> + Send + 'static>> {
        let map = self.pkce_store.clone();
        let c_chal = code_challenge.to_string();
        let c_ver = code_verifier.to_string();
        Box::pin(async move {
            let mut guard = map.write().await;
            guard.insert(c_chal, c_ver);
            Ok(())
        })
    }

    fn get_pkce_verifier(
        &self,
        code_challenge: &str,
    ) -> Pin<Box<dyn Future<Output = Result<Option<String>, OAuthError>> + Send + 'static>> {
        let map = self.pkce_store.clone();
        let key = code_challenge.to_string();
        Box::pin(async move {
            let guard = map.read().await;
            Ok(guard.get(&key).cloned())
        })
    }

    fn delete_pkce_verifier(
        &self,
        code_challenge: &str,
    ) -> Pin<Box<dyn Future<Output = Result<(), OAuthError>> + Send + 'static>> {
        let map = self.pkce_store.clone();
        let key = code_challenge.to_string();
        Box::pin(async move {
            let mut guard = map.write().await;
            guard.remove(&key);
            Ok(())
        })
    }

    fn store_csrf_state(
        &self,
        state: &str,
        _expires_in: u64,
    ) -> Pin<Box<dyn Future<Output = Result<(), OAuthError>> + Send + 'static>> {
        let map = self.csrf_store.clone();
        let key = state.to_string();
        Box::pin(async move {
            let mut guard = map.write().await;
            guard.insert(key);
            Ok(())
        })
    }

    fn get_csrf_state(
        &self,
        state: &str,
    ) -> Pin<Box<dyn Future<Output = Result<bool, OAuthError>> + Send + 'static>> {
        let map = self.csrf_store.clone();
        let key = state.to_string();
        Box::pin(async move {
            let guard = map.read().await;
            Ok(guard.contains(&key))
        })
    }

    fn delete_csrf_state(
        &self,
        state: &str,
    ) -> Pin<Box<dyn Future<Output = Result<(), OAuthError>> + Send + 'static>> {
        let map = self.csrf_store.clone();
        let key = state.to_string();
        Box::pin(async move {
            let mut guard = map.write().await;
            guard.remove(&key);
            Ok(())
        })
    }
} 