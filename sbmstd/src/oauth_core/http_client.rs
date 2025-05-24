use std::future::Future;
use std::pin::Pin;
use std::time::Duration;
use std::error::Error;
use dashmap::DashMap;
use std::sync::Arc;

use starberry_core::http::http_value::HttpMethod;

/// HTTP redirect policy configuration.
#[derive(Debug, Clone)]
pub enum RedirectPolicy {
    /// Do not follow redirections.
    None,
    /// Follow up to `u32` redirections.
    Limit(u32),
}

/// HTTP request for executing a call.
#[derive(Debug, Clone)]
pub struct HttpRequest {
    /// HTTP method (GET, POST, etc.).
    pub method: HttpMethod,
    /// Target URL.
    pub url: String,
    /// Request headers.
    pub headers: Vec<(String, String)>,
    /// Optional request body.
    pub body: Option<Vec<u8>>,
    /// Optional timeout duration.
    pub timeout: Option<Duration>,
    /// Redirect policy to use for this request.
    pub redirect_policy: RedirectPolicy,
}

/// HTTP response from executing a call.
#[derive(Debug, Clone)]
pub struct HttpResponse {
    /// HTTP status code.
    pub status: u16,
    /// Response headers.
    pub headers: Vec<(String, String)>,
    /// Response body.
    pub body: Vec<u8>,
}

/// Error type for HTTP client operations.
pub type HttpClientError = Box<dyn Error + Send + Sync>;

/// Generic HTTP client interface for OAuth flows.
pub trait OAuthHttpClient: Send + Sync + Clone + 'static {
    /// Execute an HTTP request asynchronously.
    fn execute(
        &self,
        request: HttpRequest,
    ) -> Pin<Box<dyn Future<Output = Result<HttpResponse, HttpClientError>> + Send + 'static>>;
}

/// In-memory HTTP client stub for testing.
#[derive(Clone)]
pub struct InMemoryHttpClient {
    responses: Arc<DashMap<String, HttpResponse>>,
    default_response: Option<HttpResponse>,
}

impl InMemoryHttpClient {
    /// Creates a new in-memory HTTP client with no default response.
    pub fn new() -> Self {
        Self { responses: Arc::new(DashMap::new()), default_response: None }
    }

    /// Creates a new in-memory HTTP client with a default response on miss.
    pub fn with_default(response: HttpResponse) -> Self {
        Self { responses: Arc::new(DashMap::new()), default_response: Some(response) }
    }

    /// Register a mock response for a specific URL.
    pub fn insert_response(&self, url: impl Into<String>, response: HttpResponse) {
        self.responses.insert(url.into(), response);
    }
}

impl OAuthHttpClient for InMemoryHttpClient {
    fn execute(
        &self,
        request: HttpRequest,
    ) -> Pin<Box<dyn Future<Output = Result<HttpResponse, HttpClientError>> + Send + 'static>> {
        let responses = self.responses.clone();
        let default = self.default_response.clone();
        let url = request.url.clone();
        Box::pin(async move {
            if let Some(entry) = responses.get(&url) {
                Ok(entry.value().clone())
            } else if let Some(resp) = default.clone() {
                Ok(resp)
            } else {
                Err("no mock response for url".into())
            }
        })
    }
} 