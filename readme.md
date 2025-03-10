## Latest stable version: 0.2.3 

# Starberry Web Framework 

Small, sweet, easy framework for full-stack web application 

Regex and other kinds of URLs are supported, and tree-structured URLs are used for easier URL management.

Use plain HTML for better compatibility with other web projects and ease of learning.

https://github.com/Redstone-D/starberry 

# Just updated 

**What's Next?** 

Version 0.2 has concluded, and development for version 0.3 will commence. The 0.3 update will be divided into two parts:

1. Templating system: This includes parsing expressions into templates, introducing for loops, while loops, and simple calculations within the template.
2. JSON parsing. 
3. Session and form manipulation.

**Starberry now supports plain HTML templates, and simpler URL definitions are enabled using macros.**

0.2.3 Templates now in use. Please use `starberry build` instead of `cargo build` when building exe for running. The config of the command is the same 

0.2.2 Security enhancement: The request size, connection time is restricted automatically. Middlewares are implemented but not in use. Add preload modules 

0.2.1 Enable URL reg everywhere in the projects, enabling Regex, Any and Any Path to be URL. URLs are stored in a tree structure for easier config 

0.2.0 Update the url pattern. *For this version, Regex/Path/Any is no longer supported. It will be available in the next version.* 

# How to start a server & URL reg 

**Quick start**

```rust 
use starberry::preload::*;  

#[tokio::main]  
async fn main() { 
    APP.clone().run().await; 
} 

pub static APP: SApp = Lazy::new(|| {
    App::new().build() 
}); 

#[lit_url(APP, "/")]
async fn home_route(_: HttpRequest) -> HttpResponse {
    text_response("Hello, world!") 
} 
``` 

You will be able to visit your server at localhost:3333 

Your project structure should be 

```
crate
├── src
│   ├── main.rs 
│   ├── lib.rs 
│   ├── ... 
└── templates
    ├── base.html
    ├── index.html
    └── ... 
``` 

The templates will automatically copied into the dist while you use starberry build 

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

(Request & Response) 

1. Session & Cookie manipulation 
2. Parsing form data (Finished, now fixing special character problems), uploading files 
3. Render Templates (Finished) 

# Example 

```rust 
use starberry::preload::*; 

#[tokio::main]  
async fn main() { 

    let furl = APP.clone().reg_from(&[LitUrl("flexible"), LitUrl("url"), LitUrl("may_be_changed")]); 
    furl.set_method(Arc::new(flexible_access)); 

    APP.clone().run().await; 
} 

pub static APP: SApp = Lazy::new(|| { 
    App::new()
        .binding(String::from("127.0.0.1:1111"))
        .mode(RunMode::Development)
        .workers(4) 
        .max_body_size(1024 * 1024 * 10) 
        .max_header_size(1024 * 10) 
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

static TEST_URL: SUrl = Lazy::new(|| {
    APP.reg_from(&[LitUrl("test")]) 
}); 

#[url(TEST_URL.clone(), LitUrl("/hello"))]
async fn hello(_: HttpRequest) -> HttpResponse { 
    text_response("Hello")  
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
// #[set_header_size(max_size: 2048, max_line_size: 1024, max_lines: 200)] 
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
    render_template("form.html") 
} 

async fn flexible_access(_: HttpRequest) -> HttpResponse { 
    text_response("Flexible") 
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
 
 