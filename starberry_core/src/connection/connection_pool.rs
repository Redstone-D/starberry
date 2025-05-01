// Example connection pool implementation using Tokio and async I/O 
// Not in use yet, but can be used as a reference for future implementations 

use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::net::TcpStream;
use tokio::sync::{Mutex, Semaphore, OwnedSemaphorePermit};
use std::collections::VecDeque;
use std::io::{Error, ErrorKind};
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};

pub struct PooledConnection {
    stream: Option<TcpStream>,
    pool: Arc<ConnectionPool>,
    created_at: Instant,
    _permit: OwnedSemaphorePermit,
} 

impl PooledConnection {
    fn new(stream: TcpStream, pool: Arc<ConnectionPool>, permit: OwnedSemaphorePermit) -> Self {
        pool.metrics_active_connections.fetch_add(1, Ordering::Release);
        Self {
            stream: Some(stream),
            pool,
            created_at: Instant::now(),
            _permit: permit,
        }
    }
    
    pub fn stream(&mut self) -> &mut TcpStream {
        self.stream.as_mut().expect("Stream was already taken")
    }
    
    pub fn take(mut self) -> TcpStream {
        self.pool.metrics_active_connections.fetch_sub(1, Ordering::Release);
        self.stream.take().expect("Stream was already taken")
    }
}

impl Drop for PooledConnection {
    fn drop(&mut self) {
        if let Some(stream) = self.stream.take() {
            self.pool.metrics_active_connections.fetch_sub(1, Ordering::Release);
            let pool = self.pool.clone();
            
            // Use a channel instead of direct spawn
            if let Err(e) = pool.return_sender.send(stream) {
                println!("Failed to return connection: {}", e);
            }
        }
    }
} 

#[derive(Default)]
pub struct ConnectionMetrics {
    pub created_total: AtomicUsize,
    pub reused_total: AtomicUsize,
    pub returned_total: AtomicUsize,
    pub errors_total: AtomicUsize,
    pub timeouts_total: AtomicUsize,
}

pub struct ConnectionPool {
    idle_connections: Mutex<VecDeque<(TcpStream, Instant)>>,
    max_size: usize,
    max_idle_time: AtomicU64, // Stored as milliseconds
    permits: Arc<Semaphore>,
    destination: String,
    return_sender: tokio::sync::mpsc::UnboundedSender<TcpStream>,
    metrics_active_connections: AtomicUsize,
    metrics: ConnectionMetrics,
}

impl ConnectionPool {
    pub fn new(max_size: usize, destination: String) -> Arc<Self> {
        let (return_sender, mut return_receiver) = tokio::sync::mpsc::unbounded_channel();
        
        let pool = Arc::new(ConnectionPool {
            idle_connections: Mutex::new(VecDeque::with_capacity(max_size)),
            max_size,
            max_idle_time: AtomicU64::new(30_000), // 30 seconds in ms
            permits: Arc::new(Semaphore::new(max_size)),
            destination,
            return_sender,
            metrics_active_connections: AtomicUsize::new(0),
            metrics: ConnectionMetrics::default(),
        });

        let pool_clone = Arc::clone(&pool);
        tokio::spawn(async move {
            while let Some(stream) = return_receiver.recv().await {
                pool_clone.process_returned_connection(stream).await;
            }
        });

        Self::start_cleanup_task(Arc::clone(&pool));
        pool
    } 

    async fn process_returned_connection(&self, stream: TcpStream) {
        if Self::is_alive(&stream).await {
            let mut idle = self.idle_connections.lock().await;
            if idle.len() < self.max_size {
                idle.push_back((stream, Instant::now()));
            }
        }
        self.metrics.returned_total.fetch_add(1, Ordering::Relaxed);
    } 

    /// Start a background task to clean up stale connections
    fn start_cleanup_task(pool: Arc<ConnectionPool>) {
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(60));
            loop {
                interval.tick().await;
                pool.clean_stale_connections().await;
            }
        });
    }
    
    /// Clean up connections that have exceeded the idle timeout
    async fn clean_stale_connections(&self) {
        let max_idle = Duration::from_millis(self.max_idle_time.load(Ordering::Acquire));
        let now = Instant::now();
        
        let mut idle = self.idle_connections.lock().await;
        idle.retain(|(_, created)| now.duration_since(*created) < max_idle);
    } 
    
    /// Check if a connection is still alive using async readiness
    async fn is_alive(stream: &TcpStream) -> bool {
        match stream.readable().await {
            Ok(_) => {
                match stream.try_read(&mut [0u8; 0]) {
                    Ok(0) => true,
                    Err(e) if e.kind() == ErrorKind::WouldBlock => true,
                    _ => false,
                }
            },
            Err(_) => false,
        }
    } 

    pub async fn get_connection(self: &Arc<Self>) -> Result<PooledConnection, Error> {
        let permit = match tokio::time::timeout(
            Duration::from_secs(5),
            self.permits.clone().acquire_owned()
        ).await {
            Ok(Ok(permit)) => permit,
            Ok(Err(_)) => {
                self.metrics.errors_total.fetch_add(1, Ordering::Relaxed);
                return Err(Error::new(ErrorKind::Other, "Semaphore closed"));
            },
            Err(_) => {
                self.metrics.timeouts_total.fetch_add(1, Ordering::Relaxed);
                return Err(Error::new(ErrorKind::TimedOut, "Connection timeout"));
            }
        };
    
        let stream = match self.try_get_idle().await {
            Some(stream) => {
                self.metrics.reused_total.fetch_add(1, Ordering::Relaxed);
                stream
            }
            None => {
                let stream = TcpStream::connect(&self.destination).await.map_err(|e| {
                    self.metrics.errors_total.fetch_add(1, Ordering::Relaxed);
                    e
                })?;
                self.metrics.created_total.fetch_add(1, Ordering::Relaxed);
                stream
            }
        };
    
        Ok(PooledConnection::new(
            stream,
            Arc::clone(self),
            permit
        ))
    } 

    async fn try_get_idle(&self) -> Option<TcpStream> {
        let mut idle = self.idle_connections.lock().await;
        while let Some((stream, _)) = idle.pop_front() {
            if Self::is_alive(&stream).await {
                return Some(stream);
            }
        }
        None
    }

    /// Return a connection to the pool
    async fn return_connection(&self, stream: TcpStream) {
        self.metrics.returned_total.fetch_add(1, Ordering::Relaxed);
        
        // Check if the connection is still valid
        if Self::is_alive(&stream).await {
            let mut idle = self.idle_connections.lock().await;
            
            // If we have capacity, add it to the idle pool
            if idle.len() < self.max_size {
                idle.push_back((stream, Instant::now()));
            }
            // Otherwise let it drop and close
        }
        
        // Note: We don't add back the permit here because the OwnedSemaphorePermit
        // in PooledConnection will automatically release it when dropped
    }
    
    /// Set the maximum time a connection can remain idle before being closed
    pub fn set_max_idle_time(&self, timeout: Duration) {
        // In a real implementation, we'd need atomic or mutex protection here
        // For this example, we're ignoring the actual update
    }
    
    /// Pre-warm connections in the pool
    pub async fn warm(&self, count: usize) -> Result<(), Error> {
        let mut idle = self.idle_connections.lock().await;
        let available = self.max_size.saturating_sub(idle.len());
        let count = count.min(available);

        for _ in 0..count {
            let stream = TcpStream::connect(&self.destination).await?;
            idle.push_back((stream, Instant::now()));
            self.metrics.created_total.fetch_add(1, Ordering::Relaxed);
        }
        
        Ok(())
    } 

    /// Get the current number of idle connections
    pub async fn idle_count(&self) -> usize {
        let idle = self.idle_connections.lock().await;
        idle.len()
    }
    
    /// Get the current number of active connections
    pub fn active_count(&self) -> usize {
        self.metrics_active_connections.load(Ordering::Relaxed)
    }
    
    /// Drain and close all idle connections in the pool
    pub async fn drain(&self) {
        let mut idle = self.idle_connections.lock().await;
        idle.clear(); // Drop all idle connections
    }

    /// Get a snapshot of the current metrics
    pub fn get_metrics(&self) -> ConnectionPoolMetrics {
        ConnectionPoolMetrics {
            idle: self.idle_connections.try_lock().map(|i| i.len()).unwrap_or(0),
            active: self.metrics_active_connections.load(Ordering::Relaxed),
            created: self.metrics.created_total.load(Ordering::Relaxed),
            reused: self.metrics.reused_total.load(Ordering::Relaxed),
            returned: self.metrics.returned_total.load(Ordering::Relaxed),
            errors: self.metrics.errors_total.load(Ordering::Relaxed),
            timeouts: self.metrics.timeouts_total.load(Ordering::Relaxed),
        }
    }

    // Socket configuration placeholder for future implementation
    /*
    async fn configure_socket(&self, stream: &TcpStream) -> std::io::Result<()> {
        // Example socket configuration code:
        stream.set_nodelay(true)?;
        
        // For more advanced options:
        // 1. Convert to std::net::TcpStream temporarily
        let std_stream = stream.into_std()?;
        
        // 2. Configure using socket2
        let socket = Socket::from(std_stream);
        socket.set_nodelay(true)?;
        socket.set_recv_buffer_size(64 * 1024)?;
        socket.set_send_buffer_size(64 * 1024)?;
        
        // 3. Convert back to tokio TcpStream
        let tokio_stream = TcpStream::from_std(socket.into())?;
        
        Ok(())
    }
    */
}

/// Metrics snapshot for a connection pool
#[derive(Debug, Clone, Copy)]
pub struct ConnectionPoolMetrics {
    pub idle: usize,
    pub active: usize,
    pub created: usize,
    pub reused: usize,
    pub returned: usize,
    pub errors: usize,
    pub timeouts: usize,
} 
