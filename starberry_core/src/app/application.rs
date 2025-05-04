use core::panic;
use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::hash::Hash;
use tokio::net::{TcpListener, TcpStream};

use std::pin::Pin;
use std::sync::{Arc, RwLock};
use std::time::Duration;
use std::{sync::mpsc}; 
use starberry_lib::random_string; 
use tokio::runtime::Runtime;
use std::future::Future;

use crate::app::middleware::LoggingMiddleware; 
use crate::app::urls;
use crate::context::Rc;

use super::super::http::http_value::*; 
use super::super::http::request::*;  
use super::super::http::response::*; 
use super::middleware::{self, AsyncMiddleware};
use super::urls::*;

/// RunMode enum to represent the mode of the application 
/// Production: Production mode 
/// Development: Test on developer's computer, showing the error message and some debug info. May contain sensitive info. 
/// Beta: Beta mode, showing some debug info. May contain some sensitive info. 
/// Build: Build mode. For building the starberry binary. Do not use this.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RunMode { 
    Production, 
    Development, 
    Beta, 
    Build, 
}

type Job = Pin<Box<dyn Future<Output = ()> + Send + 'static>>;

/// App struct modified to store binding address instead of TcpListener
pub struct App {
    pub root_url: Arc<Url>, 
    pub binding_address: String, // Changed from listener to binding_address
    pub mode: RunMode, 
    pub worker: usize, // Did not implemented 
    pub max_connection_time: usize, 
    pub connection_config: ParseConfig, 
    pub middlewares: Arc<Vec<Arc<dyn AsyncMiddleware>>>, 
    pub config: HashMap<String, Box<dyn Any + Send + Sync>>, 
    pub statics: HashMap<TypeId, Box<dyn Any + Send + Sync>>, 
}

/// Builder for App
pub struct AppBuilder { 
    root_url: Option<Arc<Url>>, 
    binding: Option<String>, 
    mode: Option<RunMode>, 
    worker: Option<usize>, 
    max_connection_time: Option<usize>, 
    max_header_size: Option<usize>, 
    max_body_size: Option<usize>, 
    max_line_length: Option<usize>, 
    max_headers: Option<usize>, 
    middle_wares: Option<Vec<Arc<dyn AsyncMiddleware>>>, 
    config: HashMap<String, Box<dyn Any + Send + Sync>>, 
    statics: HashMap<TypeId, Box<dyn Any + Send + Sync>>,
}

impl AppBuilder {
    pub fn new() -> Self { 
        Self {
            root_url: None,
            binding: None,
            mode: None, 
            worker: None, 
            max_connection_time: None,
            max_header_size: None,
            max_body_size: None,
            max_line_length: None,
            max_headers: None,
            middle_wares: Some(Self::default_middlewares()), 
            config: HashMap::new(), 
            statics: HashMap::new(), 
        }
    }

    pub fn default_middlewares() -> Vec<Arc<dyn AsyncMiddleware>> { 
        vec![]
    }

    pub fn root_url(mut self, root_url: Arc<Url>) -> Self { 
        self.root_url = Some(root_url); 
        self 
    }

    pub fn binding(mut self, binding: String) -> Self { 
        self.binding = Some(binding); 
        self 
    }

    pub fn mode(mut self, mode: RunMode) -> Self { 
        self.mode = Some(mode); 
        self 
    } 

    pub fn worker(mut self, threads: usize) -> Self {
        self.worker = Some(threads);
        self
    } 

    pub fn max_connection_time(mut self, max_connection_time: usize) -> Self { 
        self.max_connection_time = Some(max_connection_time); 
        self 
    }

    pub fn max_header_size(mut self, max_header_size: usize) -> Self { 
        self.max_header_size = Some(max_header_size); 
        self 
    }

    pub fn max_body_size(mut self, max_body_size: usize) -> Self { 
        self.max_body_size = Some(max_body_size); 
        self 
    }

    pub fn max_line_length(mut self, max_line_length: usize) -> Self { 
        self.max_line_length = Some(max_line_length); 
        self 
    }

    pub fn max_headers(mut self, max_headers: usize) -> Self { 
        self.max_headers = Some(max_headers); 
        self 
    }

    // Append a middleware instance created by T to the end of the vector.
    pub fn append_middleware<T: 'static + AsyncMiddleware>(mut self) -> Self {
        let middleware = Arc::new(T::return_self());
        if let Some(middle_wares) = &mut self.middle_wares {
            middle_wares.push(middleware);
        } else {
            self.middle_wares = Some(vec![middleware]);
        }
        self
    }

    // Insert a middleware instance created by T at the beginning of the vector.
    pub fn extend_middleware<T: 'static + AsyncMiddleware>(mut self) -> Self { 
        let middleware = Arc::new(T::return_self());
        if let Some(middle_wares) = &mut self.middle_wares {
            middle_wares.insert(0, middleware);
        } else {
            self.middle_wares = Some(vec![middleware]);
        }
        self
    } 

    pub fn remove_middleware<T: 'static + AsyncMiddleware>(mut self) -> Self { 
        if let Some(middle_wares) = &mut self.middle_wares {
            middle_wares.retain(|m| {
                // Keep the middleware if it's NOT of type T
                !m.as_any().is::<T>() 
            });
        } 
        self  
    } 

    pub fn set_statics<T: 'static + Send + Sync>(mut self, value: T) -> Self { 
        self.statics.insert(TypeId::of::<T>(), Box::new(value)); 
        self 
    } 

    pub fn set_config<T: 'static + Send + Sync>(mut self, key: impl Into<String>, value: T) -> Self {
        self.config.insert(key.into(), Box::new(value)); 
        self 
    } 

    /// Build method: create the `App`, storing binding address without creating a TcpListener
    pub fn build(self) -> Arc<App> { 
        let root_url = match self.root_url { 
            Some(root_url) => root_url, 
            None => {
                Arc::new(Url {
                    path: PathPattern::Literal(String::from("/")),
                    children: RwLock::new(Children::Nil),
                    method: RwLock::new(None),
                    ancestor: Ancestor::Nil,
                    middlewares: RwLock::new(MiddleWares::Nil),
                    params: RwLock::new(Params::default()),
                })
            }
        };

        // Just store the binding address, don't bind yet
        let binding_address = self.binding.unwrap_or_else(|| String::from("127.0.0.1:3003")); 
        
        let mode = self.mode.unwrap_or_else(|| RunMode::Development);
        let worker = self.worker.unwrap_or_else(|| num_cpus()); 
        let max_connection_time = self.max_connection_time.unwrap_or_else(|| 5); 
        let max_header_size = self.max_header_size.unwrap_or_else(|| 1024 * 1024); 
        let max_body_size = self.max_body_size.unwrap_or_else(|| 1024 * 512); 
        let max_line_length = self.max_line_length.unwrap_or_else(|| 1024 * 64); 
        let max_headers = self.max_headers.unwrap_or_else(|| 100);
        let connection_config = ParseConfig::new(
            max_header_size,
            max_line_length,
            max_headers,
            max_body_size
        );

        Arc::new(App {
            root_url,
            binding_address,
            mode, 
            worker, 
            max_connection_time,
            connection_config,
            middlewares: self
                .middle_wares
                .unwrap_or_else(|| Self::default_middlewares())
                .into(), 
            config: self.config, 
            statics: self.statics, 
        })
    }
}

impl App {
    pub fn new() -> AppBuilder { 
        AppBuilder::new() 
    }

    pub fn set_root_url(&mut self, root_url: Arc<Url>) { 
        self.root_url = root_url; 
    }

    pub fn get_binding(self: &Arc<Self>) -> String { 
        self.binding_address.clone()
    }

    pub fn set_mode(&mut self, mode: RunMode) { 
        self.mode = mode; 
    }

    pub fn get_mode(self: &Arc<Self>) -> RunMode { 
        self.mode.clone() 
    } 

    pub fn set_max_connection_time(&mut self, max_connection_time: usize) { 
        self.max_connection_time = max_connection_time; 
    }

    pub fn get_max_connection_time(self: &Arc<Self>) -> usize { 
        self.max_connection_time 
    }

    pub fn set_max_header_size(&mut self, max_header_size: usize) { 
        self.connection_config.set_max_header_size(max_header_size); 
    }

    pub fn get_max_header_size(self: &Arc<Self>) -> usize { 
        self.connection_config.get_max_header_size() 
    }

    pub fn set_max_body_size(&mut self, max_body_size: usize) { 
        self.connection_config.set_max_body_size(max_body_size); 
    }

    pub fn get_max_body_size(self: &Arc<Self>) -> usize { 
        self.connection_config.get_max_body_size() 
    }

    pub fn set_max_line_length(&mut self, max_line_length: usize) { 
        self.connection_config.set_max_line_length(max_line_length); 
    }

    pub fn get_max_line_length(self: &Arc<Self>) -> usize { 
        self.connection_config.get_max_line_length() 
    }

    pub fn set_max_headers(&mut self, max_headers: usize) { 
        self.connection_config.set_max_headers(max_headers); 
    }

    pub fn get_max_headers(self: &Arc<Self>) -> usize { 
        self.connection_config.get_max_headers() 
    } 

    pub fn statics<T: 'static + Send + Sync>(self: &Arc<Self>) -> Option<&T> {
        self.statics
            .get(&TypeId::of::<T>())
            .and_then(|boxed| boxed.downcast_ref::<T>())
    } 

    pub fn config<T: 'static + Send + Sync>(self: &Arc<Self>, key: &str) -> Option<&T> {
        self.config
            .get(key)
            .and_then(|boxed| boxed.downcast_ref::<T>())
    } 

    /// This function add a new url to the app. It will be added to the root url 
    /// # Arguments 
    /// * `url` - The url to add. It should be a string.
    pub fn lit_url<T: Into<String>>(
        self: &Arc<Self>, 
        url: T, 
    ) -> Arc<super::urls::Url> { 
        let url = url.into(); 
        println!("Adding url: {}", url); 
        match self.root_url
            .clone()
            .literal_url(&url, None, Some(self.middlewares.clone()), Params::default())
        {
            Ok(url) => url,
            Err(_) => super::urls::dangling_url(),
        }
    }

    /// Handle a single connection
    pub fn handle_connection(self: Arc<Self>, stream: TcpStream) {
        let app = Arc::clone(&self);
        let job = async move {
            let rc = Rc::handle(app.clone(), stream).await;
            rc.run().await; 
        };
        tokio::spawn(job);
    } 

    /// Main loop listening for connections - now creates the TcpListener at runtime
    pub async fn run(self: Arc<Self>) {
        // let runtime = tokio::runtime::Builder::new_multi_thread()
        // .worker_threads(self.worker)
        // .enable_all()
        // .build()
        // .unwrap(); 
        
        // Create TcpListener only when run() is called, within the tokio runtime
        let listener = match TcpListener::bind(&self.binding_address).await {
            Ok(listener) => listener,
            Err(e) => panic!("Binding failed on {}: {}", self.binding_address, e),
        };
        
        println!("Connection established on {}", listener.local_addr().unwrap());
        
        // Create a signal handler for clean shutdown
        let (shutdown_tx, mut shutdown_rx) = tokio::sync::oneshot::channel::<()>();
        
        // Handle Ctrl+C for clean shutdown
        tokio::spawn(async move {
            if let Ok(_) = tokio::signal::ctrl_c().await {
                println!("Received shutdown signal");
                let _ = shutdown_tx.send(());
            }
        });
        
        loop {
            tokio::select! {
                accept_result = listener.accept() => {
                    match accept_result {
                        Ok((stream, addr)) => {
                            // println!("Accepted connection from {addr}");
                            Arc::clone(&self).handle_connection(stream);
                        }
                        Err(e) => { 
                            if self.get_mode() == RunMode::Build{ 
                                eprintln!("Failed to accept connection: {e}"); 
                            } 
                        }
                    }
                }
                _ = &mut shutdown_rx => {
                    println!("Shutting down server...");
                    break;
                }
            }
        }

        tokio::time::sleep(Duration::from_secs(1)).await;
        println!("Server shutdown complete");
    }

    pub fn app_url(
        self: &Arc<Self>,
        segments: &[PathPattern]
    ) -> Result<Arc<Url>, String> {
        let mut current = self.root_url.clone();
        for seg in segments { 
            current = current.get_child_or_create(seg.clone())?; 
            current.set_middlewares(Some(self.middlewares.clone())); 
        }
        Ok(current)
    }

    pub fn reg_from(
        self: &Arc<Self>,
        segments: &[PathPattern]
    ) -> Arc<Url> { 
        match self.app_url(segments){ 
            Ok(url) => url, 
            Err(_) => {
                urls::dangling_url()
            }
        }
    }
} 

// Helper function for determining CPU count
fn num_cpus() -> usize {
    match std::thread::available_parallelism() {
        Ok(n) => n.get(),
        Err(_) => 1, // Fallback if we can't determine
    }
} 
