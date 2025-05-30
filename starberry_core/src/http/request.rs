use crate::app::config::ParseConfig; 

use super::{http_value::*, net}; 
use super::body::HttpBody;
use super::meta::HttpMeta;
use super::start_line::{HttpStartLine}; 
use std::collections::HashMap;  
use tokio::io::{AsyncRead, AsyncWrite, BufReader, BufWriter}; 

/// Represents an HTTP request with metadata and body.
/// 
/// This struct contains all information about an incoming HTTP request, 
/// including headers, method, URL, and body content.
pub struct HttpRequest {
    pub meta: HttpMeta,
    pub body: HttpBody
}

impl HttpRequest { 
    pub fn new(meta: HttpMeta, body: HttpBody) -> Self { 
        HttpRequest { meta, body } 
    } 
    
    pub fn meta(&self) -> &HttpMeta { 
        &self.meta 
    } 

    /// Parses the HTTP request from a stream, returning an `HttpRequest` instance. 
    /// The stream is expected to be a `BufReader` wrapping a `TcpStream`. 
    /// Body will not be parsed 
    pub async fn parse_lazy<R: AsyncRead + Unpin>(stream: &mut BufReader<R>, config: &ParseConfig, print_raw: bool) -> Self {
        match net::parse_lazy(stream, config, true, print_raw).await { 
            Ok((meta, body)) => Self::new(meta, body), 
            Err(_) => Self::default() 
        }
    } 

    /// Parses the HTTP request body from a stream if the body has not been parsed yet. 
    pub async fn parse_body<R: AsyncRead + Unpin>(&mut self, reader: &mut BufReader<R>, max_size: usize) {
        // if let HttpBody::Unparsed = self.body {
        //     self.body = HttpBody::parse(
        //         reader,
        //         max_size,
        //         &mut self.meta,
        //     ).await;
        // }; 
        let _ = net::parse_body(&mut self.meta, &mut self.body, reader, max_size).await; 
    } 
    
    pub async fn send<W: AsyncWrite + Unpin>(&mut self, writer: &mut BufWriter<W>) -> std::io::Result<()> { 
        net::send(&mut self.meta, &mut self.body, writer).await 
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

/// Collection of helper functions to easily create common HTTP requests. 
pub mod request_templates {
    use std::collections::HashMap;

    use crate::http::{body::HttpBody, http_value::{HttpMethod, HttpVersion}, meta::HttpMeta, start_line::HttpStartLine};

    use super::HttpRequest;
 
    pub fn get_request<T: Into<String>>(url: T) -> HttpRequest { 
        let meta = HttpMeta::new(
            HttpStartLine::new_request(
                HttpVersion::Http11,
                HttpMethod::GET,
                url.into(), 
            ),
            HashMap::new(),
        );
        let body = HttpBody::Unparsed;
        HttpRequest::new(meta, body) 
    }
} 
