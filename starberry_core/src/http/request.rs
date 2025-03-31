use crate::app::urls::Url;

use super::http_value::*; 
use std::hash::Hash;
use std::io::{BufRead, BufReader, Read};
use std::error::Error;
use std::net::TcpStream; 
use std::str;
use once_cell::sync::Lazy;
use regex::Regex; 
use std::collections::HashMap; 
use akari::Object; 

#[derive(Debug)]  
pub struct HttpRequest { 
    pub start_line: RequestStartLine, 
    pub header: RequestHeader, 
    pub body: RequestBody, 
} 

/// RequestStartLine is the first line of the HTTP request, which contains the method, path, and HTTP version. 
#[derive(Debug, Clone)] 
pub struct RequestStartLine{ 
    pub http_version: HttpVersion, 
    pub method: HttpMethod, 
    pub path: String, 
    pub url: Option<RequestPath> 
} 

pub struct ParseConfig{ 
    pub max_header_size: usize, 
    pub max_line_length: usize, 
    pub max_headers: usize, 
    pub max_body_size: usize, 
} 

impl ParseConfig{ 
    pub fn new(max_header_size: usize, max_line_length: usize, max_headers: usize , max_body_size: usize) -> Self { 
        Self { max_header_size, max_body_size, max_line_length, max_headers } 
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
        Self { max_header_size: 8192, max_body_size: 1028*1028, max_line_length: 8192, max_headers: 100 } 
    } 
}

impl RequestStartLine{ 
    /// It is used to parse the request line and create a new RequestStartLine object. 
    pub fn new(http_version: HttpVersion, method: HttpMethod, path: String) -> Self { 
        Self { http_version, method, path, url: None } 
    }  

    /// It is used to convert the RequestStartLine object to a string. 
    pub fn represent(&self) -> String { 
        format!("{} {} {}", self.http_version.to_string(), self.method.to_string(), self.path) 
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
    
        Ok(Self::new( http_version, method, path )) 
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

/// RequestHeader is a struct that represents the headers of an HTTP request. 
#[derive(Debug)] 
pub struct RequestHeader{ 
    pub header: HashMap<String, String>, 
    content_type: Option<HttpContentType>, 
    content_length: Option<usize>, 
    cookies: Option<HashMap<String, String>>, 
} 

impl RequestHeader { 
    /// It is used to create a new RequestHeader object. 
    pub fn new() -> Self { 
        Self { 
            header: HashMap::new(), 
            content_type: None, 
            content_length: None, 
            cookies: None, 
        } 
    } 

    pub fn set_header_hashmap(&mut self, header: HashMap<String, String>) { 
        self.header = header; 
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
    pub fn parse(headers: Vec<String>) -> Self{ 
        let mut header_map = HashMap::new(); 
        for line in headers { 
            let parts: Vec<&str> = line.splitn(2, ':').collect();
            if parts.len() == 2 {
                let key = parts[0].trim().to_uppercase().to_string();
                let value = parts[1].trim().to_string();
                header_map.insert(key, value);
            }
        } 
        let mut header = Self::new(); 
        header.set_header_hashmap(header_map); 
        header 
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
            None => return self.parse_content_length() 
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
    pub fn parse_content_length(&mut self) -> Option<usize>{ 
        let length = self.header.get("CONTENT-LENGTH") 
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
        match self.content_type{ 
            Some(ref content_type) => return Some(content_type.clone()), 
            None => return self.parse_content_type() 
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
        let content_type = HttpContentType::from_str(
            self.header.get("CONTENT-TYPE").unwrap_or(&"".to_owned())
        ); 
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
        let cookies = match self.header.get("COOKIE"){ 
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

#[derive(Debug)]   
pub enum RequestBody{ 
    Text(String), 
    Binary(Vec<u8>), 
    Form(UrlEncodedForm), 
    Files(MultiForm), 
    Json(Object), 
    Empty, 
} 

impl RequestBody{ 
    pub fn parse(body: Vec<u8>, header: &mut RequestHeader) -> Self { 
    
        if header.get_content_length().unwrap_or(0) == 0 { 
            return RequestBody::Empty; 
        } 

        match header.get_content_type().unwrap_or(HttpContentType::from_str("")){ 
            HttpContentType::Application { subtype, .. } if subtype == "json" => return Self::parse_json(body), 
            HttpContentType::Text { subtype, .. } if subtype == "html" => return Self::parse_text(body), 
            HttpContentType::Text { subtype, .. } if subtype == "plain"  => return Self::parse_text(body), 
            HttpContentType::Application { subtype, .. } if subtype == "x-www-form-urlencoded" => return Self::url_encoded_form(body), 
            HttpContentType::Multipart { subtype, boundary } if subtype == "form-data" => return Self::multipart_form_data(body, boundary.unwrap_or("".to_string())), 
            _ => return Self::parse_text(body), 
        } 
    } 

    pub fn parse_json(body: Vec<u8>) -> Self { 
        return RequestBody::Json(Object::from_json(
            str::from_utf8(&body)
            .unwrap_or(""))
            .unwrap_or(Object::new("")
        ));  
    } 

    pub fn parse_text(body: Vec<u8>) -> Self { 
        return RequestBody::Text(String::from_utf8_lossy(&body).to_string()); 
    } 

    pub fn parse_binary(body: Vec<u8>) -> Self { 
        return RequestBody::Binary(body); 
    } 

    pub fn url_encoded_form(body: Vec<u8>) -> Self { 
        let form_data = String::from_utf8_lossy(&body).to_string(); 
        let mut form_map = HashMap::new(); 
        for pair in form_data.split('&') { 
            let parts: Vec<&str> = pair.split('=').collect(); 
            if parts.len() == 2 { 
                form_map.insert(parts[0].to_string(), parts[1].to_string()); 
            } 
        } 
        return RequestBody::Form(UrlEncodedForm { data: form_map });  
    } 

    /// Parses a multipart form data body into a HashMap.
    ///
    /// # Arguments
    ///
    /// * `body` - The raw bytes of the multipart form data body
    /// * `boundary` - The boundary string specified in the Content-Type header
    ///
    /// # Returns
    ///
    /// A HashMap where keys are field names and values are parsed form fields
    ///
    /// # Examples
    /// 
    /// ```
    /// use starberry_core::http::request::RequestBody;  
    /// let boundary = "boundary123";
    /// let body = concat!(
    ///     "--boundary123\r\n",
    ///     "Content-Disposition: form-data; name=\"field1\"\r\n\r\n",
    ///     "value1\r\n",
    ///     "--boundary123\r\n",
    ///     "Content-Disposition: form-data; name=\"file1\"; filename=\"example.txt\"\r\n",
    ///     "Content-Type: text/plain\r\n\r\n",
    ///     "file content here\r\n",
    ///     "--boundary123--\r\n"
    /// ).as_bytes().to_vec();
    ///
    /// let form_data = RequestBody::multipart_form_data(body, boundary.to_string());
    /// let form_data = if let RequestBody::Files(ref data) = form_data { 
    ///     data 
    /// } else { 
    ///    panic!("Expected multipart form data")
    /// }; 
    /// assert_eq!(form_data.len(), 2);
    /// assert!(form_data.contains_key("field1"));
    /// assert!(form_data.contains_key("file1"));
    /// // Test the file content and filename 
    /// assert_eq!(form_data.get("file1").unwrap().filename(), Some("example.txt".to_string()));
    /// ```
    pub fn multipart_form_data(body: Vec<u8>, boundary: String) -> Self {
        
        /// Finds a subsequence within a larger sequence of bytes.
        fn find_subsequence(haystack: &[u8], needle: &[u8]) -> Option<usize> {
            haystack.windows(needle.len()).position(|window| window == needle)
        }

        /// Extracts the field name from the Content-Disposition header.
        fn extract_field_name(headers: &str) -> Option<String> {
            // Simple regex to extract name="value" from Content-Disposition
            let re = regex::Regex::new(r#"Content-Disposition:.*?name="([^"]+)""#).unwrap();
            re.captures(headers)
                .and_then(|cap| cap.get(1))
                .map(|m| m.as_str().to_string())
        }

        /// Extracts the filename from the Content-Disposition header if present.
        fn extract_filename(headers: &str) -> Option<String> {
            let re = regex::Regex::new(r#"Content-Disposition:.*?filename="([^"]+)""#).unwrap();
            re.captures(headers)
                .and_then(|cap| cap.get(1))
                .map(|m| m.as_str().to_string())
        }

        /// Extracts the content type from the Content-Type header if present.
        fn extract_content_type(headers: &str) -> Option<String> {
            let re = regex::Regex::new(r#"Content-Type:\s*(.+?)(?:\r\n|\r|\n|$)"#).unwrap();
            re.captures(headers)
                .and_then(|cap| cap.get(1))
                .map(|m| m.as_str().trim().to_string())
        } 
        
        let mut form_map: HashMap<String, MultiFormField> = HashMap::new();
        
        // The boundary in the body is prefixed with "--"
        let boundary = format!("--{}", boundary);
        let boundary_bytes = boundary.as_bytes();
        let end_boundary = format!("{}--", boundary);
        let end_boundary_bytes = end_boundary.as_bytes();
        
        // Split the body by boundaries
        let mut parts: Vec<&[u8]> = Vec::new();
        let mut start_idx = 0;
        
        while let Some(idx) = find_subsequence(&body[start_idx..], boundary_bytes) {
            // Skip the first boundary or add the part if not the first
            if start_idx > 0 {
                parts.push(&body[start_idx..start_idx + idx - 2]); // -2 to remove trailing CRLF
            }
            
            // Move past this boundary
            start_idx += idx + boundary_bytes.len();
            
            // Check if this is the end boundary
            if start_idx < body.len() && 
            body.len() - start_idx >= 2 && 
            body[start_idx..start_idx+2] == [b'-', b'-'] {
                break; // End boundary found
            }
        } 
        
        // Process each part
        for part in parts {
            if part.len() < 4 { // Minimum size for valid part
                continue;
            }
            
            // Find headers and content separation (double CRLF)
            if let Some(header_end) = find_subsequence(part, b"\r\n\r\n") {
                let headers = &part[..header_end];
                let content = &part[header_end + 4..]; // +4 to skip the double CRLF
                
                // Parse headers as UTF-8 string
                if let Ok(headers_str) = std::str::from_utf8(headers) {
                    let name = extract_field_name(headers_str);
                    let filename = extract_filename(headers_str);
                    let content_type = extract_content_type(headers_str);
                    
                    if let Some(field_name) = name {
                        if let Some(filename) = filename {
                            match form_map.get_mut(&field_name){ 
                                Some(field) => { 
                                    field.insert_file(MultiFormFieldFile::new(Some(filename), content_type, content.to_vec()));
                                }, 
                                None => { 
                                    form_map.insert(field_name.clone(), MultiFormField::new_file(MultiFormFieldFile::new(Some(filename), content_type, content.to_vec()))); 
                                } 
                            } 
                        } else {
                            // This is a text field - try to convert to UTF-8
                            if let Ok(text_value) = std::str::from_utf8(content) {
                                form_map.insert(field_name, MultiFormField::Text(text_value.to_string()));
                            } else {
                                // Fallback for non-UTF-8 field content
                                form_map.insert(field_name.clone(), MultiFormField::new_file(MultiFormFieldFile::new( None, content_type, content.to_vec()))); 
                            }
                        }
                    }
                }
            }
        }
        
        Self::Files(form_map.into()) 
    } 
}

impl HttpRequest { 
    pub fn new(method: HttpMethod, path: String) -> Self { 
        Self { 
            start_line: RequestStartLine::new(HttpVersion::Http11, method.clone(), path.clone()), 
            header: RequestHeader::new(), 
            body: RequestBody::Empty, 
        }  
    } 

    pub async fn from_request_stream(
        stream: &mut TcpStream, 
        config: &ParseConfig, 
    ) -> Result<HttpRequest, Box<dyn Error + Send + Sync>> {
        let mut buf_reader = BufReader::new(stream);
        let mut headers = Vec::new();
        
        let mut total_header_size = 0; 
        loop { 
            let mut line = String::new();
            let bytes_read = buf_reader.read_line(&mut line)?;
            // println!("Read line: {}, buffer: {}", line, bytes_read); 
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

        if headers.is_empty() {
            return Err("Empty request".into());
        }

        let start_line = RequestStartLine::parse(headers.remove(0))?; 
        let mut header = RequestHeader::parse(headers); 

        let content_length = header.get_content_length().unwrap_or(0).min(config.max_body_size); 
        let mut body_buffer = vec![0; content_length];  
                
        let mut body = RequestBody::Empty; 
        if content_length != 0 { 
            buf_reader.read_exact(&mut body_buffer)?; 
            body = RequestBody::parse(body_buffer, &mut header); 
        } 
        
        Ok(HttpRequest {
            start_line, 
            header, 
            body 
        })
    } 

    pub fn get_path(&mut self, part: usize) -> String { 
        self.start_line.get_url().url_part(part) 
    } 

    pub fn path(&self) -> &str { 
        &self.start_line.path 
    } 

    /// Returns the parsed form from the request body if it exists. 
    /// If the body is not a form, it returns None. 
    pub fn form(&self) -> Option<&UrlEncodedForm> { 
        if let RequestBody::Form(ref data) = self.body { 
            Some(data) 
        } else { 
            None 
        } 
    } 

    /// Returns a reference to the form data if it exists, or an empty HashMap if it doesn't. 
    pub fn form_or_default(&self) -> &UrlEncodedForm {
        self.form().unwrap_or_else(|| {
            static EMPTY: Lazy<UrlEncodedForm> = Lazy::new(|| HashMap::new().into()); 
            &EMPTY
        })
    }

    /// Returns the request body as parsed MultiPartFormField if it exists. 
    /// If the body is not a multipart form, it returns None. 
    pub fn files(&self) -> Option<&MultiForm> { 
        if let RequestBody::Files(ref data) = self.body { 
            Some(data) 
        } else { 
            None 
        } 
    } 

    /// Returns a reference to the parsed files data if it exists, or an empty HashMap if it doesn't. 
    pub fn files_or_default(&self) -> &MultiForm {
        self.files().unwrap_or_else(|| {
            static EMPTY: Lazy<MultiForm> = Lazy::new(|| HashMap::new().into());
            &EMPTY
        })
    } 

    /// Returns the request body as parsed JSON if it exists. 
    /// If the body is not JSON, it returns None. 
    pub fn json(&self) -> Option<&Object> { 
        if let RequestBody::Json(ref data) = self.body { 
            Some(data) 
        } else { 
            None 
        } 
    } 

    /// Returns a reference to the parsed JSON data if it exists, or an empty Object if it doesn't. 
    pub fn json_or_default(&self) -> &Object {
        self.json().unwrap_or_else(|| {
            static EMPTY: Lazy<Object> = Lazy::new(|| Object::new(""));
            &EMPTY
        }) 
    } 

    pub fn method(&self) -> HttpMethod { 
        self.start_line.method.clone()  
    } 

    pub fn get_cookies(&mut self) -> &HashMap<String, String> { 
        self.header.get_cookies() 
    } 

    pub fn get_cookie(&mut self, key: &str) -> Option<String> { 
        self.header.get_cookie(key) 
    } 

    pub fn get_cookie_or_default(&mut self, key: &str) -> String { 
        self.header.get_cookie(key).unwrap_or("".to_string()) 
    } 

} 

impl Default for HttpRequest { 
    fn default() -> Self { 
        Self { 
            start_line: RequestStartLine::new(HttpVersion::Http11, HttpMethod::GET, "/".to_string()), 
            header: RequestHeader::new(), 
            body: RequestBody::Empty, 
        } 
    } 
} 
