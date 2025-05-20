//! In-memory default implementations for OAuth core traits.

use std::{sync::Arc, pin::Pin, future::Future};
use dashmap::DashMap;
use uuid::Uuid;
use super::types::{Client, Grant, Token, TokenModel, OAuthError};
use super::oauth_provider::{ClientStore, TokenManager, Authorizer};

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