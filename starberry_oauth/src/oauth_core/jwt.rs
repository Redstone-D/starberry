//! JWT-based TokenManager for OAuth2.

use std::{pin::Pin, future::Future};
use jsonwebtoken::{EncodingKey, DecodingKey, Header, Validation, encode, decode};
use serde::{Serialize, Deserialize};
use chrono::Utc;
use super::types::{JWTAlgorithm, TokenModel, Token, Grant, OAuthError};
use super::oauth_provider::TokenManager;
use async_trait::async_trait;

/// A TokenManager that issues JWT access tokens.
pub struct JWTTokenManager {
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
    algorithm: JWTAlgorithm,
    expiration_seconds: u64,
}

impl JWTTokenManager {
    /// Create a new JWTTokenManager using HS256 and a shared secret.
    pub fn new_hs256(secret: &[u8], expiration_seconds: u64) -> Self {
        Self {
            encoding_key: EncodingKey::from_secret(secret),
            decoding_key: DecodingKey::from_secret(secret),
            algorithm: JWTAlgorithm::HS256,
            expiration_seconds,
        }
    }

    /// Create a new JWTTokenManager using RS256 and RSA key pair.
    pub fn new_rs256(private_key_pem: &[u8], public_key_pem: &[u8], expiration_seconds: u64) -> Self {
        Self {
            encoding_key: EncodingKey::from_rsa_pem(private_key_pem).expect("Invalid private key"),
            decoding_key: DecodingKey::from_rsa_pem(public_key_pem).expect("Invalid public key"),
            algorithm: JWTAlgorithm::RS256,
            expiration_seconds,
        }
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

    async fn validate_token(&self, token: &str) -> Result<Token, OAuthError> {
        let decoding_key = self.decoding_key.clone();
        let alg = self.algorithm.clone();
        let token_owned = token.to_owned();
        let mut validation = Validation::new(match alg {
            JWTAlgorithm::HS256 => jsonwebtoken::Algorithm::HS256,
            JWTAlgorithm::RS256 => jsonwebtoken::Algorithm::RS256,
        });
        let token_data = decode::<Claims>(&token_owned, &decoding_key, &validation).map_err(|_| OAuthError::InvalidToken)?;
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