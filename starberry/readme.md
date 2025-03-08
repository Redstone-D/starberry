## Latest stable version: 0.1.5 

# Starberry Web Framework 

Light-weighted framework for full-site framework, managing both the frontend and backend 

Regex and other kind of URL is supported, tree structred URL is being used for easier URL management 

https://github.com/Redstone-D/starberry 

# Just updated 

0.2.1 Enable URL reg everywhere in the projects, enabling Regex, Any and Any Path to be URL. URLs are stored in a tree structure for easier config 

0.2.0 Update the url pattern. *For this version, Regex/Path/Any is no longer supported. It will be available in the next version.* 

# How to start a server & URL reg 

**Quick start**

```rust 
#[tokio::main]  

async fn main() { 
    APP.clone().run().await; 
} 

use once_cell::sync::Lazy;
use starberry::{App, RunMode}; 
use starberry::{LitUrl, RegUrl, AnyUrl, AnyPath}; 
use starberry::urls::*; 
use starberry::{HttpRequest, HttpResponse}; 
use starberry::{text_response, html_response};  
use starberry::{lit_url, url}; 
use starberry::HttpMethod::*; 
use std::sync::Arc; 

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
``` 

You will be able to visit your server at localhost:1111. If you does not bind with 1111 port, the default port is 3333 

## Registering URL 

In Starberry, the URL can be registered anywhere in your project, in absolute or relative way 

We assume that you have created an App named APP as static variable as the code shown in quick start 

**Without a function** 

```rust 
#[lit_url(APP, "/random/split/something")]
async fn random_route(_: HttpRequest) -> HttpResponse {
    text_response("A random page") 
}  
``` 

The code above defines a absolute url, "/random/split/something" returns text showing "A random page" 

```rust 
static TEST_URL: Lazy<Arc<Url>> = Lazy::new(|| {
    APP.reg_from(&[LitUrl("test")]) 
}); 

#[url(TEST_URL.clone(), LitUrl("/hello"))]
async fn hello(_: HttpRequest) -> HttpResponse {
    text_response("Hello, world!") 
} 
``` 

The code above does this in a relative way. First we define a Literal URL (We are going to discuss other type of URLs shortly) of "test" called TEST_URL 

After that, we use url() macro, passing TEST_URL and a literal URL pattern. Starberry will concat those two together, meaning you can visit this through "/test/hello" 

This is the way we recommended for making a URL tree, so that URL management will be easier and leading to a more dynamic program in the future 

**Within a function** 

```rust 
    let furl = APP.clone().reg_from(&[LitUrl("flexible"), LitUrl("url"), LitUrl("may_be_changed")]); 
    furl.set_method(Arc::new(flexible_access)); 
``` 

Getting the URL instance from APP so that we can set URL for them. 

If you would like to have a more dynamic one, please consider use the child in the url instance (we recommand you to do so) 

## URL types 

**LitUrl(&str)** 

The normal URL, work the way as you think, just a literal 

**RegUrl(&str)** 

You input a regex, starberry will match it in the run time 

**AnyUrl** 

Also know as Any if you directly use starberry::urls::PathPatten. Accept any literal 

**AnyDir** 

Also know as AnyPath if you directly use starberry::urls::PathPatten. Accept any number of literal after this 

# TBD 

(Akatemp) 

1. Input data from macro 
2. Parsing expressions 

(Request & Response) 

1. Session & Cookie manipulation 
2. Parsing form data (Finished, now fixing special character problems), uploading files 
3. Render Templates 

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
