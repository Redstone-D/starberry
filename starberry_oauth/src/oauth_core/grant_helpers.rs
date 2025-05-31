use std::sync::Arc;
use super::oauth_client::OAuthClient;
use super::types::{Token, OAuthError, TokenModel};
use super::oauth_provider::TokenStorage;
use super::http_client::{OAuthHttpClient, HttpRequest, RedirectPolicy};
use starberry_lib::encode_url_owned;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use starberry_core::http::http_value::HttpMethod;
use serde_json::Value;
use tracing::instrument;

/// Authorization Code + PKCE flow helper.
pub struct AuthorizationCodePkceFlow<S: TokenStorage> {
    client: OAuthClient,
    storage: Arc<S>,
}

impl<S: TokenStorage> AuthorizationCodePkceFlow<S> {
    /// Constructs a new PKCE flow helper.
    pub fn new<Sec, U>(
        client_id: impl Into<String>,
        client_secret: Option<Sec>,
        authorize_url: impl Into<String>,
        token_url: impl Into<String>,
        scopes: impl IntoIterator<Item = String>,
        storage: Arc<S>,
    ) -> Self
    where
        Sec: Into<String>,
    {
        let client = OAuthClient::new(
            client_id,
            client_secret.map(|s| s.into()),
            authorize_url,
            token_url,
            scopes,
        );
        AuthorizationCodePkceFlow { client, storage }
    }

    /// Initiate the authorization code flow: store state & verifier and return auth URL.
    #[instrument(skip(self), level = "debug")]
    pub async fn initiate(&self, redirect_uri: &str, state_expires_in: u64) -> Result<String, OAuthError> {
        let url = self.client.get_authorize_url(redirect_uri);
        // Persist state and PKCE verifier
        self.storage.store_csrf_state(self.client.state(), state_expires_in).await?;
        self.storage.store_pkce_verifier(self.client.code_challenge(), self.client.code_verifier()).await?;
        Ok(url)
    }

    /// Exchange the code for a token: validate state, load verifier, call token endpoint.
    #[instrument(skip(self, http_client), level = "debug")]
    pub async fn exchange<C: OAuthHttpClient>(
        &self,
        http_client: &C,
        code: &str,
        redirect_uri: &str,
    ) -> Result<Token, OAuthError> {
        // Validate and clear state
        let valid = self.storage.get_csrf_state(self.client.state()).await?;
        if !valid {
            return Err(OAuthError::InvalidGrant);
        }
        self.storage.delete_csrf_state(self.client.state()).await?;
        // Retrieve and clear verifier
        let verifier_opt = self.storage.get_pkce_verifier(self.client.code_challenge()).await?;
        let verifier = verifier_opt.ok_or(OAuthError::InvalidGrant)?;
        self.storage.delete_pkce_verifier(self.client.code_challenge()).await?;
        // Perform exchange
        self.client.exchange_code(http_client, code, redirect_uri).await
    }
}

/// Client Credentials flow helper.
pub struct ClientCredentialsFlow {
    client_id: String,
    client_secret: String,
    token_url: String,
    scopes: Vec<String>,
}

impl ClientCredentialsFlow {
    /// Constructs a new client credentials helper.
    pub fn new<I, Sec, U>(
        client_id: I,
        client_secret: Sec,
        token_url: U,
        scopes: impl IntoIterator<Item = String>,
    ) -> Self
    where
        I: Into<String>,
        Sec: Into<String>,
        U: Into<String>,
    {
        ClientCredentialsFlow {
            client_id: client_id.into(),
            client_secret: client_secret.into(),
            token_url: token_url.into(),
            scopes: scopes.into_iter().collect(),
        }
    }

    /// Execute client credentials flow, returning an access token.
    #[instrument(skip(self, http_client), level = "debug")]
    pub async fn execute<C: OAuthHttpClient>(&self, http_client: &C) -> Result<Token, OAuthError> {
        // Build form
        let mut form = vec![
            ("grant_type", "client_credentials".to_string()),
        ];
        if !self.scopes.is_empty() {
            form.push(("scope", self.scopes.join(" ")));
        }
        let body = form
            .into_iter()
            .map(|(k, v)| format!("{}={}", encode_url_owned(k), encode_url_owned(v.as_str())))
            .collect::<Vec<_>>()
            .join("&")
            .into_bytes();
        // Build headers
        let mut headers = vec![("Content-Type".into(), "application/x-www-form-urlencoded".into())];
        let creds = format!("{}:{}", self.client_id, self.client_secret);
        let auth = format!("Basic {}", URL_SAFE_NO_PAD.encode(creds.as_bytes()));
        headers.push(("Authorization".into(), auth));
        // Build request
        let request = HttpRequest {
            method: HttpMethod::POST,
            url: self.token_url.clone(),
            headers,
            body: Some(body),
            timeout: None,
            redirect_policy: RedirectPolicy::None,
        };
        let resp = http_client.execute(request).await.map_err(|_| OAuthError::ServerError)?;
        if resp.status != 200 {
            return Err(OAuthError::InvalidGrant);
        }
        let v: Value = serde_json::from_slice(&resp.body).map_err(|_| OAuthError::ServerError)?;
        let access_token = v.get("access_token").and_then(|t| t.as_str()).unwrap_or_default().to_string();
        let refresh_token = v.get("refresh_token").and_then(|t| t.as_str()).map(|s| s.to_string());
        let expires_in = v.get("expires_in").and_then(|t| t.as_u64()).unwrap_or(0);
        let scope = v.get("scope").and_then(|t| t.as_str()).map(|s| s.to_string());
        Ok(Token { model: TokenModel::BearerOpaque, access_token, refresh_token, expires_in, scope })
    }
}

/// Refresh Token flow helper.
pub struct RefreshTokenFlow {
    client_id: String,
    client_secret: Option<String>,
    token_url: String,
}

impl RefreshTokenFlow {
    /// Constructs a new refresh token helper.
    pub fn new<I, Sec, U>(
        client_id: I,
        client_secret: Option<Sec>,
        token_url: U,
    ) -> Self
    where
        I: Into<String>,
        Sec: Into<String>,
        U: Into<String>,
    {
        RefreshTokenFlow {
            client_id: client_id.into(),
            client_secret: client_secret.map(|s| s.into()),
            token_url: token_url.into(),
        }
    }

    /// Execute refresh token flow, returning a new access token.
    #[instrument(skip(self, http_client), level = "debug")]
    pub async fn execute<C: OAuthHttpClient>(
        &self,
        http_client: &C,
        refresh_token: &str,
    ) -> Result<Token, OAuthError> {
        let mut form = vec![
            ("grant_type", "refresh_token".to_string()),
            ("refresh_token", refresh_token.to_string()),
            ("client_id", self.client_id.clone()),
        ];
        if let Some(sec) = &self.client_secret {
            form.push(("client_secret", sec.clone()));
        }
        let body = form
            .into_iter()
            .map(|(k, v)| format!("{}={}", encode_url_owned(k), encode_url_owned(v.as_str())))
            .collect::<Vec<_>>()
            .join("&")
            .into_bytes();
        let request = HttpRequest {
            method: HttpMethod::POST,
            url: self.token_url.clone(),
            headers: vec![("Content-Type".into(), "application/x-www-form-urlencoded".into())],
            body: Some(body),
            timeout: None,
            redirect_policy: RedirectPolicy::None,
        };
        let resp = http_client.execute(request).await.map_err(|_| OAuthError::ServerError)?;
        if resp.status != 200 {
            return Err(OAuthError::InvalidGrant);
        }
        let v: Value = serde_json::from_slice(&resp.body).map_err(|_| OAuthError::ServerError)?;
        let access_token = v.get("access_token").and_then(|t| t.as_str()).unwrap_or_default().to_string();
        let refresh_token = v.get("refresh_token").and_then(|t| t.as_str()).map(|s| s.to_string());
        let expires_in = v.get("expires_in").and_then(|t| t.as_u64()).unwrap_or(0);
        let scope = v.get("scope").and_then(|t| t.as_str()).map(|s| s.to_string());
        Ok(Token { model: TokenModel::BearerOpaque, access_token, refresh_token, expires_in, scope })
    }
} 