use super::cookie::{Cookie, CookieMap};
use super::http_value::{self, *}; 
use super::body::HttpBody;
use super::meta::HttpMeta;
use super::start_line::{HttpStartLine}; 
use std::collections::HashMap; 

/// Represents an HTTP request with metadata and body.
/// 
/// This struct contains all information about an incoming HTTP request, 
/// including headers, method, URL, and body content.
pub struct HttpRequest {
    pub meta: HttpMeta,
    pub body: HttpBody
}

impl HttpRequest {
    /// Creates a new HttpRequest instance.
    ///
    /// # Arguments
    ///
    /// * `meta` - The metadata containing headers and request line
    /// * `body` - The request body
    ///
    /// # Returns
    ///
    /// A new HttpRequest instance
    ///
    /// # Examples
    ///
    /// ```rust
    /// use starberry_core::http::HttpRequest;
    /// use starberry_core::http::meta::HttpMeta;
    /// use starberry_core::http::body::HttpBody;
    /// use starberry_core::http::start_line::HttpStartLine;
    /// use starberry_core::http::http_value::{HttpMethod, HttpVersion};
    /// use std::collections::HashMap;
    ///
    /// let start_line = HttpStartLine::new_request(
    ///     HttpVersion::Http11,
    ///     HttpMethod::GET,
    ///     "/api/users".to_string()
    /// );
    /// let meta = HttpMeta::new(start_line, HashMap::new());
    /// let body = HttpBody::Empty;
    ///
    /// let request = HttpRequest::new(meta, body);
    /// ```
    pub fn new(meta: HttpMeta, body: HttpBody) -> Self {
        Self {
            meta,
            body
        }
    }
    
    /// Returns the HTTP method of this request.
    ///
    /// # Returns
    ///
    /// The HTTP method (GET, POST, etc.)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use starberry_core::http::HttpRequest;
    /// use starberry_core::http::http_value::HttpMethod;
    /// 
    /// // Assuming `request` is an instance of HttpRequest
    /// let method = request.method();
    /// if method == HttpMethod::POST {
    ///     // Handle POST request
    /// }
    /// ```
    pub fn method(&self) -> HttpMethod {
        self.meta.method()
    }
    
    /// Returns the path portion of the request URL.
    ///
    /// # Returns
    ///
    /// The path string
    ///
    /// # Examples
    ///
    /// ```rust
    /// use starberry_core::http::HttpRequest;
    /// 
    /// // Assuming `request` is an instance of HttpRequest
    /// let path = request.path();
    /// if path == "/api/users" {
    ///     // Handle users endpoint
    /// }
    /// ```
    pub fn path(&self) -> String {
        self.meta.path()
    }
    
    /// Returns a specific cookie from the request by name.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the cookie to retrieve
    ///
    /// # Returns
    ///
    /// The cookie if found, or None
    ///
    /// # Examples
    ///
    /// ```rust
    /// use starberry_core::http::HttpRequest;
    /// 
    /// // Assuming `request` is an instance of HttpRequest
    /// let session_id = request.get_cookie("session_id");
    /// if let Some(cookie) = session_id {
    ///     println!("Session ID: {}", cookie.value);
    /// }
    /// ```
    pub fn get_cookie<T: AsRef<str>>(&mut self, name: T) -> Option<Cookie> {
        self.meta.get_cookie(name)
    }
    
    /// Returns all cookies from the request.
    ///
    /// # Returns
    ///
    /// A reference to the CookieMap containing all cookies
    ///
    /// # Examples
    ///
    /// ```rust
    /// use starberry_core::http::HttpRequest;
    /// 
    /// // Assuming `request` is an instance of HttpRequest
    /// let cookies = request.cookies();
    /// for (name, cookie) in cookies.iter() {
    ///     println!("Cookie {}: {}", name, cookie.value);
    /// }
    /// ```
    pub fn cookies(&mut self) -> &CookieMap {
        self.meta.get_cookies()
    }
    
    /// Returns the content type of the request.
    ///
    /// # Returns
    ///
    /// The content type if specified, or None
    ///
    /// # Examples
    ///
    /// ```rust
    /// use starberry_core::http::HttpRequest;
    /// use starberry_core::http::http_value::HttpContentType;
    /// 
    /// // Assuming `request` is an instance of HttpRequest
    /// if let Some(content_type) = request.content_type() {
    ///     if content_type == HttpContentType::ApplicationJson {
    ///         // Handle JSON request
    ///     }
    /// }
    /// ```
    pub fn content_type(&mut self) -> Option<HttpContentType> {
        self.meta.get_content_type()
    }
    
    /// Returns the content length of the request body.
    ///
    /// # Returns
    ///
    /// The content length if specified, or None
    ///
    /// # Examples
    ///
    /// ```rust
    /// use starberry_core::http::HttpRequest;
    /// 
    /// // Assuming `request` is an instance of HttpRequest
    /// if let Some(length) = request.content_length() {
    ///     if length > 1_000_000 {
    ///         // Reject large requests
    ///     }
    /// }
    /// ```
    pub fn content_length(&mut self) -> Option<usize> {
        self.meta.get_content_length()
    } 
} 

impl Default for HttpRequest {
    fn default() -> Self {
        let meta = HttpMeta::new(
            HttpStartLine::new_request(
                HttpVersion::Http11,
                HttpMethod::GET,
                "/".to_string()
            ),
            HashMap::new()
        );
        let body = HttpBody::default();
        HttpRequest::new(meta, body)
    }
} 
