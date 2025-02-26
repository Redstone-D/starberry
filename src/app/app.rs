use std::sync::Arc;
use std::{net::TcpListener, thread}; 
use std::net::TcpStream;    
use super::super::http::http_value::*; 
use super::super::http::request::*;  
use super::super::http::response::*; 
use super::urls::*; 
use std::io::Write; 
use tokio::runtime::Runtime; 
use tokio::spawn;

pub struct App {
    pub root_url: Url, 
    pub listener: TcpListener, 
    pub mode: RunMode, 
} 

/// RunMode enum to represent the mode of the application 
/// Production: Production mode 
/// Development: Test on developer's computer, showing the error message and some debug info. May contain sensitive info. 
/// Beta: Beta mode, showing some debug info. May contain some sensitive info. 
/// Build: Build mode. For building the starberry binary. Do not use this. 
pub enum RunMode { 
    Production, 
    Development, 
    Beta, 
    Build, 
} 

impl App { 
    pub async fn request(&self, request: HttpRequest) -> HttpResponse { 
        let path = request.path.clone(); 
        let mut path = path.split('/').collect::<Vec<&str>>(); 
        path.remove(0); 
        println!("{:?}", path); 
        let url = Arc::new(&self.root_url); 
        let url: Option<_> = (*url).walk(path.iter()).await; 
        if let Some(url) = url { 
            return url.run(request).await; 
        } else { 
            return HttpResponse::new(HttpVersion::Http11, StatusCode::NOT_FOUND, String::from("Not Found")); 
        } 
    }  

    // Note: This function is now synchronous, and expects that `self` is shared via an Arc.
    pub fn handle_connection(self: Arc<Self>, mut stream: TcpStream) {
        // Spawn a new OS thread for this connection.
        thread::spawn(move || {
            // Create a new Tokio runtime in this thread.
            let rt = Runtime::new().expect("Failed to create Tokio runtime");
            rt.block_on(async {
                // Parse the HTTP request from the stream.
                if let Ok(request) = HttpRequest::from_request_stream(&mut stream).await {
                    // Process the request asynchronously and send the response.
                    self.request(request).await.send(&mut stream).await;
                }
            });
        });
    } 

    pub async fn run(self: Arc<Self>) { 
        for stream in self.listener.incoming() {
            let stream = stream.unwrap();
            Arc::clone(&self).handle_connection(stream); 
        } 
    } 
} 
