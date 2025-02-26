use super::super::http::request::*; 
use super::super::http::response::*; 
use super::super::http::http_value::*; 
use std::future::Future;
use std::pin::Pin;
use std::slice::Iter; 
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
    Some(Vec<Url>),
} 

impl Url { 
    pub async fn run(&self, request: HttpRequest) -> HttpResponse { 
        // Runs the function inside it 
        if let Some(method) = &self.method { 
            return method.handle(request).await; 
        } 
        return HttpResponse::new(HttpVersion::Http11, StatusCode::NOT_FOUND, String::from("Not Found")); 
    } 

    pub fn walk<'a>(&'a self, mut path: Iter<'a, &str>) -> Pin<Box<dyn Future<Output = Option<&'a Self>> + 'a>> {
        Box::pin(async move { 
            let this = match path.next(){ 
                Some(path) => *path, 
                None => "", 
            }; 
            if this == "" { 
                return Some(self); 
            } 
            if let Children::Some(c) = &self.children {
                for child in c.iter() { 
                    // println!("{} {:?}", this, &child.path);
                    match &child.path { 
                        PathPattern::Literal(p) => { 
                            if p == this { 
                                println!("Running: {} {:?}", this, &child.path); 
                                if path.len() > 1 { 
                                    return child.walk(path).await; 
                                } else { 
                                    return Some(&child); 
                                } 
                            } 
                        }, 
                        PathPattern::Regex(p) => { 
                            let re = Regex::new(p).unwrap(); 
                            if re.is_match(this) { 
                                println!("Running: {} {:?}", this, &child.path); 
                                if path.len() > 1 { 
                                    return child.walk(path).await; 
                                } else { 
                                    return Some(&child); 
                                } 
                            } 
                        }, 
                        PathPattern::Any => { 
                            println!("Running: {} {:?}", this, &child.path); 
                            if path.len() > 1 { 
                                return child.walk(path).await; 
                            } else { 
                                return Some(&child); 
                            }  
                        },  
                        PathPattern::AnyPath => {  
                            println!("Running: {} {:?}", this, &child.path); 
                            return Some(&child);  
                        },  
                    } 
                }
            }
            None
        })
    } 

} 

