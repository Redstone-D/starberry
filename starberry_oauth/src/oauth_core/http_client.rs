use std::future::Future;
use std::pin::Pin;
use std::time::Duration;
use std::error::Error;
use dashmap::DashMap;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt, BufReader, BufWriter};
use starberry_core::connection::{ConnectionBuilder, Protocol, TxPool};
use starberry_core::http::context::HttpResCtx;
use starberry_core::http::body::HttpBody as CoreHttpBody;
use starberry_core::http::http_value::HttpMethod;
use starberry_core::http::request::HttpRequest as CoreHttpRequest;
use starberry_core::http::response::HttpResponse as CoreHttpResponse;
use starberry_core::http::meta::HttpMeta;
use starberry_core::http::start_line::HttpStartLine;
use starberry_core::http::http_value::HttpVersion;
use std::collections::HashMap;
use starberry_core::connection::Tx;
use starberry_core::http::safety::HttpSafety;

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

/// HTTP client using starberry_core ConnectionPool for raw HTTP and connection reuse
#[derive(Clone)]
pub struct CoreHttpClient {
    pools: Arc<DashMap<String, Arc<TxPool<HttpResCtx>>>>,
    max_body_size: usize,
    max_conns: usize,
}

impl CoreHttpClient {
    /// Create a new CoreHttpClient with max connections per host and max body bytes to read
    pub fn new(max_conns: usize, max_body_size: usize) -> Self {
        CoreHttpClient { pools: Arc::new(DashMap::new()), max_body_size, max_conns }
    }

    async fn get_pool(&self, host_port: &str) -> Arc<TxPool<HttpResCtx>> {
        if let Some(entry) = self.pools.get(host_port) {
            entry.clone()
        } else {
            let pool = Arc::new(TxPool::new());
            self.pools.insert(host_port.to_string(), pool.clone());
            pool
        }
    }
}

impl OAuthHttpClient for CoreHttpClient {
    fn execute(
        &self,
        request: HttpRequest,
    ) -> Pin<Box<dyn Future<Output = Result<HttpResponse, HttpClientError>> + Send + 'static>> {
        let pools = self.pools.clone();
        let max_body = self.max_body_size;
        Box::pin(async move {
            // Parse scheme and after-scheme portion
            let url = request.url.clone();
            let parts: Vec<&str> = url.split("://").collect();
            let (scheme, after) = if parts.len() >= 2 {
                (parts[0], parts[1])
            } else {
                ("http", parts[0])
            };
            let ssl = scheme.eq_ignore_ascii_case("https");
            let (host_port, path) = if let Some(idx) = after.find('/') {
                (&after[..idx], &after[idx..])
            } else {
                (after, "/")
            };
            // Get or insert the TxPool for this host_port
            let pool = if let Some(entry) = pools.get(host_port) {
                entry.clone()
            } else {
                let new_pool = Arc::new(TxPool::new());
                pools.insert(host_port.to_string(), new_pool.clone());
                new_pool
            };
            // Acquire or build a new HTTP context
            let mut ctx = if let Some(ctx) = pool.get().await {
                ctx
            } else {
                // Split host and optional port
                let mut hp = host_port.split(':');
                let host = hp.next().unwrap_or(host_port);
                let port: u16 = hp
                    .next()
                    .map_or_else(|| if ssl { 443 } else { 80 }, |p| p.parse().unwrap_or_else(|_| if ssl { 443 } else { 80 }));
                let conn = ConnectionBuilder::new(host, port)
                    .protocol(Protocol::HTTP)
                    .tls(ssl)
                    .connect()
                    .await
                    .map_err(|e| Box::new(e) as HttpClientError)?;
                HttpResCtx::new(conn, HttpSafety::new().with_max_body_size(max_body), host_port.to_string())
            };
            // Build core HttpRequest
            let mut meta = HttpMeta::new(
                HttpStartLine::new_request(
                    HttpVersion::Http11,
                    request.method.clone(),
                    path.to_string(),
                ),
                HashMap::new(),
            );
            meta.set_attribute("host", host_port.to_string());
            for (k, v) in &request.headers {
                meta.set_attribute(k.clone(), v.clone());
            }
            let body = if let Some(bytes) = &request.body {
                CoreHttpBody::Binary(bytes.clone())
            } else {
                CoreHttpBody::Unparsed
            };
            let core_req = CoreHttpRequest::new(meta, body);

            // Send the request and parse only headers in ctx.response
            ctx.request(core_req);
            ctx.send().await;
            // Take ownership of the HttpResponse out of the context
            let mut resp_to_parse = std::mem::take(&mut ctx.response);
            // Read the full body using the context's reader
            {
                let reader = &mut ctx.reader;
                resp_to_parse.parse_body(reader, &ctx.config).await;
            }
            // Return the context to the pool
            pool.release(ctx).await;
            // Build the OAuth HttpResponse from the parsed response
            let status = resp_to_parse.meta.start_line.status_code().as_u16();
            let headers = resp_to_parse
                .meta
                .get_header_hashmap()
                .iter()
                .map(|(k, v)| (k.clone(), v.as_str().to_string()))
                .collect();
            let body_bytes = match resp_to_parse.body {
                CoreHttpBody::Binary(b) => b,
                CoreHttpBody::Text(t) => t.into_bytes(),
                CoreHttpBody::Json(j) => j.into_json().as_bytes().to_vec(),
                _ => Vec::new(),
            };
            Ok(HttpResponse { status, headers, body: body_bytes })
        })
    }
} 