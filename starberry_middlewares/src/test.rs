use super::FutureResponse; 
use starberry_macro::build_macro; 

#[build_macro()]
pub async fn MyMiddleWare(req: HttpRequest) -> FutureResponse {
    println!("Middleware: Received request for {}", req.path()); 
    next(req)  
}  
