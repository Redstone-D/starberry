## Latest stable version: 0.1.5 

# Starberry Web Framework 

This is a async, light-weighted web framework 100% coded in rust. 

Regex and other kind of URL is supported, tree structred URL is being used for easier URL management 

https://github.com/Redstone-D/starberry 

# Just updated 

0.2.0 Update the url pattern. *For this version, Regex/Path/Any is no longer supported. It will be available in the next version.* 

# How to start a server 

```rust 
#[tokio::main]  

async fn main() { 
    APP.clone().run().await; 
} 

use once_cell::sync::Lazy;
use starberry::{App, RunMode};
use starberry_core::http::request::HttpRequest;
use starberry_core::http::response::HttpResponse; 
use std::sync::Arc; 
use starberry::text_response;  
use starberry::lit_url; 

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

```

# Update log 

0.1.5: Reexport the methods and enable dynamic function loading. Enable &str, String, Vec<u8> and so on to act as the response body 

You are able to start the surver by using the following codes 


```use starberry::app::app::App;
use starberry::App;
use starberry::urls; 
use starberry::text_response; 
use starberry::RunMode;

use std::time::Duration; 
use std::thread::sleep; 
use std::sync::Arc; 

pub async fn test() {
    let app = App::new(init_urls().into()) 
    .binding(String::from("127.0.0.1:1111")) 
    .mode(RunMode::Development) 
    .workers(4) 
    .build(); 
    let runner = Arc::new(app); 
    runner.run().await; 
}

pub fn init_urls() -> urls::Url {
    urls::Url {
        path: urls::PathPattern::literal_path("/"),
        children: urls::Children::Some(vec![
            Arc::new(urls::Url {
                path: urls::PathPattern::literal_path("about"),
                children: urls::Children::Nil,
                method: Some(Box::new(|_req| async {
                    text_response("About Page")
                })),
            }),
            Arc::new(urls::Url {
                path: urls::PathPattern::regex_path("[0-9]+"), 
                children: urls::Children::Nil,
                method: Some(Box::new(|_req| async {
                    text_response("Number page")
                })),
            }),
            Arc::new(urls::Url {
                path: urls::PathPattern::Any,
                children: urls::Children::Nil,
                method: Some(Box::new(|_req| async {
                    sleep(Duration::from_secs(1));
                    println!("1");
                    sleep(Duration::from_secs(1));
                    println!("2");
                    sleep(Duration::from_secs(1));
                    println!("3");
                    text_response("Async Test Page")
                })),
            }),
        ]),
        method: Some(Box::new(|_req| async {
            text_response("Home Page") 
        })), 
    }  
} 

#[tokio::main]  

async fn main() {
    test::test().await; 
}  

``` 

0.1.4: Optimized way in starting app, optimized Response class 

0.1.3: Use thread pooling, enable user to set number of threads. Use better URL approach 

0.1.2: Updated Request Analyze, Debug to not Generate Panic. Let the program capable for async (The 0.1.1 async is fake) 
