use super::cookie::{Cookie, CookieMap};
use super::http_value::{self, *}; 
use super::body::HttpBody; 
use std::collections::HashMap;
use std::fmt::Write;
use tokio::net::TcpStream; 
use tokio::io::{AsyncWriteExt, BufWriter}; 

use std::future::{ready, Ready};
use std::pin::Pin;
use std::future::Future; 

pub struct ResponseStartLine{ 
    pub http_version: HttpVersion, 
    pub status_code: StatusCode,  
} 

impl std::fmt::Display for ResponseStartLine { 
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { 
        write!(f, "{} {}", self.http_version.to_string(), self.status_code.to_string()) 
    } 
} 

/// The response header struct, 
pub struct ResponseHeader{ 
    content_type: Option<HttpContentType>, 
    content_length: Option<usize>, 
    location: Option<String>, 
    cookie: Option<CookieMap>, 
    pub header: HashMap<String, String>, 
} 

impl ResponseHeader { 
    pub fn new() -> Self { 
        Self { 
            content_type: None, 
            content_length: None, 
            location: None, 
            cookie: None, 
            header: HashMap::new()
        } 
    } 

    /// Add a header to the response header. 
    /// Existed header will be replaced. 
    pub fn add(&mut self, key: String, value: String) { 
        match key.as_str() { 
            "Content-Type" => self.content_type = Some(HttpContentType::from_str(&value)), 
            "Content-Length" => self.content_length = Some(value.parse::<usize>().unwrap()), 
            "Location" => self.location = Some(value), 
            _ => { 
                self.header.insert(key, value); 
            },
        } 
    } 

    pub fn get_content_length(&self) -> Option<usize> { 
        self.content_length 
    } 

    pub fn set_content_length(&mut self, length: usize) { 
        self.content_length = Some(length); 
    } 

    pub fn clear_content_length(&mut self) { 
        self.content_length = None; 
    } 

    pub fn get_content_type(&self) -> Option<HttpContentType> { 
        self.content_type.clone() 
    } 

    pub fn set_content_type(&mut self, content_type: HttpContentType) { 
        self.content_type = Some(content_type);  
    } 

    pub fn clear_content_type(&mut self) { 
        self.content_type = None; 
    } 

    pub fn set_location(&mut self, location: &str) { 
        self.location = Some(location.to_string());  
    } 

    pub fn clear_location(&mut self) { 
        self.location = None; 
    } 

    pub fn cookie<T: Into<String>>(mut self, key: T, cookie: Cookie) -> Self { 
        self.add_cookie(key, cookie); 
        self 
    } 

    /// Add a cookie to the response header. 
    pub fn add_cookie<T: Into<String>>(&mut self, key: T, cookie: Cookie) { 
        if self.cookie.is_none() { 
            self.cookie = Some(CookieMap::new()); 
        } 
        if let Some(ref mut cookies) = self.cookie { 
            cookies.set(key, cookie); 
        } 
    } 

    /// Clear all cookies in the response header. 
    pub fn clear_cookie(&mut self) { 
        self.cookie = None; 
    } 

    pub fn remove_cookie<T: AsRef<str>>(&mut self, name: T) -> Option<Cookie> { 
        if let Some(ref mut cookies) = self.cookie { 
            return cookies.remove(name); 
        } 
        None 
    } 

    pub fn represent(&self) -> String { 
        let mut result = String::new(); 
        if let Some(ref content_type) = self.content_type { 
            result.push_str(&format!("Content-Type: {}\r\n", content_type.to_string())); 
        }
        if let Some(length) = self.content_length { 
            result.push_str(&format!("Content-Length: {}\r\n", length)); 
        } 
        if let Some(ref location) = self.location { 
            result.push_str(&format!("Location: {}\r\n", location)); 
        } 
        if let Some(ref cookies) = self.cookie { 
            result.push_str(&format!("{}\r\n", cookies.response()));  
        } 
        for (key, value) in &self.header { 
            result.push_str(&format!("{}: {}\r\n", key, value)); 
        } 
        result 
    } 
} 

pub struct HttpResponse { 
    pub start_line: ResponseStartLine, 
    pub header: ResponseHeader, 
    pub body: HttpBody 
}  

impl HttpResponse { 
    pub fn new(
        start_line: ResponseStartLine, 
        header: ResponseHeader, 
        body: HttpBody, 
    ) -> Self { 
        Self { 
            start_line, 
            header, 
            body, 
        } 
    } 

    pub fn add_cookie<T: Into<String>>(mut self, key: T, cookie: Cookie) -> Self { 
        self.header.add_cookie(key, cookie); 
        self 
    } 

    pub async fn send(&mut self, stream: &mut TcpStream) -> std::io::Result<()> {
        let mut writer = BufWriter::new(stream);
        let mut headers = String::with_capacity(256); 
        let bin = self.body.into_static(&mut self.header).await; 
    
        write!(
            &mut headers,
            "{}\r\n\
            {}\r\n",
            self.start_line,
            self.header.represent()
        ).map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
    
        writer.write_all(headers.as_bytes()).await?;
        writer.write_all(bin).await?; 

        // println!("{}, {:?}", headers, bin); 
        writer.flush().await?; 
        
        Ok(()) 
    } 

    // /// Converts this response into a Future that resolves to itself.
    // /// Useful for middleware functions that need to return a Future<Output = HttpResponse>.
    // pub fn future(self) -> impl Future<Output = HttpResponse> + Send {
    //     ready(self)
    // }

    // /// Creates a boxed future from this response (useful for trait objects).
    // pub fn boxed_future(self) -> Pin<Box<dyn Future<Output = HttpResponse> + Send>> {
    //     Box::pin(self.future())
    // } 
} 

impl Default for HttpResponse { 
    fn default() -> Self { 
        let start_line = ResponseStartLine { 
            http_version: HttpVersion::Http11, 
            status_code: StatusCode::OK, 
        }; 
        let header = ResponseHeader::new(); 
        let body = HttpBody::default(); // Default body is empty string
        HttpResponse::new(start_line, header, body) 
    } 
} 

pub mod request_templates {
    use std::path::Path; 
    use std::collections::HashMap; 

    use akari::Object;
    use akari::TemplateManager;

    use crate::http::body::HttpBody;
    use crate::http::http_value::HttpContentType;
    use super::ResponseStartLine;  
    use super::ResponseHeader; 
    use crate::http::http_value::HttpVersion; 
    use crate::http::http_value::StatusCode; 
    use super::HttpResponse; 
 
    pub fn text_response(body: impl Into<String>) -> HttpResponse { 
        let start_line = ResponseStartLine { 
            http_version: HttpVersion::Http11, 
            status_code: StatusCode::OK, 
        }; 
        let mut header = ResponseHeader::new(); 
        header.set_content_type(HttpContentType::TextPlain()); 
        HttpResponse::new(start_line, header, HttpBody::Text(body.into())) 
    } 

    pub fn html_response(body: impl Into<Vec<u8>>) -> HttpResponse { 
        let start_line = ResponseStartLine { 
            http_version: HttpVersion::Http11, 
            status_code: StatusCode::OK, 
        }; 
        let mut header = ResponseHeader::new(); 
        header.set_content_type(HttpContentType::TextHtml()); 
        HttpResponse::new(start_line, header, HttpBody::Binary(body.into())) 
    } 

    pub fn redirect_response(url: &str) -> HttpResponse { 
        let start_line = ResponseStartLine { 
            http_version: HttpVersion::Http11, 
            status_code: StatusCode::FOUND, 
        }; 
        let mut header = ResponseHeader::new(); 
        header.set_location(url); 
        HttpResponse::new(start_line, header, HttpBody::Empty) 
    } 

    pub fn plain_template_response(file: &str) -> HttpResponse { 
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
        header.set_content_type(HttpContentType::TextHtml()); 
        HttpResponse::new(start_line, header, HttpBody::Binary(body)) 
    } 

    pub fn normal_response(status_code: StatusCode, body: impl Into<Vec<u8>>) -> HttpResponse { 
        let start_line = ResponseStartLine { 
            http_version: HttpVersion::Http11, 
            status_code, 
        }; 
        let header = ResponseHeader::new(); 
        HttpResponse::new(start_line, header, HttpBody::Binary(body.into())) 
    } 

    pub fn json_response(body: Object) -> HttpResponse { 
        let start_line = ResponseStartLine { 
            http_version: HttpVersion::Http11, 
            status_code: StatusCode::OK, 
        }; 
        let mut header = ResponseHeader::new(); 
        header.set_content_type(HttpContentType::ApplicationJson()); 
        HttpResponse::new(start_line, header, HttpBody::Json(body)) 
    } 

    pub fn template_response(file: &str, data: HashMap<String, Object>) -> HttpResponse { 
        // println!("file: {:?}", file); 
        let template_manager = TemplateManager::new("templates");
        let result = match template_manager.render(file, &data){ 
            Ok(content) => content,
            Err(err) => return text_response(err.to_string()),  
        }; 
        let start_line = ResponseStartLine { 
            http_version: HttpVersion::Http11, 
            status_code: StatusCode::OK, 
        }; 
        let mut header = ResponseHeader::new(); 
        header.set_content_type(HttpContentType::TextHtml()); 
        // println!("body: {:?}", result);
        let body = result.into_bytes(); 
        HttpResponse::new(start_line, header, HttpBody::Binary(body)) 
    }

    pub fn return_status<'a>(status_code: StatusCode) -> HttpResponse { 
        normal_response(status_code, "")
    } 
}

// pub mod akari_templates { 
//     /// This macro is used to create a template response with the given path and key-value pairs. 
//     /// It renders a template within specified path 
//     /// and inserts the key-value pairs into the template context. 
//     /// It is a convenient way to generate dynamic HTML responses. 
//     /// # Examples 
//     /// ```rust 
//     /// // akari_render!("/path/to/template", key1 = "value1", key2 = "value2"); 
//     /// // This will fail because the template does not exist. 
//     /// ``` 
//     #[macro_export]
//     macro_rules! akari_render {
//         ($path:expr) => {{
//             template_response($path, std::collections::HashMap::new())
//         }}; 
//         ($path:expr, $($key:ident = $value:tt),* $(,)?) => {{
//             let mut map = std::collections::HashMap::new();
//             $(
//                 akari_render!(@insert map, $key = $value);
//             )*
//             template_response($path, map)
//         }};
//         (@insert $map:expr, $key:ident = $value:literal) => {
//             $map.insert(stringify!($key).to_string(), object!($value));
//         };
//         (@insert $map:expr, $key:ident = $value:expr) => {
//             $map.insert(stringify!($key).to_string(), $value);
//         };
//     }     
// } 

// pub mod akari_object { 
//     /// This macro is used to create a JSON response with the given key-value pairs. 
//     /// It is a convenient way to generate JSON responses. 
//     #[macro_export]
//     macro_rules! akari_json {
//         // Forward any input to the object! macro and wrap the result in json_response
//         ($($tokens:tt)*) => {{
//             let obj = object!($($tokens)*);
//             json_response(obj)
//         }};
//     } 
// }
