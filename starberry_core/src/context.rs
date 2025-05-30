use std::pin::Pin;
use std::sync::Arc; 
use std::future::ready; 

use tokio::io::{BufReader, BufWriter, ReadHalf, WriteHalf};
use async_trait::async_trait; 

use crate::connection::Connection; 
use crate::app::application::App; 

#[async_trait] 
pub trait Rx: Sized + Send + Sync { 
    fn test_protocol(initial_bytes: &[u8]) -> bool;
    
    async fn process(app: Arc<App>, read_half: BufReader<ReadHalf<Connection>>, write_half: BufWriter<WriteHalf<Connection>>); 

    async fn process_direct(app: Arc<App>, stream: Connection) { 
        let (read_stream, write_stream) = stream.split();
        let reader = BufReader::new(read_stream);
        let writer = BufWriter::new(write_stream);
        Self::process(app, reader, writer).await; 
    } 

    /// Converts this response into a Future that resolves to itself.
    /// Useful for middleware functions that need to return a Future<Output = HttpResponse>.
    fn future(self) -> impl Future<Output = Self> + Send  
    where 
        Self: Sized + Send + 'static {
        ready(self)
    } 

    /// Creates a boxed future from this response (useful for trait objects).
    fn boxed_future(self) -> Pin<Box<dyn Future<Output = Self> + Send >> 
    where 
        Self: Sized + Send + 'static {
        Box::pin(self.future())
    }  

    fn bad_request(&mut self); 
}

#[async_trait]  
pub trait Tx: Send + Sync { 
    type Request; 
    type Response; 
    type Error; 
    async fn process(&mut self, request: Self::Request) -> Result<&mut Self::Response, Self::Error>; 
    async fn shutdown(&mut self) -> Result<(), Self::Error>; 
} 
