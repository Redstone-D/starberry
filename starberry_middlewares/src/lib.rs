pub use starberry_macro::middleware; 
pub use starberry_core; 

type FutureResponse = std::pin::Pin<std::boxed::Box<dyn std::future::Future<Output = starberry_core::http::response::HttpResponse> + Send + 'static>>; 

pub mod test; 
pub use test::MyMiddleWare; 
