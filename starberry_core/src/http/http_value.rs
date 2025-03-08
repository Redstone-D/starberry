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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HttpContentType {
    TextPlain,
    TextHtml,
    TextCss,
    TextJavascript,
    ApplicationJson,
    ApplicationXml,
    ApplicationJavascript,
    ApplicationPdf,
    ApplicationZip,
    ApplicationXWwwFormUrlEncoded,
    ApplicationOctetStream,
    ImagePng,
    ImageJpeg,
    ImageGif,
    ImageSvgXml,
    AudioMpeg,
    AudioOgg,
    VideoMp4,
    VideoWebm,
    MultipartFormData,
    Other(String),
}

impl HttpContentType {
    /// Converts a string into an HttpContentType enum variant
    pub fn from_str(content_type: &str) -> Self {
        match content_type {
            "text/plain" => Self::TextPlain,
            "text/html" => Self::TextHtml,
            "text/css" => Self::TextCss,
            "text/javascript" => Self::TextJavascript,
            "application/json" => Self::ApplicationJson,
            "application/xml" => Self::ApplicationXml,
            "application/javascript" => Self::ApplicationJavascript,
            "application/pdf" => Self::ApplicationPdf,
            "application/zip" => Self::ApplicationZip,
            "application/x-www-form-urlencoded" => Self::ApplicationXWwwFormUrlEncoded,
            "application/octet-stream" => Self::ApplicationOctetStream,
            "image/png" => Self::ImagePng,
            "image/jpeg" => Self::ImageJpeg,
            "image/gif" => Self::ImageGif,
            "image/svg+xml" => Self::ImageSvgXml,
            "audio/mpeg" => Self::AudioMpeg,
            "audio/ogg" => Self::AudioOgg,
            "video/mp4" => Self::VideoMp4,
            "video/webm" => Self::VideoWebm,
            "multipart/form-data" => Self::MultipartFormData,
            other => Self::Other(other.to_string()),
        }
    }

    /// Converts an HttpContentType enum variant into its string representation
    pub fn as_str(&self) -> &str {
        match self {
            Self::TextPlain => "text/plain",
            Self::TextHtml => "text/html",
            Self::TextCss => "text/css",
            Self::TextJavascript => "text/javascript",
            Self::ApplicationJson => "application/json",
            Self::ApplicationXml => "application/xml",
            Self::ApplicationJavascript => "application/javascript",
            Self::ApplicationPdf => "application/pdf",
            Self::ApplicationZip => "application/zip",
            Self::ApplicationXWwwFormUrlEncoded => "application/x-www-form-urlencoded",
            Self::ApplicationOctetStream => "application/octet-stream",
            Self::ImagePng => "image/png",
            Self::ImageJpeg => "image/jpeg",
            Self::ImageGif => "image/gif",
            Self::ImageSvgXml => "image/svg+xml",
            Self::AudioMpeg => "audio/mpeg",
            Self::AudioOgg => "audio/ogg",
            Self::VideoMp4 => "video/mp4",
            Self::VideoWebm => "video/webm",
            Self::MultipartFormData => "multipart/form-data",
            Self::Other(other) => other.as_str(),
        }
    }
}

impl std::fmt::Display for HttpContentType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
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
