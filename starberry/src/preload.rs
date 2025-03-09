pub use once_cell::sync::Lazy;
pub use crate::{App, RunMode}; 
pub use crate::{LitUrl, RegUrl, AnyUrl, AnyPath}; 
pub use crate::urls::*; 
pub use crate::{HttpRequest, HttpResponse}; 
pub use crate::{text_response, html_response};  
pub use crate::{lit_url, url}; 
pub use crate::HttpMethod::*; 
pub use std::sync::Arc; 
pub use std::thread::sleep; 
pub use std::time::Duration; 

pub type SApp = Lazy<Arc<App>>; 
