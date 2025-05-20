//! Cookie-based TokenManager leveraging server-side sessions in cookies.

use std::{pin::Pin, future::Future, collections::HashMap, time::{SystemTime, UNIX_EPOCH}};
use serde_json;
use super::oauth_provider::TokenManager;
use super::types::{TokenModel, Token, OAuthError, Grant};
use crate::session::session::{new_session, get_mut};

/// A TokenManager that uses session IDs in cookies for opaque tokens.
pub struct CookieTokenManager {
    /// Time-to-live for sessions (in seconds).
    ttl_secs: u64,
}

impl CookieTokenManager {
    /// Create a new CookieTokenManager with the given session TTL.
    pub fn new(ttl_secs: u64) -> Self {
        Self { ttl_secs }
    }
}

impl TokenManager for CookieTokenManager {
    fn generate_token(&self, grant: Grant) -> Pin<Box<dyn Future<Output = Result<Token, OAuthError>> + Send + 'static>> {
        let ttl = self.ttl_secs;
        Box::pin(async move {
            match grant {
                Grant::RefreshToken { token } => {
                    // Renew existing session from refresh token
                    let id = token.parse::<u64>().map_err(|_| OAuthError::InvalidGrant)?;
                    let mut session = get_mut(id).map_err(|_| OAuthError::InvalidGrant)?;
                    session.touch(ttl);
                    let expiry = session.expiry_time;
                    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
                    let expires_in = if expiry > now { expiry - now } else { 0 };
                    let scope = session.get("scope").cloned();
                    Ok(Token {
                        model: TokenModel::BearerOpaque,
                        access_token: token,
                        refresh_token: None,
                        expires_in,
                        scope,
                    })
                }
                _ => {
                    // Create a new session for the token
                    let data: HashMap<String, String> = HashMap::new();
                    // Optionally store grant info in session data:
                    // data.insert("grant".to_string(), serde_json::to_string(&grant).unwrap());
                    let session_id = new_session(data, ttl);
                    Ok(Token {
                        model: TokenModel::BearerOpaque,
                        access_token: session_id.to_string(),
                        refresh_token: None,
                        expires_in: ttl,
                        scope: None,
                    })
                }
            }
        })
    }

    fn revoke_token(&self, token: &str) -> Pin<Box<dyn Future<Output = Result<(), OAuthError>> + Send + 'static>> {
        let token_owned = token.to_string();
        Box::pin(async move {
            if let Ok(id) = token_owned.parse::<u64>() {
                if let Ok(mut session) = get_mut(id) {
                    // Expire session immediately
                    session.expiry_time = 0;
                    session.data.clear();
                }
            }
            Ok(())
        })
    }

    fn validate_token(&self, token: &str) -> Pin<Box<dyn Future<Output = Result<Token, OAuthError>> + Send + 'static>> {
        let ttl = self.ttl_secs;
        let token_owned = token.to_string();
        Box::pin(async move {
            let id = token_owned.parse::<u64>().map_err(|_| OAuthError::InvalidToken)?;
            let mut session = get_mut(id).map_err(|_| OAuthError::InvalidToken)?;
            session.touch(ttl);
            let expiry = session.expiry_time;
            let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
            let expires_in = if expiry > now { expiry - now } else { 0 };
            let scope = session.get("scope").cloned();
            Ok(Token {
                model: TokenModel::BearerOpaque,
                access_token: token_owned,
                refresh_token: None,
                expires_in,
                scope,
            })
        })
    }
} 