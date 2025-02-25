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
    pub path: String,
    pub children: Children,
    pub method: Option<Box<dyn AsyncUrlHandler>>, // Now uses our new trait
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
                    let re = Regex::new(&child.path).unwrap();
                    if re.is_match(this) {
                        if path.len() > 1 {
                            return child.walk(path).await;
                        } else { 
                            return Some(&child);
                        }
                    }
                }
            }
            None
        })
    } 

} 




