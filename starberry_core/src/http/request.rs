use crate::app::urls::Url;
use starberry_lib::decode_url_owned;

use super::http_value::*;
use akari::Object;
use once_cell::sync::Lazy;
use regex::Regex;
use std::collections::HashMap;
use std::error::Error;
use std::hash::Hash;
use tokio::io::{AsyncBufReadExt, AsyncRead, AsyncReadExt, BufReader}; 
use tokio::net::TcpStream;
use std::str;

/// RequestHeader is a struct that represents the headers of an HTTP request. 
#[derive(Debug)]
pub struct HttpMeta { 
    pub start_line: RequestStartLine, 
    pub header: HashMap<String, String>,
    content_type: Option<HttpContentType>,
    content_length: Option<usize>,
    cookies: Option<HashMap<String, String>>,
} 

/// RequestStartLine is the first line of the HTTP request, which contains the method, path, and HTTP version.
#[derive(Debug, Clone)]
pub struct RequestStartLine {
    pub http_version: HttpVersion,
    pub method: HttpMethod,
    pub path: String,
    pub url: Option<RequestPath>,
} 

pub struct ParseConfig {
    pub max_header_size: usize,
    pub max_line_length: usize,
    pub max_headers: usize,
    pub max_body_size: usize,
} 

impl ParseConfig {
    pub fn new ( 
        max_header_size: usize,
        max_line_length: usize,
        max_headers: usize,
        max_body_size: usize,
    ) -> Self {
        Self {
            max_header_size,
            max_body_size,
            max_line_length,
            max_headers,
        }
    }

    pub fn set_max_header_size(&mut self, size: usize) {
        self.max_header_size = size;
    }

    pub fn set_max_body_size(&mut self, size: usize) {
        self.max_body_size = size; 
    }

    pub fn set_max_line_length(&mut self, size: usize) {
        self.max_line_length = size;
    }

    pub fn set_max_headers(&mut self, size: usize) {
        self.max_headers = size;
    }

    pub fn get_max_header_size(&self) -> usize {
        self.max_header_size
    }

    pub fn get_max_body_size(&self) -> usize {
        self.max_body_size
    }

    pub fn get_max_line_length(&self) -> usize {
        self.max_line_length
    }

    pub fn get_max_headers(&self) -> usize {
        self.max_headers
    }

    pub fn default() -> Self {
        Self {
            max_header_size: 8192,
            max_body_size: 1028 * 1028,
            max_line_length: 8192,
            max_headers: 100,
        }
    }
}

impl RequestStartLine {
    /// It is used to parse the request line and create a new RequestStartLine object.
    pub fn new(http_version: HttpVersion, method: HttpMethod, path: String) -> Self {
        Self {
            http_version,
            method,
            path,
            url: None,
        }
    }

    /// It is used to convert the RequestStartLine object to a string.
    pub fn represent(&self) -> String {
        format!(
            "{} {} {}",
            self.http_version.to_string(),
            self.method.to_string(),
            self.path
        )
    }

    /// It is used to parse the request line and create a new RequestStartLine object.
    /// It takes a string as input and splits it into parts.
    /// If the number of parts is less than 3, it returns an error.
    /// # Arguments
    /// * `line` - A string slice that contains the request line.
    /// # Returns
    /// * `Result<Self, String>` - It returns a Result object.
    /// * If the parsing is successful, it returns Ok with a RequestStartLine object.
    /// * If the parsing fails, it returns Err with an error message.
    /// # Example
    /// ```rust
    /// use starberry_core::http::request::RequestStartLine;
    /// let request_line = "GET /index.html HTTP/1.1";
    /// let start_line = RequestStartLine::parse(request_line).unwrap();
    /// println!("{}", start_line);
    /// ```
    /// # Errors
    /// * It returns an error if the request line is malformed.
    /// * It returns an error if the number of parts is less than 3.
    pub fn parse<T: AsRef<str>>(line: T) -> Result<Self, String> {
        let line = line.as_ref();
        let parts: Vec<&str> = line.split_whitespace().collect();

        if parts.len() != 3 {
            return Err("Malformed request line".into());
        }

        let method = HttpMethod::from_string(parts[0]);
        let path = parts[1].to_string();
        let http_version = HttpVersion::from_string(parts[2]);

        Ok(Self::new(http_version, method, path))
    }

    pub fn get_url(&mut self) -> RequestPath {
        match &self.url {
            Some(url) => return url.clone(),
            None => self.parse_url(),
        }
    }

    pub fn parse_url(&mut self) -> RequestPath {
        let url = RequestPath::from_string(&self.path);
        self.url = Some(url.clone());
        url
    }

    pub fn set_url(&mut self, url: RequestPath) {
        self.url = Some(url);
    }

    pub fn clear_url(&mut self) {
        self.url = None;
    }
}

impl std::fmt::Display for RequestStartLine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {} {}", self.http_version, self.method, self.path)
    }
} 

impl HttpMeta { 
    /// It is used to create a new RequestHeader object.
    pub fn new(
        start_line: RequestStartLine, 
        headers: HashMap<String, String> 
    ) -> Self {
        Self { 
            start_line, 
            header: headers,
            content_type: None,
            content_length: None,
            cookies: None, 
        }
    } 

    pub async fn from_request_stream<R: AsyncRead + Unpin>(
        buf_reader: &mut BufReader<R>,
        config: &ParseConfig, 
        print_raw: bool, 
    ) -> Result<HttpMeta, Box<dyn Error + Send + Sync>> {
        let mut headers = Vec::new();
        let mut total_header_size = 0;
        
        // Try to fill the buffer with a single read first
        buf_reader.fill_buf().await?;
        
        // Fast path: Check if we got all headers in one go
        let buffer = buf_reader.buffer();
        if let Some((header_lines, headers_end)) = Self::extract_headers_from_buffer(buffer, config) {
            // We found the complete headers in the buffer
            if print_raw {
                println!("Fast path: got all headers in single read");
            }
            
            // Process headers from buffer
            for line in header_lines {
                if line.len() > config.max_line_length {
                    return Err("Header line too long".into());
                }
                
                total_header_size += line.len() + 2; // +2 for CRLF
                if total_header_size > config.max_header_size {
                    return Err("Headers too large".into());
                }
                
                if headers.len() >= config.max_headers {
                    return Err("Too many headers".into());
                }
                
                // Strip CRLF injection and store
                let safe_line = line.replace("\r", "");
                headers.push(safe_line);
            }
            
            // Consume the processed data from the buffer
            buf_reader.consume(headers_end);
        } else {
            // Slow path: read headers line by line as before
            if print_raw {
                println!("Slow path: reading headers line by line");
            }
            
            loop {
                let mut line = String::new();
                let bytes_read = buf_reader.read_line(&mut line).await?;
                if print_raw {
                    println!("Read line: {}, buffer: {}", line, bytes_read);
                }
                
                if bytes_read == 0 || line.trim_end().is_empty() {
                    break; // End of headers
                }
                
                // Reject requests with an extremely long header line
                if line.len() > config.max_line_length {
                    return Err("Header line too long".into());
                }
                
                total_header_size += line.len();
                
                // Enforce max header size limit
                if total_header_size > config.max_header_size {
                    return Err("Headers too large".into());
                }
                
                // Enforce max number of headers
                if headers.len() >= config.max_headers {
                    return Err("Too many headers".into());
                }
                
                // Strip CRLF injection and store the header
                let safe_line = line.trim_end().replace("\r", "");
                headers.push(safe_line);
            } 
        }
        
        if headers.is_empty() {
            return Err("Empty request".into());
        }
        
        let start_line = RequestStartLine::parse(headers.remove(0))?;
        let header = Self::parse(headers);
        
        if print_raw {
            println!("Parsed headers: {:?}", header);
            println!("Parsed start line: {:?}", start_line);
        }
        
        Ok(HttpMeta::new(start_line, header))
    } 

    /// Helper function to extract complete headers from a buffer if possible
    fn extract_headers_from_buffer<'a>(buffer: &'a [u8], config: &ParseConfig) -> Option<(Vec<&'a str>, usize)> {
        // Look for the end of headers marker (double CRLF)
        let mut i = 0;
        while i + 3 < buffer.len() {
            if buffer[i] == b'\r' && buffer[i+1] == b'\n' && 
            buffer[i+2] == b'\r' && buffer[i+3] == b'\n' {
                
                // Found end of headers
                let headers_section = std::str::from_utf8(&buffer[..i+2]).ok()?;
                
                // Split into lines
                let lines: Vec<&str> = headers_section
                    .split("\r\n")
                    .filter(|s| !s.is_empty())
                    .collect();
                    
                if lines.len() > config.max_headers {
                    return None; // Too many headers, fall back to slow path
                }
                
                return Some((lines, i + 4)); // +4 to include the final \r\n\r\n
            }
            i += 1;
        }
        
        None // Didn't find complete headers
    }   

    /// It is used to add a new header to the RequestHeader object.
    /// Taking Vector of String as input.
    /// # Arguments
    /// * `headers` - A vector of strings that contains the headers.
    /// # Returns
    /// * `Self` - It returns a RequestHeader object.
    /// # Example
    /// ```rust
    /// use starberry_core::http::request::RequestHeader;
    /// let headers = vec![
    ///    "Content-Type: text/html".to_string(),
    ///    "Content-Length: 123".to_string(),
    /// ];
    /// let request_header = RequestHeader::parse(headers);
    /// println!("{:?}", request_header);
    /// ```
    pub fn parse(headers: Vec<String>) -> HashMap<String, String> {
        let mut header_map = HashMap::new();
        for line in headers {
            let parts: Vec<&str> = line.splitn(2, ':').collect();
            if parts.len() == 2 {
                let key = parts[0].trim().to_uppercase().to_string();
                let value = parts[1].trim().to_string();
                header_map.insert(key, value);
            }
        } 
        header_map 
    } 

    pub fn set_header_hashmap(&mut self, header: HashMap<String, String>) {
        self.header = header;
    } 

    pub fn get_path(&mut self, part: usize) -> String {
        self.start_line.get_url().url_part(part)
    }

    pub fn path(&self) -> &str {
        &self.start_line.path
    } 

    pub fn method(&self) -> &HttpMethod {
        &self.start_line.method
    } 

    /// It is used to get the value of a header from the RequestHeader object.
    /// Taking a string as input.
    /// # Example
    /// ```rust
    /// use starberry_core::http::request::RequestHeader;
    /// let headers = vec![
    ///    "Content-Type: text/html".to_string(),
    ///    "Content-Length: 123".to_string(),
    /// ];
    /// let mut request_header = RequestHeader::parse(headers);
    /// let content_type = request_header.get_content_length().unwrap();
    /// println!("{:?}", content_type);
    /// ```
    pub fn get_content_length(&mut self) -> Option<usize> {
        match self.header.get("CONTENT-LENGTH") {
            Some(value) => return value.parse::<usize>().ok(),
            None => return self.parse_content_length(),
        }
    }

    /// It is used to parse the value of a header from the RequestHeader object
    /// from the hashmap into attribute.
    /// # Example
    /// ```rust
    /// use starberry_core::http::request::RequestHeader;
    /// let headers = vec![
    ///    "Content-Type: text/html".to_string(),
    ///    "Content-Length: 123".to_string(),
    /// ];
    /// let mut request_header = RequestHeader::parse(headers);
    /// request_header.parse_content_length();
    /// assert_eq!(request_header.get_content_length(), Some(123));
    /// ```
    pub fn parse_content_length(&mut self) -> Option<usize> {
        let length = self
            .header
            .get("CONTENT-LENGTH")
            .and_then(|s| s.parse::<usize>().ok())
            .unwrap_or(0);
        self.set_content_length(length);
        Some(length)
    }

    pub fn set_content_length(&mut self, length: usize) {
        self.content_length = Some(length);
    }

    pub fn clear_content_length(&mut self) {
        self.content_length = None;
    }

    /// It is used to get the value of a header from the RequestHeader object.
    /// Taking a string as input.
    /// # Example
    /// ```rust
    /// use starberry_core::http::request::RequestHeader;
    /// use starberry_core::http::http_value::HttpContentType;
    /// let headers = vec![
    ///   "Content-Type: text/html".to_string(),
    ///   "Content-Length: 123".to_string(),
    /// ];
    /// let mut request_header = RequestHeader::parse(headers);
    /// request_header.get_content_type();
    /// assert_eq!(request_header.get_content_type(), Some(HttpContentType::TextHtml));
    /// ```
    pub fn get_content_type(&mut self) -> Option<HttpContentType> {
        match self.content_type {
            Some(ref content_type) => return Some(content_type.clone()),
            None => return self.parse_content_type(),
        }
    }

    /// It is used to parse the value of a header from the RequestHeader object
    /// from the hashmap into attribute.
    /// # Example
    /// ```rust
    /// use starberry_core::http::request::RequestHeader;
    /// use starberry_core::http::http_value::HttpContentType;
    /// let headers = vec![
    ///    "Content-Type: text/html".to_string(),
    ///   "Content-Length: 123".to_string(),
    /// ];
    /// let mut request_header = RequestHeader::parse(headers);
    /// request_header.parse_content_type();
    /// assert_eq!(request_header.get_content_type(), Some(HttpContentType::TextHtml));
    /// ```
    pub fn parse_content_type(&mut self) -> Option<HttpContentType> {
        let content_type =
            HttpContentType::from_str(self.header.get("CONTENT-TYPE").unwrap_or(&"".to_owned()));
        self.set_content_type(content_type.clone());
        Some(content_type)
    }

    pub fn set_content_type(&mut self, content_type: HttpContentType) {
        self.content_type = Some(content_type);
    }

    pub fn clear_content_type(&mut self) {
        self.content_type = None;
    } 

    /// It is used to get the value of a header from the RequestHeader object.
    /// # Example
    /// ```rust
    /// use starberry_core::http::request::RequestHeader;
    /// use starberry_core::http::http_value::HttpContentType;
    /// use std::collections::HashMap;
    /// let headers = vec![
    ///   "Content-Type: text/html".to_string(),
    ///   "Content-Length: 123".to_string(),
    ///   "Cookie: sessionId=abc123; theme=dark; loggedIn=true".to_string(),
    /// ];
    /// let mut request_header = RequestHeader::parse(headers);
    /// assert_eq!(request_header.get_cookies(), &HashMap::from([("sessionId".to_string(), "abc123".to_string()), ("theme".to_string(), "dark".to_string()), ("loggedIn".to_string(), "true".to_string())]));
    pub fn get_cookies(&mut self) -> &HashMap<String, String> {
        if self.cookies.is_none() {
            self.cookies = Some(self.parse_cookies());
        }
        self.cookies.as_ref().unwrap()
    }

    pub fn get_cookie(&mut self, key: &str) -> Option<String> {
        if self.cookies.is_none() {
            self.cookies = Some(self.parse_cookies());
        }
        return self.cookies.as_ref().unwrap().get(key).cloned();
    } 

    pub fn get_cookie_or_default(&mut self, key: &str) -> String {
        self.get_cookie(key).unwrap_or("".to_string())
    } 

    /// It is used to get the value of a header from the RequestHeader object.
    /// Taking a string as input.
    /// # Example
    /// ```rust
    /// use starberry_core::http::request::RequestHeader;
    /// use starberry_core::http::http_value::HttpContentType;
    /// use std::collections::HashMap;
    /// let headers = vec![
    ///    "Content-Type: text/html".to_string(),
    ///   "Content-Length: 123".to_string(),
    ///    "Cookie: sessionId=abc123; theme=dark; loggedIn=true".to_string(),
    /// ];
    /// let mut request_header = RequestHeader::parse(headers);
    /// request_header.parse_cookies();
    /// assert_eq!(request_header.get_cookies(), HashMap::from([("sessionId".to_string(), "abc123".to_string()), ("theme".to_string(), "dark".to_string()), ("loggedIn".to_string(), "true".to_string())]));
    pub fn parse_cookies(&mut self) -> HashMap<String, String> {
        let cookies = match self.header.get("COOKIE") {
            Some(cookies) => cookies,
            None => return HashMap::new(),
        };
        let mut cookie_map = HashMap::new();
        for cookie in cookies.split(';') {
            let parts: Vec<&str> = cookie.split('=').collect();
            if parts.len() == 2 {
                cookie_map.insert(parts[0].trim().to_string(), parts[1].trim().to_string());
            }
        }
        self.cookies = Some(cookie_map.clone());
        cookie_map
    }

    pub fn set_cookies(&mut self, cookies: HashMap<String, String>) {
        self.cookies = Some(cookies);
    }

    pub fn clear_cookies(&mut self) {
        self.cookies = None;
    } 
} 

impl Default for HttpMeta { 
    fn default() -> Self {
        Self { 
            start_line: RequestStartLine::new(
                HttpVersion::Http11,
                HttpMethod::GET,
                "/".to_string(),
            ), 
            header: HashMap::new(),
            content_type: None,
            content_length: None,
            cookies: None, 
        }
    } 
}
