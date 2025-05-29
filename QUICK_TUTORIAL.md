# Starberry Documentation for 0.6.1 (Core) 

# Chapter 0: New Concept Introduced 0.5.x and 0.6.x 

### Multi Protocol and new App Mechanism 

```mermaid
flowchart TD
    RX1[Get Connection from TcpListener] --> RX2{Multiple Protocol Mode?}
    RX2 -- Yes --> RX3[Test each protocol with Rx::test()]
    RX3 --> RX4[Select first protocol returning true]
    RX4 --> RX5[Use that protocol type]
    RX5 --> RX7[Process request: read/write as needed]

    RX2 -- No --> RX6[Use the single protocol directly]
    RX6 --> RX7[Process request: read/write as needed]

    RX7 -.-> TX1[Build Tx and Connection]
    TX1 --> TX2[Tx::send(): send Response, get Request]
    TX2 --> TX3{Is connection needed?}
    TX3 -- No --> TX4[Close connection]
    TX3 -- Yes --> TX5[Keep connection open]
``` 

### Request context, Rx, Tx 

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

# Chapter 1: Hello Starberry! 

### Installation: 

You can install starberry by using the following commandL 

```
cargo install starberry 
``` 

Once it is installed, you can run the following command to create a new starberry project: 

``` 
starberry new HelloStarberry 
``` 

This will automatically creates a folder called HelloStarberry. A minimal starberry project will be generated 

### The simpliest project 

Go to `src/main.rs`, let's dig dive into each line of code and see what they do. 

```rust main.rs 
use starberry::prelude::*;
``` 

The first line of code shown above imports everything from `starberry::prelude`, this is basically everything you need for a standard starberry project 

```rust main.rs 
#[tokio::main]
async fn main() {
    APP.clone().run().await;
} 
``` 

In the tokio environment (note you don't need to import tokio because starberry did that for you automatically), you clone the App instance called APP and run it. 

Note that you must clone the instance before running it since APP is a global instance wrapped with Lazy. 

```rust 
pub static APP: SApp = once_cell::sync::Lazy::new(|| {
    App::new().build()
}); 
``` 

This defines a Static App variable by using lazy. Note that you also don't need to import Lazy since starberry did this automatically. 

```rust 
#[url(APP.lit_url("/"))] 
``` 

`APP.lit_url('/')` meaning that generate a Url instance from APP where the url is "/". By passing this into the `#[url()]` macro this links the function below with this url instance. 

```rust 
async fn home_route() -> HttpResponse {
    text_response("Hello from Starberry!")
} 
``` 

This function returns a text HttpResponse which is `Hello from Starberry!` 

After compiling the app and run it, you may visit localhost:3003 (the default port for starberry), it will returns "Hello from Starberry!" 

# Chapter 2: Basic URL pattern types 

Url is associated with "Patterns" in starberry. Url patterns is an enum consists of 4 types 

| Type name | Actual Name | Note | 
| --- | --- | --- | 
| LitUrl(String) | Literal(String) | | 
| RegUrl(String) | Regex(String) | | 
| AnyUrl() | AnyUrl | This only accepts one string. A path will not be accepted | 
| AnyDict() | AnyPath | This accepts a path, a string will also be accepted | 
| TrailingSlash() | Literal("") | | 

*Note: Type name is the function you actually use in your application to define urlpattern where the Actural name is the actual name defined in the enum. You will not use the Actual name very often 

By using `reg!` macro, you may associate patterns with App or Url instance to register them 

For example, 

```rust 
#[url(reg![&APP, LitUrl("test"), RegUrl("[0-9]+")])]  
async fn testnumber() -> HttpResponse { 
    text_response("Number page") 
} 
``` 

Then you may visit this page through the url of `/test/123` (Actually you can visit the same page with any number after /test/) 

*Note: Please notice there is no trailing slash. The url with or without trailing slash are different 

While also note that each UrlPattern can only match one part of the path. Which means that you must not use `LitUrl("aaa/bbb")`, instead you should use `LitUrl(aaa), LitUrl("bbb")` 

There is another way of registering url, 

```rust
APP.reg_from(&[LitUrl("test"), RegUrl("[0-9]+")]) 
``` 

This provides the same effect as the one above, but the grammar is much complicated. Since starberry 0.4.5, we provide the macro to register the Url 

You may also define a static url pattern for easier change: 

```rust 
static TEST_URL: SPattern = Lazy::new(|| {LitUrl("test")}); 
``` 

Then you may use 

```rust 
#[url(reg![&APP, TEST_URL, RegUrl("[0-9]+")])]   
``` 

To register. 

Note if you use the old pattern, you must use the `.clone` method and add a Reference sign `&` before the TEST_URL. This is the reason why we strongly don't recommand you to use the old grammar. 

# Chapter 3: Return something dynamic & Introduction to Akari 

### Getting the nunmer from the url 

Starberry accepts dynamic url pattern, but how we are able to be benefited from this ability? Such as returning the number the user input after `/test/` 

In some other frameworks, we may define something like this: 

```python 
@app.url("/route/<number>") 
def number_function(number): 
    return render(number) 
``` 

Unfortunately, due to the support of Regex, starberry don't allow you to define names for each part of the url. 

You can only access the url through `req.get_path(<int: The part of the url>)` 

Let's change that function a little bit, 

```rust 
#[url(reg![&APP, LitUrl("test"), RegUrl("[0-9]+")])]  
async fn testnumber() -> HttpResponse { 
    text_response(req.get_path(1)) 
} 
``` 

This will get the **second** part of the url, which is the number part, in a text format. 

### Returning a Json or Template 

Except for text, akari gives us the ability to return in Json or Templates. 

We may use `akari_template!()` or `akari_json!()` to return them, below are 2 examples of how to do this 

```rust 
#[url(reg![&APP, LitUrl("test"), RegUrl("[0-9]+")])]  
async fn testnumber() -> HttpResponse { 
    akari_json!({ 
        number: req.get_path(1)
    }) // Return {number: ###} 
} 
``` 

OR 

```rust
#[url(reg![&APP, LitUrl("test"), RegUrl("[0-9]+")])]  
async fn testnumber() -> HttpResponse { 
    akari_template!({
        "number.html":
        number=req.get_path(1)
    }) 
} 
``` 

Where you need to add a new file into template folder which is at the same level as the src folder, 

```html number.html 
<h1>-[ number ]-</h1> 
``` 

So that a header will render the number the user input 

### Akari Json and Introdcution to Akari Templates 

With the build-in ability of reading json files and templating in akari, we can easily manipulate json and render dynamic templates 

Akari json, officially akari object, can be constructed by using the `object!` macro. It is a json-like structure, and you are able to read and write data from it. 

```rust 
use akari::object; 
object!({
    number: 3, 
    string: "Hello", 
    array: [1, 2, 3], 
    object: { 
        a: 1, 
        b: 2, 
        c: 3 
    }
}) 
``` 

Then you can create a Json. 

Where you can also use 

```rust 
use akari::object; 
use akari::Object; // Notice the capital O meaning that Object is a struct not macro 

let json = r#"{"key": "value", "number": 42, "list": [1, 2, 3]}"#; 
let obj = Object::from_json(json).expect("Failed to parse JSON"); 
let dir = "D://test/test.json"; 
Object::from_jsonf(dir).unwrap_or(Object::None); // Read a json from a file 
obj.into_jsonf(dir); // Write obj into the dir 
``` 

While various of methods are provided to read a value in json. 

Be carefun about the difference between `obj.to_string()`, `obj.string()` and `obj.into_json()` 

Akari template is a templating language, which wrap its syntax in `-[ ... ]-`. We will further discuss this in Chapter 9 

# Chapter 4: HttpRequest and HttpResponse 

# Chapter 5: Deep seek into Http 

HttpRequest and HttpResponse consists of two parts, HttpMeta and HttpBody 

HttpMeta is "Lazy loading", which means that when generating the HttpMeta, it will just fetch all data as unparsed from the Buffered Reader and store it in a `HashMap: <String, String>`. 

Where when user tries to use the associated method, such as `get_content_type()`, starberry will do the following things 

(1): Check whether content_type has been cached 
(2): If it is being cached, directly return the cached data. **IT WILL NOT CHECK WHETHER THE HASHMAP HAS BEEN MODIFIED**, since the hashmap is designed to be write for only once, when it is initializing 
(3): If not, get the String data of content type from the hashmap, and convert it into HttpContentType. Cache it and return the data 

The same design is introduced in HttpBody also. The response will not be automatically being parsed. When you call the body getting method such as `Rc::form()` or `Rc::json()`, it first reads the buffer then store the compiled data into cache. 

Since 0.5, the Response and Request shares the same HttpBody and HttpMeta. However in 0.4 these attributes only applies to Request side 

# Chapter 6: Request context, Standard Middleware and Building Application 

### Request Context 

Request context, (we call this "Rc" for short in starberry) is a struct consisting the Request Meta, Request Body, Request Stream, Response, locals and params. 

As discussed in Chapter 4 and 5, we already know how to read data from request and send response. In this chapter we are going to focus more on Stream, locals and params. 

Stream is a buffer reader, reading the TcpStream that the user send to the server. Since the RequestBody is lazy load, you may read the body on your own without relying on Starberry's HttpBody. While it is technically possible to read data the user send after we receive the request, however this does not happen in Http1.1. Starberry will be able to handle Http2.0 response later on 

For locals and params, they are 2 sets of data passing through Middleware Chain (which we are going to discuss later on). 

Where params stores a value in the type-based params storage. Any previous value of the same type will be replaced. 

You may do something like 

```rust 
// Store authentication information
req.set_param(User { id: 123, name: "Alice".to_string() });
 
// Store timing information
req.set_param(RequestTimer::start()); 

// In an authentication process. We do not write this syntax in actual middlewares 
if let Some(user) = req.param::<User>() {
    println!("Request by: {}", user.name);
    // Proceed with authenticated user
} else {
    return HttpResponse::unauthorized();
} 

// Update a request timer
if let Some(timer) = req.param_mut::<RequestTimer>() {
    timer.mark("after_db_query");
} 

// Take ownership of a value
if let Some(token) = req.take_param::<AuthToken>() {
    // Use and consume the token
    validate_token(token);
} 
``` 

To store data. No key is provided. 

However for locals, it stores a value in the string-based locals storage with the given key. Any previous value with the same key will be replaced 

```rust 
req.set_local("user_id", 123);
req.set_local("is_premium", true);
req.set_local("cart_items", vec!["item1", "item2"]); 

// In a request handler
if let Some(is_premium) = req.local::<bool>("is_premium") {
    if *is_premium {
        // Show premium content
    }
}

// With different types
let user_id = req.local::<i32>("user_id");
let items = req.local::<Vec<String>>("cart_items"); 

// Modify a list of items
if let Some(items) = req.local_mut::<Vec<String>>("cart_items") {
    items.push("new_item".to_string());
} 

// Take ownership of a value
if let Some(token) = req.take_local::<String>("session_token") {
    // Use and consume the token
    validate_and_destroy_token(token);
} 
``` 

### Standard Middleware 

Installing sbmstd to use starberry's standard middleware library 

### Building Application 

In Chapter 1, we talked about starting a fairly application 

```rust 
pub static APP: SApp = once_cell::sync::Lazy::new(|| {
    App::new().build()
}); 
``` 

Now let's deep dive into setting configs, middlewares and settings in the application 

The statement 

```rust 
App::new() 
``` 

Initiates an AppBuilder instance. For the AppBuilder instance a set methods passing its owned value in while returning a modified owned value out is provided. 

After manipulating and setting the variable into the AppBuilder, we use 

```rust 
AppBuilder::build() 
``` 

To build and return a App instance. **Once a APP instance is built, you are not allowed to change its config**. 

For example, in the Starberry example project the following code is provided 

```rust 
    App::new() 
        .binding(String::from("127.0.0.1:1111"))
        .mode(RunMode::Build)
        .max_body_size(1024 * 1024 * 10) 
        .max_header_size(1024 * 10) 
        .append_middleware::<PrintLog>() 
        .append_middleware::<MyMiddleWare2>() 
        .insert_middleware::<MyMiddleWare1>() 
        .set_config("serect_key", "key") 
        .set_statics("static".to_string())
        .build() 
``` 

Let's talk about each function 

- Binding: The way of accessing the application 
- Mode: Production (Production environment), Development (Developing the application), Beta (Testing the application publically), Build (Internal testing for Starberry development) 
- Max body size, max header size 
- Append middleware: Append a middleware in the end of the middleware chain 
- Insert middleware: Insert a middleware in the head of the middleware chain 
- Set config: the same as set_local in Rc 
- Set statics: the same as set_params in Rc 

After that the APP is built and run 

# Chapter 7: Form, file and Akari Json 

# Chapter 8: Cookies & Session 

# Chapter 9: Advanced Akari operations & templating 

### Akari Object 

Akari Object, as discussed in Chapter 3, is a Json like data structure. This also leads to the result of that Akari Object is never type safe 

Akari Object is able to hold 5 types of value, they are: numerical, boolean, String, List and Object 

You may extract the value from Akari by using `match` or `if let`, just like any other Rust enums. You may also use a shorthand, just append the data type after the value, such as 

```rust 
let a = object!(3); 
assert!(a.numerical, 3.0); // Akari stores numericals as float 
assert!(a.boolean, true); // Akari applies auto conversion, if != 0.0 then return true 
``` 

### Akari template 

**Note: Some grammar does not implemented in 0.2.2 (The version used in default in Starberry 0.4.7), but you may update this manually** 

**Extend, Insert and Blocks** 

**Accessing dictionaries and lists** 

# Chapter 10: Middlewares 

### The easiest middleware 

Defining a new middleware in Starberry is farily simple. By using `#[middleware]` macro, starberry will automatically transfer your function into a middleware. The middleware will have the same struct name as the function. 

Please write your function name in UpperCamelCase, not the normal sname_case 

```rust 
#[middleware]
pub async fn MyMiddleWare1(){ 
    println!("Middleware: Received request for {}, start processing", req.path()); 
    next(req)  
}  
``` 

The code above shows a most simple middleware, which has the functionality of printing the path of requests 

Let's break down line by line. 

```rust 
#[middleware] 
pub async fn MyMiddleWare1(){ 
``` 

The first line gives the middleware name. It accepts one argument, Rc. The reason why we keep the parameter empty is that starberry automatically added `mut req: Rc` in the parameter for us, so we don't need to write it 

If you really want to change the name, you may explicitly write this into the parameter, then you can use your custom name 

```rust 
    println!("Middleware: Received request for {}, start processing", req.path());  
``` 

Do the thing this middleware needed to do 

```rust 
    next(req)  
``` 

The next is a keyword in starberry middleware. You need to pass your current request context into it, then it will pass the Rc into the next middleware, the middleware chain continues 

```rust 
} 
``` 

Closing the middleware function, and directly returns next(req) 

### Run after the middleware chain 

```rust 
#[middleware]
pub async fn MyMiddleWare2(){ 
    let path = req.path().to_owned(); 
    let a = next(req).await; 
    println!("Middleware: Received request for {}, end processing", path); // You cannot access to req here 
    a.boxed_future() // You must return a future 
}  
``` 

The middleware above prints a statement after the middleware chain. 

Note that now you must use the boxed_future since async middleware expects you return a future 

### Early abort  

```rust 
#[middleware]
pub async fn MyMiddleWare3(){ 
    if req.path() == "/directly_return" { 
        req.response = text_response("Directly return"); 
        req.boxed_future() 
    } else {
        next(req) 
    } 
} 
``` 

This middleware checks whether the path is `/directly_return`, if true it directly returns a text_response by setting the req's response into a HttpResponse and not passing the Rc into the middleware chain 

### Accessing values 

Please refer to the chapter of Request Context. In the middleware you may read and write in request contest's locals and params for passing values 
