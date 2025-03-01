use super::http_value::{self, *}; 
use std::collections::HashMap;
use std::net::TcpStream; 
use std::io::Write; 

pub struct ResponseStartLine{ 
    pub http_version: HttpVersion, 
    pub status_code: StatusCode,  
} 

impl std::fmt::Display for ResponseStartLine { 
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { 
        write!(f, "{} {}", self.http_version.to_string(), self.status_code.to_string()) 
    } 
} 

pub struct ResponseHeader{ 
    pub header: HashMap<String, String>, 
} 

impl ResponseHeader { 
    pub fn new() -> Self { 
        Self { header: HashMap::new() } 
    } 

    pub fn add(&mut self, key: String, value: String) { 
        self.header.insert(key, value); 
    } 

    pub fn set_content_length(&mut self, length: usize) { 
        self.add(String::from("CONTENT-LENGTH"), length.to_string()); 
    } 

    pub fn set_content_type(&mut self, content_type: HttpContentType) { 
        self.add(String::from("CONTENT-TYPE"), content_type.to_string()); 
    } 

    pub fn represent(&self) -> String { 
        let mut result = String::new(); 
        for (key, value) in &self.header { 
            result.push_str(&format!("{}: {}\r\n", key, value)); 
        } 
        result 
    } 
} 

pub struct HttpResponse { 
    pub start_line: ResponseStartLine, 
    pub header: ResponseHeader, 
    pub body: String, 
}  

impl HttpResponse { 
    pub fn new(start_line: ResponseStartLine, 
        header: ResponseHeader, 
        body: String
    ) -> Self { 
        Self { 
            start_line, 
            header, 
            body 
        } 
    } 

    pub fn set_content_length(mut self) -> Self { 
        self.header.set_content_length(self.body.len()); 
        self 
    }  

    pub async fn send(&self, stream: &mut TcpStream) { 
        let response = format!("{}\r\n{}\r\n{}", self.start_line, self.header.represent(), self.body); 
        // println!("{}", response); 
        stream.write_all(response.as_bytes()).unwrap(); 
    } 

    pub fn text_response(body: String) -> Self { 
        let start_line = ResponseStartLine { 
            http_version: HttpVersion::Http11, 
            status_code: StatusCode::OK, 
        }; 
        let mut header = ResponseHeader::new(); 
        header.set_content_type(HttpContentType::TextPlain); 
        Self::new(start_line, header, body).set_content_length() 
    } 

    pub fn normal_response(status_code: StatusCode, body: String) -> Self { 
        let start_line = ResponseStartLine { 
            http_version: HttpVersion::Http11, 
            status_code, 
        }; 
        let header = ResponseHeader::new(); 
        Self::new(start_line, header, body) 
    } 
} 

pub fn return_status(status_code: StatusCode) -> HttpResponse { 
    HttpResponse::normal_response(status_code, String::new()) 
} 
