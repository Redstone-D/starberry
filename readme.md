# Starberry Web Framework 

This is a async, light-weighted web framework 100% coded in rust. 

Regex and other kind of URL is supported, tree structred URL is being used for easier URL management 

This is still in developmental stage 

https://github.com/Redstone-D/starberry 

# Just updated 

0.1.3: Use thread pooling, enable user to set number of threads. Use better URL approach 

0.1.2: Updated Request Analyze, Debug to not Generate Panic. Let the program capable for async (The 0.1.1 async is fake) 

## How to start a server? 

`
pub async fn test() {
    let app = Arc::new(App {
        root_url: init_urls(),
        listener: TcpListener::bind("127.0.0.1:3003").unwrap(),
        mode: RunMode::Development,
    });
    app.run().await;
}
` 
