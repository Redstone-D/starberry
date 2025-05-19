use std::any::Any;
use std::collections::HashMap;
use std::sync::Arc;
use std::io::BufReader;
use tokio::net::TcpStream;
// Assuming HttpMeta, HttpBody, HttpResponse, App, and Url are defined in the crate
// If not, these will need to be imported or defined accordingly, e.g.:
// use crate::http::{HttpMeta, HttpBody, HttpResponse};
// use crate::app::App;  // Hypothetical import for App
// use url::Url;  // For Url, assuming the url crate is used

pub struct Rc {
    pub meta: crate::connection::Connection,  // Assuming Connection from connection.rs for SQL handling
    pub body: Vec<u8>,  // Simple buffer for SQL query results
    pub reader: BufReader<TcpStream>,  // Keep as is, since it's used in SQL connections
    pub app: Arc<String>,  // Simplified or removed if not needed; using String as placeholder
    pub endpoint: Arc<String>,  // Change to a string for database URL
    pub response: Result<String, crate::connection::ConnectionError>,  // Use existing ConnectionError for SQL errors

    /// Type-based extension storage, typically used by middleware
    /// Each type can have exactly one value
    params: HashMap<std::any::TypeId, Box<dyn Any + Send + Sync>>,
    
    /// String-based extension storage, typically used by application code
    /// Multiple values of the same type can be stored with different keys
    locals: HashMap<String, Box<dyn Any + Send + Sync>>,
}
