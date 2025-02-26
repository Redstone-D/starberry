## Latest stable version: N/A 

# Starberry Web Framework 

This is a async, light-weighted web framework 100% coded in rust. 

Regex and other kind of URL is supported, tree structred URL is being used for easier URL management 

https://github.com/Redstone-D/starberry 

# Just updated 

0.1.3: Use thread pooling, enable user to set number of threads. Use better URL approach 

0.1.2: Updated Request Analyze, Debug to not Generate Panic. Let the program capable for async (The 0.1.1 async is fake) 

## How to start a server? 

```
use starberry::app::app::App;
use starberry::app::urls;
use starberry::http::http_value::*;
use starberry::http::response::*;
use std::sync::Arc; 
use starberry::app::app::RunMode;
use std::time::Duration; 
use std::thread::sleep; 

pub async fn test() {
    let mut app = App::new(init_urls().into()); 
    app.set_binding("127.0.0.1:1111"); 
    app.set_mode(RunMode::Development); 
    app.set_workers(4); 
    let runner = Arc::new(app); 
    runner.run().await; 
}

pub fn init_urls() -> urls::Url {
    urls::Url {
        path: urls::PathPattern::Literal("/".to_string()),
        children: urls::Children::Some(vec![
            Arc::new(urls::Url {
                path: urls::PathPattern::Literal("about".to_string()),
                children: urls::Children::Nil,
                method: Some(Box::new(|_req| async {
                    HttpResponse::new( 
                        HttpVersion::Http11, 
                        StatusCode::OK,
                        String::from("About Page"),
                    )
                })),
            }),
            Arc::new(urls::Url {
                path: urls::PathPattern::Regex("[0-9]+".to_string()),
                children: urls::Children::Nil,
                method: Some(Box::new(|_req| async {
                    HttpResponse::new(
                        HttpVersion::Http11,
                        StatusCode::OK,
                        String::from("Number page"),
                    )
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
                    HttpResponse::new(
                        HttpVersion::Http11,
                        StatusCode::OK,
                        String::from("Async Test Page"),
                    )
                })),
            }),
        ]),
        method: Some(Box::new(|_req| async {
            HttpResponse::new(
                HttpVersion::Http11,
                StatusCode::OK,
                String::from("Home Page"),
            )
        })), 
    }  
}

``` 
