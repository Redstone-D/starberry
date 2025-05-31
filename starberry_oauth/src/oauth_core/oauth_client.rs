use uuid::Uuid;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use ring::rand::{SecureRandom, SystemRandom};
use starberry_lib::{encode_url_owned};
use super::crypto::pkce_code_challenge;
use starberry_core::http::http_value::HttpMethod;
use super::http_client::{OAuthHttpClient, HttpRequest, HttpResponse, RedirectPolicy};
use super::types::{Token, OAuthError, TokenModel};
use serde_json;
use tracing::instrument;

/// OAuth2 client that encapsulates all OAuth2 client state, including PKCE and CSRF.
#[derive(Clone, Debug)]
pub struct OAuthClient {
    /// Client identifier.
    pub client_id: String,
    /// Optional client secret.
    pub client_secret: Option<String>,
    /// OAuth2 authorization endpoint URL.
    pub authorize_url: String,
    /// OAuth2 token endpoint URL.
    pub token_url: String,
    /// Requested scopes.
    pub scopes: Vec<String>,
    /// CSRF state parameter.
    pub state: String,
    /// PKCE code verifier.
    pub code_verifier: String,
    /// PKCE code challenge derived from the verifier.
    pub code_challenge: String,
}

impl OAuthClient {
    /// Creates a new OAuthClient with the given parameters, generating a CSRF state
    /// and PKCE code verifier & challenge.
    pub fn new(
        client_id: impl Into<String>,
        client_secret: Option<impl Into<String>>,
        authorize_url: impl Into<String>,
        token_url: impl Into<String>,
        scopes: impl IntoIterator<Item = String>,
    ) -> Self {
        let client_id = client_id.into();
        let client_secret = client_secret.map(|s| s.into());
        let authorize_url = authorize_url.into();
        let token_url = token_url.into();
        let scopes = scopes.into_iter().collect();
        // Generate CSRF state
        let state = Uuid::new_v4().to_string();
        // Generate PKCE verifier
        let rng = SystemRandom::new();
        let mut buf = [0u8; 32];
        rng.fill(&mut buf).expect("PKCE code verifier generation failed");
        let code_verifier = URL_SAFE_NO_PAD.encode(&buf);
        // Generate PKCE challenge
        let code_challenge = pkce_code_challenge(&code_verifier);

        OAuthClient {
            client_id,
            client_secret,
            authorize_url,
            token_url,
            scopes,
            state,
            code_verifier,
            code_challenge,
        }
    }

    /// Returns a reference to the PKCE code verifier.
    pub fn code_verifier(&self) -> &str {
        &self.code_verifier
    }

    /// Returns a reference to the CSRF state.
    pub fn state(&self) -> &str {
        &self.state
    }

    /// Returns a reference to the PKCE code challenge.
    pub fn code_challenge(&self) -> &str {
        &self.code_challenge
    }

    /// Constructs the authorization URL with the appropriate query parameters.
    pub fn get_authorize_url(&self, redirect_uri: &str) -> String {
        let mut params = vec![
            ("response_type", "code".to_string()),
            ("client_id", self.client_id.clone()),
            ("redirect_uri", redirect_uri.to_string()),
            ("scope", self.scopes.join(" ")),
            ("state", self.state.clone()),
            ("code_challenge", self.code_challenge.clone()),
            ("code_challenge_method", "S256".to_string()),
        ];
        let query = params
            .iter()
            .map(|(k, v)| format!("{}={}", encode_url_owned(k), encode_url_owned(v)))
            .collect::<Vec<_>>()
            .join("&");
        format!("{}?{}", self.authorize_url, query)
    }

    /// Exchanges an authorization code for an access token using a generic HTTP client.
    #[instrument(skip(self, http_client), level = "debug")]
    pub async fn exchange_code<C: OAuthHttpClient>(
        &self,
        http_client: &C,
        code: &str,
        redirect_uri: &str,
    ) -> Result<Token, OAuthError> {
        // Build URL-encoded form body
        let mut form = vec![
            ("grant_type", "authorization_code".to_string()),
            ("code", code.to_string()),
            ("redirect_uri", redirect_uri.to_string()),
            ("client_id", self.client_id.clone()),
            ("code_verifier", self.code_verifier.clone()),
        ];
        if let Some(secret) = &self.client_secret {
            form.push(("client_secret", secret.clone()));
        }
        let body_bytes = form
            .iter()
            .map(|(k, v)| format!("{}={}", encode_url_owned(k), encode_url_owned(v)))
            .collect::<Vec<_>>()
            .join("&")
            .into_bytes();

        let request = HttpRequest {
            method: HttpMethod::POST,
            url: self.token_url.clone(),
            headers: vec![("Content-Type".to_string(), "application/x-www-form-urlencoded".to_string())],
            body: Some(body_bytes),
            timeout: None,
            redirect_policy: RedirectPolicy::None,
        };
        let response = http_client
            .execute(request)
            .await
            .map_err(|_| OAuthError::ServerError)?;
        if response.status != 200 {
            return Err(OAuthError::InvalidGrant);
        }
        // Manually parse JSON into Token
        let v: serde_json::Value = serde_json::from_slice(&response.body).map_err(|_| OAuthError::ServerError)?;
        let access_token = v.get("access_token").and_then(|t| t.as_str()).unwrap_or_default().to_string();
        let refresh_token = v.get("refresh_token").and_then(|t| t.as_str()).map(|s| s.to_string());
        let expires_in = v.get("expires_in").and_then(|t| t.as_u64()).unwrap_or(0);
        let scope = v.get("scope").and_then(|t| t.as_str()).map(|s| s.to_string());
        Ok(Token {
            model: TokenModel::BearerOpaque,
            access_token,
            refresh_token,
            expires_in,
            scope,
        })
    }
} 