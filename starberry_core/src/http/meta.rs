use crate::app::urls::Url;
use starberry_lib::decode_url_owned; 
use super::cookie::{Cookie, CookieMap}; 

use super::http_value::*; 
use super::start_line::HttpStartLine; 
use akari::Object;
use once_cell::sync::Lazy;
use regex::Regex;
use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::hash::Hash;
use tokio::io::{AsyncBufReadExt, AsyncRead, AsyncReadExt, BufReader}; 
use tokio::net::TcpStream;
use std::str; 

/// RequestHeader is a struct that represents the headers of an HTTP request. 
#[derive(Debug)]
pub struct HttpMeta { 
    pub start_line: HttpStartLine, 
    pub header: HashMap<String, HeaderValue>, 
    content_type: Option<HttpContentType>,
    content_length: Option<usize>,
    cookies: Option<CookieMap>, 
    location: Option<String>, 
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

/// Represents a value for an HTTP header, which can be either a single string or multiple values.
/// 
/// HTTP headers can sometimes have multiple values, which are typically combined with commas,
/// but some special headers like Set-Cookie maintain separate values.
#[derive(Debug, Clone)]
pub enum HeaderValue {
    /// A single header value
    Single(String),
    /// Multiple header values
    Multiple(Vec<String>),
}

impl HeaderValue { 
    /// Create a new HeaderValue from a single string.
    /// 
    /// # Arguments
    /// 
    /// * `value` - A string that represents the header value.
    /// 
    /// # Returns
    /// 
    /// A new HeaderValue containing a single value.
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// use starberry_core::http::meta::HeaderValue;
    /// let header = HeaderValue::new("application/json");
    /// ```
    pub fn new<T: Into<String>>(value: T) -> Self {
        HeaderValue::Single(value.into())
    }

    /// Append a new value to the HeaderValue.
    /// 
    /// If the HeaderValue is a single value, it will convert it to a multiple value.
    /// Values are typically combined with comma separators for standard HTTP headers.
    /// 
    /// # Arguments 
    /// 
    /// * `value` - A string that represents the header value to append.
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// use starberry_core::http::meta::HeaderValue;
    /// let mut header_value = HeaderValue::new("text/html");
    /// header_value.append("charset=UTF-8");
    /// assert_eq!(header_value.as_str(), "text/html, charset=UTF-8");
    /// ```
    pub fn append<T: Into<String>>(&mut self, value: T) {
        match self {
            HeaderValue::Single(s) => {
                let mut values = vec![s.clone()];
                values.push(value.into());
                *self = HeaderValue::Multiple(values);
            }
            HeaderValue::Multiple(v) => v.push(value.into()),
        }
    }

    /// Convert the HeaderValue to a string representation.
    /// 
    /// Multiple values are joined with a comma and space, following HTTP header conventions.
    /// 
    /// # Returns
    /// 
    /// A string representation of the header value(s).
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// use starberry_core::http::meta::HeaderValue;
    /// let mut header_value = HeaderValue::new("text/html");
    /// header_value.append("application/xhtml+xml");
    /// assert_eq!(header_value.as_str(), "text/html, application/xhtml+xml");
    /// ```
    pub fn as_str(&self) -> String {
        match self {
            HeaderValue::Single(s) => s.clone(),
            HeaderValue::Multiple(v) => v.join(", "),
        }
    }

    /// Returns the number of values in this HeaderValue.
    /// 
    /// # Returns
    /// 
    /// * `usize` - 1 for a single value, or the count of values for multiple values.
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// use starberry_core::http::meta::HeaderValue;
    /// let mut header = HeaderValue::new("text/html");
    /// assert_eq!(header.len(), 1);
    /// 
    /// header.append("application/json");
    /// assert_eq!(header.len(), 2);
    /// ```
    pub fn len(&self) -> usize {
        match self {
            HeaderValue::Single(_) => 1,
            HeaderValue::Multiple(v) => v.len(),
        }
    }

    /// Checks if the HeaderValue is empty.
    /// 
    /// A HeaderValue is considered empty if it contains no values or only empty strings.
    /// 
    /// # Returns
    /// 
    /// `true` if the header value is empty, `false` otherwise.
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// use starberry_core::http::meta::HeaderValue;
    /// let empty_header = HeaderValue::new("");
    /// assert!(empty_header.is_empty());
    /// 
    /// let header = HeaderValue::new("application/json");
    /// assert!(!header.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        match self {
            HeaderValue::Single(s) => s.is_empty(),
            HeaderValue::Multiple(v) => v.is_empty() || v.iter().all(|s| s.is_empty()),
        }
    }

    /// Attempts to get a value at the specified index.
    /// 
    /// For a single value, only index 0 is valid.
    /// For multiple values, any valid index within the range of values is accepted.
    /// 
    /// # Arguments
    /// 
    /// * `index` - The index of the value to retrieve.
    /// 
    /// # Returns
    /// 
    /// * `Option<&String>` - The value at the specified index, or None if the index is out of bounds.
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// use starberry_core::http::meta::HeaderValue;
    /// let mut header = HeaderValue::new("text/html");
    /// assert_eq!(header.try_get(0), Some(&"text/html".to_string()));
    /// assert_eq!(header.try_get(1), None);
    /// 
    /// header.append("application/json");
    /// assert_eq!(header.try_get(1), Some(&"application/json".to_string()));
    /// ```
    pub fn try_get(&self, index: usize) -> Option<&String> {
        match self {
            HeaderValue::Single(s) if index == 0 => Some(s),
            HeaderValue::Single(_) => None,
            HeaderValue::Multiple(v) => v.get(index),
        }
    }

    /// Gets a value at the specified index, or returns an empty string if the index is out of bounds.
    /// 
    /// # Arguments
    /// 
    /// * `index` - The index of the value to retrieve.
    /// 
    /// # Returns
    /// 
    /// The string at the specified index, or an empty string if the index is out of bounds.
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// use starberry_core::http::meta::HeaderValue;
    /// let header = HeaderValue::new("text/html");
    /// assert_eq!(header.get(0), "text/html");
    /// assert_eq!(header.get(1), ""); // Out of bounds returns empty string
    /// ```
    pub fn get(&self, index: usize) -> String {
        self.try_get(index).cloned().unwrap_or_default()
    }

    /// Gets a value at the specified index, or returns the provided default if the index is out of bounds.
    /// 
    /// # Arguments
    /// 
    /// * `index` - The index of the value to retrieve.
    /// * `default` - The default value to return if the index is out of bounds.
    /// 
    /// # Returns
    /// 
    /// The string at the specified index, or the default if the index is out of bounds.
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// use starberry_core::http::meta::HeaderValue;
    /// let header = HeaderValue::new("text/html");
    /// assert_eq!(header.get_or(0, "default"), "text/html");
    /// assert_eq!(header.get_or(1, "default"), "default"); // Out of bounds returns default
    /// ```
    pub fn get_or<S: Into<String>>(&self, index: usize, default: S) -> String {
        self.try_get(index).cloned().unwrap_or_else(|| default.into())
    }

    /// Add a value to the header without combining it with existing values.
    /// 
    /// This is useful for headers like Set-Cookie where each value should be treated
    /// as a separate header instance rather than being combined with commas.
    /// 
    /// # Arguments
    /// 
    /// * `value` - The value to add.
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// use starberry_core::http::meta::HeaderValue;
    /// let mut cookies = HeaderValue::new("sessionId=abc123; Path=/");
    /// cookies.add_without_combining("theme=dark; Path=/; Max-Age=3600");
    /// 
    /// // Each cookie is kept as a separate value
    /// assert_eq!(cookies.try_get(0), Some(&"sessionId=abc123; Path=/".to_string()));
    /// assert_eq!(cookies.try_get(1), Some(&"theme=dark; Path=/; Max-Age=3600".to_string()));
    /// 
    /// // When we use as_str() they'll still be combined with commas for API consistency
    /// // but should be treated separately when used with headers like Set-Cookie
    /// ```
    pub fn add_without_combining<T: Into<String>>(&mut self, value: T) {
        match self {
            HeaderValue::Single(_) => {
                let original = std::mem::replace(self, HeaderValue::Multiple(Vec::new()));
                if let HeaderValue::Single(s) = original {
                    *self = HeaderValue::Multiple(vec![s, value.into()]);
                }
            }
            HeaderValue::Multiple(v) => v.push(value.into()),
        }
    }

    /// Attempts to get the first value in this HeaderValue.
    /// 
    /// # Returns
    /// 
    /// * `Option<&String>` - The first value, or None if there are no values.
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// use starberry_core::http::meta::HeaderValue;
    /// let mut header = HeaderValue::new("text/html");
    /// header.append("application/json");
    /// assert_eq!(header.try_first(), Some(&"text/html".to_string()));
    /// ```
    pub fn try_first(&self) -> Option<&String> {
        match self {
            HeaderValue::Single(value) => Some(value),
            HeaderValue::Multiple(values) if !values.is_empty() => Some(&values[0]),
            _ => None,
        }
    }

    /// Gets the first value in this HeaderValue, or an empty string if there are no values.
    /// 
    /// # Returns
    /// 
    /// The first value, or an empty string if there are no values.
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// use starberry_core::http::meta::HeaderValue;
    /// let header = HeaderValue::new("text/html");
    /// assert_eq!(header.first(), "text/html");
    /// 
    /// let empty: HeaderValue = HeaderValue::Multiple(vec![]);
    /// assert_eq!(empty.first(), "");
    /// ```
    pub fn first(&self) -> String {
        self.try_first().cloned().unwrap_or_default()
    }

    /// Gets the first value in this HeaderValue, or the provided default if there are no values.
    /// 
    /// # Arguments
    /// 
    /// * `default` - The default value to return if there are no values.
    /// 
    /// # Returns
    /// 
    /// The first value, or the default if there are no values.
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// use starberry_core::http::meta::HeaderValue;
    /// let header = HeaderValue::new("text/html");
    /// assert_eq!(header.first_or("default"), "text/html");
    /// 
    /// let empty: HeaderValue = HeaderValue::Multiple(vec![]);
    /// assert_eq!(empty.first_or("default"), "default");
    /// ```
    pub fn first_or<S: Into<String>>(&self, default: S) -> String {
        self.try_first().cloned().unwrap_or_else(|| default.into())
    }

    /// Gets all values as a vector of string references.
    /// 
    /// # Returns
    /// 
    /// A vector containing references to all values.
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// use starberry_core::http::meta::HeaderValue;
    /// let mut header = HeaderValue::new("text/html");
    /// header.append("application/json");
    /// 
    /// let values = header.values();
    /// assert_eq!(values.len(), 2);
    /// assert_eq!(values[0], &"text/html".to_string());
    /// assert_eq!(values[1], &"application/json".to_string());
    /// ```
    pub fn values(&self) -> Vec<&String> {
        match self {
            HeaderValue::Single(value) => vec![value],
            HeaderValue::Multiple(values) => values.iter().collect(),
        }
    }
}

/// Implements conversion from a string to HeaderValue.
///
/// This enables more ergonomic creation of HeaderValue instances.
///
/// # Examples
/// 
/// ```rust
/// use starberry_core::http::meta::HeaderValue;
/// let header: HeaderValue = "text/html".to_string().into();
/// assert_eq!(header.first(), "text/html");
/// ```
impl From<String> for HeaderValue {
    fn from(value: String) -> Self {
        HeaderValue::new(value)
    }
}

/// Implements conversion from a string slice to HeaderValue.
///
/// This enables more ergonomic creation of HeaderValue instances.
///
/// # Examples
/// 
/// ```rust
/// use starberry_core::http::meta::HeaderValue;
/// let header: HeaderValue = "text/html".into();
/// assert_eq!(header.first(), "text/html");
/// ```
impl From<&str> for HeaderValue {
    fn from(value: &str) -> Self {
        HeaderValue::new(value.to_string())
    }
}

/// Implements iterator for HeaderValue to easily iterate over all values.
///
/// # Examples
/// 
/// ```rust
/// use starberry_core::http::meta::HeaderValue;
/// let mut header = HeaderValue::new("text/html");
/// header.append("application/json");
/// 
/// let mut values = Vec::new();
/// for value in header {
///     values.push(value);
/// }
/// assert_eq!(values, vec!["text/html", "application/json"]);
/// ```
impl IntoIterator for HeaderValue {
    type Item = String;
    type IntoIter = std::vec::IntoIter<String>;

    fn into_iter(self) -> Self::IntoIter {
        match self {
            HeaderValue::Single(s) => vec![s].into_iter(),
            HeaderValue::Multiple(v) => v.into_iter(),
        }
    }
}

/// Implements conversion from HeaderValue to a vector of strings.
///
/// # Examples
/// 
/// ```rust
/// use starberry_core::http::meta::HeaderValue;
/// let mut header = HeaderValue::new("text/html");
/// header.append("application/json");
/// 
/// let values: Vec<String> = header.into();
/// assert_eq!(values, vec!["text/html", "application/json"]);
/// ```
impl From<HeaderValue> for Vec<String> {
    fn from(header_value: HeaderValue) -> Self {
        match header_value {
            HeaderValue::Single(s) => vec![s],
            HeaderValue::Multiple(v) => v,
        }
    }
}

/// Implements conversion from HeaderValue to a string.
///
/// Multiple values are joined with commas and spaces.
///
/// # Examples
/// 
/// ```rust
/// use starberry_core::http::meta::HeaderValue;
/// let mut header = HeaderValue::new("text/html");
/// header.append("application/json");
/// 
/// let value: String = header.into();
/// assert_eq!(value, "text/html, application/json");
/// ```
impl From<HeaderValue> for String {
    fn from(header_value: HeaderValue) -> Self {
        match header_value {
            HeaderValue::Single(s) => s,
            HeaderValue::Multiple(v) => v.join(", "),
        }
    }
} 

impl HttpMeta { 
    /// It is used to create a new RequestHeader object.
    pub fn new(
        start_line: HttpStartLine, 
        headers: HashMap<String, HeaderValue> 
    ) -> Self {
        Self { 
            start_line, 
            header: headers,
            content_type: None,
            content_length: None,
            cookies: None, 
            location: None, 
        }
    } 

    pub async fn from_stream<R: AsyncRead + Unpin>(
        buf_reader: &mut BufReader<R>,
        config: &ParseConfig,
        print_raw: bool,
        is_request: bool,
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
                    return Err(format!("Header line too long").into());
                }
                
                total_header_size += line.len() + 2; // +2 for CRLF
                if total_header_size > config.max_header_size {
                    return Err(format!("Headers too large").into());
                }
                
                if headers.len() >= config.max_headers {
                    return Err(format!("Too many headers").into());
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
                
                // Reject with an extremely long header line
                if line.len() > config.max_line_length {
                    return Err(format!("Header line too long").into());
                }
                
                total_header_size += line.len();
                
                // Enforce max header size limit
                if total_header_size > config.max_header_size {
                    return Err(format!("Headers too large").into());
                }
                
                // Enforce max number of headers
                if headers.len() >= config.max_headers {
                    return Err(format!("Too many headers").into());
                }
                
                // Strip CRLF injection and store the header
                let safe_line = line.trim_end().replace("\r", "");
                headers.push(safe_line);
            } 
        }
        
        if headers.is_empty() {
            return Err(format!("Empty {}", if is_request { "request" } else { "response" }).into());
        }
        
        // Parse the start line according to whether it's a request or response
        let start_line = Self::parse_start_line(&headers.remove(0), is_request);
        
        // Parse headers with special handling for specific header names
        let header = Self::parse_headers(headers, is_request);
        
        if print_raw {
            println!("Parsed headers: {:?}", header);
            println!("Parsed start line: {:?}", start_line);
        }
        
        Ok(HttpMeta::new(start_line, header))
    }
    
    // Helper function to parse the start line
    fn parse_start_line(line: &str, is_request: bool) -> HttpStartLine {
        if is_request {
            HttpStartLine::parse_request(line)
        } else {
            HttpStartLine::parse_response(line)
        }
    }
    
    // Helper function to parse headers with special handling for specific header types
    fn parse_headers(header_lines: Vec<String>, is_response: bool) -> HashMap<String, HeaderValue> {
        let mut headers: HashMap<String, HeaderValue> = HashMap::new();
        
        // // List of headers that should not be combined (kept as separate values)
        // // This is especially important for responses with multiple Set-Cookie headers
        // let non_combinable_headers: HashSet<&str> = [
        //     "set-cookie",
        //     // Add other headers that should not be combined if needed 
        // ].iter().cloned().collect();
        
        for line in header_lines {
            if let Some(colon_pos) = line.find(':') {
                let (key, value) = line.split_at(colon_pos);
                
                // Normalize the header name (case-insensitive in HTTP)
                let header_name = key.trim().to_lowercase();
                
                // Remove the colon and trim whitespace from the value
                let header_value = value[1..].trim().to_string();
                
                // Check if this is a special header that should not be combined
                // let is_non_combinable = is_response && non_combinable_headers.contains(header_name.as_str());
                
                match headers.get_mut(&header_name) {
                    Some(existing_value) => { 
                        existing_value.add_without_combining(header_value);  
                        // For special headers like Set-Cookie, add without combining
                        // if is_non_combinable {
                        //     existing_value.add_without_combining(header_value);
                        // } else {
                        //     // For regular headers, append (typically combined with commas)
                        //     existing_value.append(header_value);
                        // }
                    }
                    None => {
                        // First occurrence of this header
                        headers.insert(header_name, HeaderValue::new(header_value));
                    }
                }
            }
        }
        
        headers
    }
    
    // Expose the specific methods that call the shared implementation
    pub async fn from_request_stream<R: AsyncRead + Unpin>(
        buf_reader: &mut BufReader<R>,
        config: &ParseConfig, 
        print_raw: bool, 
    ) -> Result<HttpMeta, Box<dyn Error + Send + Sync>> {
        Self::from_stream(buf_reader, config, print_raw, true).await 
    }
    
    pub async fn from_response_stream<R: AsyncRead + Unpin>(
        buf_reader: &mut BufReader<R>,
        config: &ParseConfig, 
        print_raw: bool, 
    ) -> Result<HttpMeta, Box<dyn Error + Send + Sync>> {
        Self::from_stream(buf_reader, config, print_raw, false).await 
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

    pub fn set_header_hashmap(&mut self, header: HashMap<String, HeaderValue>) {
        self.header = header;
    } 

    /// Returns the hashed, unparsed header. 
    /// Note this reference is not intended for you to mutate. 
    /// If yo do want to mutate, please use .set_attribute() method 
    pub fn get_header_hashmap(&self) -> &HashMap<String, HeaderValue> { 
        &self.header 
    } 

    /// 
    pub fn set_attribute<T: Into<String>, S: Into<HeaderValue>>(&mut self, key: T, value: S) { 
        self.header.insert(key.into().trim().to_lowercase(), value.into()); 
    } 

    pub fn get_path(&mut self, part: usize) -> String {
        self.start_line.get_url().url_part(part)
    }

    pub fn path(&self) -> String {
        self.start_line.path() 
    } 

    pub fn method(&self) -> HttpMethod {
        self.start_line.method() 
    } 

    /// Gets the content length from the HTTP meta data.
    ///
    /// Returns the cached content length if available, otherwise parses
    /// the content-length header from the headers map.
    ///
    /// # Returns
    ///
    /// * `Option<usize>` - The content length, or None if not available or invalid.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use starberry_core::http::meta::HttpMeta;
    /// use starberry_core::http::meta::HeaderValue;
    /// use std::collections::HashMap;
    ///
    /// let mut headers = HashMap::new();
    /// headers.insert("content-length".to_string(), HeaderValue::new("123"));
    /// let mut meta = HttpMeta::new(Default::default(), headers);
    /// 
    /// assert_eq!(meta.get_content_length(), Some(123));
    /// ``` 
    pub fn get_content_length(&mut self) -> Option<usize> {
        if let Some(length) = self.content_length {
            return Some(length);
        }
        self.parse_content_length()
    } 

    /// Parses the Content-Length header from the headers map and stores it in the content_length field.
    ///
    /// # Returns
    ///
    /// * `Option<usize>` - The parsed Content-Length value, or None if not present or invalid.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use starberry_core::http::meta::HttpMeta;
    /// use starberry_core::http::meta::HeaderValue;
    /// use std::collections::HashMap;
    ///
    /// let mut headers = HashMap::new();
    /// headers.insert("content-length".to_string(), HeaderValue::new("123"));
    /// let mut meta = HttpMeta::new(Default::default(), headers);
    /// 
    /// let length = meta.parse_content_length();
    /// assert_eq!(length, Some(123));
    /// assert_eq!(meta.get_content_length(), Some(123));
    /// ``` 
    pub fn parse_content_length(&mut self) -> Option<usize> {
        let length = self
            .header
            .get("content-length")
            .and_then(|s| s.first().parse::<usize>().ok()); 
        self.content_length = length;
        length 
    }

    /// Sets the content_length field.
    ///
    /// # Arguments
    ///
    /// * `length` - The content length to set.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use starberry_core::http::meta::HttpMeta;
    /// 
    /// let mut meta = HttpMeta::default();
    /// meta.set_content_length(456);
    /// 
    /// assert_eq!(meta.get_content_length(), Some(456));
    /// ```
    pub fn set_content_length(&mut self, length: usize) {
        self.content_length = Some(length);
    }  

    /// Clears the cached content_length field without modifying the header map.
    ///
    /// Note that it will **NOT** clear the value in the HashMap.
    /// To remove both the cached field and the header, use `delete_content_length()`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use starberry_core::http::meta::HttpMeta;
    /// use starberry_core::http::meta::HeaderValue;
    /// 
    /// let mut meta = HttpMeta::default();
    /// meta.set_content_length(123);
    /// meta.clear_content_length();
    /// 
    /// // The content-length header in the HashMap is still intact
    /// // but the cached value is cleared
    /// ```
    pub fn clear_content_length(&mut self) {
        self.content_length = None;
    } 

    /// Deletes the Content-Length header completely, clearing both the cached field
    /// and removing it from the header map.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use starberry_core::http::meta::HttpMeta;
    /// use starberry_core::http::meta::HeaderValue;
    /// 
    /// let mut meta = HttpMeta::default();
    /// meta.set_header("content-length", "123");
    /// meta.delete_content_length();
    /// 
    /// // Both the cached field and the header are now removed
    /// assert!(meta.get_header("content-length").is_none());
    /// ```
    pub fn delete_content_length(&mut self) {
        self.content_length = None;
        self.header.remove("content-length");
    }

    /// Gets the content type from the HTTP meta data.
    ///
    /// Returns the cached content type if available, otherwise parses
    /// the content-type header from the headers map.
    ///
    /// # Returns
    ///
    /// * `Option<HttpContentType>` - The content type, or None if not available.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use starberry_core::http::meta::HttpMeta;
    /// use starberry_core::http::meta::HeaderValue;
    /// use starberry_core::http::http_value::HttpContentType;
    /// use std::collections::HashMap;
    ///
    /// let mut headers = HashMap::new();
    /// headers.insert("content-type".to_string(), HeaderValue::new("text/html"));
    /// let mut meta = HttpMeta::new(Default::default(), headers);
    /// 
    /// assert_eq!(meta.get_content_type(), Some(HttpContentType::TextHtml));
    /// ``` 
    pub fn get_content_type(&mut self) -> Option<HttpContentType> {
        if let Some(ref content_type) = self.content_type {
            return Some(content_type.clone());
        }
        self.parse_content_type()
    } 

    /// Parses the Content-Type header from the headers map and stores it in the content_type field.
    ///
    /// # Returns
    ///
    /// * `Option<HttpContentType>` - The parsed Content-Type value, or None if not present.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use starberry_core::http::meta::HttpMeta;
    /// use starberry_core::http::meta::HeaderValue;
    /// use starberry_core::http::http_value::HttpContentType;
    /// use std::collections::HashMap;
    ///
    /// let mut headers = HashMap::new();
    /// headers.insert("content-type".to_string(), HeaderValue::new("text/html"));
    /// let mut meta = HttpMeta::new(Default::default(), headers);
    /// 
    /// let content_type = meta.parse_content_type();
    /// assert_eq!(content_type, Some(HttpContentType::TextHtml));
    /// ``` 
    pub fn parse_content_type(&mut self) -> Option<HttpContentType> {
        // Try lowercase first, then uppercase for backward compatibility
        let content_type_str = self.header
            .get("content-type") 
            .map(|value| value.first()); 
        
        if let None = content_type_str { 
            return None; 
        }; 

        let content_type_str = content_type_str.unwrap();    
        let content_type = HttpContentType::from_str(&content_type_str);
        self.set_content_type(content_type.clone());
        Some(content_type)
    } 

    /// Sets the content_type field.
    ///
    /// # Arguments
    ///
    /// * `content_type` - The content type to set.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use starberry_core::http::meta::HttpMeta;
    /// use starberry_core::http::http_value::HttpContentType;
    /// 
    /// let mut meta = HttpMeta::default();
    /// meta.set_content_type(HttpContentType::ApplicationJson);
    /// 
    /// assert_eq!(meta.get_content_type(), Some(HttpContentType::ApplicationJson));
    /// ```
    pub fn set_content_type(&mut self, content_type: HttpContentType) {
        self.content_type = Some(content_type);
    } 

    /// Clears the cached content_type field without modifying the header map.
    ///
    /// This method invalidates the cached content_type value, which will cause
    /// subsequent calls to `get_content_type()` to re-parse the value from the
    /// headers map.
    ///
    /// Note that it will **NOT** clear the value in the headers map.
    /// To remove both the cached field and the header, use `delete_content_type()`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use starberry_core::http::meta::HttpMeta;
    /// use starberry_core::http::meta::HeaderValue;
    /// use starberry_core::http::http_value::HttpContentType;
    /// use std::collections::HashMap;
    ///
    /// let mut headers = HashMap::new();
    /// headers.insert("content-type".to_string(), HeaderValue::new("text/html"));
    /// let mut meta = HttpMeta::new(Default::default(), headers);
    /// 
    /// // Parse the value into the cache
    /// let content_type = meta.get_content_type();
    /// assert_eq!(content_type, Some(HttpContentType::TextHtml));
    /// 
    /// // Clear the cache only
    /// meta.clear_content_type();
    /// 
    /// // The header is still intact and will be re-parsed
    /// assert_eq!(meta.get_content_type(), Some(HttpContentType::TextHtml));
    /// ```
    pub fn clear_content_type(&mut self) {
        self.content_type = None;
    }

    /// Deletes the Content-Type header completely, clearing both the cached field
    /// and removing it from the header map.
    ///
    /// This method removes the content-type header from the headers map and
    /// clears the cached content_type value. Subsequent calls to `get_content_type()`
    /// will return a default value unless a new content-type is set.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use starberry_core::http::meta::HttpMeta;
    /// use starberry_core::http::meta::HeaderValue;
    /// use starberry_core::http::http_value::HttpContentType;
    /// use std::collections::HashMap;
    ///
    /// let mut headers = HashMap::new();
    /// headers.insert("content-type".to_string(), HeaderValue::new("text/html"));
    /// let mut meta = HttpMeta::new(Default::default(), headers);
    /// 
    /// // Delete both the cache and header
    /// meta.delete_content_type();
    /// 
    /// // The header is gone
    /// assert!(meta.get_header("content-type").is_none());
    /// 
    /// // And get_content_type will now return a default value
    /// assert_eq!(meta.get_content_type().unwrap(), HttpContentType::from_str(""));
    /// ```
    pub fn delete_content_type(&mut self) {
        self.content_type = None;
        self.header.remove("content-type");
    }

    /// Gets the cookies from the HTTP meta data.
    ///
    /// Returns the cached cookies if available, otherwise parses
    /// the cookie header from the headers map.
    ///
    /// # Returns
    ///
    /// A reference to the cookies map.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use starberry_core::http::meta::HttpMeta;
    /// use starberry_core::http::meta::HeaderValue;
    /// use starberry_core::http::cookie::{Cookie, CookieMap};
    /// use std::collections::HashMap;
    ///
    /// let mut headers = HashMap::new();
    /// headers.insert("cookie".to_string(), HeaderValue::new("sessionId=abc123; theme=dark"));
    /// let mut meta = HttpMeta::new(Default::default(), headers);
    /// 
    /// let cookies = meta.get_cookies();
    /// assert_eq!(cookies.get("sessionId").unwrap().value(), "abc123");
    /// assert_eq!(cookies.get("theme").unwrap().value(), "dark");
    /// ```
    pub fn get_cookies(&mut self) -> &CookieMap {
        if self.cookies.is_none() { 
            self.cookies = Some(self.parse_cookies());
        }
        self.cookies.as_ref().unwrap()
    } 

    /// Gets a specific cookie by key.
    ///
    /// If the cookies haven't been parsed yet, parses them from the headers map.
    ///
    /// # Arguments
    ///
    /// * `key` - The cookie key to look up.
    ///
    /// # Returns
    ///
    /// * `Option<Cookie>` - The cookie if found, or None.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use starberry_core::http::meta::HttpMeta;
    /// use starberry_core::http::meta::HeaderValue;
    /// use std::collections::HashMap;
    ///
    /// let mut headers = HashMap::new();
    /// headers.insert("cookie".to_string(), HeaderValue::new("sessionId=abc123; theme=dark"));
    /// let mut meta = HttpMeta::new(Default::default(), headers);
    /// 
    /// let session_cookie = meta.get_cookie("sessionId");
    /// assert_eq!(session_cookie.unwrap().value(), "abc123");
    /// ```
    pub fn get_cookie<T: AsRef<str>>(&mut self, key: T) -> Option<Cookie> {
        if self.cookies.is_none() {
            self.cookies = Some(self.parse_cookies());
        }
        self.cookies.as_ref().unwrap().get(key).cloned()
    } 

    /// Gets a specific cookie by key, returning a default cookie if not found.
    ///
    /// # Arguments
    ///
    /// * `key` - The cookie key to look up.
    ///
    /// # Returns
    ///
    /// The cookie if found, or a default empty cookie.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use starberry_core::http::meta::HttpMeta;
    /// use starberry_core::http::meta::HeaderValue;
    /// use std::collections::HashMap;
    ///
    /// let mut headers = HashMap::new();
    /// headers.insert("cookie".to_string(), HeaderValue::new("sessionId=abc123"));
    /// let mut meta = HttpMeta::new(Default::default(), headers);
    /// 
    /// // Existing cookie
    /// let session_cookie = meta.get_cookie_or_default("sessionId");
    /// assert_eq!(session_cookie.value(), "abc123");
    /// 
    /// // Non-existent cookie returns default
    /// let nonexistent = meta.get_cookie_or_default("nonexistent");
    /// assert_eq!(nonexistent.value(), "");
    /// ```
    pub fn get_cookie_or_default<T: AsRef<str>>(&mut self, key: T) -> Cookie {
        self.get_cookie(key).unwrap_or_else(|| Cookie::new(""))
    } 

    /// Parses cookies from either request Cookie header or response Set-Cookie headers,
    /// depending on the type of HTTP message (request or response).
    ///
    /// # Returns
    ///
    /// A CookieMap containing the parsed cookies.
    ///
    /// # Examples
    ///
    /// ```rust
    /// // For a request with a Cookie header
    /// use starberry_core::http::meta::{HttpMeta, HeaderValue};
    /// use starberry_core::http::http_value::HttpStartLine;
    /// use std::collections::HashMap;
    ///
    /// let mut headers = HashMap::new();
    /// headers.insert("cookie".to_string(), HeaderValue::new("sessionId=abc123; theme=dark"));
    /// let mut meta = HttpMeta::new(HttpStartLine::parse_request("GET / HTTP/1.1"), headers);
    /// 
    /// let cookies = meta.parse_cookies();
    /// assert_eq!(cookies.get("sessionId").unwrap().value, "abc123");
    ///
    /// // For a response with Set-Cookie headers
    /// use starberry_core::http::meta::{HttpMeta, HeaderValue};
    /// use starberry_core::http::http_value::HttpStartLine;
    /// use std::collections::HashMap;
    ///
    /// let mut headers = HashMap::new();
    /// headers.insert("set-cookie".to_string(), HeaderValue::new("sessionId=abc123; Path=/; Secure"));
    /// let mut meta = HttpMeta::new(HttpStartLine::parse_response("HTTP/1.1 200 OK"), headers);
    /// 
    /// let cookies = meta.parse_cookies();
    /// assert_eq!(cookies.get("sessionId").unwrap().value, "abc123");
    /// assert_eq!(cookies.get("sessionId").unwrap().get_path(), Some("/".to_string()));
    /// assert_eq!(cookies.get("sessionId").unwrap().get_secure(), Some(true));
    /// ```
    pub fn parse_cookies(&self) -> CookieMap {
        // Check if this is a request or response
        if self.start_line.is_request() {
            self.parse_request_cookies()
        } else {
            self.parse_response_cookies()
        }
    }
    
    /// Parses cookies from the request Cookie header.
    ///
    /// # Returns
    ///
    /// A CookieMap containing the parsed cookies.
    fn parse_request_cookies(&self) -> CookieMap { 
        let cookie_header = self.header.get("cookie");
        
        match cookie_header {
            Some(header_value) => match header_value {
                HeaderValue::Single(cookie_str) => CookieMap::parse(cookie_str),
                HeaderValue::Multiple(cookie_strs) => {
                    // Combine multiple cookie headers into one map
                    let mut cookie_map = CookieMap::new();
                    for cookie_str in cookie_strs {
                        let parsed = CookieMap::parse(cookie_str);
                        for (k, v) in parsed.0.into_iter() {
                            cookie_map.set(k, v);
                        }
                    }
                    cookie_map
                }
            },
            None => CookieMap::default()
        }
    }

    /// Parses cookies from the response Set-Cookie headers.
    ///
    /// # Returns
    ///
    /// A CookieMap containing the parsed cookies with their attributes.
    fn parse_response_cookies(&self) -> CookieMap {
        let set_cookie_header = self.header.get("set-cookie");
        
        match set_cookie_header {
            Some(HeaderValue::Single(s)) => CookieMap::parse_set_cookies([s.as_str()]),
            Some(HeaderValue::Multiple(v)) => {
                CookieMap::parse_set_cookies(v.iter().map(|s| s.as_str()))
            },
            None => CookieMap::default()
        }
    } 

    pub fn set_cookies(&mut self, cookies: CookieMap) { 
        self.cookies = Some(cookies);
    } 

    /// Add a cookie to the HTTP meta data.
    ///
    /// # Arguments
    ///
    /// * `key` - The cookie key.
    /// * `cookie` - The cookie to add.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use starberry_core::http::meta::HttpMeta;
    /// use starberry_core::http::cookie::Cookie;
    /// 
    /// let mut meta = HttpMeta::default();
    /// meta.add_cookie("sessionId", Cookie::new("abc123"));
    /// 
    /// assert_eq!(meta.get_cookie("sessionId").unwrap().value(), "abc123");
    /// ```
    pub fn add_cookie<T: Into<String>>(&mut self, key: T, cookie: Cookie) { 
        if self.cookies.is_none() { 
            self.cookies = Some(CookieMap::new()); 
        }         if let Some(ref mut cookies) = self.cookies { 
            cookies.set(key, cookie); 
        } 
    } 

    /// Clears the cached cookies field without modifying the header map.
    ///
    /// This method invalidates the cached cookies value, which will cause
    /// subsequent calls to `get_cookies()` to re-parse the value from the
    /// headers map.
    ///
    /// Note that it will **NOT** clear the value in the headers map.
    /// To remove both the cached field and the header, use `delete_cookies()`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use starberry_core::http::meta::HttpMeta;
    /// use starberry_core::http::meta::HeaderValue;
    /// use std::collections::HashMap;
    ///
    /// let mut headers = HashMap::new();
    /// headers.insert("cookie".to_string(), HeaderValue::new("sessionId=abc123"));
    /// let mut meta = HttpMeta::new(Default::default(), headers);
    /// 
    /// // Parse the value into the cache
    /// let cookies = meta.get_cookies();
    /// assert_eq!(cookies.get("sessionId").unwrap().value(), "abc123");
    /// 
    /// // Clear the cache only
    /// meta.clear_cookies();
    /// 
    /// // The header is still intact and will be re-parsed
    /// assert_eq!(meta.get_cookies().get("sessionId").unwrap().value(), "abc123");
    /// ```
    pub fn clear_cookies(&mut self) {
        self.cookies = None;
    }

    /// Deletes the Cookie header completely, clearing both the cached field
    /// and removing it from the header map.
    ///
    /// This method removes the cookie header from the headers map and
    /// clears the cached cookies value. Subsequent calls to `get_cookies()`
    /// will return an empty cookie map unless new cookies are set.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use starberry_core::http::meta::HttpMeta;
    /// use starberry_core::http::meta::HeaderValue;
    /// use std::collections::HashMap;
    ///
    /// let mut headers = HashMap::new();
    /// headers.insert("cookie".to_string(), HeaderValue::new("sessionId=abc123"));
    /// let mut meta = HttpMeta::new(Default::default(), headers);
    /// 
    /// // Delete both the cache and header
    /// meta.delete_cookies();
    /// 
    /// // The header is gone
    /// assert!(meta.get_header("cookie").is_none());
    /// 
    /// // And get_cookies will now return an empty map
    /// assert!(meta.get_cookies().is_empty());
    /// ```
    pub fn delete_cookies(&mut self) {
        self.cookies = None;
        self.header.remove("cookie"); 
        self.header.remove("set-cookie"); 
    } 

    /// Gets the location header from the HTTP meta data.
    ///
    /// Returns the cached location if available, otherwise parses
    /// the location header from the headers map.
    ///
    /// # Returns
    ///
    /// * `Option<String>` - The location, or None if not available.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use starberry_core::http::meta::HttpMeta;
    /// use starberry_core::http::meta::HeaderValue;
    /// use std::collections::HashMap;
    ///
    /// let mut headers = HashMap::new();
    /// headers.insert("location".to_string(), HeaderValue::new("/redirect"));
    /// let mut meta = HttpMeta::new(Default::default(), headers);
    /// 
    /// assert_eq!(meta.get_location(), Some("/redirect".to_string()));
    /// ```
    pub fn get_location(&mut self) -> Option<String> {
        if let Some(ref loc) = self.location {
            return Some(loc.clone());
        }
        self.parse_location()
    } 

    /// Parses the Location header from the headers map and stores it in the location field.
    ///
    /// # Returns
    ///
    /// * `Option<String>` - The location value, or None if not present.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use starberry_core::http::meta::HttpMeta;
    /// use starberry_core::http::meta::HeaderValue;
    /// use std::collections::HashMap;
    ///
    /// let mut headers = HashMap::new();
    /// headers.insert("location".to_string(), HeaderValue::new("/redirect"));
    /// let mut meta = HttpMeta::new(Default::default(), headers);
    /// 
    /// let location = meta.parse_location();
    /// assert_eq!(location, Some("/redirect".to_string()));
    /// ```
    pub fn parse_location(&mut self) -> Option<String> {
        // Try both lowercase and uppercase for backward compatibility
        let location = self.header
            .get("location") 
            .map(|value| value.first());
        
        self.set_location(location.clone());
        location
    } 

    /// Sets the location field.
    ///
    /// # Arguments
    ///
    /// * `location` - The location to set.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use starberry_core::http::meta::HttpMeta;
    /// 
    /// let mut meta = HttpMeta::default();
    /// meta.set_location(Some("/redirect".to_string()));
    /// 
    /// assert_eq!(meta.get_location(), Some("/redirect".to_string()));
    /// ```
    pub fn set_location(&mut self, location: Option<String>) {
        self.location = location;
    } 

    /// Clears the cached location field without modifying the header map.
    ///
    /// This method invalidates the cached location value, which will cause
    /// subsequent calls to `get_location()` to re-parse the value from the
    /// headers map.
    ///
    /// Note that it will **NOT** clear the value in the headers map.
    /// To remove both the cached field and the header, use `delete_location()`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use starberry_core::http::meta::HttpMeta;
    /// use starberry_core::http::meta::HeaderValue;
    /// use std::collections::HashMap;
    ///
    /// let mut headers = HashMap::new();
    /// headers.insert("location".to_string(), HeaderValue::new("/redirect"));
    /// let mut meta = HttpMeta::new(Default::default(), headers);
    /// 
    /// // Parse the value into the cache
    /// let location = meta.get_location();
    /// assert_eq!(location, Some("/redirect".to_string()));
    /// 
    /// // Clear the cache only
    /// meta.clear_location();
    /// 
    /// // The header is still intact and will be re-parsed
    /// assert_eq!(meta.get_location(), Some("/redirect".to_string()));
    /// ```
    pub fn clear_location(&mut self) {
        self.location = None;
    }

    /// Deletes the Location header completely, clearing both the cached field
    /// and removing it from the header map.
    ///
    /// This method removes the location header from the headers map and
    /// clears the cached location value. Subsequent calls to `get_location()`
    /// will return None unless a new location is set.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use starberry_core::http::meta::HttpMeta;
    /// use starberry_core::http::meta::HeaderValue;
    /// use std::collections::HashMap;
    ///
    /// let mut headers = HashMap::new();
    /// headers.insert("location".to_string(), HeaderValue::new("/redirect"));
    /// let mut meta = HttpMeta::new(Default::default(), headers);
    /// 
    /// // Delete both the cache and header
    /// meta.delete_location();
    /// 
    /// // The header is gone
    /// assert!(meta.get_header("location").is_none());
    /// 
    /// // And get_location will now return None
    /// assert_eq!(meta.get_location(), None);
    /// ```
    pub fn delete_location(&mut self) {
        self.location = None;
        self.header.remove("location");
    } 

        /// Serializes the HTTP meta data to a string representation.
    ///
    /// This method generates a properly formatted HTTP header section,
    /// including the start line and all headers.
    ///
    /// # Returns
    ///
    /// A string containing the start line and all headers, formatted
    /// according to the HTTP protocol.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use starberry_core::http::meta::HttpMeta;
    /// use starberry_core::http::meta::HeaderValue;
    /// use starberry_core::http::http_value::{HttpStartLine, HttpVersion, HttpMethod};
    /// use std::collections::HashMap;
    ///
    /// // Create a request meta
    /// let mut meta = HttpMeta::new(
    ///     HttpStartLine::new_request(
    ///         HttpVersion::Http11,
    ///         HttpMethod::GET,
    ///         "/index.html".to_string()
    ///     ),
    ///     HashMap::new()
    /// );
    /// meta.set_header("host", "example.com");
    ///
    /// let http_string = meta.represent();
    /// assert!(http_string.starts_with("GET /index.html HTTP/1.1\r\n"));
    /// assert!(http_string.contains("host: example.com\r\n"));
    /// assert!(http_string.ends_with("\r\n\r\n"));
    /// ```
    pub fn represent(&self) -> String {
        let mut result = String::new();
        let mut handled_headers = HashSet::new();
        
        // Add the start line (works for both request and response)
        result.push_str(&format!("{}\r\n", self.start_line));
        
        // Process field values first (they have priority)
        
        // Add content-type if present
        if let Some(ref content_type) = self.content_type {
            result.push_str(&format!("content-type: {}\r\n", content_type));
            handled_headers.insert("content-type".to_string());
        }
        
        // Add content-length if present
        if let Some(content_length) = self.content_length {
            result.push_str(&format!("content-length: {}\r\n", content_length));
            handled_headers.insert("content-length".to_string());
        }
        
        // Add location if present
        if let Some(ref location) = self.location {
            result.push_str(&format!("location: {}\r\n", location));
            handled_headers.insert("location".to_string());
        }
        
        // Add cookies based on whether this is a request or response
        if let Some(ref cookies) = self.cookies {
            if self.start_line.is_request() {
                // For requests, we use the Cookie header
                let cookie_header = cookies.request();
                if !cookie_header.is_empty() {
                    result.push_str(&format!("{}\r\n", cookie_header));
                    handled_headers.insert("cookie".to_string());
                }
            } else {
                // For responses, we use Set-Cookie headers
                let cookie_header = cookies.response();
                if !cookie_header.is_empty() {
                    result.push_str(&format!("{}\r\n", cookie_header));
                    handled_headers.insert("set-cookie".to_string());
                }
            }
        }
        
        // Now process any remaining headers from the hashmap
        for (key, value) in &self.header {
            if !handled_headers.contains(key) {
                match value {
                    HeaderValue::Single(v) => {
                        // Single values get a single header line
                        result.push_str(&format!("{}: {}\r\n", key, v));
                    },
                    HeaderValue::Multiple(values) => {
                        // Multiple values each get their own header line
                        for v in values {
                            result.push_str(&format!("{}: {}\r\n", key, v));
                        }
                    }
                }
            }
        }
        
        // End headers with an extra CRLF
        result.push_str("\r\n");
        
        result 
    } 
} 

impl Default for HttpMeta { 
    fn default() -> Self {
        Self { 
            start_line: HttpStartLine::new_request( 
                HttpVersion::Http11,
                HttpMethod::GET,
                "/".to_string(),
            ), 
            header: HashMap::new(),
            content_type: None,
            content_length: None,
            cookies: None, 
            location: None, 
        }
    } 
}
