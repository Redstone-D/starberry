use starberry::prelude::*; 
use example::APP; 

#[tokio::main]
async fn main() {
    APP.clone().run().await;
} 

mod resource;
