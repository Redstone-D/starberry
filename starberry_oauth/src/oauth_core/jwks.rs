//! JWKS caching for RS256 JWT validation.

use std::{sync::Arc, time::{Duration, Instant}};
use dashmap::DashMap;
use tokio::sync::RwLock;
use serde::Deserialize;
use serde_json;
use super::types::OAuthError;
use super::http_client::{CoreHttpClient, OAuthHttpClient, HttpRequest, RedirectPolicy};
use starberry_core::http::http_value::HttpMethod;
use jsonwebtoken::DecodingKey;
use tracing::instrument;

/// A JWK as represented in a JWKS endpoint.
#[derive(Debug, Deserialize)]
struct Jwk {
    kty: String,
    kid: Option<String>,
    #[serde(rename = "use")]
    use_: Option<String>,
    n: String,
    e: String,
}

/// A JWKS response containing multiple keys.
#[derive(Debug, Deserialize)]
struct JwkSet {
    keys: Vec<Jwk>,
}

/// A cache of JWKs fetched from a JWKS URI with automatic refresh.
#[derive(Clone)]
pub struct JwksCache {
    client: CoreHttpClient,
    uri: String,
    keys: Arc<DashMap<String, (String, String)>>,
    ttl: Duration,
    last_refresh: Arc<RwLock<Instant>>,
}

impl JwksCache {
    /// Create a new JWKS cache with the given CoreHttpClient, endpoint URI, and TTL.
    /// Immediately fetches and caches the keys.
    pub async fn new(
        client: CoreHttpClient,
        uri: impl Into<String>,
        ttl: Duration,
    ) -> Result<Self, OAuthError> {
        let uri = uri.into();
        let cache = JwksCache {
            client,
            uri: uri.clone(),
            keys: Arc::new(DashMap::new()),
            ttl,
            last_refresh: Arc::new(RwLock::new(Instant::now() - ttl - Duration::from_secs(1))),
        };
        cache.fetch_and_store().await?;
        Ok(cache)
    }

    #[instrument(skip(self), level = "debug")]
    async fn fetch_and_store(&self) -> Result<(), OAuthError> {
        let req = HttpRequest {
            method: HttpMethod::GET,
            url: self.uri.clone(),
            headers: Vec::new(),
            body: None,
            timeout: None,
            redirect_policy: RedirectPolicy::None,
        };
        let resp = self.client.execute(req).await.map_err(|_| OAuthError::ServerError)?;
        if resp.status != 200 {
            return Err(OAuthError::ServerError);
        }
        let jwks: JwkSet = serde_json::from_slice(&resp.body).map_err(|_| OAuthError::ServerError)?;
        self.keys.clear();
        for jwk in jwks.keys {
            if let Some(kid) = jwk.kid {
                self.keys.insert(kid, (jwk.n, jwk.e));
            }
        }
        let mut write_lock = self.last_refresh.write().await;
        *write_lock = Instant::now();
        Ok(())
    }

    /// Get the DecodingKey for the given JWK key ID, refreshing cache if expired or missing.
    #[instrument(skip(self), level = "debug")]
    pub async fn get(&self, kid: &str) -> Result<DecodingKey, OAuthError> {
        let elapsed = Instant::now().duration_since(*self.last_refresh.read().await);
        if self.keys.get(kid).is_none() || elapsed > self.ttl {
            self.fetch_and_store().await?;
        }
        if let Some(entry) = self.keys.get(kid) {
            let (n, e) = entry.value();
            DecodingKey::from_rsa_components(n, e).map_err(|_| OAuthError::ServerError)
        } else {
            Err(OAuthError::InvalidToken)
        }
    }
} 