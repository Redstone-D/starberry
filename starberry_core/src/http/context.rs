use std::collections::HashMap;
use std::sync::Arc;
use async_trait::async_trait;
use tokio::io::{BufReader, BufWriter, ReadHalf, WriteHalf};
use akari::Value;
use once_cell::sync::Lazy;
use crate::app::{application::App, urls::Url};
use crate::connection::Connection;
use crate::context::{Rx, Tx};
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
use super::response::request_templates; 

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
    pub async fn run(mut self) {
        let endpoint = self.endpoint.clone();
        self.request_check(&endpoint); 
        let parsed = endpoint.run(self);
        parsed.await.send_response().await;
    } 
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
    pub async fn send_response(mut self) {
        let _ = self.response.send(&mut self.writer).await;
    }
    pub fn meta(&self) -> &HttpMeta {
        &self.request.meta
    }
    pub fn app(&self) -> Arc<App> {
        self.app.clone()
    }
    pub fn endpoint(&self) -> Arc<Url<HttpReqCtx>> {
        self.endpoint.clone()
    }
    pub async fn parse_body(&mut self) {
        self.request.parse_body(
            &mut self.reader,
            self.endpoint.get_params::<MaxBodySize>().and_then(|size| Some(size.get())).unwrap_or(self.app.get_max_body_size()), 
        ).await;
    } 
    pub async fn form(&mut self) -> Option<&UrlEncodedForm> {
        self.parse_body().await; // Await the Future<Output = ()>
        if let HttpBody::Form(ref data) = self.request.body {
            Some(data)
        } else {
            None
        }
    }
    pub async fn form_or_default(&mut self) -> &UrlEncodedForm {
        match self.form().await {
            Some(form) => form,
            None => {
                static EMPTY: Lazy<UrlEncodedForm> = Lazy::new(|| HashMap::new().into());
                &EMPTY
            }
        }
    }
    pub async fn files(&mut self) -> Option<&MultiForm> {
        self.parse_body().await; // Await the Future<Output = ()>
        if let HttpBody::Files(ref data) = self.request.body {
            Some(data)
        } else {
            None
        }
    }
    pub async fn files_or_default(&mut self) -> &MultiForm {
        match self.files().await {
            Some(files) => files,
            None => {
                static EMPTY: Lazy<MultiForm> = Lazy::new(|| HashMap::new().into());
                &EMPTY
            }
        }
    }
    pub async fn json(&mut self) -> Option<&Value> {
        self.parse_body().await; // Await the Future<Output = ()>
        if let HttpBody::Json(ref data) = self.request.body {
            Some(data)
        } else {
            None
        }
    }
    pub async fn json_or_default(&mut self) -> &Value {
        match self.json().await {
            Some(json) => json,
            None => {
                static EMPTY: Lazy<Value> = Lazy::new(|| Value::new(""));
                &EMPTY
            }
        }
    }
    pub fn get_path(&mut self, part: usize) -> String {
        self.request.meta.get_path(part)
    }
    pub fn path(&self) -> String {
        self.request.meta.path()
    }
    pub fn get_arg_index<S: AsRef<str>>(&self, arg: S) -> Option<usize> {
        self.endpoint.get_segment_index(arg.as_ref())
    }
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
    pub fn get_cookies(&mut self) -> &CookieMap {
        self.request.meta.get_cookies()
    }
    pub fn get_cookie(&mut self, key: &str) -> Option<Cookie> {
        self.request.meta.get_cookie(key)
    }
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
        self.response = request_templates::return_status(StatusCode::NOT_FOUND) 
    }
}

pub struct HttpResCtx {
    pub request: HttpRequest,
    pub response: HttpResponse,
    pub reader: BufReader<ReadHalf<Connection>>,
    pub writer: BufWriter<WriteHalf<Connection>>,
}
impl HttpResCtx {
    pub fn new(mut request: HttpRequest, connection: Connection, host: impl Into<String>) -> Self {
        let (reader, writer) = connection.split(); 
        if request.meta.get_host().is_none() {
            request.meta.set_host(Some(host.into()));
        } 
        Self {
            request,
            response: HttpResponse::default(),
            reader: BufReader::new(reader),
            writer: BufWriter::new(writer)
        }
    }
    pub async fn send(&mut self) {
        self.request.send(&mut self.writer).await;
        self.response = HttpResponse::parse_lazy(&mut self.reader, &ParseConfig::default(), false).await;
    }
} 

#[async_trait]  
impl Tx for HttpResCtx { 
    type Request = HttpRequest; 
    type Response = HttpResponse; 
    type Error = (); 
    async fn process(&mut self, request: Self::Request) -> Result<&mut Self::Response, Self::Error> {
        self.request = request; 
        self.send().await; 
        Ok(&mut self.response)
    }
}

#[cfg(test)]
mod test {
    use std::collections::HashMap; 
    use crate::{connection::{ConnectionBuilder, Protocol}, http::{context::HttpResCtx, http_value::{HttpMethod, HttpVersion}, meta::HttpMeta, start_line::HttpStartLine}};
    #[tokio::test]
    async fn request_a_page() {
        let builder = ConnectionBuilder::new("example.com", 443)
            .protocol(Protocol::HTTP)
            .tls(true);
        let connection = builder.connect().await.unwrap();
        // 6. Create a request context 
        let request = crate::http::request::request_templates::get_request(); 
        let mut request = HttpResCtx::new(request, connection, "example.com");
        request.send().await;
        request.response.parse_body(
            &mut request.reader,
            1024 * 1024,
        ).await;
        println!("{:?}, {:?}", request.response.meta, request.response.body);
    }
}  
