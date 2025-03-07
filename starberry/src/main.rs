#[tokio::main]  

async fn main() { 
    APP.clone().run().await; 
} 

use once_cell::sync::Lazy;
use starberry::{App, RunMode};
use starberry_core::http::request::HttpRequest;
use starberry_core::http::response::HttpResponse; 
use std::sync::Arc;
use std::thread::sleep; 
use starberry::text_response;  
use starberry::lit_url; 
use std::time::Duration; 

pub static APP: Lazy<Arc<App>> = Lazy::new(|| {
    App::new()
        .binding(String::from("127.0.0.1:1111"))
        .mode(RunMode::Development)
        .workers(4)
        .build() 
}); 

#[lit_url(APP, "/")]
async fn home_route(_: HttpRequest) -> HttpResponse {
    text_response("Hello, world!") 
} 


#[lit_url(APP, "/random/split/something")]
async fn random_route(_: HttpRequest) -> HttpResponse {
    text_response("A random page") 
} 

#[lit_url(APP, "/async_test")]
async fn async_test(_: HttpRequest) -> HttpResponse {
    sleep(Duration::from_secs(1));
    println!("1");
    sleep(Duration::from_secs(1));
    println!("2");
    sleep(Duration::from_secs(1));
    println!("3");
    text_response("Async Test Page") 
} 

#[lit_url(APP, "/async_test2")] 
async fn async_test2(_: HttpRequest) -> HttpResponse {
    sleep(Duration::from_secs(1));
    println!("1");
    sleep(Duration::from_secs(1));
    println!("2");
    sleep(Duration::from_secs(1));
    println!("3");
    text_response("Async Test Page") 
} 
