## Latest stable version: 0.3.3 

# Starberry Web Framework 

Small, sweet, easy framework for full-stack web application 

Regex and other kinds of URLs are supported, and tree-structured URLs are used for easier URL management. 

Example project: https://github.com/Field-of-Dreams-Studio/starberry-example/tree/main 

Use plain HTML&Akari Template for better compatibility with other web projects and ease of learning. 

Learn more about Akari template: https://crates.io/crates/akari 

https://github.com/Redstone-D/starberry 

# Just updated 

0.4.5: Update with tokio, enable m:n scaling 

Better url reg macro. Enable debugging in Build mode. Note don't use this mode in production 

0.4.4: Enable early return if the request does not match with the allowed request method & allow content types 

(Bug fix) The blank project will no longer use the old syntax. `starberry version` impled   

0.4.3: Enable passing arguments and locals into Rc 

0.4.2: Rc is used to send the reponse. Now the request body will not be automatically being parsed 

(Important Syntax Update) Now the middleware chain passes the request context. When you registering function to url, the type passing in can be implied. You may return Rc or HttpRequest 

0.4.1: Updated middleware syntax 

0.4.0: Wrap the Request with Request context struct, providing access to App and Url config. Change the name preload into prelude 

(Important syntax update) Please accept Rc as argument instead of HttpRequest when you define an endpoint 

**Updates going to happen in 0.4 version** 

- [x] Warp HttpResponse with HttpContext 
- [x] No need to write mut HttpRequest in the argument, it will be automatically added 
- [x] Enabling function to read URL configuration & rules 
- [ ] Removing cookies 

- [x] Middlewares 
- [ ] Standard middlewre library for starberry (Oauth) 

- [x] Better URL configuration 
- [x] Url registering macro 
- [x] m:n thread scale 
- [x] Early abropt 

**Starberry now supports templateing through akari, and simpler URL definitions are enabled using macros.** 

## Why use Starberry? 

Do not need to know how starberry works, just use it, its simple 

**SIMPLE** 

In starberry you only need to do this to parse the form and return the form data as json 

```rust 
#[url(APP.lit_url("/submit"))] 
async fn handle_form() -> HttpResponse { 
    let form = req.form_or_default(); 
    akari_json!({ 
        name: form.get_or_default("name"), 
        age: form.get_or_default("age") 
    }) 
} 
``` 

While for setting a Cookie 

```rust 
#[url(APP.lit_url("/cookie"))] 
async fn set_cookie() -> HttpResponse { 
    text_response("Cookie Set").add_cookie( 
        Cookie::new("global_cookie", "something").path("/")  
    ) 
} 
``` 

**FULL-STACK** 

You are able to return a template in a dynamic way in starberry 

```rust 
#[url(APP.lit_url("/template"))] 
async fn template() -> HttpResponse { 
    akari_template!(
        "template.html", 
        title="My Website - Home", 
        page_title="Welcome to My Website", 
        show_message=true, 
        message="Hello, world!", 
        items=[1, 2, 3, 4, 5] 
    )
} 
``` 

template.html
```HTML
-[ template "base.html" ]-

-[ block head ]-
<link rel="stylesheet" href="style.css">
<meta name="description" content="My awesome page">
-[ endblock ]-

-[ block header ]-
<h1>-[ page_title ]-</h1>
<nav>
    <ul>
        <li><a href="/">Home</a></li>
        <li><a href="/about">About</a></li>
        <li><a href="/contact">Contact</a></li>
    </ul>
</nav>
-[ endblock ]-

-[ block content ]-
<div class="container">
    <h2>Welcome to our website</h2>
    
    -[ if show_message ]-
        <div class="message">-[ message ]-</div>
    -[ endif ]-
    
    <ul class="items">
        -[ for item items ]-
            <li class="item">-[ item ]-</li>
        -[ endfor ]-
    </ul>
</div>
-[ endblock ]- 
``` 

base.html 
```HTML 
<!DOCTYPE html>
<html>
<head>
    <title>-[ title ]-</title>
    -[ block head ]-
    <!-- Default head content -->
    -[ endblock ]-
</head>
<body>
    <header>
        -[ block header ]-
        <h1>Default Site Header</h1>
        -[ endblock ]-
    </header>
    
    <main>
        -[ block content ]-
        <p>Default content - override this</p>
        -[ endblock ]-
    </main>
    
    <footer>
        -[ block footer ]-
        <p>&copy; 2025 Starberry</p>
        -[ endblock ]-
    </footer>
</body>
</html>
``` 

# How to start a server & URL reg 

**Quick start**

```rust 
use starberry::prelude::*;  

#[tokio::main]  
async fn main() { 
    APP.clone().run().await; 
} 

pub static APP: SApp = Lazy::new(|| {
    App::new().build() 
}); 

#[lit_url(APP, "/")]
async fn home_route() -> HttpResponse {
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
#[url(APP.lit_url("/random/split/something"))]
async fn random_route() -> HttpResponse {
    text_response("A random page") 
}   
``` 

The code above defines a absolute url, "/random/split/something" returns text showing "A random page" 

```rust 
static TEST_URL: Lazy<Arc<Url>> = Lazy::new(|| {
    APP.reg_from(&[LitUrl("test")]) 
}); 

#[url(reg![&APP, LitUrl("hello")])]
async fn hello() -> HttpResponse {
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

# Http Resonse and Request 

After getting the request, you may use 

```rust 
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
``` 

For URL Coded form (datatype is application/x-www-form-urlencoded) you will get a hashmap of data and its form name by using this code 

```rust 
if *request.method() == POST { 
    match request.files() { 
        Some(form) => { 
            return text_response(form.get("file").unwrap().data().unwrap().to_owned()); 
        } 
        None => { 
            return text_response("Error parsing form"); 
        }  
    }  
} 
``` 

You may get the file 

# Update log 

0.3.3: Data sent from Url_encoded_form is now being automatically decoded. You may set random key for the Application. Default will be a 32 char long string generated randomly. Short cuts of form access has been enabled. redirect_response() function is now available to provide a redirect response. 

(Important Syntax Upgrade) akari_render! and akari_json! now can plug in functions, expressions, nested objects and so on inside, no need to first define then use. akari_render! now can accept zero arguments. Upgraded Akari into 0.2.0 

0.3.2: Re-export Url trait properly, enabled cookie manipulation. Enable request.get_path() to get segments of URL. Bug fix: Now "any" url can be proporly used. Upgraded Akari into 0.1.3 

0.3.1: Enabled reading files from request, multiple file in a single input can also be handled. Now nested JSON is supported and you may use akari_json! to directly return a JSON object. (Bug fix) Now starberry run is enabled. Optimized form reading 

0.3.0: Akari template in use. You may call `akari_render!` to return a HttpResponse using the template system. Json response are also ready for use. You may parse a json using Object module, a json can be generated using `object!` macro. 

The main program is updated. You may use `starberry new` to start a new project for starberry, also `starberry release` command is ready in use 

Read more about akari: https://crates.io/crates/akari 

0.2.3 Templates now in use. Please use `starberry build` instead of `cargo build` when building exe for running. The config of the command is the same 

0.2.2 Security enhancement: The request size, connection time is restricted automatically. Middlewares are implemented but not in use. Add preload modules 

0.2.1 Enable URL reg everywhere in the projects, enabling Regex, Any and Any Path to be URL. URLs are stored in a tree structure for easier config 

0.2.0 Update the url pattern. *For this version, Regex/Path/Any is no longer supported. It will be available in the next version.* 

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
 



