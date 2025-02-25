use super::http_value::*; 
use std::io::{BufReader, BufRead};
use std::error::Error;
use std::net::TcpStream;
use regex::Regex; 

pub struct HttpRequest { 
    pub method: HttpMethod, 
    pub path: String, 
} 

impl HttpRequest { 
    pub fn new(method: HttpMethod, path: String) -> Self { 
        Self { method, path } 
    } 

    pub fn from_request_stream(request_stream: &TcpStream) -> Result<Self, Box<dyn Error>> {
        let buf_reader = BufReader::new(request_stream); 
        let mut request_lines = buf_reader.lines(); 
        let request_line = request_lines.next().unwrap()?; // Need to be changed. called `Option::unwrap()` on a `None` value 
        let parts: Vec<&str> = request_line.split_whitespace().collect(); 
        let method = HttpMethod::from_string(parts[0]); 
        let path = parts[1].to_string(); 
        for line in request_lines { 
            let line = line?; 
            println!("{}//", line); 
            if line == "" { 
                break; 
            } 
        } 
        Ok(Self { method, path }) 
    } 

    pub fn url_match(&self, reurl: &str) -> bool { 
        let re = Regex::new(reurl).unwrap(); 
        re.is_match(&self.path) 
    } 
} 

