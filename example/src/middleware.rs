use starberry::prelude::*; 

#[middleware]
pub async fn MyMiddleWare1(){ 
    println!("Middleware: Received request for {}, start processing", req.path()); 
    next(req).await   
}  

#[middleware]
pub async fn MyMiddleWare2(){ 
    let path = req.path().to_owned(); 
    let a = next(req).await; 
    println!("Middleware: Received request for {}, end processing", path); // You cannot access to req here 
    a 
}  

#[middleware]
pub async fn MyMiddleWare3(){ 
    if req.path() == "/directly_return" { 
        req.response = text_response("Directly return"); 
        req 
    } else {
        next(req).await 
    } 
} 

#[middleware] 
pub async fn MyMiddleWare4(){ 
    println!("Middleware: Received request for {}, start processing", req.path()); 
    req.locals.set("some_value", 5); 
    req.params.set(true); 
    next(req).await 
} 

#[middleware] 
pub async fn MyMiddleWare5(){ 
    req = next(req).await; 
    let value = req.locals.take::<i32>("some_value").unwrap_or(0); 
    let param = req.params.take::<bool>().unwrap_or(false); 
    println!("Local: {}, Params: {}", value, param); 
    req 
}
