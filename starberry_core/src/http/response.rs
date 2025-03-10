use super::http_value::{self, *}; 
use std::collections::HashMap;
use std::net::TcpStream; 
use std::io::{BufWriter, Write}; 

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
    pub body: Box<dyn AsRef<[u8]> + Send + Sync>, // Change to trait object
}  

impl HttpResponse { 
    pub fn new(
        start_line: ResponseStartLine, 
        header: ResponseHeader, 
        body: impl AsRef<[u8]> + Send + Sync + 'static, 
    ) -> Self { 
        Self { 
            start_line, 
            header, 
            body: Box::new(body), // Store as Box<dyn>
        } 
    } 

    pub fn set_content_length(mut self) -> Self { 
        self.header.set_content_length(self.body.as_ref().as_ref().len());  
        self 
    }  

    pub async fn send(&self, stream: &mut TcpStream) {
        let mut writer = BufWriter::new(stream);
    
        let start_line_bytes = format!("{}\r\n", self.start_line).into_bytes();
        let headers_bytes = format!("{}\r\n", self.header.represent()).into_bytes();
        let body_bytes = self.body.as_ref().as_ref();
    
        writer.write_all(&start_line_bytes).unwrap();
        writer.write_all(&headers_bytes).unwrap();
        writer.write_all(body_bytes).unwrap();
        
        writer.flush().unwrap(); // Ensure all data is sent
    } 
} 

pub mod request_templates {
    use std::path::Path;

    use crate::http::http_value::HttpContentType;
    use super::ResponseStartLine;  
    use super::ResponseHeader; 
    use crate::http::http_value::HttpVersion; 
    use crate::http::http_value::StatusCode; 
    use super::HttpResponse; 
 
    pub fn text_response(body: impl AsRef<[u8]> + Send + Sync + 'static) -> HttpResponse { 
        let start_line = ResponseStartLine { 
            http_version: HttpVersion::Http11, 
            status_code: StatusCode::OK, 
        }; 
        let mut header = ResponseHeader::new(); 
        header.set_content_type(HttpContentType::TextPlain); 
        HttpResponse::new(start_line, header, body).set_content_length() 
    } 

    pub fn html_response(body: impl AsRef<[u8]> + Send + Sync + 'static) -> HttpResponse { 
        let start_line = ResponseStartLine { 
            http_version: HttpVersion::Http11, 
            status_code: StatusCode::OK, 
        }; 
        let mut header = ResponseHeader::new(); 
        header.set_content_type(HttpContentType::TextHtml); 
        HttpResponse::new(start_line, header, body).set_content_length() 
    } 

    pub fn render_template(file: &str) -> HttpResponse { 
        let start_line = ResponseStartLine { 
            http_version: HttpVersion::Http11, 
            status_code: StatusCode::OK, 
        }; 
        let mut header = ResponseHeader::new(); 
        let file_path = Path::new("templates").join(file);
        let body = match std::fs::read(file_path) { 
            Ok(content) => content,
            Err(_) => return return_status(StatusCode::NOT_FOUND), 
        }; 
        header.set_content_type(HttpContentType::TextHtml); 
        HttpResponse::new(start_line, header, body).set_content_length() 
    } 

    pub fn normal_response(status_code: StatusCode, body: impl AsRef<[u8]> + Send + Sync + 'static) -> HttpResponse { 
        let start_line = ResponseStartLine { 
            http_version: HttpVersion::Http11, 
            status_code, 
        }; 
        let header = ResponseHeader::new(); 
        HttpResponse::new(start_line, header, body) 
    } 

    pub fn return_status<'a>(status_code: StatusCode) -> HttpResponse { 
        normal_response(status_code, "")
    } 
}

