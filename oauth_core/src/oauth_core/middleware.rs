use std::{sync::Arc, any::Any, pin::Pin, future::Future};
use starberry_core::http::context::HttpReqCtx;
use starberry_core::app::middleware::AsyncMiddleware;
use super::oauth_provider::{ClientStore, TokenManager, Authorizer};
use super::memory::{InMemoryClientStore, InMemoryTokenManager, InMemoryAuthorizer};
use super::types::OAuthContext;
use starberry_core::http::http_value::StatusCode;
use starberry_core::http::response::request_templates::return_status;
use uuid::Uuid;
use std::collections::HashMap;
use starberry_core::http::http_value::HttpMethod;
use starberry_core::http::cookie::Cookie;
use starberry_core::http::response::request_templates::html_response;
use std::sync::{Mutex, OnceLock};
use std::time::{Instant, Duration};
use starberry_macro::middleware; 


/// OAuth2 middleware layer with configurable stores and endpoints.
#[derive(Clone)]
pub struct OAuthLayer {
    client_store: Arc<dyn ClientStore>,
    token_manager: Arc<dyn TokenManager>,
    authorizer: Arc<dyn Authorizer>,
    authorize_endpoint: String,
    token_endpoint: String,
}

impl OAuthLayer {
    /// Creates a new OAuthLayer with in-memory defaults and standard endpoints.
    pub fn new() -> Self {
        OAuthLayer {
            client_store: Arc::new(InMemoryClientStore::new(Vec::new())),
            token_manager: Arc::new(InMemoryTokenManager::new()),
            authorizer: Arc::new(InMemoryAuthorizer::new()),
            authorize_endpoint: "/oauth/authorize".into(),
            token_endpoint: "/oauth/token".into(),
        }
    }

    /// Sets a custom client store.
    pub fn client_store(mut self, store: Arc<dyn ClientStore>) -> Self {
        self.client_store = store;
        self
    }

    /// Sets a custom token manager.
    pub fn token_manager(mut self, manager: Arc<dyn TokenManager>) -> Self {
        self.token_manager = manager;
        self
    }

    /// Sets a custom authorizer.
    pub fn authorizer(mut self, authorizer: Arc<dyn Authorizer>) -> Self {
        self.authorizer = authorizer;
        self
    }

    /// Overrides the authorization endpoint path.
    pub fn authorize_endpoint<S: Into<String>>(mut self, path: S) -> Self {
        self.authorize_endpoint = path.into();
        self
    }

    /// Overrides the token endpoint path.
    pub fn token_endpoint<S: Into<String>>(mut self, path: S) -> Self {
        self.token_endpoint = path.into();
        self
    }

    /// Use JWT access tokens with HS256 signing.
    pub fn use_jwt_hs256(mut self, secret: &[u8], expiration_seconds: u64) -> Self {
        use super::jwt::JWTTokenManager;
        self.token_manager = Arc::new(JWTTokenManager::new_hs256(secret, expiration_seconds));
        self
    }

    /// Use JWT access tokens with RS256 signing.
    pub fn use_jwt_rs256(mut self, private_key_pem: &[u8], public_key_pem: &[u8], expiration_seconds: u64) -> Self {
        use super::jwt::JWTTokenManager;
        self.token_manager = Arc::new(JWTTokenManager::new_rs256(private_key_pem, public_key_pem, expiration_seconds));
        self
    }

    /// Use database-backed opaque tokens.
    pub fn use_db(mut self, pool: starberry_sql::sql::pool::SqlPool, expiration_seconds: u64) -> Self {
        use super::db::DBTokenManager;
        self.token_manager = Arc::new(DBTokenManager::new(pool, expiration_seconds));
        self
    }

    /// Use cookie-based opaque tokens backed by sessions.
    pub fn use_cookie(mut self, ttl_secs: u64) -> Self {
        use super::cookie::CookieTokenManager;
        self.token_manager = Arc::new(CookieTokenManager::new(ttl_secs));
        self
    }
}

// Simple token bucket for rate limiting
struct TokenBucket {
    capacity: u32,
    tokens: f64,
    last: Instant,
    refill_rate: f64, // tokens per second
}

impl TokenBucket {
    fn new(capacity: u32, refill_rate: f64) -> Self {
        Self { capacity, tokens: capacity as f64, last: Instant::now(), refill_rate }
    }
    fn consume(&mut self, amount: u32) -> bool {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last).as_secs_f64();
        self.tokens = (self.tokens + elapsed * self.refill_rate).min(self.capacity as f64);
        self.last = now;
        if self.tokens >= amount as f64 {
            self.tokens -= amount as f64;
            true
        } else {
            false
        }
    }
}

// Global rate limiter map: key -> TokenBucket
static RATE_LIMITERS: OnceLock<Mutex<HashMap<String, TokenBucket>>> = OnceLock::new();

impl AsyncMiddleware<HttpReqCtx> for OAuthLayer {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn return_self() -> Self {
        OAuthLayer::new()
    }

    fn handle<'a>(
        &'a self,
        mut req: HttpReqCtx,
        next: Box<dyn Fn(HttpReqCtx) -> Pin<Box<dyn Future<Output = HttpReqCtx> + Send>> + Send + Sync + 'static>,
    ) -> Pin<Box<dyn Future<Output = HttpReqCtx> + Send + 'static>> {
        let authorize_path = self.authorize_endpoint.clone();
        let token_path = self.token_endpoint.clone();
        let client_store = self.client_store.clone();
        let token_manager = self.token_manager.clone();
        let authorizer = self.authorizer.clone();

        Box::pin(async move {
            let full_path = req.path();
            let (path_only, query_string) = if let Some((p, q)) = full_path.split_once('?') { (p, q) } else { (full_path.as_str(), "") };
            if path_only == authorize_path {
                let method = req.meta().method();
                if method == HttpMethod::GET {
                    // Parse query parameters
                    let mut params: HashMap<String, String> = HashMap::new();
                    for pair in query_string.split('&') {
                        let mut iter = pair.splitn(2, '=');
                        if let (Some(k), Some(v)) = (iter.next(), iter.next()) {
                            params.insert(k.to_string(), v.to_string());
                        }
                    }
                    let client_id = params.get("client_id").cloned().unwrap_or_default();
                    // Rate limit GET authorize per client_id
                    {
                        let map_mutex = RATE_LIMITERS.get_or_init(|| Mutex::new(HashMap::new()));
                        let mut map = map_mutex.lock().unwrap();
                        let bucket = map.entry(client_id.clone()).or_insert_with(|| TokenBucket::new(10, 10.0 / 60.0));
                        if !bucket.consume(1) {
                            req.response = return_status(StatusCode::TOO_MANY_REQUESTS);
                            return req;
                        }
                    }
                    let redirect_uri = params.get("redirect_uri").cloned().unwrap_or_default();
                    let response_type = params.get("response_type").cloned().unwrap_or_default();
                    let scope = params.get("scope").cloned().unwrap_or_default();
                    let state = params.get("state").cloned().unwrap_or_default();
                    // Enforce PKCE for public clients: fetch client and require code_challenge
                    let client_res = client_store.get_client(&client_id).await;
                    let client = if let Ok(c) = client_res { c } else {
                        req.response = return_status(StatusCode::BAD_REQUEST);
                        return req;
                    };
                    let is_public = client.secret.is_none();
                    let code_challenge_opt = params.get("code_challenge").cloned();
                    let code_challenge_method = params.get("code_challenge_method").cloned().unwrap_or_else(|| "plain".to_string());
                    if is_public && code_challenge_opt.is_none() {
                        req.response = return_status(StatusCode::BAD_REQUEST);
                        return req;
                    }
                    let code_challenge = code_challenge_opt.unwrap_or_default();

                    // Generate CSRF token and render consent form
                    let csrf_token = Uuid::new_v4().to_string();
                    let html = format!(r#"<!DOCTYPE html>
<html><body>
<h1>Authorize access for client {}</h1>
<form method=\"POST\" action=\"{}\"> 
    <input type=\"hidden\" name=\"csrf_token\" value=\"{}\" />
    <input type=\"hidden\" name=\"client_id\" value=\"{}\" />
    <input type=\"hidden\" name=\"redirect_uri\" value=\"{}\" />
    <input type=\"hidden\" name=\"response_type\" value=\"{}\" />
    <input type=\"hidden\" name=\"scope\" value=\"{}\" />
    <input type=\"hidden\" name=\"state\" value=\"{}\" />
    <input type=\"hidden\" name=\"code_challenge\" value=\"{}\" />
    <input type=\"hidden\" name=\"code_challenge_method\" value=\"{}\" />
    <p>Requested scopes: {}</p>
    <button type=\"submit\" name=\"action\" value=\"approve\">Approve</button>
    <button type=\"submit\" name=\"action\" value=\"deny\">Deny</button>
</form>
</body></html>"#,
                        client_id, authorize_path, csrf_token, client_id,
                        redirect_uri, response_type, scope, state,
                        code_challenge, code_challenge_method, scope
                    );

                    req.response = html_response(html)
                        .add_cookie("csrf_token", Cookie::new(csrf_token).path("/").secure(true).http_only(true));
                    return req;
                } else if method == HttpMethod::POST {
                    // Parse and validate CSRF and extract PKCE data without overlapping borrows
                    let (client_id_post, code_challenge_post) = {
                        // Extract CSRF token from cookie
                        let cookie_token = req.get_cookie("csrf_token").map(|c| c.value.clone()).unwrap_or_default();
                        // Parse form data
                        let f = req.form_or_default().await;
                        let submitted = f.get("csrf_token").cloned().unwrap_or_default();
                        if submitted != cookie_token {
                            req.response = return_status(StatusCode::BAD_REQUEST);
                            return req;
                        }
                        let client_id_val = f.get("client_id").cloned().unwrap_or_default();
                        let code_challenge_val = f.get("code_challenge").cloned().unwrap_or_default();
                        (client_id_val, code_challenge_val)
                    };
                    // Enforce PKCE for public clients
                    let client_res_post = client_store.get_client(&client_id_post).await;
                    let client_post = if let Ok(c) = client_res_post { c } else {
                        req.response = return_status(StatusCode::BAD_REQUEST);
                        return req;
                    };
                    if client_post.secret.is_none() && code_challenge_post.is_empty() {
                        req.response = return_status(StatusCode::BAD_REQUEST);
                        return req;
                    }
                    // CSRF and PKCE validated; proceed to consent processing
                } else {
                    req.response = return_status(StatusCode::METHOD_NOT_ALLOWED);
                    return req;
                }
            } else if path_only == token_path {
                // Obtain client IP from X-Forwarded-For header (populated by Nginx real_ip module)
                let ip = if let Some(hv) = req.meta().header.get("x-forwarded-for") {
                    hv.as_str().split(',').next().unwrap_or("").trim().to_string()
                } else {
                    "unknown".to_string()
                };
                // Rate limit token endpoint per client IP
                {
                    let map_mutex = RATE_LIMITERS.get_or_init(|| Mutex::new(HashMap::new()));
                    let mut map = map_mutex.lock().unwrap();
                    let bucket = map.entry(ip).or_insert_with(|| TokenBucket::new(10, 10.0 / 60.0));
                    if !bucket.consume(1) {
                        req.response = return_status(StatusCode::TOO_MANY_REQUESTS);
                        return req;
                    }
                }
                // TODO: implement token endpoint logic
            } else {
                // Protected: validate Bearer/JWT token and inject OAuthContext
                let token_opt = req.meta().header.get("authorization")
                    .map(|hv| hv.as_str())
                    .and_then(|s| s.strip_prefix("Bearer ").map(str::to_string));
                let token_str = if let Some(t) = token_opt {
                    t
                } else {
                    req.response = return_status(StatusCode::UNAUTHORIZED);
                    return req;
                };
                let token_res = token_manager.validate_token(&token_str).await;
                let token = if let Ok(tok) = token_res { tok } else {
                    req.response = return_status(StatusCode::UNAUTHORIZED);
                    return req;
                };
                let scopes = token.scope.clone()
                    .map(|s| s.split(' ').map(str::to_string).collect())
                    .unwrap_or_default();
                let client_id = token.access_token.clone();
                let oauth_ctx = OAuthContext { client_id, scopes, token: token.clone() };
                req.params.set(oauth_ctx);
            }
            next(req).await
        })
    }
}