#![allow(non_snake_case)] 
#![allow(non_camel_case_types)] 

use std::{collections::HashMap, hash::Hash};

use once_cell::sync::Lazy;

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

impl PartialEq<&HttpMethod> for HttpMethod { 
    fn eq(&self, other: &&HttpMethod) -> bool { 
        self == *other 
    } 
} 

impl PartialEq<HttpMethod> for &HttpMethod {
    fn eq(&self, other: &HttpMethod) -> bool {
        **self == *other
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
    METHOD_NOT_ALLOWED = 405, 
    UNSUPPORTED_MEDIA_TYPE = 415, 
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
            StatusCode::METHOD_NOT_ALLOWED => "405 Method Not Allowed".to_string(), 
            StatusCode::UNSUPPORTED_MEDIA_TYPE => "415 Unsupported Media Type".to_string(), 
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
            StatusCode::METHOD_NOT_ALLOWED => 405, 
            StatusCode::UNSUPPORTED_MEDIA_TYPE => 415, 
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
            405 => StatusCode::METHOD_NOT_ALLOWED, 
            415 => StatusCode::UNSUPPORTED_MEDIA_TYPE, 
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
            "405 Method Not Allowed" => StatusCode::METHOD_NOT_ALLOWED, 
            "415 Unsupported Media Type" => StatusCode::UNSUPPORTED_MEDIA_TYPE, 
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

#[derive(Debug, Clone)] 
pub struct UrlEncodedForm{ 
    pub data: HashMap<String, String>  
} 

impl UrlEncodedForm{ 
    /// Creates a new UrlEncodedForm with an empty HashMap. 
    pub fn new() -> Self { 
        Self { data: HashMap::new() } 
    } 

    /// Inserts a key-value pair into the UrlEncodedForm. 
    pub fn insert(&mut self, key: String, value: String) { 
        self.data.insert(key, value); 
    } 

    /// Gets the value from the UrlEncodedForm. 
    pub fn get(&self, key: &str) -> Option<&String> { 
        self.data.get(key) 
    } 

    pub fn get_or_default(&self, key: &str) -> &String { 
        if let Some(value) = self.data.get(key) { 
            return value; 
        } 
        static EMPTY: Lazy<String> = Lazy::new(|| "".to_string()); 
        &EMPTY 
    } 

    /// Gets all values from the UrlEncodedForm. 
    pub fn get_all(&self) -> &HashMap<String, String> { 
        &self.data 
    } 
} 

impl From<HashMap<String, String>> for UrlEncodedForm { 
    fn from(data: HashMap<String, String>) -> Self { 
        Self { data } 
    } 
} 

/// Represents a multipart form data. 
#[derive(Debug, Clone)] 
pub struct MultiForm{ 
    data: HashMap<String, MultiFormField> 
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

impl From<HashMap<String, MultiFormField>> for MultiForm { 
    fn from(data: HashMap<String, MultiFormField>) -> Self { 
        Self { data } 
    } 
} 

impl MultiForm{ 
    /// Creates a new MultiForm with an empty HashMap. 
    pub fn new() -> Self { 
        Self { data: HashMap::new() } 
    } 

    /// Inserts a field into the MultiForm. 
    pub fn insert(&mut self, key: String, value: MultiFormField) { 
        self.data.insert(key, value); 
    } 

    /// Gets the field from the MultiForm. 
    pub fn get(&self, key: &str) -> Option<&MultiFormField> { 
        self.data.get(key) 
    } 

    /// Gets all fields from the MultiForm. 
    pub fn get_all(&self) -> &HashMap<String, MultiFormField> { 
        &self.data 
    } 

    /// Gets the files from the MultiForm. 
    pub fn get_text(&self, key: &str) -> Option<&String> { 
        if let Some(field) = self.data.get(key) { 
            if let MultiFormField::Text(value) = field { 
                return Some(value); 
            } 
        } 
        None 
    } 

    pub fn get_text_or_default(&self, key: &str) -> String { 
        if let Some(field) = self.data.get(key) { 
            if let MultiFormField::Text(value) = field { 
                return value.clone(); 
            } 
        } 
        "".to_string() 
    } 

    /// Gets the files from the MultiForm. 
    pub fn get_files(&self, key: &str) -> Option<&Vec<MultiFormFieldFile>> { 
        if let Some(field) = self.data.get(key) { 
            if let MultiFormField::File(files) = field { 
                return Some(files); 
            } 
        } 
        None 
    } 

    /// Gets the files from the MultiForm. 
    /// This function returns an empty vector if the field is not found or if it is not a file. 
    pub fn get_files_or_default(&self, key: &str) -> &Vec<MultiFormFieldFile> { 
        if let Some(field) = self.data.get(key) { 
            if let MultiFormField::File(files) = field { 
                return files; 
            } 
        } 
        static EMPTY: Lazy<Vec<MultiFormFieldFile>> = Lazy::new(|| Vec::new()); 
        &EMPTY 
    } 

    /// Get the first file from the MultiForm. 
    pub fn get_first_file(&self, key: &str) -> Option<&MultiFormFieldFile> { 
        if let Some(field) = self.data.get(key) { 
            if let MultiFormField::File(files) = field { 
                return files.first(); 
            } 
        } 
        None 
    } 

    /// Get the first file from the MultiForm. 
    /// This function returns the first file as a MultiFormFieldFile. 
    pub fn get_first_file_or_default(&self, key: &str) -> &MultiFormFieldFile { 
        if let Some(field) = self.get_first_file(key) { 
            return field; 
        } 
        static EMPTY: Lazy<MultiFormFieldFile> = Lazy::new(|| MultiFormFieldFile::default()); 
        &EMPTY 
    } 

    /// Get the first file content from the MultiForm. 
    /// This function returns the first file content as a byte slice. 
    pub fn get_first_file_content(&self, key: &str) -> Option<&[u8]> { 
        if let Some(field) = self.data.get(key) { 
            if let MultiFormField::File(files) = field { 
                return files.first().map(|file| file.data.as_slice()); 
            } 
        } 
        None 
    } 

    /// Get the first file content from the MultiForm. 
    /// This function returns the first file content as a byte vector. 
    /// This function returns an empty vector if the field is not found or if it is not a file. 
    pub fn get_first_file_content_or_default(&self, key: &str) -> &[u8] { 
        if let Some(content) = self.get_first_file_content(key) { 
            return content; 
        } 
        static EMPTY: Lazy<Vec<u8>> = Lazy::new(|| Vec::new()); 
        &EMPTY 
    }
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

impl Default for MultiFormField { 
    /// Creates a new MultiFormField with an empty string. 
    fn default() -> Self { 
        Self::Text("".to_string()) 
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

impl Default for MultiFormFieldFile { 
    fn default() -> Self { 
        Self { filename: None, content_type: None, data: Vec::new() } 
    } 
} 

pub struct CookieResponse{ 
    pub name: String, 
    pub value: String, 
    pub path: Option<String>, 
    pub domain: Option<String>, 
    pub expires: Option<String>, 
    pub max_age: Option<String>, 
    pub secure: Option<bool>, 
    pub http_only: Option<bool>, 
} 

impl CookieResponse{ 
    /// Creates a new CookieResponse with the given name and value. 
    /// This function initializes the cookie with default values for path, domain, expires, max_age, secure, and http_only. 
    /// It returns a CookieResponse instance. 
    /// # Examples 
    /// ```rust 
    /// use starberry_core::http::http_value::CookieResponse; 
    /// let cookie = CookieResponse::new("session_id", 123456).domain("example.com".to_string()).path("/".to_string()).expires("Wed, 21 Oct 2025 07:28:00 GMT".to_string()).secure(true).http_only(true); 
    /// ``` 
    pub fn new<T: ToString, S: ToString>(name: S, value: T) -> Self { 
        Self { 
            name: name.to_string(), 
            value: value.to_string(), 
            path: None, 
            domain: None, 
            expires: None, 
            max_age: None, 
            secure: None, 
            http_only: None, 
        } 
    } 

    pub fn get_name(&self) -> &str { 
        &self.name 
    } 

    pub fn set_name<T: ToString>(&mut self, name: T) { 
        self.name = name.to_string(); 
    } 

    pub fn get_value(&self) -> &str { 
        &self.value  
    } 

    pub fn set_value<T: ToString>(&mut self, value: T) { 
        self.value = value.to_string(); 
    } 

    pub fn path<T: ToString>(self, path: T) -> Self { 
        Self { path: Some(path.to_string()), ..self } 
    } 

    pub fn get_path(&self) -> Option<String> { 
        self.path.clone() 
    } 

    pub fn set_path<T: ToString> (&mut self, path: T) { 
        self.path = Some(path.to_string()); 
    } 

    pub fn clear_path(&mut self) { 
        self.path = None; 
    } 

    pub fn domain<T: ToString>(self, domain: T) -> Self { 
        Self { domain: Some(domain.to_string()), ..self } 
    } 

    pub fn get_domain(&self) -> Option<String> { 
        self.domain.clone() 
    } 

    pub fn set_domain<T: ToString> (&mut self, domain: T) { 
        self.domain = Some(domain.to_string()); 
    } 

    pub fn clear_domain(&mut self) { 
        self.domain = None; 
    } 

    pub fn expires<T: ToString> (self, expires: T) -> Self { 
        Self { expires: Some(expires.to_string()), ..self } 
    } 

    pub fn get_expires(&self) -> Option<String> { 
        self.expires.clone() 
    } 

    pub fn set_expires<T: ToString> (&mut self, expires: T) { 
        self.expires = Some(expires.to_string()); 
    } 

    pub fn clear_expires(&mut self) { 
        self.expires = None; 
    } 

    pub fn max_age<T: ToString> (self, max_age: T) -> Self { 
        Self { max_age: Some(max_age.to_string()), ..self } 
    } 

    pub fn get_max_age(&self) -> Option<String> { 
        self.max_age.clone() 
    } 

    pub fn set_max_age<T: ToString> (&mut self, max_age: T) { 
        self.max_age = Some(max_age.to_string()); 
    } 

    pub fn clear_max_age(&mut self) { 
        self.max_age = None; 
    } 

    pub fn secure(self, secure: bool) -> Self { 
        Self { secure: Some(secure), ..self } 
    } 

    pub fn get_secure(&self) -> Option<bool> { 
        self.secure.clone() 
    } 

    pub fn set_secure(&mut self, secure: bool) { 
        self.secure = Some(secure); 
    } 

    pub fn clear_secure(&mut self) { 
        self.secure = None; 
    } 

    pub fn http_only(self, http_only: bool) -> Self { 
        Self { http_only: Some(http_only), ..self } 
    } 

    pub fn get_http_only(&self) -> Option<bool> { 
        self.http_only.clone() 
    } 

    pub fn set_http_only(&mut self, http_only: bool) { 
        self.http_only = Some(http_only); 
    } 

    pub fn clear_http_only(&mut self) { 
        self.http_only = None; 
    } 

    pub fn to_string(&self) -> String { 
        let mut result = format!("{}={}", self.name, self.value.to_string()); 
        if let Some(ref path) = self.path { 
            result.push_str(&format!("; Path={}", path)); 
        } 
        if let Some(ref domain) = self.domain { 
            result.push_str(&format!("; Domain={}", domain)); 
        } 
        if let Some(ref expires) = self.expires { 
            result.push_str(&format!("; Expires={}", expires)); 
        } 
        if let Some(ref max_age) = self.max_age { 
            result.push_str(&format!("; Max-Age={}", max_age)); 
        } 
        if let Some(ref secure) = self.secure { 
            if *secure { 
                result.push_str("; Secure"); 
            } 
        } 
        if let Some(ref http_only) = self.http_only { 
            if *http_only { 
                result.push_str("; HttpOnly"); 
            } 
        } 
        result 
    } 
} 

impl std::fmt::Display for CookieResponse { 
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { 
        write!(f, "{}", self.to_string()) 
    } 
} 

#[derive(Debug, Clone)] 
pub struct RequestPath{ 
    path: Vec<String> 
} 

impl RequestPath{ 
    pub fn new(path: Vec<String>) -> Self{ 
        Self { path }  
    } 

    pub fn to_string(&self) -> String{ 
        let mut result = String::new(); 
        for part in &self.path { 
            result.push('/'); 
            result.push_str(part); 
        } 
        result 
    } 

    pub fn from_string(url: &str) -> Self{ 
        let mut path = Vec::new(); 
        let parts: Vec<&str> = url.split('/').collect(); 
        for part in parts { 
            if !part.is_empty() { 
                path.push(part.to_string()); 
            } 
        } 
        Self { path } 
    } 

    pub fn url_part(&self, part: usize) -> String{ 
        if part < 0 { 
            return self.path[self.path.len() as usize + part as usize].clone(); 
        } else if part >= self.path.len() { 
            return "".to_string(); 
        } 
        self.path[part].clone()  
    }
} 
