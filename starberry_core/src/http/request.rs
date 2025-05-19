use crate::connection::Connection;

use super::http_value::*; 
use super::body::HttpBody;
use super::meta::{HttpMeta, ParseConfig};
use super::start_line::{HttpStartLine}; 
use std::collections::HashMap;  
use tokio::io::{AsyncRead, BufReader}; 

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
        // Create one BufReader up-front, pass this throughout.
        let mut reader = BufReader::new(stream); 
        let meta = match HttpMeta::from_request_stream(
            &mut reader, 
            config, 
            print_raw, 
        ).await {
            Ok(meta) => meta,
            Err(e) => {
                println!("Error parsing request: {}", e); 
                return Self::default(); 
            }
        }; 

        let body = HttpBody::Unparsed; 

        Self::new(meta, body) 
    } 

    /// Parses the HTTP request body from a stream if the body has not been parsed yet. 
    pub async fn parse_body<R: AsyncRead + Unpin>(&mut self, reader: &mut BufReader<R>, max_size: usize) {
        if let HttpBody::Unparsed = self.body {
            self.body = HttpBody::parse(
                reader,
                max_size,
                &mut self.meta,
            ).await;
        }
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
