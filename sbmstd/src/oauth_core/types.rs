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
#[derive(Debug, Clone)]
pub enum JWTAlgorithm {
    HS256,
    RS256,
}

/// Supported access token models.
#[derive(Debug, Clone)]
pub enum TokenModel {
    /// Opaque bearer tokens.
    BearerOpaque,
    /// JSON Web Tokens with a signing algorithm.
    JWT { algorithm: JWTAlgorithm },
}

/// OAuth2 token representation.
#[derive(Debug, Clone)]
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
#[derive(Debug)]
pub enum OAuthError {
    /// The client authentication failed.
    InvalidClient,
    /// The provided grant is invalid or expired.
    InvalidGrant,
    /// The provided token is invalid or expired.
    InvalidToken,
    /// Client or user is not authorized to perform this request.
    Unauthorized,
    /// Generic server-side error.
    ServerError,
}

/// Context data for authenticated OAuth requests.
#[derive(Debug, Clone)]
pub struct OAuthContext {
    /// The client identifier or subject of the token.
    pub client_id: String,
    /// Granted scopes associated with the token.
    pub scopes: Vec<String>,
    /// The underlying token information.
    pub token: Token,
} 