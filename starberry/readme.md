# Starberry Web Framework

![Latest Version](https://img.shields.io/badge/version-0.4.7-brightgreen)
[![Crates.io](https://img.shields.io/crates/v/starberry)](https://crates.io/crates/starberry)
[![MIT License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

> Small, sweet, easy framework for full-stack Rust web applications

## 📋 Overview

Starberry is a lightweight, intuitive web framework focused on simplicity and productivity. It supports regex-based routing, tree-structured URLs, and integrates seamlessly with the Akari templating system.

**[Example project](https://github.com/Field-of-Dreams-Studio/starberry-example/tree/main)**

## ✨ Key Features

- **Simple API**: Intuitive request/response handling with minimal boilerplate
- **Full-Stack**: Built-in template rendering with Akari templates
- **Flexible Routing**: Support for regex patterns, literal URLs, and nested routes
- **Asynchronous**: Built with Tokio for efficient async handling
- **Form Handling**: Easy processing of form data and file uploads
- **Middleware Support**: Create reusable request processing chains 
- **Multi-Protocal Support**: Starberry is now planning to handle ingoing or outgoing http(s), ws(s), db and other tcp protocals. This feature will come with the **0.6 stable version** 

## 🎇 New Features 

## 🚀 Getting Started

### Installation

Install starberry bu using 

```
cargo install starberry 
``` 

After installing starberry, use 

```
starberry new <Proj_name> 
``` 

To create a new project 

### Quick Start

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

Visit your server at `http://localhost:3003`

### Project Structure

```
project/
├── src/
│   ├── main.rs
│   ├── lib.rs
│   └── ... 
├── programfiles/ 
│   ├── config.json 
│   └── ... 
└── templates/
    ├── base.html
    ├── index.html
    └── ... 
``` 

Program file folder is used to store the config of the program data generated during the process of running the program. The files are automatically copied to the `dist` directory when you run `starberry build`. 

Templates are automatically copied to the `dist` directory when you run `starberry build`. 

### Stable Versions 

| Version | Download | Notes | 
| --- | --- | --- | 
| 0.6.4 | Not released | | 
| 0.4.7 | `cargo install starberry@0.4.7` | Async + Request Context | 
| 0.3.3 | `cargo install starberry@0.3.3` | Sync Starberry | 

## 📝 Usage Guide

### URL Registration

#### Method 1: Using Macros (Recommended)

```rust
// Absolute URL
#[url(APP.lit_url("/random/split/something"))]
async fn random_route() -> HttpResponse {
    text_response("A random page")
}

// Relative URL with parent
#[url(reg![&APP, LitUrl("hello")])]
async fn hello() -> HttpResponse {
    text_response("Hello, world!")
}
```

#### Method 2: Dynamic Registration

```rust
let furl = APP.clone().reg_from(&[LitUrl("flexible"), LitUrl("url")]);
furl.set_method(Arc::new(flexible_access));
```

### URL Pattern Types

| Type | Description | Example |
|------|-------------|---------|
| `LitUrl(&str)` | Matches literal path segment | `LitUrl("users")` |
| `RegUrl(&str)` | Matches regex pattern | `RegUrl("[0-9]+")` |
| `AnyUrl` | Matches any single path segment | `AnyUrl` |
| `AnyDir` | Matches any number of path segments | `AnyDir` |

### Request Handling

```rust
#[url(APP.lit_url("/submit"))]
async fn handle_form() -> HttpResponse {
    if request.method() == POST {
        // Form data (application/x-www-form-urlencoded)
        let form = req.form_or_default().await;
        
        // Access form fields
        let name = form.get_or_default("name");
        let age = form.get_or_default("age");
        
        // File uploads
        if let Some(files) = request.files().await {
            if let Some(file) = files.get("file") {
                // Process file
                let file_data = file.data().unwrap();
                // ...
            }
        }
        
        return akari_json!({
            name: form.get_or_default("name"),
            age: form.get_or_default("age")
        });
    }
    
    text_response("Method not allowed").with_status(405)
}
```

### Templating

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

**Template Example:**

```html
-[ template "base.html" ]-

-[ block head ]-
<link rel="stylesheet" href="style.css">
<meta name="description" content="My awesome page">
-[ endblock ]-

-[ block content ]-
<div class="container">
    <h2>-[ page_title ]-</h2>
    
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

### Working with Cookies

```rust
#[url(APP.lit_url("/cookie"))]
async fn set_cookie() -> HttpResponse {
    text_response("Cookie Set").add_cookie(
        Cookie::new("global_cookie", "something").path("/")
    )
}
```

### JSON Responses

```rust
#[url(APP.lit_url("/api/data"))]
async fn api_data() -> HttpResponse {
    akari_json!({
        success: true,
        data: {
            id: 1,
            name: "Example",
            values: [10, 20, 30]
        }
    })
}
```

### Redirects

```rust
#[url(APP.lit_url("/redirect"))]
async fn redirect() -> HttpResponse {
    redirect_response("/new-location")
}
```

## 📋 Changelog 

### 0.6.0 
- Change the design of APP to enable multi-protocol 
- Added database support 

### 0.5.1 
- Reform Cookie and CookieMap, deleted the CookieResponse struct 
- Merge Request's Meta & Body and Response's into one unified struct 
- **Important Bug Fix** 
    - Able to send Redirect Response. Redirect Response will send nothing in previous 0.5.x version 
    - Starberry now can be compiled on Windows devices 

### 0.5.0 
- Enable sending requests through a Connection 
- Argumented URL 

### 0.4.x and earlier 
- Request Context holding all contexts of a request 
- Simplified middleware definition pattern & Standard middleware library 
- Added Akari templating support
- Added cookie manipulation
- File upload handling
- Form data processing improvements 

## 🔮 Planned Updates

**All planned updates for 0.4 is already finished** 

(0.5 ver) 
- [x] Standard middleware library (Session) 
- [x] Merge HttpRequestBody and HttpResponseBody together 
- [x] Enable sending simple requests to external server 
- [x] Enable form and file send() & get_size() out to external server 
- [x] URL segment parameters 

- [x] Merge HttpRequestMeta and HttpResponseMeta together 
- [x] Optimize Cookie data structure, unify Request Cookie and Response Cookie together 
- [ ] Grammar sugar for Regex 

- [ ] Middleware libraries (OAuth, Surreal) 
- [ ] Static file serving 
- [ ] Logging 

## 📚 Learn More

Learn more about Akari template: https://crates.io/crates/akari

## 📄 License

MIT License
 
 