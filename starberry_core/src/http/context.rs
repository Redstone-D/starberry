use std::collections::HashMap;
use std::sync::Arc;
use async_trait::async_trait;
use tokio::io::{AsyncWriteExt, BufReader, BufWriter, ReadHalf, WriteHalf};
use akari::Value;
use once_cell::sync::Lazy;
use crate::app::{application::App, urls::Url};
use crate::connection::Connection;
use crate::connection::{Rx, Tx};
use crate::extensions::{Locals, Params};
use crate::http::cookie::{Cookie, CookieMap};
use crate::http::request::HttpRequest; 
use crate::http::safety::*; 
use crate::http::{
    http_value::HttpMethod,
    form::{
        UrlEncodedForm,
        MultiForm
    },
    meta::HttpMeta,
    body:: HttpBody,
    response::HttpResponse
};

use crate::app::config::ParseConfig;

use super::http_value::StatusCode; 
use super::response::response_templates; 

/// The `RequestContext` struct is used to hold the context of a request.
pub struct HttpReqCtx {
    pub request: HttpRequest,
    pub reader: BufReader<ReadHalf<Connection>>,
    pub writer: BufWriter<WriteHalf<Connection>>,
    pub app: Arc<App>,
    pub endpoint: Arc<Url<HttpReqCtx>>,
    pub response: HttpResponse,
    pub params: Params, 
    pub locals: Locals,
} 

impl HttpReqCtx { 

    /// Creates a new Request Context 
    pub fn new(
        request: HttpRequest,
        reader: BufReader<ReadHalf<Connection>>,
        writer: BufWriter<WriteHalf<Connection>>,
        app: Arc<App>,
        endpoint: Arc<Url<HttpReqCtx>>,
    ) -> Self {
        Self {
            request,
            reader,
            writer,
            app,
            endpoint,
            response: HttpResponse::default(),
            params: Default::default(),
            locals: Default::default(), 
        }
    } 

    /// Handles the request by parsing it and creating a new `HttpReqCtx`. 
    pub async fn handle(
        app: Arc<App>, 
        mut reader: BufReader<ReadHalf<Connection>>, 
        writer: BufWriter<WriteHalf<Connection>> 
    ) -> Self {
        // Create one BufReader up-front, pass this throughout.
        let request = HttpRequest::parse_lazy(
            &mut reader,
            &app.connection_config,
            app.get_mode() == crate::app::application::RunMode::Build,
        ).await;
        let endpoint = app
            .root_url
            .clone()
            .walk_str(&request.meta.path())
            .await;
        // let endpoint = dangling_url();
        Self::new(request, reader, writer, app.clone(), endpoint.clone())
    } 

    /// Runs the endpoint and sending the response. 
    pub async fn run(mut self) {
        let endpoint = self.endpoint.clone();
        self.request_check(&endpoint); 
        let parsed = endpoint.run(self);
        parsed.await.send_response().await;
    } 

    /// Checks whether the request fulfills the endpoint's security requirements. 
    pub fn request_check(&mut self, endpoint: &Arc<Url<HttpReqCtx>>) -> bool { 
        (match endpoint.get_params::<MaxBodySize>() { 
            Some(max_size) => max_size.check(self.request.meta.get_content_length().unwrap_or(0)),
            None => true 
        }) && (match endpoint.get_params::<AllowedMethods>() { 
            Some(max_size) => max_size.check(&self.request.meta.method()),
            None => true 
        }) && (match endpoint.get_params::<AllowedContentTypes>() { 
            Some(max_size) => max_size.check(&self.request.meta.get_content_type().unwrap_or_default()),
            None => true 
        }) 
    } 

    /// Sends the response 
    pub async fn send_response(mut self) {
        let _ = self.response.send(&mut self.writer).await;
    } 

    /// Returns the meta in the request as reference 
    pub fn meta(&mut self) -> &mut HttpMeta {
        &mut self.request.meta
    } 

    /// Returns the Arc<App> to the user 
    pub fn app(&self) -> Arc<App> {
        self.app.clone()
    } 

    /// Returns the reader of the request 
    pub fn endpoint(&self) -> Arc<Url<HttpReqCtx>> {
        self.endpoint.clone() 
    } 

    /// Parses the body of the request, reading it into the `HttpBody` field of the request. 
    /// Note that request body will not be automatically parsed unless this function is called 
    /// The automatic parsing is not recommended, as it can lead to performance issues and security vulnerabilities. 
    /// If you didn't parse body, the body will be `HttpBody::Unparsed`. 
    pub async fn parse_body(&mut self) {
        self.request.parse_body(
            &mut self.reader,
            self.endpoint.get_params::<MaxBodySize>().and_then(|size| Some(size.get())).unwrap_or(self.app.get_max_body_size()), 
        ).await;
    }  

    /// Returns the body of the request as a reference to `HttpBody`. 
    pub async fn form(&mut self) -> Option<&UrlEncodedForm> {
        self.parse_body().await; // Await the Future<Output = ()>
        if let HttpBody::Form(ref data) = self.request.body {
            Some(data)
        } else {
            None
        }
    } 

    /// Returns the body of the request as a reference to `UrlEncodedForm`, or an empty form if not present. 
    pub async fn form_or_default(&mut self) -> &UrlEncodedForm {
        match self.form().await {
            Some(form) => form,
            None => {
                static EMPTY: Lazy<UrlEncodedForm> = Lazy::new(|| HashMap::new().into());
                &EMPTY
            }
        }
    } 

    /// Returns the body of the request as a reference to `MultiForm`. 
    pub async fn files(&mut self) -> Option<&MultiForm> {
        self.parse_body().await; // Await the Future<Output = ()>
        if let HttpBody::Files(ref data) = self.request.body {
            Some(data)
        } else {
            None
        }
    } 

    /// Returns the body of the request as a reference to `MultiForm`, or an empty form if not present. 
    pub async fn files_or_default(&mut self) -> &MultiForm {
        match self.files().await {
            Some(files) => files,
            None => {
                static EMPTY: Lazy<MultiForm> = Lazy::new(|| HashMap::new().into());
                &EMPTY
            }
        }
    } 

    /// Returns the body of the request as a reference to `HttpBody::Binary`. 
    pub async fn json(&mut self) -> Option<&Value> {
        self.parse_body().await; // Await the Future<Output = ()>
        if let HttpBody::Json(ref data) = self.request.body {
            Some(data)
        } else {
            None
        }
    } 

    /// Returns the body of the request as a reference to `HttpBody::Binary`, or an empty JSON if not present. 
    pub async fn json_or_default(&mut self) -> &Value {
        match self.json().await {
            Some(json) => json,
            None => {
                static EMPTY: Lazy<Value> = Lazy::new(|| Value::new(""));
                &EMPTY
            }
        }
    } 

    /// Get the path by using index 
    pub fn get_path(&mut self, part: usize) -> String {
        self.request.meta.get_path(part)
    } 

    /// Get the whole path 
    pub fn path(&self) -> String {
        self.request.meta.path()
    } 

    /// Get the index of the part given its name 
    pub fn get_arg_index<S: AsRef<str>>(&self, arg: S) -> Option<usize> {
        self.endpoint.get_segment_index(arg.as_ref())
    } 
    
    /// Get the preferred by the user 
    pub fn get_preferred_language(&mut self) -> Option<String> {
        self.request.meta.get_lang().map(|lang_dict| { 
            lang_dict.most_preferred() 
        }) 
    } 

    /// Get the preferred by the user with a default value 
    pub fn get_preferred_language_or_default<T: AsRef<str>>(&mut self, default: T) -> String {
        self.get_preferred_language().unwrap_or_else(|| default.as_ref().to_string())
    } 

    /// Get the part of the url by using its given name 
    pub fn get_arg<S: AsRef<str>>(&mut self, arg: S) -> Option<String> {
        match self.get_arg_index(arg.as_ref()) {
            Some(index) => Some(self.request.meta.get_path(index)),
            None => None,
        }
    } 

    /// Returns the method of the request.
    pub fn method(&mut self) -> HttpMethod {
        self.request.meta.method()
    } 

    /// Get teh full cookie map 
    pub fn get_cookies(&mut self) -> &CookieMap {
        self.request.meta.get_cookies()
    } 

    /// Get a single cookie 
    pub fn get_cookie(&mut self, key: &str) -> Option<Cookie> {
        self.request.meta.get_cookie(key)
    } 

    /// Get a cookie. If not found a default cookie will be returned 
    pub fn get_cookie_or_default<T: AsRef<str>>(&mut self, key: T) -> Cookie {
        self.request.meta.get_cookie_or_default(key)
    } 
} 

#[async_trait] 
impl Rx for HttpReqCtx { 
    async fn process(
        app: Arc<App>, 
        reader: BufReader<ReadHalf<Connection>>, 
        writer: BufWriter<WriteHalf<Connection>>, 
    ) {
        let handler = Self::handle(app, reader, writer).await;
        handler.run().await; 
    } 

    fn test_protocol(initial_bytes: &[u8]) -> bool {
        // Check for HTTP methods
        initial_bytes.starts_with(b"GET") || 
        initial_bytes.starts_with(b"POST") ||
        initial_bytes.starts_with(b"PUT") ||
        initial_bytes.starts_with(b"DELETE")
    } 

    fn bad_request(&mut self) { 
        self.response = response_templates::return_status(StatusCode::NOT_FOUND) 
    }
}

/// The `HttpResCtx` struct is a transmit layer of Http 
pub struct HttpResCtx {
    pub request: HttpRequest,
    pub response: HttpResponse, 
    pub host: String, 
    pub reader: BufReader<ReadHalf<Connection>>,
    pub writer: BufWriter<WriteHalf<Connection>>,
} 

impl HttpResCtx {
    pub fn new(connection: Connection, host: impl Into<String>) -> Self {
        let (reader, writer) = connection.split(); 
        Self {
            request: HttpRequest::default(),
            response: HttpResponse::default(), 
            host: host.into(), 
            reader: BufReader::new(reader),
            writer: BufWriter::new(writer)
        }
    } 
    pub fn request(&mut self, mut request: HttpRequest) { 
        if request.meta.get_host().is_none() {
            request.meta.set_host(Some(self.host.clone()));
        }; 
        self.request = request; 
    }
    pub async fn send(&mut self) {
        let _ = self.request.send(&mut self.writer).await;
        self.response = HttpResponse::parse_lazy(&mut self.reader, &ParseConfig::default(), false).await;
    }
} 

#[async_trait]  
impl Tx for HttpResCtx { 
    type Request = HttpRequest; 
    type Response = HttpResponse; 
    type Error = std::io::Error; 
    async fn process(&mut self, request: Self::Request) -> Result<&mut Self::Response, Self::Error> {
        self.request(request); 
        self.send().await; 
        Ok(&mut self.response)
    } 
    async fn shutdown(&mut self) -> Result<(), Self::Error> { 
        self.writer.shutdown().await 
    }
}

#[cfg(test)]
mod test { 
    use crate::{connection::{ConnectionBuilder, Protocol}, connection::transmit::Tx, http::{context::HttpResCtx, http_value::{HttpMethod, HttpVersion}, meta::HttpMeta, request::request_templates, start_line::HttpStartLine}};
    #[tokio::test]
    async fn request_a_page() {
        let builder = ConnectionBuilder::new("example.com", 443)
            .protocol(Protocol::HTTP)
            .tls(true);
        let connection = builder.connect().await.unwrap(); 
        let mut request = HttpResCtx::new(connection, "example.com");
        let _ = request.process(request_templates::get_request("/")).await;
        request.response.parse_body(
            &mut request.reader,
            1024 * 1024,
        ).await;
        println!("{:?}, {:?}", request.response.meta, request.response.body);
    }
}  
