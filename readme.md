# Starberry Web Framework

![Latest Version](https://img.shields.io/badge/version-0.6.4-brightgreen)
[![Crates.io](https://img.shields.io/crates/v/starberry)](https://crates.io/crates/starberry)
[![MIT License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

> Small, sweet, easy framework for full-stack Rust web applications

## 📋 Overview

Starberry is a lightweight, intuitive web framework focused on simplicity and productivity. It supports regex-based routing, tree-structured URLs, and integrates seamlessly with the Akari templating system.

**[Example project](https://github.com/Field-of-Dreams-Studio/starberry-example/tree/main)** 

MSRV: 1.86 

## ✨ Key Features

- **Simple API**: Intuitive request/response handling with minimal boilerplate
- **Full-Stack**: Built-in template rendering with Akari templates
- **Flexible Routing**: Support for regex patterns, literal URLs, and nested routes
- **Asynchronous**: Built with Tokio for efficient async handling
- **Form Handling**: Easy processing of form data and file uploads
- **Middleware Support**: Create reusable request processing chains 
- **Multi-Protocol Support**: Starberry is now planning to handle ingoing or outgoing http(s), db and other tcp protocals 

## 🎇 New Features 

### Multi Protocol and new App Mechanism 

```mermaid
flowchart TD
    RX1[Get Connection from TcpListener]
    RX2{Multiple Protocol Mode?}
    RX3["Test each protocol with Rx::test()"]
    RX4[Select first protocol returning true]
    RX5[Use that protocol type]
    RX6[Use the single protocol directly]
    RX7[Process request: read/write as needed]
    TX1[Build Tx and Connection]
    TX2["Tx::send(): send Response, get Request"]
    TX3{Is connection needed?}
    TX4[Close connection]
    TX5[Keep connection open]

    RX1 --> RX2
    RX2 -- Yes --> RX3
    RX3 --> RX4
    RX4 --> RX5
    RX5 --> RX7
    RX2 -- No --> RX6
    RX6 --> RX7
    RX7 --////--> TX1
    TX1 --> TX2
    TX2 --> TX3
    TX3 -- No --> TX4
    TX3 -- Yes --> TX5
``` 

As the diagram shown above, Rx (Receive) and Tx (Transmit) are the 2 most important traits introduced in Starberry 0.6. All of the protocols are wrapped with them. 

This means that you will be able to define your own protocol in App, and very soon you will be able to use socket and ftp in starberry. Currently Http and Sql has been implemented 

For Http, a simple method is provided for sending a request to a server 

```rust 
let response = HttpResCtx::send_request(
    "https://api.pmine.org",                                        // Host, protocol, port 
    get_request("/num/change/lhsduifhsjdbczfjgszjdhfgxyjey/36/2"),  // Request content 
    HttpSafety::new().with_max_body_size(25565),                    // Safety configuration 
)
.await
.unwrap(); 
``` 

There are also other protocols that does not provide a shortcut or you maybe want to use more advanced settings for http request, you may use the following way 

(1) Building a connection with the server 

```rust
let builder = ConnectionBuilder::new("example.com", 443)
    .protocol(Protocol::HTTP)
    .tls(true);
let connection = builder.connect().await.unwrap(); 
``` 

(2) Create a new Tx (HttpResCtx is the struct implemented Tx in Http) 

```rust 
let mut request = HttpResCtx::new(
    connection, 
    HttpSafety::new().with_max_body_size(25565), 
    "example.com"
); 
``` 

(3) Insert the request by using the `Tx::prcess()` function 

```rust 
let _ = request.process(request_templates::get_request("/")).await; 
``` 

Because we need to borrow the reader from the HttpResCtx so we didn't use the &mut Response from the process() function, we directly use HttpReqCtx.response to further process 

```rust 
request.response.parse_body(
    &mut request.reader,
    1024 * 1024,
).await;
println!("{:?}, {:?}", request.response.meta, request.response.body); 
``` 

### Protocol Registry 

Protocol registry now manages middleware chains and handler registration through a unified API 

The `App` type no longer owns a global `Url` tree or middleware list. 

URLs and middleware must now be configured on the **protocol** level during protocol registration. 

You

```rust 
/// APP only have one inbound protocol 
pub static APP: SAPP = Lazy::new(|| {
    App::new()
        .single_protocol(ProtocolBuilder::<HttpReqCtx>::new()) 
        .build() 
});  


/// APP need multiple inbound protocols 
pub static APP_MULTI: SApp = Lazy::new(|| {
    App::new() 
        .protocol(HandlerBuilder::new()
            .protocol(ProtocolBuilder::<HttpReqCtx>::new()) 
            // .protocol(ProtocolBuilder:: /* .. /* ) <- Add another protocol 
        )
        .build() 
}); 
``` 

### SQL Support (Starberry SQL v0.6.0)

Starberry now includes first-class PostgreSQL support via the `starberry_sql` crate. This async Rust library provides:

- Simple query building (`SqlQuery`/`sql!` macro)
- Type-safe row mapping with `FromRow`
- Connection pooling (`SqlPool`)
- Full transaction support
- Prepared statements & batch execution

**Basic Example:**
```rust
use starberry_sql::{DbConnectionBuilder, sql, FromRow, SslMode};

#[derive(FromRow)]
struct User { id: i32, name: String }

#[lit_url(APP, "/users")]
async fn get_users(mut ctx: HttpReqCtx) -> HttpResponse {
    let builder = DbConnectionBuilder::new("localhost", 5432)
        .database("my_db")
        .ssl_mode(SslMode::Disable);
    
    let mut conn = builder.connect().await.unwrap();
    let users: Vec<User> = sql!("SELECT id, name FROM users")
        .fetch_all_as(&mut conn)
        .await
        .unwrap();
    
    json_response(&users)
} 
``` 

### Safty setting for APP & Cors support 

We are now able to add config to both APP and urls. For APP, we are able to set the config/static by using methods 

```rust 
pub static APP: SApp = Lazy::new(|| {
    App::new()
        .single_protocol(ProtocolBuilder::<HttpReqCtx>::new() 
            .append_middleware::<CookieSession>() // Append the cors middleware. The cors middleware's setting is a AppCorsSettings data  
        ) 
        .set_config( // Add a config data, which is a TypeID, Any HashMap 
            prelude::cors_settings::AppCorsSettings::new() // Store a data with the type of AppCorsSettings into config. The middleware will be able to figure this stored data out by using its TypeID 
        ) 
        .set_local( // Set a static data, which is a String, Any HashMap. When getting data you need to specify both String and Type 
            "key": "MySuperSecuredAdminKeyWhichIsSoLongThatCanProtectTheServerFromAnyAttack" 
        )
        .build() 
}); 
``` 

While for the Url, we are able to store params by using the `#[url]` macro 

```rust 
#[url(APP.reg_from(&[TEST_URL.clone(), LitUrl("get")]), config=[HttpSafety::new().with_allowed_method(HttpMethod::GET)])]  
async fn get_only() -> HttpResponse { 
    text_response("Get only")  
} 

#[url(APP.reg_from(&[TEST_URL.clone(), LitUrl("post")]), config=[HttpSafety::new().with_allowed_methods(vec![HttpMethod::POST])])]  
async fn post_only() -> HttpResponse { 
    text_response("Post only")  
}
``` 

### Unify Http Request and Http Response 

You may see in the new Http mod, request and responses are in the same structure of 

```rust 
pub struct HttpRequest {
    pub meta: HttpMeta,
    pub body: HttpBody
}

pub struct HttpResponse { 
    pub meta: HttpMeta, 
    pub body: HttpBody 
}  
``` 

Where `HttpMeta` and `HttpBody` both implemented different methods for sending/parsing request/response 

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
│   ├── base.html
|   ├── index.html
|   └── ... 
└── build.rs 
``` 

Program file folder is used to store the config of the program data generated during the process of running the program. The files are automatically copied to the `dist` directory when you run `starberry build`. 

Templates are automatically copied to the `dist` directory when you run `starberry build`. 

### Quick tutorials 

You may visit our webpage to go through the quick tutorial for starberry 

https://fds.rs/starberry/tutorial/0.6.4/ <- New version 

https://fds.rs/starberry/tutorial/0.4.7/ 

### Stable Versions 

| Version | Download | Notes | 
| --- | --- | --- | 
| 0.6.4 | `cargo install starberry@0.6.4` | Multi Protocol Support | 
| 0.4.7 | `cargo install starberry@0.4.7` | Async + Request Context | 
| 0.3.3 | `cargo install starberry@0.3.3` | Sync Starberry | 

## 📋 Changelog 

### 0.6.4 
- Middleware added in core crate to prevent bad request 
- API optimized & Macro update 
- Bug fixes for content reading and parsing 

### 0.6.3 
- Bug fix for multiple set-cookie headers 
- Request templates now added with simpler methods for http request 
- Content Disposition now is available in http_value 
- Url structed and its constructive macro updated, setting associaled data/config and middleware is possible 
- **Important Workflow Change** 
    - A build.rs file will be generated automatically from 0.6.3-rc2, `starberry build/run/release` now behaves the same as `cargo build/run` 

### 0.6.2 
- Bug fix for Regex Url 
- Protocol Registery handling middleware and handlers 
- **Important Syntax Change** 
    - Now APP is no longer associated with a Url & Middleware. Url will be associated with protocol 
    - This means that you will need to to middleware operation when you creating protocol 
    - Please read more in our documentation: https://fds.rs/starberry/tutorial/0.6.4/ 

### 0.6.1 
- Language struct built in Http mod 
- Static file serving response (with auto assigned content type) 
- Bug fix & Update Akari 

### 0.6.0 
- Change the design of APP to enable multi-protocol 
- Added database support 
- Languages are readable from headers 

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

**All planned updates for 0.6 is already finished** 

## 📚 Learn More

Learn more about Akari template: https://crates.io/crates/akari/ 

Go to our homepage: https://fds.rs 

## 📄 License

MIT License
