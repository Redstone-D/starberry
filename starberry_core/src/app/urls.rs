use super::super::http::request::*; 
use super::super::http::response::*; 
use super::super::http::http_value::*;
use super::middleware; 
use std::future::Future;
use std::pin::Pin;
use std::slice::Iter; 
use std::sync::Arc;
use std::sync::OnceLock;
use std::sync::RwLock;
use regex::Regex; 
pub static ROOT_URL: OnceLock<Url> = OnceLock::new();  
use super::super::app::middleware::*; 

pub trait AsyncUrlHandler: Send + Sync + 'static {
    fn handle(
        &self, 
        req: HttpRequest
    ) -> Pin<Box<dyn Future<Output = HttpResponse> + Send + 'static>>;
}

impl<F, Fut> AsyncUrlHandler for F
where
    F: Fn(HttpRequest) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = HttpResponse> + Send + 'static,
{
    fn handle(
        &self, 
        req: HttpRequest
    ) -> Pin<Box<dyn Future<Output = HttpResponse> + Send + 'static>> {
        Box::pin(self(req))
    }
}

pub struct Url {
    pub path: PathPattern,
    pub children: RwLock<Children>, 
    pub ancestor: Ancestor, 
    pub method: RwLock<Option<Arc<dyn AsyncUrlHandler>>>, 
    pub middlewares: RwLock<MiddleWares>,  
} 

#[derive(Clone, Debug)] 
pub enum PathPattern { 
    Literal(String), 
    Regex(String), 
    Any,
    AnyPath, 
} 

impl PathPattern{ 
    pub fn literal_path<T: Into<String>>(path: T) -> Self { 
        Self::Literal(path.into()) 
    } 

    pub fn regex_path<T: Into<String>>(path: T) -> Self { 
        Self::Regex(path.into()) 
    } 

    pub fn any() -> Self { 
        Self::Any 
    } 

    pub fn any_path() -> Self { 
        Self::AnyPath 
    } 
} 

pub mod path_pattern_creator { 
    use super::PathPattern; 

    /// Creates a literal path pattern. 
    /// This is a wrapper around the literal_path function. 
    /// This is useful for creating path patterns that are not regex. 
    pub fn literal_path<T: Into<String>>(path: T) -> PathPattern { 
        PathPattern::Literal(path.into())  
    } 

    /// Creates a regex path pattern. 
    /// This is a wrapper around the regex_path function. 
    /// This is useful for creating path patterns that are regex. 
    pub fn regex_path<T: Into<String>>(path: T) -> PathPattern { 
        PathPattern::Regex(path.into())  
    } 

    /// Creates a any pattern. 
    /// You may use this to match any string. 
    /// This is faster then regex when any string should be passed into the same endpoint 
    pub fn any() -> PathPattern { 
        PathPattern::Any 
    } 

    /// Creates a any path pattern. 
    /// This is useful for matching any path. 
    /// This is faster then regex when any path should be passed into the same endpoint 
    pub fn any_path() -> PathPattern { 
        PathPattern::AnyPath 
    } 
}

impl PartialEq for PathPattern {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (PathPattern::Literal(l), PathPattern::Literal(r)) => l == r,
            (PathPattern::Regex(l), PathPattern::Regex(r)) => l == r,
            (PathPattern::Any, PathPattern::Any) => true,
            (PathPattern::AnyPath, PathPattern::AnyPath) => true,
            _ => false,
        }
    } 
} 

impl std::fmt::Display for PathPattern {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PathPattern::Literal(path) => write!(f, "Literal: {}", path),
            PathPattern::Regex(path) => write!(f, "Regex: {}", path),
            PathPattern::Any => write!(f, "*"),
            PathPattern::AnyPath => write!(f, "**"),
        }
    }
} 

pub enum Children {
    Nil,
    Some(Vec<Arc<Url>>),
} 

pub enum Ancestor {
    Nil,
    Some(Arc<Url>), 
} 

impl std::fmt::Display for Url {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { 
        // create a string having all output msg of childrens 
        let mut children_str = String::new(); 
        let mut func_str = String::new(); 
        // Look for whether the fuction is None or not 
        if let Some(_) = self.method.read().unwrap().as_ref() { 
            func_str.push_str(&format!("Function Exists, ")); 
        } else { 
            func_str.push_str("None, "); 
        } 
        if let Children::Some(children) = &*self.children.read().unwrap() { 
            for child in children.iter() { 
                children_str.push_str(&format!("{}\n", child)); 
            } 
        } else { 
            children_str.push_str(""); 
        } 
        write!(f, "Url: {}, Function: {}, {{{}}}", self.path, func_str, children_str) 
    }
} 

#[derive(Clone)]
pub enum MiddleWares { 
    Nil, 
    MiddleWare(Arc<Vec<Arc<dyn AsyncMiddleware>>>), 
} 

impl MiddleWares{ 
    pub fn new() -> Self { 
        Self::Nil 
    } 

    pub fn add_middleware(&mut self, middleware: Arc<dyn AsyncMiddleware>) { 
        match self { 
            MiddleWares::Nil => { 
                *self = MiddleWares::MiddleWare(Arc::new(vec![middleware])); 
            } 
            MiddleWares::MiddleWare(middlewares) => { 
                let middlewares = Arc::make_mut(middlewares); 
                middlewares.push(middleware); 
            } 
        } 
    } 

    /// Remove a specific middleware based on pointer equality.
    /// Returns true if the middleware was found and removed.
    pub fn remove_middleware(&mut self, target: &Arc<dyn AsyncMiddleware>) -> bool {
        match self {
            MiddleWares::Nil => false,
            MiddleWares::MiddleWare(middlewares) => {
                // Get a mutable reference to the inner vector.
                let middlewares = Arc::make_mut(middlewares);
                if let Some(pos) = middlewares.iter().position(|m| Arc::ptr_eq(m, target)) {
                    middlewares.remove(pos);
                    // If there are no more middlewares, set self to Nil.
                    if middlewares.is_empty() {
                        *self = MiddleWares::Nil;
                    }
                    return true;
                }
                false
            }
        }
    }

    pub fn get_middlewares(&self) -> Option<Arc<Vec<Arc<dyn AsyncMiddleware>>>> { 
        match self { 
            MiddleWares::Nil => None, 
            MiddleWares::MiddleWare(middlewares) => Some(middlewares.clone()), 
        } 
    }  

    pub fn parse( 
        middlewares: &Option<Arc<Vec<Arc<dyn AsyncMiddleware>>>> 
    ) -> MiddleWares { 
        match middlewares { 
            Some(middlewares) => MiddleWares::MiddleWare(middlewares.clone()), 
            None => MiddleWares::Nil, 
        } 
    } 
} 

impl IntoIterator for MiddleWares {
    type Item = Arc<dyn AsyncMiddleware>;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        match self {
            MiddleWares::Nil => Vec::new().into_iter(),
            MiddleWares::MiddleWare(vec_arc) => {
                // Try to unwrap the Arc, if possible.
                // If unwrapping fails (because other clones exist), we clone the inner vector.
                match Arc::try_unwrap(vec_arc) {
                    Ok(vec) => vec.into_iter(),
                    Err(arc) => (*arc).clone().into_iter(),
                }
            }
        }
    }
} 

impl Url { 
    pub async fn run(&self, request: HttpRequest) -> HttpResponse { 
        let handler_opt = { 
            let guard = self.method.read().unwrap();
            guard.clone()
        }; 
        // Lock the middleware 
        let middlewares = { 
            let guard = self.middlewares.read().unwrap(); 
            guard.clone() 
        }; 
        // Runs the function inside it 
        if let Some(method) = handler_opt { 
            // Whether middleware found, by using lf let middleware 
            if let MiddleWares::MiddleWare(_) = middlewares {  
                let base = Arc::new(move |req: HttpRequest| {
                    method.handle(req)
                }) as Arc<dyn Fn(HttpRequest) -> Pin<Box<dyn Future<Output = HttpResponse> + Send>> + Send + Sync>;
                
                // Fold the middleware chain (iterate in reverse order so the first added middleware runs first)
                let chain = middlewares.clone().into_iter().rev().fold(base, |next, mw| {
                    let next_clone = next.clone();
                    Arc::new(move |req: HttpRequest| {
                        // Clone next_clone for each call so the closure doesn't consume it.
                        let next_fn = next_clone.clone();
                        mw.handle(req, Box::new(move |r| next_fn(r)))
                    }) as Arc<dyn Fn(HttpRequest) -> Pin<Box<dyn Future<Output = HttpResponse> + Send>> + Send + Sync>
                }); 
                
                // Now call the complete chain with the request.
                return chain(request).await 
            } else { 
                return method.handle(request).await; 
            }
            // return method.handle(request).await; 
        } 
        return request_templates::return_status(StatusCode::NOT_FOUND); 
    } 

    /// Walk the URL tree based on the path segments.
    /// Returns Some(Arc<Self>) if a matching URL is found, otherwise None.
    pub fn walk<'a>(
        self: Arc<Self>,
        mut path: Iter<'a, &str>,
    ) -> Pin<Box<dyn Future<Output = Option<Arc<Self>>> + Send + 'a>> { 
        
        // Print path 
        // println!("Walking: {:?}", path); 

        // We immediately figure out the "this_segment"
        let this_segment = match path.next() {
            Some(s) => *s,
            None => "",
        }; 

        // Acquire a read lock to inspect the children.
        let guard = self.children.read().unwrap();
        // We only proceed if there are actually some children in the vector:
        let children = if let Children::Some(children) = &*guard {
            children.clone() 
        } else {
            return Box::pin(async { None });
        };
        drop(guard); // Not strictly necessary, but clarifies we no longer need the lock

        // Now create the async portion to iterate over the children
        Box::pin(async move {
            for child_url in children.iter() {
                match &child_url.path {
                    PathPattern::Literal(p) => {
                        if p == this_segment { 
                            // println!("Found literal match: {}, {}, Paths: {:?}", p, this_segment, path); 
                            if path.len() >= 1 { 
                                return child_url.clone().walk(path).await;
                            } else {
                                return Some(child_url.clone());
                            }
                        }
                    }
                    PathPattern::Regex(regex_str) => {
                        let re = Regex::new(regex_str).unwrap(); 
                        // println!("Comparing Regex match: {}, {}, Paths: {:?}", re, this_segment, path);  
                        if re.is_match(this_segment) { 
                            if path.len() > 1 {
                                return child_url.clone().walk(path).await;
                            } else {
                                return Some(child_url.clone());
                            }
                        }
                    }
                    PathPattern::Any => {
                        if path.len() > 1 {
                            return child_url.clone().walk(path).await;
                        } else {
                            return Some(child_url.clone());
                        }
                    }
                    PathPattern::AnyPath => {
                        return Some(child_url.clone());
                    }
                }
            }
            None
        })
    } 

    /// Runs the handler (if any) attached to this URL.
    /// If no handler exists, returns `NOT_FOUND`.
    pub fn run_child(
        self: Arc<Self>,
        request: HttpRequest,
    ) -> Pin<Box<dyn Future<Output = HttpResponse> + Send + 'static>> {
        Box::pin(async move {
            let handler_opt = {
                let guard = self.method.read().unwrap();
                guard.clone() 
            };
            if let Some(handler) = handler_opt {
                handler.handle(request).await
            } else {
                request_templates::return_status(StatusCode::NOT_FOUND)
            }
        })
    } 

    /// Delete a child URL under this URL. 
    /// If the child URL doesn't exist, it returns an error. 
    /// # Arguments 
    /// * `child` - The child URL to delete. 
    /// # Returns 
    /// * `Ok(())` - The child URL was deleted. 
    /// * `Err(String)` - An error message. 
    pub fn kill_child(self: &Arc<Self>, child: PathPattern) -> Result<(), String> { 
        // Acquire a write lock
        let mut guard = self.children.write().unwrap(); 
        match &mut *guard { 
            Children::Nil => Err(format!("No children found")), 
            Children::Some(children) => { 
                // Find the child and remove it 
                if let Some(pos) = children.iter().position(|c| c.path == child) { 
                    children.remove(pos); 
                    Ok(()) 
                } else { 
                    Err(format!("Child not found: {}", child)) 
                } 
            } 
        } 
    } 

    /// Creates a new child URL under this URL. 
    /// If the child URL already exists, it deletes it first. 
    /// If it doesn't exist, it creates a new one and returns it. 
    /// # Arguments 
    /// * `child` - The child URL to create. 
    /// * `function` - The function to run when this URL is accessed. Wrapped in Option 
    /// # Returns 
    /// * `Ok(Arc<Url>)` - The child URL. 
    /// * `Err(String)` - An error message. 
    /// # Note 
    /// This function is not async, but it can be used in an async context. 
    pub fn childbirth(
        self: &Arc<Self>, 
        child: PathPattern, 
        function: Option<Arc<dyn AsyncUrlHandler>>, 
        middleware: Option<Arc<Vec<Arc<dyn AsyncMiddleware>>>>, 
    ) -> Result<Arc<Url>, String> { 
        println!("Creating child URL: {:?}", child); 
        
        // First, do a quick check if the child already exists:
        if self.clone().child_exists(&child) {
            self.kill_child(child.clone())?; 
        } 

        // Create the new child URL
        let new_child = Arc::new(Url { 
            path: child,
            children: RwLock::new(Children::Nil),
            ancestor: Ancestor::Some(Arc::clone(&self)),
            method: RwLock::new(function), 
            middlewares: RwLock::new(MiddleWares::parse(&middleware)), 
        });

        // Now lock for writing and insert the new child
        let mut guard = self.children.write().unwrap();
        match &mut *guard {
            Children::Nil => {
                *guard = Children::Some(vec![new_child.clone()]);
            }
            Children::Some(vec_children) => {
                vec_children.push(new_child.clone());
            }
        }

        Ok(new_child)
    }

    pub fn get_children(self: Arc<Self>, child: PathPattern) -> Result<Arc<Url>, String> {
        // Acquire a read lock
        let guard = self.children.read().unwrap();
        match &*guard {
            Children::Nil => Err(format!("No children found")),
            Children::Some(children) => {
                for child_url in children.iter() {
                    if child_url.path == child {
                        return Ok(child_url.clone());
                    } 
                }
                Err(format!("Child not found: {}", child))
            }
        } 
    } 

    pub fn default_url(self: &Arc<Self>, path: PathPattern) -> Arc<Self> { 
        // Create a new URL with the default path 
        let new_url = Arc::new(Url { 
            path, 
            children: RwLock::new(Children::Nil), 
            ancestor: Ancestor::Nil, 
            method: RwLock::new(None), 
            middlewares: RwLock::new(MiddleWares::Nil), 
        }); 
        new_url 
    } 

    /// Get a child URL or create it if it doesn't exist. 
    /// # Arguments 
    /// * `child` - The child URL to get or create. 
    /// # Returns 
    /// * `Ok(Arc<Url>)` - The child URL. 
    /// * `Err(String)` - An error message. 
    /// # Note 
    /// This function is not async, but it can be used in an async context. 
    pub fn get_child_or_create(self: Arc<Self>, child: PathPattern) -> Result<Arc<Self>, String> {
        {
            let guard = self.children.read().unwrap();
            match &*guard {
                Children::Nil => {
                    // No children at all, so there's nothing to return.
                }
                Children::Some(children) => {
                    // Check each child to see if it matches `child`
                    for child_url in children.iter() {
                        if child_url.path == child {
                            // If we find it, return immediately 
                            // println!("Child found: {}", child_url); 
                            return Ok(child_url.clone());
                        }
                    }
                }
            }
        } 
        // println!("Child not found, creating new one: {:?}", child); 
        self.childbirth(child, None, None) 
    }
    

    pub fn child_exists(self: Arc<Self>, child: &PathPattern) -> bool {
        // Acquire a read lock
        let guard = self.children.read().unwrap();
        match &*guard {
            Children::Nil => false,
            Children::Some(children) => {
                children.iter().any(|c| c.path == *child)
            }
        }
    } 

    /// Register a child URL with a function. 
    pub fn literal_url(
        self: Arc<Self>, 
        path: &str, 
        function: Arc<dyn AsyncUrlHandler>, 
        middleware: Option<Arc<Vec<Arc<dyn AsyncMiddleware>>>> 
    ) -> Result<Arc<Url>, String> { 
        println!("Changing url into path pattern: {}", path); 
        // Remove the first slash if exist 
        let path = if path.starts_with('/') { 
            &path[1..] 
        } else { 
            path 
        }; 
        // Use register, convert the path to a Vec<PathPattern> 
        let path_vec: Vec<PathPattern> = path.split('/').map(|s| PathPattern::literal_path(s)).collect(); 
        println!("Path vector: {:?}", path_vec); 
        // Call register with the path_vec and function 
        let result = self.register(path_vec, Some(function), middleware);
        // Return the result 
        match result { 
            Ok(url) => Ok(url), 
            Err(e) => Err(format!("Error registering URL: {}", e)), 
        } 
    } 

    /// Register a URL with a function. 
    pub fn register(
        self: Arc<Self>, 
        path: Vec<PathPattern>, 
        function: Option<Arc<dyn AsyncUrlHandler>>, 
        middleware: Option<Arc<Vec<Arc<dyn AsyncMiddleware>>>>
    ) -> Result<Arc<Self>, String> { 
        println!("Registering URL: {:?}", path); 
        if path.len() == 1 { 
            return self.childbirth(path[0].clone(), function, middleware); 
        } else { 
            println!("Recursion: Registering child URL: {:?}", path[0]); 
            return self.get_child_or_create(path[0].clone())?.register(path[1..].to_vec(), function, middleware); 
        } 
    } 

    pub fn look_for_child(self: Arc<Self>, path: PathPattern) -> Option<Arc<Self>> { 
        // Acquire a read lock
        let guard = self.children.read().unwrap();
        match &*guard {
            Children::Nil => None,
            Children::Some(children) => {
                for child in children.iter() {
                    if child.path == path {
                        return Some(child.clone());
                    }
                }
                None
            }
        }
    } 

    pub fn set_method(&self, handler: Arc<dyn AsyncUrlHandler>) {
        let mut guard = self.method.write().unwrap();
        *guard = Some(handler); 
    } 

    pub fn set_middlewares(&self, middlewares: Option<Arc<Vec<Arc<dyn AsyncMiddleware>>>>) {
        let mut guard = self.middlewares.write().unwrap(); 
        match middlewares { 
            Some(middlewares) => { 
                *guard = MiddleWares::MiddleWare(middlewares); 
            } 
            None => { 
                *guard = MiddleWares::Nil; 
            } 
        } 
    } 
} 

pub fn dangling_url() -> Arc<Url> { 
    Arc::new(Url { 
        path: PathPattern::Any, 
        children: RwLock::new(Children::Nil), 
        ancestor: Ancestor::Nil, 
        method: RwLock::new(None), 
        middlewares: RwLock::new(MiddleWares::Nil), 
    }) 
} 
