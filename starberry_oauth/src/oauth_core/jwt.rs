//! JWT-based TokenManager for OAuth2.

use std::{pin::Pin, future::Future};
use jsonwebtoken::{EncodingKey, DecodingKey, Header, Validation, encode, decode, decode_header};
use serde::{Serialize, Deserialize};
use chrono::Utc;
use super::types::{JWTAlgorithm, TokenModel, Token, Grant, OAuthError};
use super::oauth_provider::TokenManager;
use async_trait::async_trait;
use super::jwks::JwksCache;
use tracing::instrument;

/// A TokenManager that issues JWT access tokens.
pub struct JWTTokenManager {
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
    algorithm: JWTAlgorithm,
    expiration_seconds: u64,
    issuer: Option<String>,
    audience: Option<String>,
    jwks_cache: Option<JwksCache>,
}

impl JWTTokenManager {
    /// Create a new JWTTokenManager using HS256 and a shared secret.
    pub fn new_hs256(secret: &[u8], expiration_seconds: u64) -> Self {
        Self {
            encoding_key: EncodingKey::from_secret(secret),
            decoding_key: DecodingKey::from_secret(secret),
            algorithm: JWTAlgorithm::HS256,
            expiration_seconds,
            issuer: None,
            audience: None,
            jwks_cache: None,
        }
    }

    /// Create a new JWTTokenManager using RS256 and RSA key pair.
    pub fn new_rs256(private_key_pem: &[u8], public_key_pem: &[u8], expiration_seconds: u64) -> Self {
        Self {
            encoding_key: EncodingKey::from_rsa_pem(private_key_pem).expect("Invalid private key"),
            decoding_key: DecodingKey::from_rsa_pem(public_key_pem).expect("Invalid public key"),
            algorithm: JWTAlgorithm::RS256,
            expiration_seconds,
            issuer: None,
            audience: None,
            jwks_cache: None,
        }
    }

    /// Configure expected issuer and audience.
    pub fn with_claims(mut self, issuer: impl Into<String>, audience: impl Into<String>) -> Self {
        self.issuer = Some(issuer.into());
        self.audience = Some(audience.into());
        self
    }

    /// Configure a JWKS caching layer for RS256 key retrieval.
    pub fn with_jwks(mut self, jwks_cache: JwksCache) -> Self {
        self.jwks_cache = Some(jwks_cache);
        self
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String,
    exp: usize,
    scope: Option<String>,
}

#[async_trait]
impl TokenManager for JWTTokenManager {
    #[instrument(skip(self, grant), level = "debug")]
    async fn generate_token(&self, grant: Grant) -> Result<Token, OAuthError> {
        let encoding_key = self.encoding_key.clone();
        let alg = self.algorithm.clone();
        let exp_secs = self.expiration_seconds as usize;
        // Determine subject and scope based on grant.
        let (sub, scope) = match grant {
            Grant::AuthorizationCode { code, .. } => (code, None),
            Grant::ClientCredentials => ("client_credentials".to_string(), None),
            Grant::RefreshToken { token } => (token, None),
            Grant::ResourceOwnerPassword { username, .. } => (username, None),
            Grant::DeviceCode { device_code, .. } => (device_code, None),
        };
        let now = Utc::now().timestamp() as usize;
        let claims = Claims { sub, exp: now + exp_secs, scope };
        let header = match alg {
            JWTAlgorithm::HS256 => Header::new(jsonwebtoken::Algorithm::HS256),
            JWTAlgorithm::RS256 => Header::new(jsonwebtoken::Algorithm::RS256),
        };
        let token_str = encode(&header, &claims, &encoding_key).map_err(|_| OAuthError::ServerError)?;
        Ok(Token {
            model: TokenModel::JWT { algorithm: alg },
            access_token: token_str,
            refresh_token: None,
            expires_in: exp_secs as u64,
            scope: None,
        })
    }

    async fn revoke_token(&self, _token: &str) -> Result<(), OAuthError> {
        // Stateless JWT; revocation requires blacklist if needed
        Ok(())
    }

    #[instrument(skip(self, token), level = "debug")]
    async fn validate_token(&self, token: &str) -> Result<Token, OAuthError> {
        let alg = self.algorithm.clone();
        let token_owned = token.to_string();
        // Set up validation checks
        let mut validation = Validation::new(match alg {
            JWTAlgorithm::HS256 => jsonwebtoken::Algorithm::HS256,
            JWTAlgorithm::RS256 => jsonwebtoken::Algorithm::RS256,
        });
        validation.validate_exp = true;
        if let Some(ref iss) = self.issuer {
            validation.set_issuer(&[iss.clone()]);
        }
        if let Some(ref aud) = self.audience {
            validation.set_audience(&[aud.clone()]);
        }
        // Determine decoding key: JWKS cache overrides static key
        let decoding_key = if let Some(cache) = &self.jwks_cache {
            let header = decode_header(&token_owned).map_err(|_| OAuthError::InvalidToken)?;
            let kid = header.kid.ok_or(OAuthError::InvalidToken)?;
            cache.get(&kid).await.map_err(|_| OAuthError::InvalidToken)?
        } else {
            self.decoding_key.clone()
        };
        let token_data = decode::<Claims>(&token_owned, &decoding_key, &validation)
            .map_err(|_| OAuthError::InvalidToken)?;
        let claims = token_data.claims;
        let now = Utc::now().timestamp() as usize;
        let expires_in = if claims.exp > now { (claims.exp - now) as u64 } else { 0 };
        Ok(Token {
            model: TokenModel::JWT { algorithm: alg },
            access_token: token_owned.clone(),
            refresh_token: None,
            expires_in,
            scope: claims.scope,
        })
    }
} 