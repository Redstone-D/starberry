//! OAuth2 core primitives: Client, Grant, Token and errors.

/// Represents an OAuth 2.0 client application.
#[derive(Debug, Clone)]
pub struct Client {
    /// Client identifier.
    pub id: String,
    /// Optional client secret.
    pub secret: Option<String>,
    /// Allowed redirect URIs.
    pub redirect_uris: Vec<String>,
}

/// OAuth2 grant types.
#[derive(Debug, Clone)]
pub enum Grant {
    /// Authorization code grant with optional PKCE verifier.
    AuthorizationCode {
        /// The authorization code received from the authorization server.
        code: String,
        /// PKCE code verifier, if used for enhanced security.
        code_verifier: Option<String>,
    },
    /// Client credentials grant.
    ClientCredentials,
    /// Refresh token grant (with rotation).
    RefreshToken {
        /// The refresh token being exchanged for a new access token.
        token: String,
    },
    /// Resource Owner Password Credentials grant (optional).
    ResourceOwnerPassword {
        /// The resource owner's username.
        username: String,
        /// The resource owner's password.
        password: String,
    },
    /// Device Code grant (optional).
    DeviceCode {
        /// The device code issued by the authorization server.
        device_code: String,
        /// The user code if provided by the server.
        user_code: Option<String>,
    },
}

/// Core JWT algorithm options.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum JWTAlgorithm {
    HS256,
    RS256,
}

/// Supported access token models.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum TokenModel {
    /// Opaque bearer tokens.
    BearerOpaque,
    /// JSON Web Tokens with a signing algorithm.
    JWT { algorithm: JWTAlgorithm },
}

/// OAuth2 token representation.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Token {
    /// The model of the access token.
    pub model: TokenModel,
    /// Access token string or serialized JWT.
    pub access_token: String,
    /// Optional refresh token (for rotation).
    pub refresh_token: Option<String>,
    /// Lifetime in seconds.
    pub expires_in: u64,
    /// Optional granted scopes.
    pub scope: Option<String>,
}

/// Core OAuth2 error kinds.
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub enum OAuthError {
    /// The client authentication failed.
    InvalidClient,
    /// The provided grant is invalid.
    InvalidGrant,
    /// The provided token is invalid.
    InvalidToken,
    /// The token has expired.
    TokenExpired,
    /// The request requires more scopes.
    InsufficientScopes,
    /// CSRF token mismatch detected.
    CsrfMismatch,
    /// Request was rate-limited.
    RateLimited,
    /// Underlying HTTP error, with description.
    HttpError(String),
    /// Client is not authorized to access this resource.
    Unauthorized,
    /// Generic server-side error.
    ServerError,
}

/// Context data for authenticated OAuth requests.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct OAuthContext {
    /// The client identifier or subject of the token.
    pub client_id: String,
    /// Granted scopes associated with the token.
    pub scopes: Vec<String>,
    /// The underlying token information.
    pub token: Token,
}

// Robust error-to-response mapping
use starberry_core::http::start_line::HttpStartLine;
use starberry_core::http::http_value::{HttpVersion, StatusCode};
use starberry_core::http::response::HttpResponse;
use starberry_core::http::response::request_templates::normal_response;
use starberry_core::http::http_value::HttpContentType;
use serde_json::{json, to_vec};
use tracing::warn;

impl OAuthError {
    /// Convert this OAuth error into an HTTP JSON response with proper status.
    pub fn into_response(&self) -> HttpResponse {
        // Map to status, error code, and description
        let (status, code, description) = match self {
            OAuthError::InvalidClient => (StatusCode::UNAUTHORIZED, "invalid_client", "Client authentication failed"),
            OAuthError::InvalidGrant => (StatusCode::BAD_REQUEST, "invalid_grant", "Invalid grant provided"),
            OAuthError::InvalidToken => (StatusCode::UNAUTHORIZED, "invalid_token", "The token is invalid"),
            OAuthError::TokenExpired => (StatusCode::UNAUTHORIZED, "invalid_token", "The token has expired"),
            OAuthError::InsufficientScopes => (StatusCode::FORBIDDEN, "insufficient_scope", "Insufficient scopes for this request"),
            OAuthError::CsrfMismatch => (StatusCode::BAD_REQUEST, "invalid_request", "CSRF token mismatch"),
            OAuthError::RateLimited => (StatusCode::TOO_MANY_REQUESTS, "rate_limited", "Too many requests"),
            OAuthError::HttpError(err) => (StatusCode::BAD_GATEWAY, "http_error", err.as_str()),
            OAuthError::Unauthorized => (StatusCode::FORBIDDEN, "unauthorized_client", "Client not authorized"),
            OAuthError::ServerError => (StatusCode::INTERNAL_SERVER_ERROR, "server_error", "Internal server error"),
        };
        // Structured log
        warn!(error = ?self, error_code = code, http_status = %status, "OAuth error occurred");
        // Build JSON body bytes
        let json_val = json!({ "error": code, "error_description": description });
        let body_bytes = to_vec(&json_val).unwrap_or_default();
        // Build response with JSON body and proper status
        let mut resp = normal_response(status.clone(), body_bytes);
        resp.meta.set_content_type(HttpContentType::ApplicationJson());
        resp.meta.start_line = HttpStartLine::new_response(HttpVersion::Http11, status);
        resp
    }
} 