use std::pin::Pin; 
use std::future::Future; 
use super::super::context::Rc; 
use std::any::Any; 
use crate::http::response::request_templates::return_status;
use crate::http::http_value::StatusCode;

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

/// Middleware to enforce HTTPS by rejecting non-HTTPS requests
pub struct HttpsEnforcement;

impl AsyncMiddleware for HttpsEnforcement {
    fn as_any(&self) -> &dyn Any { self }

    fn return_self() -> Self where Self: Sized {
        HttpsEnforcement
    }

    fn handle<'a>(
        &'a self,
        mut ctx: Rc,
        next: Box<dyn Fn(Rc) -> Pin<Box<dyn Future<Output = Rc> + Send>> + Send + Sync + 'static>,
    ) -> Pin<Box<dyn Future<Output = Rc> + Send + 'static>> {
        Box::pin(async move {
            // Check X-Forwarded-Proto header for HTTPS enforcement
            let proto_opt = ctx.meta().header.get("x-forwarded-proto")
                .map(|hv| hv.first().to_lowercase());
            if proto_opt != Some("https".to_string()) {
                // Reject non-HTTPS
                ctx.response = return_status(StatusCode::UPGRADE_REQUIRED);
                return ctx;
            }
            // Continue to next middleware or handler
            next(ctx).await
        })
    }
} 
