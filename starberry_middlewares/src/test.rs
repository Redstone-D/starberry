use super::FutureResponse; 
use starberry_macro::middleware; 

#[middleware]
pub async fn MyMiddleWare(req: HttpRequest) -> FutureResponse {
    println!("Middleware: Received request for {}", req.path()); 
    next(req)  
}  
