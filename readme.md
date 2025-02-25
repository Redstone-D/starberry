# Starberry Web Frame 

This is still in developmental stage 

https://github.com/Redstone-D/starberry 

## How to start a server? 

`
let app = Arc::new(App {
    root_url: init_urls(),
    listener: TcpListener::bind("127.0.0.1:3003").unwrap(), 
    mode: RunMode::Development, 
});
app.run().await; 
` 
