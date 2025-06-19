use starberry::{prelude::*, ContentDisposition};   

pub use crate::APP; 

static TEST_URL: SPattern = Lazy::new(|| {LitUrl("async")}); 

#[url(APP.reg_from(&[TEST_URL.clone(), LitUrl("channel1")]))] 
async fn async_test() -> HttpResponse {
    sleep(Duration::from_secs(1));
    println!("1");
    sleep(Duration::from_secs(1)); 
    println!("2");
    sleep(Duration::from_secs(1));
    println!("3");
    text_response("Async Test Page") 
} 

#[url(APP.reg_from(&[TEST_URL.clone(), RegUrl("channel2")]))]  
async fn async_test2() -> HttpResponse {
    sleep(Duration::from_secs(1));
    println!("1");
    sleep(Duration::from_secs(1));
    println!("2");
    sleep(Duration::from_secs(1));
    println!("3");
    text_response("Async Test Page") 
} 

#[url(reg![&APP, TEST_URL, RegUrl("[0-9]+")])]  
async fn testa() -> HttpResponse { 
    text_response("Number page") 
} 

#[url(APP.reg_from(&[TEST_URL.clone(), LitUrl("get_serect_key")]))]  
async fn get_serect_key() -> HttpResponse {
    text_response(req.app.config().get::<String>().cloned().unwrap_or("No key".to_string()))  
}   

#[url(APP.reg_from(&[TEST_URL.clone(), LitUrl("file")]))] 
async fn file() -> HttpResponse {
    let file_path = "programfiles/example.txt"; // Adjust the path as needed 
    match std::fs::read(file_path) {
        Ok(content) => normal_response(StatusCode::OK, content).content_disposition(ContentDisposition::attachment("example.txt")),
        Err(e) => text_response(format!("Error reading file: {}", e)),
    }
} 
