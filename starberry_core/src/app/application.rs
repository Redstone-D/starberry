use core::panic;
use std::pin::Pin;
use std::sync::{Arc, Mutex, RwLock};
use std::{net::TcpListener, thread, sync::mpsc}; 
use std::net::TcpStream;    
use tokio::runtime::Runtime;

use super::super::http::http_value::*; 
use super::super::http::request::*;  
use super::super::http::response::*; 
use super::urls::*; 

pub struct App {
    pub root_url: Arc<Url>, 
    pub listener: TcpListener, 
    pub mode: RunMode, 
    pub pool: ThreadPool, 
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
                        println!("Worker {id} got a job; executing.");
                        rt.block_on(job);
                    }
                    Err(_) => {
                        println!("Worker {id} exiting.");
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
} 

impl AppBuilder { 
    pub fn new() -> Self { 
        Self { root_url: None, binding: None, mode: None, workers: None } 
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

    pub fn build(self) -> Arc<App> { 
        let root_url = match self.root_url{ 
            Some(root_url) => root_url, 
            None => { 
                Arc::new(Url { 
                    path: PathPattern::Literal(String::from("/")), 
                    children: RwLock::new(Children::Nil), 
                    method: RwLock::new(None), 
                    ancestor: Ancestor::Nil, 
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
        Arc::new(App { root_url, listener: binding, mode, pool: workers }) 
    } 
}

impl App { 
    pub fn new() -> AppBuilder { 
        AppBuilder::new() 
    } 

    pub fn set_root_url(&mut self, root_url: Arc<Url>) { 
        self.root_url = root_url; 
    } 

    pub fn set_binding(&mut self, binding: &str) { 
        self.listener = TcpListener::bind(binding).unwrap(); 
    } 

    pub fn set_mode(&mut self, mode: RunMode) { 
        self.mode = mode; 
    } 

    pub fn set_workers(&mut self, workers: usize) { 
        self.pool = ThreadPool::new(workers); 
    } 

    /// This function add a new url to the app. It will be added to the root url 
    /// # Arguments 
    /// * `url` - The url to add. It should be a string. 
    pub fn literal_url<T: Into<String>>(self: &Arc<Self>, url: T, function: Arc<dyn AsyncUrlHandler>) -> Result<Arc<super::urls::Url>, String> { 
        let url = url.into(); 
        self.root_url.clone().literal_url(&url, function) 
    } 

    pub async fn request(&self, request: HttpRequest) -> HttpResponse { 
        let path = request.path.clone(); 
        let mut path = path.split('/').collect::<Vec<&str>>(); 
        path.remove(0); 
        println!("{:?}", path); 
        let url: Option<_> = Arc::clone(&self.root_url).walk(path.iter()).await; 
        if let Some(url) = url { 
            return url.run(request).await; 
        } else { 
            return request_templates::return_status(StatusCode::NOT_FOUND);  
        } 
    }  

    // Note: This function is now synchronous, and expects that `self` is shared via an Arc.
    pub fn handle_connection(self: Arc<Self>, mut stream: TcpStream) {
        // Spawn a new OS thread for this connection.
        println!("New connection from {}", stream.peer_addr().unwrap()); 
        let app = Arc::clone(&self); 
        let job = async move {
            if let Ok(request) = HttpRequest::from_request_stream(&mut stream).await {
                // Process the request asynchronously and send the response.
                app.request(request).await.send(&mut stream).await;
            }
        };
        // Box the async closure and pass it to the thread pool.
        self.pool.execute(Box::pin(job)); 
    }

    pub async fn run(self: Arc<Self>) { 
        for stream in self.listener.incoming() {
            let stream = stream.unwrap();
            Arc::clone(&self).handle_connection(stream); 
        } 
    } 
} 
