use std::pin::Pin; 
use std::future::Future; 
use super::super::context::Rc; 
use std::any::Any; 

pub trait AsyncMiddleware: Send + Sync + 'static { 
    fn as_any(&self) -> &dyn Any; 

    fn return_self() -> Self where Self: Sized; 

    fn handle<'a>( 
        &self,
        rc: Rc,
        next: Box<dyn Fn(Rc) -> Pin<Box<dyn Future<Output = Rc> + Send>> + Send + Sync + 'static>,
    ) -> Pin<Box<dyn Future<Output = Rc> + Send + 'static>>;
} 

pub struct LoggingMiddleware;

impl AsyncMiddleware for LoggingMiddleware {
    fn handle<'a>(
        &'a self,
        context: Rc, 
        next: Box<dyn Fn(Rc) -> Pin<Box<dyn Future<Output = Rc> + Send>> + Send + Sync + 'a>,
    ) -> Pin<Box<dyn Future<Output = Rc> + Send + 'static>> {
        println!("Logging: Received request for {}", context.path());
        next(context) 
    }
    
    fn as_any(&self) -> &dyn Any {
        self
    } 

    fn return_self() -> Self {
        LoggingMiddleware
    } 
} 
