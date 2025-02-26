use super::http_value::*; 
use std::net::TcpStream; 
use std::io::Write; 


pub struct HttpResponse { 
    pub http_version: HttpVersion, 
    pub status_code: StatusCode, 
    pub body: String, 
}  

impl HttpResponse { 
    pub fn new(http_version: HttpVersion, 
        status_code: StatusCode, 
        body: String
    ) -> Self { 
        Self { http_version, status_code, body } 
    } 

    pub async fn send(&self, stream: &mut TcpStream) { 
        let response = format!("{} {} \r\n\r\n{}", self.http_version.to_string(), self.status_code.to_string(), self.body); 
        // println!("{}", response); 
        stream.write_all(response.as_bytes()).unwrap(); 
    } 
} 

pub fn abrupt(status_code: StatusCode) -> HttpResponse { 
    HttpResponse::new(HttpVersion::Http11, status_code, String::from("")) 
} 
