use std::collections::HashMap;
use std::str::Bytes; 
use super::http_value::*; 
use super::form::*; 
use super::meta::HttpMeta; 
use starberry_lib::decode_url_owned;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt}; 
use akari::Value; 

static EMPTY: Vec<u8> = Vec::new();

#[derive(Debug, Clone)]
pub enum HttpBody {
    Text(String),
    Binary(Vec<u8>),
    Form(UrlEncodedForm),
    Files(MultiForm),
    Json(Value),
    Empty,
    Unparsed,
} 

impl HttpBody { 
    pub async fn parse<R: AsyncRead + Unpin>(
        buf_reader: &mut tokio::io::BufReader<R>, 
        max_size: usize,
        header: &mut HttpMeta,
    ) -> Self {
        let parsed;
        let content_length = header.get_content_length().unwrap_or(0).min(max_size);
println!("Content‐Length header says: {}", content_length);
        if content_length == 0 {
            parsed = Self::Empty;
        } else {

            let mut body_buffer = vec![0; content_length];
            buf_reader
                .read_exact(&mut body_buffer)
                .await
                .expect("failed to read exactly content_length bytes");
            println!("Read {} bytes", body_buffer.len());
            println!("Body buffer: {:?}", body_buffer); 

            parsed = match header
                .get_content_type()
                .unwrap_or(HttpContentType::from_str(""))
            {
                HttpContentType::Application { subtype, .. } if subtype == "json" => {
                    Self::parse_json(body_buffer)
                }
                HttpContentType::Text { subtype, .. } if subtype == "html" => {
                    Self::parse_text(body_buffer)
                }
                HttpContentType::Text { subtype, .. } if subtype == "plain" => {
                    Self::parse_text(body_buffer)
                }
                HttpContentType::Application { subtype, .. }
                    if subtype == "x-www-form-urlencoded" =>
                {
                    Self::parse_form(body_buffer)
                }
                HttpContentType::Multipart { subtype, boundary } 
                    if subtype == "form-data" => 
                { 
                    Self::parse_files(body_buffer, boundary.unwrap_or("".to_string())) 
                }
                _ => Self::parse_text(body_buffer),
            }
        }

        parsed
    } 

    /// Write a response body to the TcpStream buffer 
    /// This will automatically set the content length and content type for the meta if it is not set 
    pub async fn into_static(&mut self, meta: &mut HttpMeta) -> &[u8] { 
        match self { 
            Self::Text(_) => { 
                self.text_into_binary(); 
                let bin = self.raw(); 
                if let None = meta.get_content_length() {
                    meta.set_content_length(bin.len()); 
                } 
                if let None = meta.get_content_type() {
                    meta.set_content_type(HttpContentType::TextHtml()); 
                } 
                meta.set_content_type(HttpContentType::TextPlain()); 
                bin 
            } 
            Self::Binary(_) => { 
                let bin = self.raw(); 
                if let None = meta.get_content_length() {
                    meta.set_content_length(bin.len()); 
                } 
                if let None = meta.get_content_type() {
                    meta.set_content_type(HttpContentType::ApplicationOctetStream()); 
                } 
                bin 
            } 
            Self::Json(_) => { 
                self.json_into_binary(); 
                let bin = self.raw(); 
                if let None = meta.get_content_length() {
                    meta.set_content_length(bin.len()); 
                } 
                if let None = meta.get_content_type() {
                    meta.set_content_type(HttpContentType::ApplicationJson()); 
                } 
                bin 
            } 
            Self::Form(_) => { 
                self.form_into_binary(); 
                let bin = self.raw(); 
                if let None = meta.get_content_length() {
                    meta.set_content_length(bin.len()); 
                } 
                if let None = meta.get_content_type() {
                    meta.set_content_type(HttpContentType::ApplicationUrlEncodedForm()); 
                } 
                bin 
            } 
            Self::Files(_) => { 
                let boundary = if let Some(
                    HttpContentType::Multipart { subtype: _, boundary: Some(boundary_value) }
                ) = meta.get_content_type() { 
                    boundary_value // Or boundary_value.to_string() depending on the type
                } else {
                    // Default boundary if none provided
                    "----DefaultBoundary7MA4YWxkTrZu0gW".to_string()
                }; 
                self.files_into_binary(&boundary); 
                let bin = self.raw(); 
                if let None = meta.get_content_length() {
                    meta.set_content_length(bin.len()); 
                } 
                if let None = meta.get_content_type() {
                    meta.set_content_type(HttpContentType::Multipart{ 
                        subtype: "form-data".to_string(), 
                        boundary: Some(boundary), 
                    }); 
                } 
                bin 
            }
            _ => {
                if let None = meta.get_content_length() {
                    meta.set_content_length(0); 
                } 
                &EMPTY 
            } 
        } 
    } 

    pub fn parse_json(body: Vec<u8>) -> Self {
        return Self::Json(
            Value::from_json(std::str::from_utf8(&body).unwrap_or("")).unwrap_or(Value::new("")),
        );
    }   

    /// Change Self::Json into Self::Binary 
    pub fn json_into_binary(&mut self) {
        match self {
            Self::Json(json) => {
                let binary = json.into_json().as_bytes().to_vec();
                *self = Self::Binary(binary);
            } 
            _ => {}
        } 
    } 

    pub fn parse_text(body: Vec<u8>) -> Self { 
        println!("Text body: {:?}", body); 
        return Self::Text(String::from_utf8_lossy(&body).to_string());
    } 

    /// Change Self::Text into Self::Binary 
    pub fn text_into_binary(&mut self) {
        match self {
            Self::Text(text) => {
                let binary = text.as_bytes().to_vec();
                *self = Self::Binary(binary);
            }
            _ => {}
        } 
    } 

    pub fn parse_binary(body: Vec<u8>) -> Self {
        return Self::Binary(body);
    } 

    /// Get the raw data for **BINARY** http body 
    /// A non binary Http Body must first convert into binary in order to get the bin data 
    pub fn raw<'a>(&'a self) -> &'a [u8] {
        match self {
            Self::Binary(data) => data, 
            _ => &EMPTY,
        }
    }     

    pub fn parse_form(body: Vec<u8>) -> Self {
        let form = UrlEncodedForm::parse(body);
        return Self::Form(form);
    }  

    pub fn form_into_binary(&mut self) {
        match self {
            Self::Form(form) => {
                let binary = form.to_string().into(); 
                *self = Self::Binary(binary);
            }
            _ => {}
        }
    }   

    pub fn parse_files(body: Vec<u8>, boundary: String) -> Self {
        let files = MultiForm::parse(body, boundary);
        return Self::Files(files);
    } 

    pub fn files_into_binary(&mut self, boundary: &String) {
        match self {
            Self::Files(files) => {
                let binary = files.to_string(boundary).into(); 
                *self = Self::Binary(binary);
            }
            _ => {}
        }
    } 

}

impl Default for HttpBody {
    fn default() -> Self {
        Self::Unparsed 
    }
} 
