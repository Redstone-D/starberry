# Please refer to the following link 

https://crates.io/crates/starberry 

# The standard starberry middleware library 

# Print Log 

### Function 

By appending `PrintLog` middleware into APP, APP will print a log for each Http Request 

### APP Statics & Configs 

N/A 

### Request Context 

N/A 

### Example 

```rust 
pub static APP: SApp = Lazy::new(|| {
    App::new().append_middleware::<PrintLog>().build()
}); 
``` 

# Session 

### Function 

By using `Session` middleware, you will be able to store session in the memory 

### APP Statics & Configs 

**session_ttl: u64**, set the time for session to expire 

### Request Context 

**SessionRW**, access the Session content from this 

### Example 

```rust 
pub static APP: SApp = Lazy::new(|| {
    App::new().append_middleware::<PrintLog>().build()
}); 
``` 
