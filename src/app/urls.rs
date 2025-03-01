use super::super::http::request::*; 
use super::super::http::response::*; 
use super::super::http::http_value::*; 
use std::future::Future;
use std::pin::Pin;
use std::slice::Iter; 
use std::sync::Arc;
use std::sync::OnceLock;
use regex::Regex; 
pub static ROOT_URL: OnceLock<Url> = OnceLock::new();  

pub trait AsyncUrlHandler: Send + Sync + 'static {
    fn handle(
        &self, 
        req: HttpRequest
    ) -> Pin<Box<dyn Future<Output = HttpResponse> + Send + '_>>;
}

impl<F, Fut> AsyncUrlHandler for F
where
    F: Fn(HttpRequest) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = HttpResponse> + Send + 'static,
{
    fn handle(
        &self, 
        req: HttpRequest
    ) -> Pin<Box<dyn Future<Output = HttpResponse> + Send + '_>> {
        Box::pin(self(req))
    }
}

pub struct Url {
    pub path: PathPattern,
    pub children: Children,
    pub method: Option<Box<dyn AsyncUrlHandler>>, // Now uses our new trait
}

#[derive(Clone, Debug)]
pub enum PathPattern {
    Literal(String),
    Regex(String), 
    Any,
    AnyPath,
}

pub enum Children {
    Nil,
    Some(Vec<Arc<Url>>),
} 

impl Url { 
    pub async fn run(&self, request: HttpRequest) -> HttpResponse { 
        // Runs the function inside it 
        if let Some(method) = &self.method { 
            return method.handle(request).await; 
        } 
        return return_status(StatusCode::NOT_FOUND); 
    } 

    pub fn walk<'a>(self: Arc<Self>, mut path: Iter<'a, &str>) -> Pin<Box<dyn Future<Output = Option<Arc<Self>>> + Send + 'a>> {
        Box::pin(async move {
            let this = match path.next() {
                Some(path) => *path,
                None => "",
            };

            if this == "" {
                return Some(self); // Return `Arc<Self>` directly
            }

            if let Children::Some(children) = &self.children {
                for child_url in children.iter() {
                    let child = Arc::clone(child_url); // Clone `Arc` to make it sendable
                    match &child.path {
                        PathPattern::Literal(p) => {
                            if p == this {
                                if path.len() > 1 {
                                    return child.walk(path).await; // Recurse
                                } else {
                                    return Some(child); // Return the child directly
                                }
                            }
                        }
                        PathPattern::Regex(p) => {
                            let re = Regex::new(&p).unwrap();
                            if re.is_match(this) {
                                if path.len() > 1 {
                                    return child.walk(path).await; // Recurse
                                } else {
                                    return Some(child); // Return the child directly
                                }
                            }
                        }
                        PathPattern::Any => {
                            if path.len() > 1 {
                                return child.walk(path).await; // Recurse
                            } else {
                                return Some(child); // Return the child directly
                            }
                        }
                        PathPattern::AnyPath => {
                            return Some(child); // Return the child directly
                        }
                    }
                }
            }
            None // No match found
        })
    } 
} 

