use std::io::BufReader;
use std::pin::Pin;
use std::{collections::HashMap, sync::Arc}; 
use std::net::TcpStream; 
use std::future::{ready, Ready}; 
use std::any::{Any, TypeId}; 

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

    /// Type-based extension storage, typically used by middleware
    /// Each type can have exactly one value
    params: HashMap<TypeId, Box<dyn Any + Send + Sync>>,
    
    /// String-based extension storage, typically used by application code
    /// Multiple values of the same type can be stored with different keys
    locals: HashMap<String, Box<dyn Any + Send + Sync>>,
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
            params: HashMap::new(),
            locals: HashMap::new(), 
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

    pub async fn run(mut self) { 
        let endpoint = self.endpoint.clone(); 
        if !endpoint.clone().request_check(&mut self).await { 
            self.response.send(self.reader.get_mut()); 
            return; 
        }
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

        //
    // Type-based params methods (for middleware)
    //
    
    /// Stores a value in the type-based params storage.
    /// Any previous value of the same type will be replaced.
    /// 
    /// # Examples
    ///
    /// ```rust
    /// let mut req = HttpRequest::default();
    /// 
    /// // Store authentication information
    /// req.set_param(User { id: 123, name: "Alice".to_string() });
    /// 
    /// // Store timing information
    /// req.set_param(RequestTimer::start());
    /// ```
    pub fn set_param<T: 'static + Send + Sync>(&mut self, value: T) {
        self.params.insert(TypeId::of::<T>(), Box::new(value));
    }
    
    /// Retrieves a reference to a value from the type-based params storage.
    /// Returns `None` if no value of this type has been stored.
    ///
    /// # Examples
    ///
    /// ```rust
    /// // In an authentication middleware
    /// if let Some(user) = req.param::<User>() {
    ///     println!("Request by: {}", user.name);
    ///     // Proceed with authenticated user
    /// } else {
    ///     return HttpResponse::unauthorized();
    /// }
    /// ```
    pub fn param<T: 'static + Send + Sync>(&self) -> Option<&T> {
        self.params
            .get(&TypeId::of::<T>())
            .and_then(|boxed| boxed.downcast_ref::<T>())
    }
    
    /// Retrieves a mutable reference to a value from the type-based params storage.
    /// Returns `None` if no value of this type has been stored.
    ///
    /// # Examples
    ///
    /// ```rust
    /// // Update a request timer
    /// if let Some(timer) = req.param_mut::<RequestTimer>() {
    ///     timer.mark("after_db_query");
    /// }
    /// ```
    pub fn param_mut<T: 'static + Send + Sync>(&mut self) -> Option<&mut T> {
        self.params
            .get_mut(&TypeId::of::<T>())
            .and_then(|boxed| boxed.downcast_mut::<T>())
    }
    
    /// Removes a value from the type-based params storage and returns it.
    /// Returns `None` if no value of this type has been stored.
    ///
    /// # Examples
    ///
    /// ```rust
    /// // Take ownership of a value
    /// if let Some(token) = req.take_param::<AuthToken>() {
    ///     // Use and consume the token
    ///     validate_token(token);
    /// }
    /// ```
    pub fn take_param<T: 'static + Send + Sync>(&mut self) -> Option<T> {
        self.params
            .remove(&TypeId::of::<T>())
            .and_then(|boxed| boxed.downcast::<T>().ok())
            .map(|boxed| *boxed)
    }
    
    //
    // String-based locals methods (for application code)
    //
    
    /// Stores a value in the string-based locals storage with the given key.
    /// Any previous value with the same key will be replaced.
    ///
    /// # Examples
    ///
    /// ```rust
    /// let mut req = HttpRequest::default();
    ///
    /// // Store various data with descriptive keys
    /// req.set_local("user_id", 123);
    /// req.set_local("is_premium", true);
    /// req.set_local("cart_items", vec!["item1", "item2"]);
    /// ```
    pub fn set_local<T: 'static + Send + Sync>(&mut self, key: impl Into<String>, value: T) {
        self.locals.insert(key.into(), Box::new(value));
    }
    
    /// Retrieves a reference to a value from the string-based locals storage by key.
    /// Returns `None` if no value with this key exists or if the type doesn't match.
    ///
    /// # Examples
    ///
    /// ```rust
    /// // In a request handler
    /// if let Some(is_premium) = req.local::<bool>("is_premium") {
    ///     if *is_premium {
    ///         // Show premium content
    ///     }
    /// }
    ///
    /// // With different types
    /// let user_id = req.local::<i32>("user_id");
    /// let items = req.local::<Vec<String>>("cart_items");
    /// ```
    pub fn local<T: 'static + Send + Sync>(&self, key: &str) -> Option<&T> {
        self.locals
            .get(key)
            .and_then(|boxed| boxed.downcast_ref::<T>())
    }
    
    /// Retrieves a mutable reference to a value from the string-based locals storage by key.
    /// Returns `None` if no value with this key exists or if the type doesn't match.
    ///
    /// # Examples
    ///
    /// ```rust
    /// // Modify a list of items
    /// if let Some(items) = req.local_mut::<Vec<String>>("cart_items") {
    ///     items.push("new_item".to_string());
    /// }
    /// ```
    pub fn local_mut<T: 'static + Send + Sync>(&mut self, key: &str) -> Option<&mut T> {
        self.locals
            .get_mut(key)
            .and_then(|boxed| boxed.downcast_mut::<T>())
    }
    
    /// Removes a value from the string-based locals storage and returns it.
    /// Returns `None` if no value with this key exists or if the type doesn't match.
    ///
    /// # Examples
    ///
    /// ```rust
    /// // Take ownership of a value
    /// if let Some(token) = req.take_local::<String>("session_token") {
    ///     // Use and consume the token
    ///     validate_and_destroy_token(token);
    /// }
    /// ```
    pub fn take_local<T: 'static + Send + Sync>(&mut self, key: &str) -> Option<T> {
        self.locals
            .remove(key)
            .and_then(|boxed| boxed.downcast::<T>().ok())
            .map(|boxed| *boxed)
    }
    
    /// Returns all keys currently stored in the locals map
    ///
    /// # Examples
    ///
    /// ```rust
    /// // Inspect what data is attached to the request
    /// for key in req.local_keys() {
    ///     println!("Request has data with key: {}", key);
    /// }
    /// ```
    pub fn local_keys(&self) -> Vec<&str> {
        self.locals.keys().map(|s| s.as_str()).collect()
    }
    
    //
    // Utility bridging methods
    //
    
    /// Exports a param value to the locals storage with the given key.
    /// The value must implement Clone. Does nothing if the param doesn't exist.
    ///
    /// # Examples
    ///
    /// ```rust
    /// // Make the authenticated user available in locals for convenience
    /// req.export_param_to_local::<User>("current_user");
    /// ```
    pub fn export_param_to_local<T: 'static + Clone + Send + Sync>(&mut self, key: impl Into<String>) {
        if let Some(value) = self.param::<T>() {
            let cloned = value.clone();
            self.set_local(key, cloned);
        }
    }
    
    /// Imports a local value into the params storage.
    /// The value must implement Clone. Does nothing if the local doesn't exist.
    ///
    /// # Examples
    ///
    /// ```rust
    /// // Make a manually set user available to middleware expecting it in params
    /// req.import_local_to_param::<User>("manual_user");
    /// ```
    pub fn import_local_to_param<T: 'static + Clone + Send + Sync>(&mut self, key: &str) {
        if let Some(value) = self.local::<T>(key) {
            let cloned = value.clone();
            self.set_param(cloned);
        }
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
