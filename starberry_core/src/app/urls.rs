use crate::extensions::ParamValue;

use super::super::http::response::*; 
use super::super::http::http_value::*; 
use super::super::connection::Rx; 
use super::super::extensions::ParamsClone; 
use std::future::Future;
use std::pin::Pin;
use std::slice::Iter; 
use std::sync::Arc; 
use std::sync::RwLock; 
use regex::Regex; 
// pub static ROOT_URL: OnceLock<Url> = OnceLock::new();  
use super::super::app::middleware::*; 

pub struct Url<R: Rx> {
    pub path: PathPattern,
    pub children: RwLock<Children<R>>, 
    pub ancestor: Ancestor<R>, 
    pub method: RwLock<Option<Arc<dyn AsyncFinalHandler<R>>>>, 
    pub middlewares: RwLock<Vec<Arc<dyn AsyncMiddleware<R>>>>,  
    pub params: RwLock<ParamsClone>, 
} 

#[derive(Clone, Debug)] 
pub enum PathPattern { 
    Literal(String), // A literal path, e.g. "foo"
    Regex(String), // A regex path, e.g. "\d+" 
    Pattern(String, String), // A regex pattern with a pattern name associated with it 
    Any, // A wildcard path, e.g. "*" 
    Argument(String), // A path with an argument 
    AnyPath, // A wildcard path with a trailing slash, e.g. "**" 
} 

impl PathPattern{ 
    pub fn literal_path<T: Into<String>>(path: T) -> Self { 
        Self::Literal(path.into()) 
    } 

    pub fn regex_path<T: Into<String>>(path: T) -> Self { 
        Self::Regex(path.into()) 
    } 

    pub fn regex_pattern<T: Into<String>, A: Into<String>>(path: T, name: A) -> Self { 
        Self::Pattern(path.into(), name.into())
    } 

    pub fn any() -> Self { 
        Self::Any 
    } 

    pub fn argument<A: Into<String>>(name: A) -> Self { 
        Self::Argument(name.into()) 
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

    pub fn trailing_slash() -> PathPattern { 
        PathPattern::Literal("".to_string()) 
    } 

    /// Creates a regex path pattern. 
    /// This is a wrapper around the regex_path function. 
    /// This is useful for creating path patterns that are regex. 
    pub fn regex_path<T: Into<String>>(path: T) -> PathPattern { 
        PathPattern::Regex(path.into())  
    } 

    /// Creates a regex path pattern with a variable name. 
    /// The variable name will be able to search the user's input into the url at this segment. 
    /// This is a wrapper around the regex_path function. 
    /// This is useful for creating path patterns that are regex. 
    pub fn regex_pattern<T: Into<String>, A: Into<String>>(path: T, name: A) -> PathPattern { 
        PathPattern::regex_pattern(path.into(), name.into()) 
    } 

    /// Creates a any pattern. 
    /// You may use this to match any string. 
    /// This is faster then regex when any string should be passed into the same endpoint 
    pub fn any() -> PathPattern { 
        PathPattern::Any 
    } 

    /// Creates a any pattern with a variable name. 
    /// This is useful for matching any string. 
    /// This is faster then regex when any string should be passed into the same endpoint 
    pub fn argument<A: Into<String>>(name: A) -> PathPattern { 
        PathPattern::Argument(name.into()) 
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
            PathPattern::Pattern(path, arg) => write!(f, "Regex {}: {}", arg, path), 
            PathPattern::Any => write!(f, "*"), 
            PathPattern::Argument(arg) => write!(f, "* {}", arg), 
            PathPattern::AnyPath => write!(f, "**"),
        } 
    }
} 

pub enum Children<R: Rx> {
    Nil,
    Some(Vec<Arc<Url<R>>>),
} 

pub enum Ancestor<R: Rx> {
    Nil,
    Some(Arc<Url<R>>), 
} 

impl<R: Rx> std::fmt::Display for Url<R> {
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

impl<R: Rx + 'static> Url<R> { 
    pub async fn run(&self, mut rx: R) -> R { 
        let final_handler = { 
            let guard = self.method.read().unwrap();
            guard.clone()
        }; 
        // Lock the middleware 
        let middlewares = { 
            let guard = self.middlewares.read().unwrap(); 
            guard.clone() 
        }; 
        // Runs the function inside it 
        if let Some(method) = final_handler { 
            run_chain(middlewares, method, rx).await 
            // return method.handle(request).await; 
        } else { 
            rx.bad_request(); 
            rx 
        }  
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
            let mut best_fit: Option<Arc<Url<R>>> = None; 
            for child_url in children.iter() { 
                // println!("Comparing: {}, {}", child_url.path, this_segment);  
                match &child_url.path { 

                    // Matching the literal paths 
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

                    // Matches the Regex Path 
                    PathPattern::Regex(regex_str) | PathPattern::Pattern(regex_str, _ ) => {
                        let re = Regex::new(regex_str).unwrap(); 
                        // println!("Comparing Regex match: {}, {}, Paths: {:?}", re, this_segment, path);  
                        if re.is_match(this_segment) { 
                            if path.len() >= 1 {
                                return child_url.clone().walk(path).await;
                            } else {
                                return Some(child_url.clone());
                            }
                        }
                    } 

                    // Matching the Any path 
                    PathPattern::Any | PathPattern::Argument(_) => {
                        if path.len() >= 1 { 
                            // println!("Found any match: {}, Paths: {:?}", this_segment, path); 
                            return child_url.clone().walk(path).await;
                        } else {
                            return Some(child_url.clone());
                        }
                    } 

                    // Else 
                    PathPattern::AnyPath => {
                        best_fit = Some(child_url.clone());
                    }
                }
            }
            best_fit 
        })
    } 

    pub async fn walk_str(self: Arc<Self>, path: &str) -> Arc<Url<R>> { 
        let mut path = path.split('/').collect::<Vec<&str>>(); 
        path.remove(0); 
        // println!("Walking: {:?}", path); 
        // Call walk with the iterator 
        self.walk(path.iter()).await.unwrap_or_else(|| { 
            // If no match is found, return a default URL 
            dangling_url() 
        }) 
    } 

    /// Get the index of segment of the URL by using the argument name 
    /// If two url pattern have the same name, it will return the last one 
    /// It will return none if no match is found 
    pub fn get_segment_index<S: AsRef<str>>(self: &Arc<Self>, name: S) -> Option<usize> { 
        let mut index = None; 
        self._step_get_segment_index(name.as_ref(), &mut index); 
        index 
    } 

    /// The recursive function will check the ancestor 
    /// During the first call, the index is None 
    fn _step_get_segment_index(self: &Arc<Self>, match_path: &str, index: &mut Option<usize>) { 
        if let None = index {    
            match &self.path { 
                PathPattern::Argument(arg) | PathPattern::Pattern(_, arg) => { 
                    if arg == &match_path { 
                        *index = Some(0); 
                    } 
                } 
                _ => {} 
            } 
        } 

        match &self.ancestor { 
            Ancestor::Nil => {
                if let Some(i) = index {
                    if *i > 0 { 
                        *i -= 1; 
                    } 
                } 
            } 
            Ancestor::Some(ancestor) => { 
                if let Some(i) = index {
                    *i += 1; 
                } 
                ancestor._step_get_segment_index(match_path, index);
            }
        }
    } 

    /// Retrieves a cloned value of type `T` from the URL's parameter storage.
    /// Returns `Some(T)` if the parameter exists and matches the type, `None` otherwise. 
    pub fn get_params<T: ParamValue + Clone + 'static>(&self) -> Option<T> {
        let params = self.params.read().unwrap(); 
        params.get::<T>().cloned()
    }

    /// Stores a value in the URL's parameter storage, overwriting any existing value
    /// of the same type. This only affects the current URL node, not its ancestors.
    pub fn set_params<T: ParamValue + 'static>(&self, value: T) {
        self.params.write().unwrap().set(value);
    } 

    /// Runs the handler (if any) attached to this URL.
    /// If no handler exists, returns `NOT_FOUND`.
    pub fn run_child(
        self: Arc<Self>,
        mut rc: R,
    ) -> Pin<Box<dyn Future<Output = R> + Send>> {
        Box::pin(async move {
            let handler_opt = {
                let guard = self.method.read().unwrap();
                guard.clone() 
            };
            if let Some(handler) = handler_opt {
                return handler.handle(rc).await; 
            } else { 
                rc.bad_request();
                return rc; 
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
    /// * `middleware` - The middleware to run when this URL is accessed. Wrapped in Option 
    /// * `params` - The parameters to use for this URL. Wrapped in Option 
    /// # Returns 
    /// * `Ok(Arc<Url>)` - The child URL. 
    /// * `Err(String)` - An error message. 
    /// # Note 
    /// This function is not async, but it can be used in an async context. 
    pub fn childbirth(
        self: &Arc<Self>, 
        child: PathPattern, 
        function: Option<Arc<dyn AsyncFinalHandler<R>>>, 
        middleware: Vec<Arc<dyn AsyncMiddleware<R>>>, 
        params: ParamsClone, 
    ) -> Result<Arc<Url<R>>, String> { 
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
            middlewares: RwLock::new(middleware), 
            params: RwLock::new(self.combine_params_for(&params)),  
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

    pub fn get_children(self: Arc<Self>, child: PathPattern) -> Result<Arc<Url<R>>, String> {
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
            middlewares: RwLock::new(vec!()), 
            params: RwLock::new(ParamsClone::new()), 
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
        self.childbirth(child, None, vec![], ParamsClone::default()) 
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
        function: Option<Arc<dyn AsyncFinalHandler<R>>>, 
        middleware: Vec<Arc<dyn AsyncMiddleware<R>>>, 
        params: ParamsClone, 
    ) -> Result<Arc<Url<R>>, String> { 
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
        let result = self.register(path_vec, function, middleware, params);
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
        function: Option<Arc<dyn AsyncFinalHandler<R>>>, 
        middleware: Vec<Arc<dyn AsyncMiddleware<R>>>, 
        params: ParamsClone, 
    ) -> Result<Arc<Self>, String> { 
        println!("Registering URL: {:?}", path); 
        if path.len() == 1 { 
            return self.childbirth(path[0].clone(), function, middleware, params); 
        } else { 
            println!("Recursion: Registering child URL: {:?}", path[0]); 
            return self.get_child_or_create(path[0].clone())?.register(path[1..].to_vec(), function, middleware, params); 
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

    pub fn set_method(&self, handler: Arc<dyn AsyncFinalHandler<R>>) {
        let mut guard = self.method.write().unwrap();
        *guard = Some(handler); 
    } 

    pub fn set_middlewares(&self, middlewares: Vec<Arc<dyn AsyncMiddleware<R>>>) {
        let mut guard = self.middlewares.write().unwrap(); 
        *guard = middlewares; 
    } 

    pub fn combine_params_for(&self, params: &ParamsClone) -> ParamsClone { 
        let guard = self.params.read().unwrap(); 
        let mut original = (*guard).clone(); 
        original.combine(params); 
        return original 
    } 

} 

pub fn dangling_url<R: Rx>() -> Arc<Url<R>> { 
    Arc::new(Url { 
        path: PathPattern::Any, 
        children: RwLock::new(Children::Nil), 
        ancestor: Ancestor::Nil, 
        method: RwLock::new(None), 
        middlewares: RwLock::new(vec!()), 
        params: RwLock::new(ParamsClone::default()), 
    }) 
} 
