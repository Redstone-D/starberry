use once_cell::sync::Lazy;
use starberry::{App, RunMode}; 
use starberry::{LitUrl, RegUrl, AnyUrl, AnyPath}; 
use starberry::urls::*; 
use starberry::{HttpRequest, HttpResponse}; 
use starberry::{text_response, html_response};  
use starberry::{lit_url, url}; 
use starberry::HttpMethod::*; 
use std::sync::Arc; 
use std::thread::sleep; 
use std::time::Duration; 

#[tokio::main]  
async fn main() { 
    let furl = APP.clone().reg_from(&[LitUrl("flexible"), LitUrl("url"), LitUrl("may_be_changed")]); 
    furl.set_method(Arc::new(flexible_access)); 

    APP.clone().run().await; 
} 

pub static APP: Lazy<Arc<App>> = Lazy::new(|| {
    App::new()
        .binding(String::from("127.0.0.1:1111"))
        .mode(RunMode::Development)
        .workers(4)
        .build() 
}); 

#[lit_url(APP, "/")]
async fn home_route(_: HttpRequest) -> HttpResponse { 
    html_response("<h1>Home</h1>") 
} 

#[lit_url(APP, "/random/split/something")]
async fn random_route(_: HttpRequest) -> HttpResponse {
    text_response("A random page") 
}  

static TEST_URL: Lazy<Arc<Url>> = Lazy::new(|| {
    APP.reg_from(&[LitUrl("test")]) 
}); 

#[url(TEST_URL.clone(), LitUrl("/hello"))]
async fn hello(_: HttpRequest) -> HttpResponse {
    text_response("Hello, world!") 
} 

#[url(TEST_URL.clone(), LitUrl("/async_test"))] 
async fn async_test(_: HttpRequest) -> HttpResponse {
    sleep(Duration::from_secs(1));
    println!("1");
    sleep(Duration::from_secs(1));
    println!("2");
    sleep(Duration::from_secs(1));
    println!("3");
    text_response("Async Test Page") 
} 

#[url(TEST_URL.clone(), RegUrl("/async_test2"))]  
async fn async_test2(_: HttpRequest) -> HttpResponse {
    sleep(Duration::from_secs(1));
    println!("1");
    sleep(Duration::from_secs(1));
    println!("2");
    sleep(Duration::from_secs(1));
    println!("3");
    text_response("Async Test Page") 
} 

#[url(TEST_URL.clone(), RegUrl("[0-9]+"))]  
async fn testa(_: HttpRequest) -> HttpResponse { 
    text_response("Number page") 
} 

#[url(TEST_URL.clone(), LitUrl("form"))]  
async fn test_form(request: HttpRequest) -> HttpResponse { 
    println!("Request to this dir"); 
    if *request.method() == POST { 
        match request.form() { 
            Some(form) => { 
                return text_response(format!("Form data: {:?}", form)); 
            } 
            None => { 
                return text_response("Error parsing form"); 
            }  
        } 
    } 
    let file: String = match std::fs::read_to_string("form.html"){ 
        Ok(file) => file, 
        Err(e) => { 
            println!("Error reading file: {}", e); 
            return text_response("Error reading file"); 
        } 
    }; 
    html_response(file) 
} 

async fn flexible_access(_: HttpRequest) -> HttpResponse { 
    text_response("Flexible") 
} 
