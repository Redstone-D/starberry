//! Database-backed TokenManager for opaque tokens.

use std::{pin::Pin, future::Future};
use uuid::Uuid;
use chrono::Utc;
use super::types::{TokenModel, Token, OAuthError, Grant};
use super::oauth_provider::TokenManager;
use starberry_core::sql::builder::SqlQuery;
use starberry_core::sql::pool::SqlPool;

/// A TokenManager that persists opaque tokens in the database.
pub struct DBTokenManager {
    pool: SqlPool,
    expiry_seconds: u64,
}

impl DBTokenManager {
    /// Create a new DBTokenManager with a connection pool and token TTL.
    pub fn new(pool: SqlPool, expiry_seconds: u64) -> Self {
        Self { pool, expiry_seconds }
    }
}

impl TokenManager for DBTokenManager {
    fn generate_token(&self, _grant: Grant) -> Pin<Box<dyn Future<Output = Result<Token, OAuthError>> + Send + 'static>> {
        let pool = self.pool.clone();
        let exp_secs = self.expiry_seconds;
        Box::pin(async move {
            // Create new opaque token pair
            let access_token = Uuid::new_v4().to_string();
            let refresh_token = Some(Uuid::new_v4().to_string());
            let now = Utc::now().timestamp();
            let expires_at = now + exp_secs as i64;
            // Persist to database (ensure you have a table `oauth_tokens` with matching columns)
            let sql = "INSERT INTO oauth_tokens (access_token, refresh_token, expires_at, scope) VALUES ($1, $2, $3, $4)";
            SqlQuery::new(sql)
                .bind(access_token.clone())
                .bind(refresh_token.clone().unwrap())
                .bind(expires_at)
                .bind(None::<String>)
                .execute_pool(&pool)
                .await
                .map_err(|_| OAuthError::ServerError)?;
            Ok(Token {
                model: TokenModel::BearerOpaque,
                access_token,
                refresh_token,
                expires_in: exp_secs,
                scope: None,
            })
        })
    }

    fn revoke_token(&self, token: &str) -> Pin<Box<dyn Future<Output = Result<(), OAuthError>> + Send + 'static>> {
        let pool = self.pool.clone();
        let token_owned = token.to_owned();
        Box::pin(async move {
            let sql = "DELETE FROM oauth_tokens WHERE access_token = $1";
            SqlQuery::new(sql)
                .bind(token_owned)
                .execute_pool(&pool)
                .await
                .map_err(|_| OAuthError::ServerError)?;
            Ok(())
        })
    }

    fn validate_token(&self, token: &str) -> Pin<Box<dyn Future<Output = Result<Token, OAuthError>> + Send + 'static>> {
        let pool = self.pool.clone();
        let token_owned = token.to_owned();
        Box::pin(async move {
            let sql = "SELECT expires_at, refresh_token, scope FROM oauth_tokens WHERE access_token = $1";
            let row = SqlQuery::new(sql)
                .bind(token_owned.clone())
                .fetch_one_pool(&pool)
                .await
                .map_err(|_| OAuthError::InvalidToken)?;
            // Parse expiration
            let expires_at: i64 = row.get("expires_at").ok_or(OAuthError::InvalidToken)?
                .parse().map_err(|_| OAuthError::InvalidToken)?;
            let now = Utc::now().timestamp();
            if expires_at < now {
                return Err(OAuthError::InvalidToken);
            }
            let refresh_token = row.get("refresh_token").cloned();
            let scope = row.get("scope").cloned();
            let expires_in = (expires_at - now) as u64;
            Ok(Token {
                model: TokenModel::BearerOpaque,
                access_token: token_owned,
                refresh_token,
                expires_in,
                scope,
            })
        })
    }
} 