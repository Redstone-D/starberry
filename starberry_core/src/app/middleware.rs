use std::pin::Pin; 
use std::future::Future;
use std::sync::Arc; 
use crate::http::context::HttpReqCtx;

use crate::connection::Rx; 
use std::any::Any; 

/// A boxed future returning `R`.
pub type BoxFuture<R> = Pin<Box<dyn Future<Output = R> + Send + 'static>>; 

pub trait AsyncMiddleware<R: Rx>: Send + Sync + 'static { 
    fn as_any(&self) -> &dyn Any; 

    /// Used when creating the mddleware 
    fn return_self() -> Self where Self: Sized; 

    fn handle<'a>( 
        &self,
        rc: R,
        next: Box<dyn Fn(R) -> Pin<Box<dyn Future<Output = R> + Send>> + Send + Sync + 'static>,
    ) -> Pin<Box<dyn Future<Output = R> + Send + 'static>>; 
} 

/// The “final handler” trait that sits at the end of a middleware chain.
pub trait AsyncFinalHandler<R>: Send + Sync + 'static {
    /// Consume the request‐context and return a future yielding the (possibly modified) context.
    fn handle(&self, ctx: R) -> BoxFuture<R>;
} 

/// Blanket impl: any async fn or closure `Fn(R) -> impl Future<Output=R>` becomes an AsyncFinalHandler<R>.
impl<F, Fut, R> AsyncFinalHandler<R> for F
where
    F: Fn(R) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = R> + Send + 'static,
{
    fn handle(&self, ctx: R) -> BoxFuture<R> {
        Box::pin((self)(ctx))
    }
} 

/// The middleware‐chain builder and executor.
pub struct MiddlewareChain<R> {
    inner: Arc<dyn Fn(R) -> BoxFuture<R> + Send + Sync + 'static>,
}

impl<R> MiddlewareChain<R>
where
    R: Rx + Send + 'static,
{
    /// Build a chain from:
    ///  - `middlewares`: the Vec of AsyncMiddleware<R> in the order you want them to run
    ///  - `final_handler`: the AsyncFinalHandler<R> that executes last
    pub fn new(
        middlewares: Vec<Arc<dyn AsyncMiddleware<R>>>,
        final_handler: Arc<dyn AsyncFinalHandler<R>>,
    ) -> Self {
        // Wrap the final handler in a Fn(R)->Future
        let final_fn: Arc<dyn Fn(R) -> BoxFuture<R> + Send + Sync + 'static> =
            Arc::new(move |ctx| final_handler.handle(ctx));

        // Fold the middlewares in reverse so that the first element runs first
        let chain = middlewares.into_iter().rev().fold(final_fn, |next, mw| {
            let next_clone = next.clone();
            Arc::new(move |ctx: R| {
                // Each middleware calls the `next_fn` when ready to proceed
                let next_fn = next_clone.clone();
                mw.handle(ctx, Box::new(move |r| next_fn(r)))
            }) as Arc<dyn Fn(R) -> BoxFuture<R> + Send + Sync + 'static>
        });

        MiddlewareChain { inner: chain }
    }

    /// Drive the chain to completion, returning the final context.
    pub async fn run(&self, ctx: R) -> R {
        (self.inner)(ctx).await
    }
} 

/// A helper that builds and runs a middleware chain in one call.
pub async fn run_chain<R: Rx + 'static>(
    middlewares: Vec<Arc<dyn AsyncMiddleware<R>>>,
    final_handler: Arc<dyn AsyncFinalHandler<R>>,
    ctx: R,
) -> R {
    let chain = MiddlewareChain::new(middlewares, final_handler);
    chain.run(ctx).await
} 

pub struct LoggingMiddleware;

impl AsyncMiddleware<HttpReqCtx> for LoggingMiddleware {
    fn handle<'a>(
        &'a self,
        context: HttpReqCtx, 
        next: Box<dyn Fn(HttpReqCtx) -> Pin<Box<dyn Future<Output = HttpReqCtx> + Send>> + Send + Sync + 'a>,
    ) -> Pin<Box<dyn Future<Output = HttpReqCtx> + Send + 'static>> {
        println!("Logging: Received request for {}", context.path());
        next(context) 
    }
    
    fn as_any(&self) -> &dyn Any {
        self
    } 

    fn return_self() -> Self {
        LoggingMiddleware
    } 
} 
