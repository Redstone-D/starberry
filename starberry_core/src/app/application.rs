use core::panic;
// use std::collections::HashMap; 
use tokio::net::{TcpListener, TcpStream};

// use starberry_lib::random_string;
use std::future::Future;
use std::pin::Pin; 
use std::sync::Arc;
use std::time::Duration;
// use tokio::runtime::Runtime;

use crate::app::protocol::{ProtocolHandlerBuilder, ProtocolRegistryBuilder};
use crate::app::urls;
use crate::connection::Connection;
use crate::connection::Rx;

use crate::extensions::{Params, Locals}; 
use crate::http::context::HttpReqCtx;

// use super::middleware::AsyncMiddleware;
use super::protocol::ProtocolRegistryKind;
use super::urls::*;

/// RunMode enum to represent the mode of the application
/// Production: Production mode
/// Development: Test on developer's computer, showing the error message and some debug info. May contain sensitive info.
/// Beta: Beta mode, showing some debug info. May contain some sensitive info.
/// Build: Build mode. For testing starberry itself. It will print out any information possible. 
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
    pub binding_address: String,
    pub handler: ProtocolRegistryKind, // Changed from listener to binding_address
    pub mode: RunMode,
    pub worker: usize, // Did not implemented
    pub max_connection_time: usize, 
    pub config: Params,
    pub statics: Locals,
}

/// Builder for App
pub struct AppBuilder {
    binding_address: Option<String>,
    handler: Option<ProtocolRegistryKind>,
    mode: Option<RunMode>,
    worker: Option<usize>,
    max_connection_time: Option<usize>, 
    config: Params, 
    statics: Locals, 
}

impl AppBuilder {
    pub fn new() -> Self {
        Self {
            binding_address: None,
            handler: None,
            mode: None,
            worker: None,
            max_connection_time: None, 
            config: Params::new(),  
            statics: Locals::new(), 
        }
    }

    pub fn binding<T: Into<String>>(mut self, binding: T) -> Self {
        self.binding_address = Some(binding.into());
        self
    }

    pub fn handler(mut self, protocol: ProtocolRegistryKind) -> Self {
        self.handler = Some(protocol);
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

    pub fn statics(mut self, statics: Locals) -> Self {
        self.statics = statics; 
        self
    } 

    pub fn config(mut self, config: Params) -> Self {
        self.config = config; 
        self
    } 

    /// Build method: create the `App`, storing binding address without creating a TcpListener
    pub fn build(self) -> Arc<App> {
        let handler = match self.handler {
            Some(root_url) => root_url,
            None => ProtocolRegistryBuilder::new()
                .protocol(ProtocolHandlerBuilder::<HttpReqCtx>::new())
                .build(),
        };

        let binding_address = self
            .binding_address
            .unwrap_or_else(|| String::from("127.0.0.1:3003"));
        let mode = self.mode.unwrap_or_else(|| RunMode::Development);
        let worker = self.worker.unwrap_or_else(|| num_cpus());
        let max_connection_time = self.max_connection_time.unwrap_or_else(|| 5);  

        Arc::new(App {
            handler,
            binding_address,
            mode,
            worker,
            max_connection_time, 
            config: self.config,
            statics: self.statics,
        })
    }
}

impl App {
    pub fn new() -> AppBuilder {
        AppBuilder::new()
    }

    pub fn get_protocol_address<T: Rx>(&self) -> String {
        unimplemented!()
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

    pub fn config(self: &Arc<Self>) -> &Params {
        &self.config 
    } 

    pub fn statics(self: &Arc<Self>) -> &Locals {
        &self.statics
    } 

    /// This function add a new url to the app. It will be added to the root url
    /// # Arguments
    /// * `url` - The url to add. It should be a string.
    pub fn lit_url<R: Rx + 'static, T: Into<String>>(
        self: &Arc<Self>,
        url: T,
    ) -> Arc<super::urls::Url<R>> {
        match self.handler.lit_url::<R, _>(url) {
            Ok(url) => url,
            Err(e) => {
                eprintln!("{}", e);
                dangling_url()
            }
        }
    }

    pub fn reg_from<R: Rx + 'static>(self: &Arc<Self>, segments: &[PathPattern]) -> Arc<Url<R>> {
        match self.handler.reg_from::<R>(segments) {
            Ok(url) => url,
            Err(e) => {
                eprintln!("{}", e);
                urls::dangling_url()
            }
        }
    }

    /// Handle a single connection
    pub fn handle_connection(self: Arc<Self>, stream: TcpStream) {
        let duration = Duration::from_secs(self.max_connection_time as u64);
        let app = self.clone();
        // 1) spawn the actual connection job
        // let handle = tokio::spawn(async move {
        //     self.handler.run(app, Connection::Tcp(stream)).await;
        // });
        // 2) in parallel, sleep then abort
        tokio::spawn(async move {
            tokio::select! { 
                _ = self.handler.run(app, Connection::Tcp(stream)) => {}, 
                _ = tokio::time::sleep(duration) => {
                    // Timed out: forcefully close
                    eprintln!("⚠️ Connection timed out after {:?}", duration);
                    // Note: dropping the reader/writer will close the socket
                } 
            }  
            // tokio::time::sleep(duration).await;
            // if !handle.is_finished() {
            //     handle.abort();
            //     eprintln!("Connection timed out after {:?}", duration);
            // }
        });
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

        println!(
            "Connection established on {}",
            listener.local_addr().unwrap()
        );

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
                            println!("Accepted connection from {addr}");
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
}

// Helper function for determining CPU count
fn num_cpus() -> usize {
    match std::thread::available_parallelism() {
        Ok(n) => n.get(),
        Err(_) => 1, // Fallback if we can't determine
    }
}
