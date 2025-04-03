use std::{collections::HashMap, sync::Arc}; 

use akari::Object;

use crate::app::{application::App, urls::Url};
use crate::http::{http_value::{HttpMethod, MultiForm, UrlEncodedForm}, request::HttpRequest}; 

/// The `RequestContext` struct is used to hold the context of a request. 
pub struct Rc { 
    pub request: HttpRequest, 
    pub app: Arc<App>, 
    pub endpoint: Arc<Url>, 
} 

impl Rc  { 
    pub fn new(request: HttpRequest, app: Arc<App>, endpoint: Arc<Url>) -> Self { 
        Rc  { request, app, endpoint } 
    } 

    pub fn request(&self) -> &HttpRequest { 
        &self.request 
    } 

    pub fn app(&self) -> Arc<App> { 
        self.app.clone() 
    } 

    pub fn endpoint(&self) -> Arc<Url> { 
        self.endpoint.clone() 
    } 

    pub fn get_path(&mut self, part: usize) -> String { 
        self.request.get_path(part) 
    }

    pub fn path(&self) -> &str { 
        self.request.path() 
    } 

    /// Returns the parsed form from the request body if it exists. 
    /// If the body is not a form, it returns None. 
    pub fn form(&self) -> Option<&UrlEncodedForm>  { 
        self.request.form() 
    } 

    /// Returns a reference to the form data if it exists, or an empty HashMap if it doesn't. 
    pub fn form_or_default(&self) -> &UrlEncodedForm { 
        self.request.form_or_default() 
    } 

    
    /// Returns the request body as parsed MultiPartFormField if it exists. 
    /// If the body is not a multipart form, it returns None.  
    pub fn files(&self) -> Option<&MultiForm> { 
        self.request.files() 
    } 

    /// Returns a reference to the parsed files data if it exists, or an empty HashMap if it doesn't. 
    pub fn files_or_default(&self) -> &MultiForm { 
        self.request.files_or_default() 
    } 

    /// Returns the request body as parsed JSON if it exists. 
    /// If the body is not JSON, it returns None. 
    pub fn json(&self) -> Option<&Object> { 
        self.request.json() 
    } 

    /// Returns a reference to the parsed JSON data if it exists, or an empty Object if it doesn't. 
    pub fn json_or_default(&self) -> &Object { 
        self.request.json_or_default() 
    } 

    /// Returns the method of the request. 
    pub fn method(&self) -> &HttpMethod { 
        self.request.method() 
    } 

    pub fn get_cookies(&mut self) -> &HashMap<String, String> { 
        self.request.get_cookies() 
    } 

    pub fn get_cookie(&mut self, key: &str) -> Option<String> { 
        self.request.get_cookie(key) 
    } 

    pub fn get_cookie_or_default(&mut self, key: &str) -> String { 
        self.request.get_cookie_or_default(key) 
    } 
} 
