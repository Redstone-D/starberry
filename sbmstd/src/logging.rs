pub use starberry_macro::middleware; 
pub use starberry_core::context::Rc; 
pub use starberry_core::app::middleware::AsyncMiddleware; 

#[middleware] 
pub async fn PrintLog(){ 
    req = next(req).await;  
    print!("[Request Received] Method: "); 
    print!("{}, ", req.method()); 
    print!("Path: "); 
    print!("{}, ", req.path()); 
    print!("Status Code: "); 
    println!("{}, ", req.response.meta.start_line.status_code()); 
    req.boxed_future() // Abropting the middleware chain 
} 
