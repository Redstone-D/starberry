use super::http_value::*; 
use std::io::{BufRead, BufReader};
use std::error::Error;
use std::net::TcpStream;
use std::sync::Arc;
use regex::Regex; 
use std::collections::HashMap; 
use tokio::sync::Mutex; 

#[derive(Debug)]  
pub struct HttpRequest { 
    pub method: HttpMethod, 
    pub path: String, 
    pub header: HashMap<String, String>, 
} 

impl HttpRequest { 
    pub fn new(method: HttpMethod, path: String) -> Self { 
        Self { method, path, header: HashMap::new() } 
    } 

    pub async fn from_request_stream(
        stream: &mut TcpStream,
    ) -> Result<Self, Box<dyn Error + Send + Sync>> {
        let mut buf_reader = BufReader::new(stream);
        let mut headers = Vec::new();

        // Read header lines until an empty line is encountered.
        loop {
            let mut line = String::new();
            let bytes_read = buf_reader.read_line(&mut line)?;
            if bytes_read == 0 || line.trim().is_empty() {
                break;
            }
            // println!("{}", line.trim());
            headers.push(line.trim().to_string());
        }

        if headers.is_empty() {
            return Err("Empty request".into());
        }

        // The first header line is the request line.
        let request_line = headers.remove(0);
        println!("{:?}", request_line);
        let parts: Vec<&str> = request_line.split_whitespace().collect();
        if parts.len() < 2 {
            return Err("Malformed request line".into());
        }

        let method = HttpMethod::from_string(parts[0]);
        let path = parts[1].to_string();

        let mut header_map = HashMap::new();
        for line in headers {
            let parts: Vec<&str> = line.splitn(2, ':').collect();
            if parts.len() == 2 {
                let key = parts[0].trim().to_string();
                let value = parts[1].trim().to_string();
                header_map.insert(key, value);
            }
        }

        Ok(HttpRequest {
            method,
            path,
            header: header_map,
        })
    } 

    pub fn url_match(&self, reurl: &str) -> bool { 
        let re = Regex::new(reurl).unwrap(); 
        re.is_match(&self.path) 
    } 
} 

