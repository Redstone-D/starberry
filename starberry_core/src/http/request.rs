use super::http_value::*; 
use std::hash::Hash;
use std::io::{BufRead, BufReader, Read};
use std::error::Error;
use std::net::TcpStream;
use regex::Regex; 
use std::collections::HashMap; 

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
        Self { http_version, method, path } 
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
    
        Ok(Self { http_version, method, path }) 
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
} 

impl RequestHeader { 
    /// It is used to create a new RequestHeader object. 
    pub fn new() -> Self { 
        Self { header: HashMap::new() } 
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
        Self { header: header_map } 
    } 

    /// It is used to get the value of a header from the RequestHeader object. 
    pub fn content_length(&self) -> Option<usize> { 
        self.header.get("CONTENT-LENGTH") 
            .and_then(|s| s.parse::<usize>().ok()) 
    } 
} 

#[derive(Debug)]   
pub enum RequestBody{ 
    Text(String), 
    Binary(Vec<u8>), 
    Form(HashMap<String, String>), 
    Json(String), 
    Empty, 
} 

impl RequestBody{ 
    pub fn parse(body: Vec<u8>, header: &RequestHeader) -> Self { 
        let content_type = header.header.get("CONTENT-TYPE").unwrap_or(&String::new()).to_string(); 
        let content_length = header.content_length().unwrap_or(0); 

        if content_length == 0 { 
            return RequestBody::Empty; 
        } 

        if content_type.contains("application/json") { 
            return RequestBody::Json(String::from_utf8_lossy(&body).to_string()); 
        } else if content_type.contains("application/x-www-form-urlencoded") { 
            let form_data = String::from_utf8_lossy(&body).to_string(); 
            let mut form_map = HashMap::new(); 
            for pair in form_data.split('&') { 
                let parts: Vec<&str> = pair.split('=').collect(); 
                if parts.len() == 2 { 
                    form_map.insert(parts[0].to_string(), parts[1].to_string()); 
                } 
            } 
            return RequestBody::Form(form_map); 
        } else if content_type.contains("text/plain") { 
            return RequestBody::Text(String::from_utf8_lossy(&body).to_string()); 
        } else { 
            return RequestBody::Binary(body); 
        }  
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
        let header = RequestHeader::parse(headers); 

        let content_length = header.content_length().unwrap_or(0).min(config.max_body_size); 
        let mut body_buffer = vec![0; content_length];  
                
        let mut body = RequestBody::Empty; 
        if content_length != 0 { 
            buf_reader.read_exact(&mut body_buffer)?; 
            body = RequestBody::parse(body_buffer, &header); 
        } 
        
        Ok(HttpRequest {
            start_line, 
            header, 
            body 
        })
    } 

    pub fn path(&self) -> &str { 
        &self.start_line.path 
    } 

    pub fn form(&self) -> Option<&HashMap<String, String>> { 
        if let RequestBody::Form(ref data) = self.body { 
            Some(data) 
        } else { 
            None 
        } 
    } 

    pub fn method(&self) -> &HttpMethod { 
        &self.start_line.method 
    } 
} 
