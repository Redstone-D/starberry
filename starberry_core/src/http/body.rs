use crate::http::safety::HttpSafety;

use super::form::*;
use super::http_value::*;
use super::meta::HttpMeta; 
use akari::Value;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncBufReadExt};

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
        header: &mut HttpMeta, 
        parse_config: &HttpSafety 
    ) -> Self {
        let parsed;
        // let content_length = header.get_content_length().unwrap_or(0).min(max_size);
        // // println!("Content‐Length header says: {}", content_length);

        let body_buffer = Self::read_binary_info(buf_reader, header, parse_config)
            .await
            .expect("Failed to read body buffer"); 
        // println!("Read {} bytes", body_buffer.len());
        // println!("Body buffer: {:?}", body_buffer);

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
            HttpContentType::Application { subtype, .. } if subtype == "x-www-form-urlencoded" => {
                Self::parse_form(body_buffer)
            }
            HttpContentType::Multipart { subtype, boundary } if subtype == "form-data" => {
                Self::parse_files(body_buffer, boundary.unwrap_or("".to_string()))
            }
            _ => Self::parse_text(body_buffer),
        };

        parsed
    }

    pub async fn read_binary_info<R: AsyncRead + Unpin>(
        buf_reader: &mut tokio::io::BufReader<R>, 
        header: &mut HttpMeta, 
        parse_config: &HttpSafety, 
    ) -> std::io::Result<Vec<u8>> { 

        /// Reads body with Content-Length
        async fn read_content_length_body<R: AsyncRead + Unpin>(
            buf_reader: &mut tokio::io::BufReader<R>,
            safety_setting: &HttpSafety,
            content_length: usize, 
        ) -> std::io::Result<Vec<u8>> { 
            let effective_content_length = std::cmp::min(content_length, safety_setting.effective_body_size()); 
            let mut body_buffer = vec![0; effective_content_length];
            buf_reader.read_exact(&mut body_buffer).await?;
            Ok(body_buffer)
        }

        /// Reads chunked transfer encoding body
        async fn read_chunked_body<R: AsyncRead + Unpin>(
            buf_reader: &mut tokio::io::BufReader<R>, 
            header: &mut HttpMeta,  
            safety_setting: &HttpSafety, 
        ) -> std::io::Result<Vec<u8>> {
            let mut body_buffer = Vec::new();
            let mut current_size = 0;

            loop {
                // Read chunk size line
                let mut size_line = String::new();
                buf_reader.read_line(&mut size_line).await?;
                let chunk_size_str = size_line.trim_end_matches(|c| c == '\r' || c == '\n');
                
                // Parse chunk size
                let chunk_size = usize::from_str_radix(chunk_size_str, 16).map_err(|_| {
                    std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid chunk size")
                })?;

                if chunk_size == 0 {
                    break; // End of chunks
                }

                // Check size limit
                current_size += chunk_size; 
                if !safety_setting.check_body_size(current_size) {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        "Chunked body exceeds maximum size",
                    ));
                }

                // Read chunk data
                let mut chunk_data = vec![0; chunk_size];
                buf_reader.read_exact(&mut chunk_data).await?;
                body_buffer.extend_from_slice(&chunk_data);

                // Read trailing CRLF
                let mut crlf = [0; 2];
                buf_reader.read_exact(&mut crlf).await?;
                if crlf != [b'\r', b'\n'] {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        "Invalid chunk terminator",
                    ));
                }
            }

            // Read trailing headers (if any)
            header.append_from_request_stream(buf_reader, safety_setting, false).await.map_err(|_| std::io::Error::new(std::io::ErrorKind::NetworkUnreachable, "Error parsing headers"))?;

            Ok(body_buffer)
        } 

        // Read raw body data 
        let encoding = header.get_encoding().unwrap_or_default(); 
        let raw_data = if encoding.transfer().is_chunked() {
            read_chunked_body(buf_reader, header, parse_config).await?
        } else {
            let content_length = header.get_content_length().unwrap_or(0);
            read_content_length_body(buf_reader, parse_config, content_length).await?
        };

        // Apply decompression based on Transfer-Encoding
        let raw_data = encoding.content().decode_compressed(raw_data)?; 

        Ok(raw_data)
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
                let boundary = if let Some(HttpContentType::Multipart {
                    subtype: _,
                    boundary: Some(boundary_value),
                }) = meta.get_content_type()
                {
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
                    meta.set_content_type(HttpContentType::Multipart {
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
        // println!("Text body: {:?}", body);
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
