use std::io::BufReader;
use std::pin::Pin;
use std::{collections::HashMap, sync::Arc}; 
use std::net::TcpStream; 
use std::future::{ready, Ready}; 

use akari::Object;
use once_cell::sync::Lazy;

use crate::app::{application::App, urls::Url};
use crate::http::response::HttpResponse;
use crate::http::{http_value::{HttpMethod, MultiForm, UrlEncodedForm}, request::{HttpMeta, HttpRequestBody}}; 

pub trait SendResponse { 
    fn send(&self, stream: &mut TcpStream); 
} 

/// The `RequestContext` struct is used to hold the context of a request. 
pub struct Rc { 
    pub meta: HttpMeta, 
    pub body: HttpRequestBody, 
    pub reader: BufReader<TcpStream>, 
    pub app: Arc<App>, 
    pub endpoint: Arc<Url>, 
    pub response: HttpResponse, 
} 

impl Rc  { 
    pub fn new(
        meta: HttpMeta,
        body: HttpRequestBody,
        reader: BufReader<TcpStream>,
        app: Arc<App>,
        endpoint: Arc<Url>,
    ) -> Self {
        Self {
            meta,
            body,
            reader,
            app,
            endpoint,
            response: HttpResponse::default(),
        }
    } 

    pub async fn handle(app: Arc<App>, stream: TcpStream) -> Self {
        // Create one BufReader up-front, pass this throughout.
        let mut reader = BufReader::new(stream);
        let meta = HttpMeta::from_request_stream(&mut reader, &app.connection_config)
            .await
            .unwrap_or_default();

        let body = HttpRequestBody::Unparsed;
        let endpoint = app
            .root_url
            .clone()
            .walk_str(meta.path())
            .await;

        Rc::new(meta, body, reader, app.clone(), endpoint.clone())
    } 

    pub async fn run(self) { 
        let endpoint = self.endpoint.clone(); 
        let parsed = endpoint.run(self); 
        parsed.await.send_response(); 
    } 

    pub fn send_response(mut self) {
        self.response.send(self.reader.get_mut());
    } 

    pub fn meta(&self) -> &HttpMeta { 
        &self.meta 
    } 

    pub fn app(&self) -> Arc<App> { 
        self.app.clone() 
    } 

    pub fn endpoint(&self) -> Arc<Url> { 
        self.endpoint.clone() 
    } 

    pub fn parse_body(&mut self) {
        if let HttpRequestBody::Unparsed = self.body {
            self.body = HttpRequestBody::parse(
                &mut self.reader,
                self.endpoint.get_max_body_size().unwrap_or(self.app.get_max_body_size()),
                &mut self.meta.header,
            );
        }
    } 

    pub fn get_path(&mut self, part: usize) -> String { 
        self.meta.get_path(part) 
    }

    pub fn path(&self) -> &str { 
        self.meta.path() 
    } 

    /// Returns the method of the request. 
    pub fn method(&mut self) -> &HttpMethod { 
        self.meta.method() 
    } 

    pub fn get_cookies(&mut self) -> &HashMap<String, String> { 
        self.meta.get_cookies() 
    } 

    pub fn get_cookie(&mut self, key: &str) -> Option<String> { 
        self.meta.get_cookie(key) 
    } 

    pub fn get_cookie_or_default(&mut self, key: &str) -> String { 
        self.meta.get_cookie_or_default(key) 
    } 

    pub fn form(&mut self) -> Option<&UrlEncodedForm> {
        self.parse_body();
        if let HttpRequestBody::Form(ref data) = self.body {
            Some(data)
        } else {
            None
        }
    }

    pub fn form_or_default(&mut self) -> &UrlEncodedForm {
        self.form().unwrap_or_else(|| {
            static EMPTY: Lazy<UrlEncodedForm> = Lazy::new(|| HashMap::new().into());
            &EMPTY
        })
    }

    pub fn files(&mut self) -> Option<&MultiForm> {
        self.parse_body(); 
        if let HttpRequestBody::Files(ref data) = self.body {
            Some(data)
        } else {
            None
        }
    }

    pub fn files_or_default(&mut self) -> &MultiForm {
        self.files().unwrap_or_else(|| {
            static EMPTY: Lazy<MultiForm> = Lazy::new(|| HashMap::new().into());
            &EMPTY
        })
    }

    pub fn json(&mut self) -> Option<&Object> {
        self.parse_body();  
        if let HttpRequestBody::Json(ref data) = self.body {
            Some(data)
        } else {
            None
        }
    }

    pub fn json_or_default(&mut self) -> &Object {
        self.json().unwrap_or_else(|| {
            static EMPTY: Lazy<Object> = Lazy::new(|| Object::new(""));
            &EMPTY
        })
    } 

    /// Converts this response into a Future that resolves to itself.
    /// Useful for middleware functions that need to return a Future<Output = HttpResponse>.
    pub fn future(self) -> impl Future<Output = Rc> + Send {
        ready(self)
    }

    /// Creates a boxed future from this response (useful for trait objects).
    pub fn boxed_future(self) -> Pin<Box<dyn Future<Output = Rc> + Send>> {
        Box::pin(self.future())
    }  
} 
