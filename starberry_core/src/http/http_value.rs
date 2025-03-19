#![allow(non_snake_case)] 
#![allow(non_camel_case_types)] 

use std::collections::HashMap;

#[derive(Debug, Clone)]  
pub enum HttpVersion { 
    Http09,
    Http10,
    Http11,
    Http20,
    Http30, 
    Unknown, 
} 

impl HttpVersion { 
    pub fn to_string(&self) -> String { 
        match self { 
            HttpVersion::Http09 => "HTTP/0.9".to_string(), 
            HttpVersion::Http10 => "HTTP/1.0".to_string(), 
            HttpVersion::Http11 => "HTTP/1.1".to_string(), 
            HttpVersion::Http20 => "HTTP/2.0".to_string(), 
            HttpVersion::Http30 => "HTTP/3.0".to_string(), 
            _ => "UNKNOWN".to_string(), 
        } 
    } 

    pub fn from_string(version: &str) -> Self { 
        match version { 
            "HTTP/0.9" => HttpVersion::Http09, 
            "HTTP/1.0" => HttpVersion::Http10, 
            "HTTP/1.1" => HttpVersion::Http11, 
            "HTTP/2.0" => HttpVersion::Http20, 
            "HTTP/3.0" => HttpVersion::Http30, 
            _ => HttpVersion::Unknown,  
        }  
    }  
} 

impl std::fmt::Display for HttpVersion { 
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { 
        write!(f, "{}", self.to_string()) 
    } 
} 

#[derive(Debug, Clone, PartialEq)] 
pub enum HttpMethod { 
    GET, 
    POST, 
    PUT, 
    DELETE, 
    HEAD, 
    OPTIONS, 
    PATCH, 
    TRACE, 
    CONNECT, 
    UNKNOWN, 
} 

impl HttpMethod { 
    pub fn to_string(&self) -> String { 
        match self { 
            HttpMethod::GET => "GET".to_string(), 
            HttpMethod::POST => "POST".to_string(), 
            HttpMethod::PUT => "PUT".to_string(), 
            HttpMethod::DELETE => "DELETE".to_string(), 
            HttpMethod::HEAD => "HEAD".to_string(), 
            HttpMethod::OPTIONS => "OPTIONS".to_string(), 
            HttpMethod::PATCH => "PATCH".to_string(), 
            HttpMethod::TRACE => "TRACE".to_string(), 
            HttpMethod::CONNECT => "CONNECT".to_string(), 
            _ => "UNKNOWN".to_string(), 
        } 
    } 

    pub fn from_string(method: &str) -> Self { 
        match method { 
            "GET" => HttpMethod::GET, 
            "POST" => HttpMethod::POST, 
            "PUT" => HttpMethod::PUT, 
            "DELETE" => HttpMethod::DELETE, 
            "HEAD" => HttpMethod::HEAD, 
            "OPTIONS" => HttpMethod::OPTIONS, 
            "PATCH" => HttpMethod::PATCH, 
            "TRACE" => HttpMethod::TRACE, 
            "CONNECT" => HttpMethod::CONNECT, 
            _ => HttpMethod::UNKNOWN,  
        }  
    }  
} 

impl std::fmt::Display for HttpMethod { 
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { 
        write!(f, "{}", self.to_string()) 
    } 
} 

pub enum StatusCode { 
    OK = 200, 
    CREATED = 201, 
    ACCEPTED = 202, 
    NO_CONTENT = 204, 
    MOVED_PERMANENTLY = 301, 
    FOUND = 302, 
    NOT_MODIFIED = 304, 
    BAD_REQUEST = 400, 
    UNAUTHORIZED = 401, 
    FORBIDDEN = 403, 
    NOT_FOUND = 404, 
    INTERNAL_SERVER_ERROR = 500, 
    NOT_IMPLEMENTED = 501, 
    BAD_GATEWAY = 502, 
    SERVICE_UNAVAILABLE = 503,  
    GATEWAY_TIMEOUT = 504, 
} 

impl StatusCode { 
    pub fn to_string(&self) -> String { 
        match self { 
            StatusCode::OK => "200 OK".to_string(), 
            StatusCode::CREATED => "201 Created".to_string(), 
            StatusCode::ACCEPTED => "202 Accepted".to_string(), 
            StatusCode::NO_CONTENT => "204 No Content".to_string(), 
            StatusCode::MOVED_PERMANENTLY => "301 Moved Permanently".to_string(), 
            StatusCode::FOUND => "302 Found".to_string(), 
            StatusCode::NOT_MODIFIED => "304 Not Modified".to_string(), 
            StatusCode::BAD_REQUEST => "400 Bad Request".to_string(), 
            StatusCode::UNAUTHORIZED => "401 Unauthorized".to_string(), 
            StatusCode::FORBIDDEN => "403 Forbidden".to_string(), 
            StatusCode::NOT_FOUND => "404 Not Found".to_string(), 
            StatusCode::INTERNAL_SERVER_ERROR => "500 Internal Server Error".to_string(), 
            StatusCode::NOT_IMPLEMENTED => "501 Not Implemented".to_string(), 
            StatusCode::BAD_GATEWAY => "502 Bad Gateway".to_string(), 
            StatusCode::SERVICE_UNAVAILABLE => "503 Service Unavailable".to_string(), 
            StatusCode::GATEWAY_TIMEOUT => "504 Gateway Timeout".to_string(),  
        } 
    } 

    pub fn to_u16(&self) -> u16 { 
        match self { 
            StatusCode::OK => 200, 
            StatusCode::CREATED => 201, 
            StatusCode::ACCEPTED => 202, 
            StatusCode::NO_CONTENT => 204, 
            StatusCode::MOVED_PERMANENTLY => 301, 
            StatusCode::FOUND => 302, 
            StatusCode::NOT_MODIFIED => 304, 
            StatusCode::BAD_REQUEST => 400, 
            StatusCode::UNAUTHORIZED => 401, 
            StatusCode::FORBIDDEN => 403, 
            StatusCode::NOT_FOUND => 404, 
            StatusCode::INTERNAL_SERVER_ERROR => 500, 
            StatusCode::NOT_IMPLEMENTED => 501, 
            StatusCode::BAD_GATEWAY => 502, 
            StatusCode::SERVICE_UNAVAILABLE => 503,  
            StatusCode::GATEWAY_TIMEOUT => 504,  
        } 
    } 

    pub fn from_u16(code: u16) -> Self { 
        match code { 
            200 => StatusCode::OK, 
            201 => StatusCode::CREATED, 
            202 => StatusCode::ACCEPTED, 
            204 => StatusCode::NO_CONTENT, 
            301 => StatusCode::MOVED_PERMANENTLY, 
            302 => StatusCode::FOUND, 
            304 => StatusCode::NOT_MODIFIED, 
            400 => StatusCode::BAD_REQUEST, 
            401 => StatusCode::UNAUTHORIZED, 
            403 => StatusCode::FORBIDDEN, 
            404 => StatusCode::NOT_FOUND, 
            500 => StatusCode::INTERNAL_SERVER_ERROR, 
            501 => StatusCode::NOT_IMPLEMENTED, 
            502 => StatusCode::BAD_GATEWAY, 
            503 => StatusCode::SERVICE_UNAVAILABLE,  
            _ => StatusCode::INTERNAL_SERVER_ERROR, 
        } 
    } 

    pub fn from_string(code: &str) -> Self { 
        match code { 
            "200 OK" => StatusCode::OK, 
            "201 Created" => StatusCode::CREATED, 
            "202 Accepted" => StatusCode::ACCEPTED, 
            "204 No Content" => StatusCode::NO_CONTENT, 
            "301 Moved Permanently" => StatusCode::MOVED_PERMANENTLY, 
            "302 Found" => StatusCode::FOUND, 
            "304 Not Modified" => StatusCode::NOT_MODIFIED, 
            "400 Bad Request" => StatusCode::BAD_REQUEST, 
            "401 Unauthorized" => StatusCode::UNAUTHORIZED, 
            "403 Forbidden" => StatusCode::FORBIDDEN, 
            "404 Not Found" => StatusCode::NOT_FOUND, 
            "500 Internal Server Error" => StatusCode::INTERNAL_SERVER_ERROR, 
            "501 Not Implemented" => StatusCode::NOT_IMPLEMENTED, 
            "502 Bad Gateway" => StatusCode::BAD_GATEWAY, 
            "503 Service Unavailable" => StatusCode::SERVICE_UNAVAILABLE,  
            _ => StatusCode::INTERNAL_SERVER_ERROR, 
        } 
    } 
} 

/// Represents the content type of an HTTP message. 
/// This enum is used to parse and construct HTTP headers related to content type. 
/// It includes well-known content types like text, application, image, audio, video, and multipart. 
/// It also includes a generic Other variant for any other content types. 
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HttpContentType {
    // Well-known content types
    Text { subtype: String, charset: Option<String> },
    Application { subtype: String, parameters: Option<Vec<(String, String)>> },
    Image { subtype: String },
    Audio { subtype: String },
    Video { subtype: String },
    Multipart { subtype: String, boundary: Option<String> },
    Other { type_name: String, subtype: String, parameters: Option<Vec<(String, String)>> },
} 

impl HttpContentType {
    /// Converts a string into an HttpContentType enum variant 
    /// This function parses the content type string and extracts the main type, subtype, and any parameters.
    /// It supports well-known content types like text, application, image, audio, video, and multipart. ]
    /// 
    /// # Examples 
    /// 
    /// ```rust 
    /// use starberry_core::http::http_value::HttpContentType; 
    /// let content_type = HttpContentType::from_str("text/html; charset=UTF-8"); 
    /// assert_eq!(content_type, HttpContentType::Text { subtype: "html".to_string(), charset: Some("UTF-8".to_string()) }); 
    /// ``` 
    pub fn from_str(content_type: &str) -> Self {
        let parts: Vec<&str> = content_type.split(';').collect();
        let main_part = parts[0].trim();
        let mut parameters = Vec::new();

        for part in &parts[1..] {
            let param_parts: Vec<&str> = part.split('=').collect();
            if param_parts.len() == 2 {
                parameters.push((param_parts[0].trim().to_string(), param_parts[1].trim().to_string()));
            }
        }

        let (type_name, subtype) = if let Some(pos) = main_part.find('/') {
            (&main_part[..pos], &main_part[pos + 1..])
        } else {
            ("unknown", "unknown")
        };

        match type_name {
            "text" => Self::Text { 
                subtype: subtype.to_string(), 
                charset: Self::find_value_from_vec(&parameters, "charset"),
            }, 
            "application" => Self::Application { 
                subtype: subtype.to_string(), 
                parameters: Some(parameters) 
            },
            "image" => Self::Image { 
                subtype: subtype.to_string() 
            },
            "audio" => Self::Audio { 
                subtype: subtype.to_string() 
            },
            "video" => Self::Video { 
                subtype: subtype.to_string() 
            },
            "multipart" => Self::Multipart { 
                subtype: subtype.to_string(), 
                boundary: Self::find_value_from_vec(&parameters, "boundary"),  
            },
            _ => Self::Other {
                type_name: type_name.to_string(), 
                subtype: subtype.to_string(), 
                parameters: Some(parameters) 
            },
        } 
    } 

    /// Find value from Vec<(String, String)> 
    /// # Examples 
    /// ```rust 
    /// use starberry_core::http::http_value::HttpContentType; 
    /// let vec = vec![("key1".to_string(), "value1".to_string()), ("key2".to_string(), "value2".to_string())]; 
    /// let value = HttpContentType::find_value_from_vec(&vec, "key1"); 
    /// assert_eq!(value, Some("value1".to_string())); 
    /// ``` 
    pub fn find_value_from_vec(vec: &Vec<(String, String)>, key: &str) -> Option<String> { 
        for (k, v) in vec { 
            if k == key { 
                return Some(v.clone()); 
            } 
        } 
        None 
    } 

    /// Converts an HttpContentType enum variant into its string representation
    pub fn to_string(&self) -> String {
        match self {
            HttpContentType::Text { subtype, .. } => format!("text/{}", subtype),
            HttpContentType::Application { subtype, .. } => format!("application/{}", subtype),
            HttpContentType::Image { subtype } => format!("image/{}", subtype),
            HttpContentType::Audio { subtype } => format!("audio/{}", subtype),
            HttpContentType::Video { subtype } => format!("video/{}", subtype),
            HttpContentType::Multipart { subtype, .. } => format!("multipart/{}", subtype),
            HttpContentType::Other { type_name, subtype, .. } => format!("{}/{}", type_name, subtype),
        }
    } 

    pub fn TextHtml() -> Self { 
        Self::Text { subtype: "html".to_string(), charset: Some("UTF-8".to_string()) } 
    } 

    pub fn TextPlain() -> Self { 
        Self::Text { subtype: "plain".to_string(), charset: Some("UTF-8".to_string()) } 
    } 

    pub fn ApplicationJson() -> Self { 
        Self::Application { subtype: "json".to_string(), parameters: Some(vec![("charset".to_string(), "UTF-8".to_string())]) } 
    } 

    pub fn ApplicationXml() -> Self { 
        Self::Application { subtype: "xml".to_string(), parameters: Some(vec![("charset".to_string(), "UTF-8".to_string())]) } 
    } 

    pub fn ApplicationOctetStream() -> Self { 
        Self::Application { subtype: "octet-stream".to_string(), parameters: Some(vec![("charset".to_string(), "UTF-8".to_string())]) } 
    } 
}

impl std::fmt::Display for HttpContentType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string())
    }
} 

pub struct HeaderConstructor{ 
    pub headers: Vec<HeaderAttribute>
} 

impl HeaderConstructor{ 
    pub fn build<T: Into<String>>(string: T) -> Self { 
        let mut headers = Vec::new(); 
        let string = string.into(); 
        let parts: Vec<&str> = string.split(';').collect(); 
        for part in parts { 
            let part = part.trim(); 
            if !part.is_empty() { 
                headers.push(HeaderAttribute::build(part)); 
            } 
        } 
        Self { headers } 
    }
}

pub struct HeaderAttribute{ 
    pub main_value: String, 
    pub attributes: HashMap<String, String>, 
} 

impl HeaderAttribute{ 
    pub fn build<T: Into<String>>(part: T) -> Self{ 
        let part = part.into(); 
        let mut attributes = HashMap::new(); 
        let main_value = part.split(':').next().unwrap_or("").trim().to_string(); 
        for attr in part.split(';').skip(1) { 
            let attr_parts: Vec<&str> = attr.split('=').collect(); 
            if attr_parts.len() == 2 { 
                attributes.insert(attr_parts[0].trim().to_string(), attr_parts[1].trim().to_string()); 
            } 
        } 
        Self { main_value, attributes } 
    }
} 

/// Represents a field in a multipart form.
#[derive(Debug, Clone)]
pub enum MultiFormField {
    Text(String),
    File(Vec<MultiFormFieldFile>)
} 

/// Represents a file in a multipart form. 
#[derive(Debug, Clone)]
pub struct MultiFormFieldFile {
    filename: Option<String>,
    content_type: Option<String>, 
    data: Vec<u8>,
} 

impl MultiFormField { 
    pub fn new_text(value: String) -> Self {
        Self::Text(value) 
    } 
    
    pub fn new_file(files: MultiFormFieldFile) -> Self {
        Self::File(vec![files])  
    } 

    /// Creates a new MultiFormField with a file. 
    /// This function takes a filename, content type, and data as parameters. 
    /// It returns a MultiFormField::File variant. 
    /// When the Field is Text type, it will change it into a File type. 
    pub fn insert_file(&mut self, file: MultiFormFieldFile) {
        if let Self::File(files) = self {
            files.push(file); 
        } else {
            *self = Self::File(vec![file]); 
        }
    }    

    /// Gets the files value from the MultiFormField. 
    pub fn get_files(&self) -> Option<&Vec<MultiFormFieldFile>> {
        if let Self::File(files) = self {
            Some(files) 
        } else {
            None 
        } 
    } 
}

impl MultiFormFieldFile{ 
    pub fn new(filename: Option<String>, content_type: Option<String>, data: Vec<u8>) -> Self { 
        Self { filename, content_type, data } 
    } 

    pub fn filename(&self) -> Option<String> { 
        self.filename.clone() 
    } 

    pub fn content_type(&self) -> Option<String> { 
        self.content_type.clone() 
    } 

    pub fn data(&self) -> &[u8] { 
        &self.data 
    } 
} 
