use std::pin::Pin; 
use std::future::Future; 
use super::super::http::request::HttpRequest; 
use super::super::http::response::HttpResponse; 
use std::any::Any; 

pub trait AsyncMiddleware: Send + Sync + 'static { 
    fn as_any(&self) -> &dyn Any; 

    fn return_self() -> Self where Self: Sized; 

    fn handle<'a>( 
        &self,
        req: HttpRequest,
        next: Box<dyn Fn(HttpRequest) -> Pin<Box<dyn Future<Output = HttpResponse> + Send>> + Send + Sync + 'static>,
    ) -> Pin<Box<dyn Future<Output = HttpResponse> + Send + 'static>>;
} 

pub struct LoggingMiddleware;

impl AsyncMiddleware for LoggingMiddleware {
    fn handle<'a>(
        &'a self,
        req: HttpRequest, 
        next: Box<dyn Fn(HttpRequest) -> Pin<Box<dyn Future<Output = HttpResponse> + Send>> + Send + Sync + 'a>,
    ) -> Pin<Box<dyn Future<Output = HttpResponse> + Send + 'static>> {
        println!("Logging: Received request for {}", req.path());
        next(req) 
    }
    
    fn as_any(&self) -> &dyn Any {
        self
    } 

    fn return_self() -> Self {
        LoggingMiddleware
    } 
} 
