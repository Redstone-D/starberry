use super::http_value::*; 
use std::io::{BufRead, BufReader, Read};
use std::error::Error;
use std::net::TcpStream;
use regex::Regex; 
use std::collections::HashMap; 

#[derive(Debug)]  
pub struct HttpRequest { 
    pub method: HttpMethod, 
    pub path: String, 
    pub header: HashMap<String, String>, 
    pub body: Option<String>, 
} 

impl HttpRequest { 
    pub fn new(method: HttpMethod, path: String) -> Self { 
        Self { method, path, header: HashMap::new(), body: None } 
    } 

    pub fn content_length(&self) -> Option<usize> { 
        self.header.get("CONTENT-LENGTH") 
            .and_then(|s| s.parse::<usize>().ok()) 
    } 

    pub async fn from_request_stream(
        stream: &mut TcpStream,
    ) -> Result<HttpRequest, Box<dyn Error + Send + Sync>> {
        let mut buf_reader = BufReader::new(stream);
        let mut headers = Vec::new();
        
        // Read headers until an empty line is encountered
        loop {
            let mut line = String::new();
            let bytes_read = buf_reader.read_line(&mut line)?;
            if bytes_read == 0 || line.trim().is_empty() {
                break;
            }
            // Trim the line to handle cases without \r\n
            headers.push(line.trim().to_string());
        }
    
        if headers.is_empty() {
            return Err("Empty request".into());
        }
    
        // Parse request line (method and path)
        let request_line = headers.remove(0);
        let parts: Vec<&str> = request_line.split_whitespace().collect();
        if parts.len() < 2 {
            return Err("Malformed request line".into());
        }
    
        let method = HttpMethod::from_string(parts[0]);
        let path = parts[1].to_string();
    
        // Parse headers
        let mut header_map = HashMap::new(); 
        for line in headers {
            let parts: Vec<&str> = line.splitn(2, ':').collect();
            if parts.len() == 2 {
                let key = parts[0].trim().to_uppercase().to_string();
                let value = parts[1].trim().to_string();
                header_map.insert(key, value);
            }
        }
    
        let content_length = header_map.get("CONTENT-LENGTH").and_then(|s| s.parse::<usize>().ok()); 
        // Now, if the request has a body, let's try to read it
        let body = if let Some(len) = content_length {
            let mut body_buffer = vec![0; len];
            buf_reader.read_exact(&mut body_buffer)?;
    
            // Check if body length matches Content-Length
            if body_buffer.len() != len {
                return Err("Content-Length mismatch".into());
            }
    
            Some(String::from_utf8_lossy(&body_buffer).to_string())
        } else {
            None // No body
        };
    
        Ok(HttpRequest {
            method,
            path,
            header: header_map,
            body,
        })
    }  

    pub fn url_match(&self, reurl: &str) -> bool { 
        let re = Regex::new(reurl).unwrap(); 
        re.is_match(&self.path) 
    } 
} 

