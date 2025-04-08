use core::panic;
use std::ops::RangeFrom; 
use std::pin::Pin;
use std::sync::{Arc, Mutex, RwLock};
use std::time::Duration;
use std::{net::TcpListener, thread, sync::mpsc}; 
use std::net::TcpStream;    
use starberry_lib::random_string; 
use tokio::runtime::Runtime; 

use crate::app::middleware::{LoggingMiddleware}; 
use crate::app::urls;
use crate::context::Rc;

use super::super::http::http_value::*; 
use super::super::http::request::*;  
use super::super::http::response::*; 
use super::middleware::{self, AsyncMiddleware};
use super::urls::*;  

pub struct App {
    pub root_url: Arc<Url>, 
    pub listener: TcpListener, 
    pub mode: RunMode, 
    pub pool: ThreadPool, 
    pub secret_key: String, 
    pub max_connection_time: usize, 
    pub connection_config: ParseConfig, 
    pub middlewares: Arc<Vec<Arc<dyn AsyncMiddleware>>>, 
} 

/// RunMode enum to represent the mode of the application 
/// Production: Production mode 
/// Development: Test on developer's computer, showing the error message and some debug info. May contain sensitive info. 
/// Beta: Beta mode, showing some debug info. May contain some sensitive info. 
/// Build: Build mode. For building the starberry binary. Do not use this. 
#[derive(Clone, Debug)]
pub enum RunMode { 
    Production, 
    Development, 
    Beta, 
    Build, 
} 

struct ThreadPool {
    workers: Vec<Worker>,
    sender: mpsc::Sender<Job>,
}

type Job = Pin<Box<dyn Future<Output = ()> + Send + 'static>>;

impl ThreadPool {
    pub fn new(size: usize) -> ThreadPool {
        assert!(size > 0);
        let (sender, receiver) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));
        let mut workers = Vec::with_capacity(size);
        for id in 0..size {
            // Pass a separate receiver to each worker
            let worker_receiver = Arc::clone(&receiver);
            println!("Creating worker {id}"); 
            workers.push(Worker::new(id, worker_receiver));
        }
        ThreadPool { workers, sender }
    }

    pub fn execute<F>(&self, f: F)
    where
        F: Future<Output = ()> + Send + 'static,
    {
        let job = Box::pin(f); // Pin the future here
        self.sender.send(job).unwrap();
    }
}

struct Worker {
    id: usize,
    thread: thread::JoinHandle<()>,
}

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Worker {
        let thread = thread::spawn(move || {
            // Each worker has its own runtime to execute the jobs
            let rt = Runtime::new().unwrap();

            loop {
                let job = receiver.lock().unwrap().recv();
                match job {
                    Ok(job) => {
                        // println!("Worker {id} got a job; executing.");
                        rt.block_on(job);
                    }
                    Err(_) => {
                        // println!("Worker {id} exiting.");
                        break; // If there are no more jobs, the worker exits.
                    }
                }
            }
        });

        Worker { id, thread }
    }
} 

pub struct AppBuilder { 
    root_url: Option<Arc<Url>>, 
    binding: Option<String>, 
    mode: Option<RunMode>, 
    workers: Option<usize>, 
    secret_key: Option<String>, 
    max_connection_time: Option<usize>, 
    max_header_size: Option<usize>, 
    max_body_size: Option<usize>, 
    max_line_length: Option<usize>, 
    max_headers: Option<usize>, 
    middle_wares: Option<Vec<Arc<dyn AsyncMiddleware>>>, 
} 

impl AppBuilder { 
    pub fn new() -> Self { 
        Self { root_url: None, binding: None, mode: None, workers: None, secret_key: None, max_connection_time: None, max_header_size: None, max_body_size: None, max_line_length: None, max_headers: None, middle_wares: Some(Self::default_middlewares()) } 
    } 

    pub fn default_middlewares() -> Vec<Arc<dyn AsyncMiddleware>> { 
        vec![Arc::new(LoggingMiddleware)]  
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

    pub fn workers(mut self, workers: usize) -> Self { 
        self.workers = Some(workers); 
        self 
    } 

    pub fn secret_key(mut self, secret_key: String) -> Self { 
        self.secret_key = Some(secret_key); 
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
    pub fn insert_middleware<T: 'static + AsyncMiddleware>(mut self) -> Self { 
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

    pub fn build(self) -> Arc<App> { 
        let root_url = match self.root_url{ 
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
        let port = self.binding.unwrap_or_else(|| String::from("127.0.0.1:3003")); 
        let binding = match TcpListener::bind(&port) { 
            Ok(binding) => binding, 
            Err(e) => panic!("Binding failed in {}, error: {}", &port, e), 
        };  
        let mode = self.mode.unwrap_or_else(|| RunMode::Development); 
        let workers = ThreadPool::new(self.workers.unwrap_or_else(|| 4)); 
        let secret_key = self.secret_key.unwrap_or_else(|| random_string(32));
        let max_connection_time = self.max_connection_time.unwrap_or_else(|| 5); 
        let max_header_size = self.max_header_size.unwrap_or_else(|| 1024 * 1024); 
        let max_body_size = self.max_body_size.unwrap_or_else(|| 1024 * 512 ); 
        let max_line_length = self.max_line_length.unwrap_or_else(|| 1024 * 64); 
        let max_headers = self.max_headers.unwrap_or_else(|| 100); 
        let connection_config = ParseConfig::new(max_header_size, max_line_length, max_headers, max_body_size); 
        Arc::new(App { root_url, listener: binding, mode, pool: workers, secret_key, max_connection_time, connection_config, middlewares: self.middle_wares.unwrap_or_else(|| Self::default_middlewares()).into() }) 
    } 
}

impl App { 
    pub fn new() -> AppBuilder { 
        AppBuilder::new() 
    } 

    pub fn set_root_url(&mut self, root_url: Arc<Url>) { 
        self.root_url = root_url; 
    } 

    pub async fn set_binding(&mut self, binding: &str) { 
        self.listener = TcpListener::bind(binding).unwrap(); 
    } 

    pub fn get_binding(self: &Arc<Self>) -> String { 
        self.listener.local_addr().unwrap().to_string() 
    } 

    pub fn set_mode(&mut self, mode: RunMode) { 
        self.mode = mode; 
    } 

    pub fn get_mode(self: &Arc<Self>) -> RunMode { 
        self.mode.clone() 
    } 

    pub fn set_workers(&mut self, workers: usize) { 
        self.pool = ThreadPool::new(workers); 
    } 

    pub fn get_workers(self: &Arc<Self>) -> usize { 
        self.pool.workers.len() 
    } 

    pub fn set_secret_key(&mut self, secret_key: String) { 
        self.secret_key = secret_key; 
    } 

    pub fn get_secret_key(self: &Arc<Self>) -> &str { 
        &self.secret_key 
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

    /// This function add a new url to the app. It will be added to the root url 
    /// # Arguments 
    /// * `url` - The url to add. It should be a string. 
    // pub fn literal_url<T: Into<String>>(
    //     self: &Arc<Self>, 
    //     url: T, 
    //     function: Arc<dyn AsyncUrlHandler>, 
    // ) -> Result<Arc<super::urls::Url>, String> { 
    //     let url = url.into(); 
    //     self.root_url.clone().literal_url(&url, function, Some(self.middlewares.clone()), default::Default::default()) 
    // } 
    pub fn lit_url<T: Into<String>>(
        self: &Arc<Self>, 
        url: T, 
    ) -> Arc<super::urls::Url> { 
        let url = url.into(); 
        println!("Adding url: {}", url); 
        match self.root_url.clone().literal_url(&url, None, Some(self.middlewares.clone()), Params::default()) { 
            Ok(url) => url, 
            Err(_) => super::urls::dangling_url(), 
        }
    } 

    // Note: This function is now synchronous, and expects that `self` is shared via an Arc.
    #[allow(unused_mut)]
    pub fn handle_connection(self: Arc<Self>, mut stream: TcpStream) { 
        // Spawn a new OS thread for this connection. 
        let app = Arc::clone(&self); 
        let job = async move { 
            let rc = Rc::handle(app.clone(), stream).await; 
            rc.run().await; 
        }; 
        // Box the async closure and pass it to the thread pool.
        self.pool.execute(Box::pin(job)); 
    }

    pub async fn run(self: Arc<Self>) { 
        // println!("Urls: {}", self.root_url); 
        println!("Connection established from {}", self.listener.local_addr().unwrap()); 
        for stream in self.listener.incoming() {
            let stream = stream.unwrap();  
            stream.set_read_timeout(Some(Duration::from_secs(5))).unwrap();  
            Arc::clone(&self).handle_connection(stream); 
        } 
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
                // eprintln!("Error getting url: {}", e);  
                urls::dangling_url() 
            } 
        } 
    } 
} 


