use std::{
    any::{Any, TypeId}, future::Future, pin::Pin, sync::Arc
};
use tokio::io::{
    AsyncBufReadExt,
    AsyncWriteExt,
    BufReader,
    BufWriter,
    ReadHalf,
    WriteHalf,
};
use crate::{app::{middleware::{AsyncMiddleware, AsyncMiddlewareChain}, urls::{PathPattern, Url}}, connection::{Connection, Rx}, extensions::ParamsClone};
use super::application::App; 

// type TestFn = fn(&[u8]) -> bool;

// type HandlerFn<R: Rx> =
//     fn(Arc<App>, Arc<Url<R>>, BufReader<ReadHalf<Connection>>, BufWriter<WriteHalf<Connection>>)
//         -> Pin<Box<dyn Future<Output = ()> + Send>>;

/// Internal struct tying a single protocol's detection function (`test`)
/// to its processing function (`handle`).
/// Concrete handler for a specific protocol
struct ProtocolHandler<R: Rx> {
    root_handler: Arc<Url<R>>, 
    middlewares: AsyncMiddlewareChain<R>, 
} 

impl<R: Rx> ProtocolHandler<R> { 
    pub fn new(
        root_handler: Arc<Url<R>>,
        middlewares: AsyncMiddlewareChain<R>,
    ) -> Self {
        Self { 
            root_handler,
            middlewares,    
        }
    }
}

pub trait ProtocolHandlerTrait: Send + Sync {
    /// A function pointer to inspect the first bytes of a connection
    /// and decide whether a protocol should handle it.
    /// Returns `true` if the given buffer matches the protocol signature.
    fn test(&self, buf: &[u8]) -> bool; 

    /// A function pointer that, given the `App` and split I/O halves wrapped
    /// in buffered reader/writer, returns a boxed `Future` that drives the
    /// protocol handler to completion.
    fn handle(
        &self,
        app: Arc<App>,
        reader: BufReader<ReadHalf<Connection>>,
        writer: BufWriter<WriteHalf<Connection>>,
    ) -> Pin<Box<dyn Future<Output = ()> + Send>>; 

    /// Allows downcasting to the concrete `ProtocolHandler<R>` type.
    fn as_any(&self) -> &dyn Any; 

    /// Like `as_any`, but for mutable downcasting.
    fn as_any_mut(&mut self) -> &mut dyn Any;
} 

impl<R: Rx + 'static> ProtocolHandlerTrait for ProtocolHandler<R> {
    fn test(&self, buf: &[u8]) -> bool {
        R::test_protocol(buf)
    }

    fn handle(
        &self,
        app: Arc<App>,
        reader: BufReader<ReadHalf<Connection>>,
        writer: BufWriter<WriteHalf<Connection>>,
    ) -> Pin<Box<dyn Future<Output = ()> + Send>> {
        let root_handler = self.root_handler.clone();
        Box::pin(async move {
            R::process(app, root_handler, reader, writer).await;
        })
    } 

    fn as_any(&self) -> &dyn Any {
        self
    } 

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    } 
} 

/// Registry for multiple protocol handlers
/// using a simple `Vec<ProtocolHandler>` for O(n) dispatch.
/// This avoids hash lookups and TypeId overhead, trading for a small
/// linear scan over handlers in registration order.
pub struct ProtocolRegistry {
    /// Ordered list of protocol handlers (test + handle).
    handlers: Vec<Arc<dyn ProtocolHandlerTrait>>,
}

impl ProtocolRegistry {
    /// Construct an empty registry with no protocols registered.
    pub fn new() -> Self {
        Self {
            handlers: Vec::new(),
        }
    }

    /// Register a protocol `P` that implements `Rx + 'static`.
    /// This pushes its `test_protocol` and `process` functions
    /// onto the `handlers` vector, preserving registration order.
    pub fn register<R: Rx + 'static>(&mut self, root_handler: Arc<Url<R>>, middleware_chain: AsyncMiddlewareChain<R>) {
        self.handlers.push(Arc::new(ProtocolHandler::new(root_handler, middleware_chain)));
    } 

    /// Attempt to detect and run one of the registered protocols.
    ///
    /// Steps:
    /// 1. Split the `Connection` into read/write halves.
    /// 2. Peek at the initial bytes without consuming them.
    /// 3. Iterate in registration order and run the first matching protocol.
    /// 4. If no match is found, cleanly shutdown the write half.
    pub async fn run_multi(&self, app: Arc<App>, conn: Connection) {
        // 1) split into raw halves
        let (read_half, write_half) = conn.split();
        let mut reader = BufReader::new(read_half);
        let mut writer = BufWriter::new(write_half);

        // 2) peek at buffered data without consuming
        let buf = reader.fill_buf().await.unwrap_or(&[]);
        let n = buf.len();

        // 3) test each registered protocol in order
        for handler in &self.handlers {
            if handler.test(&buf[..n]) {
                // 4) if test passes, dispatch to this protocol's handler
                handler.handle(app.clone(), reader, writer).await;
                return;
            }
        }

        // 5) no protocol matched → close the connection gracefully
        let _ = writer.shutdown().await;
    }
}

/// Enum used in `App` to select between single‐protocol mode
/// (direct dispatch to one protocol P) and multi‐protocol mode
/// (detection loop over a `ProtocolRegistry`).
pub enum ProtocolRegistryKind {
    /// Single‐protocol mode. Stores only the handler function for zero‐overhead dispatch.
    Single(Arc<dyn ProtocolHandlerTrait>), 
    /// Multi‐protocol mode. Contains a full `ProtocolRegistry`.
    Multi(ProtocolRegistry),
} 


pub struct ProtocolHandlerBuilder<R: Rx + 'static> {
    url: Arc<Url<R>>,
    middlewares: Vec<Arc<dyn AsyncMiddleware<R>>>,
}

impl<R: Rx> ProtocolHandlerBuilder<R> {
    pub fn new() -> Self {
        Self {
            url: Arc::new(Url::default()),
            middlewares: Vec::new(), 
        }
    }

    pub fn with_default_middlewares(mut self) -> Self {
        self.middlewares = Self::default_middlewares();
        self
    }

    pub fn default_middlewares() -> Vec<Arc<dyn AsyncMiddleware<R>>> {
        vec![
            // Add your default middleware implementations here
        ]
    } 

    pub fn set_url(mut self, url: Arc<Url<R>>) -> Self { 
        self.url = url; 
        self 
    }

    // Append a middleware instance created by T to the end of the vector.
    pub fn append_middleware<M>(mut self) -> Self
    where
        M: AsyncMiddleware<R> + 'static,
    {
        self.middlewares.push(Arc::new(M::return_self()));
        self
    }

    // Insert a middleware instance created by T at the beginning of the vector.
    pub fn prepend_middleware<M>(mut self) -> Self
    where
        M: AsyncMiddleware<R> + Default + 'static,
    {
        self.middlewares.insert(0, Arc::new(M::default()));
        self
    }

    pub fn remove_middleware<M>(mut self) -> Self
    where
        M: 'static,
    {
        self.middlewares.retain(|m| {
            m.as_any().type_id() != TypeId::of::<M>()
        });
        self
    }

    pub fn build(self) -> Arc<dyn ProtocolHandlerTrait> {
        Arc::new(ProtocolHandler::new(self.url, self.middlewares))
    }
}

pub struct ProtocolRegistryBuilder {
    handlers: Vec<Arc<dyn ProtocolHandlerTrait>>,
}

impl ProtocolRegistryBuilder {
    pub fn new() -> Self {
        Self { handlers: Vec::new() }
    }

    pub fn protocol<R: Rx>(mut self, builder: ProtocolHandlerBuilder<R>) -> Self {
        self.handlers.push(builder.build());
        self
    }

    pub fn build(self) -> ProtocolRegistryKind {
        match self.handlers.len() {
            // 0 => ProtocolRegistryKind::empty(), 
            1 => ProtocolRegistryKind::Single(self.handlers.into_iter().next().unwrap()) ,
            _ => ProtocolRegistryKind::Multi(ProtocolRegistry{handlers: self.handlers}),
        }
    }
} 

impl ProtocolRegistryKind {
    /// Construct a `Single` variant for protocol `P`, avoiding any
    /// loops or lookups. This is the fastest path when you know at
    /// compile time which protocol to run.
    pub fn single<R: Rx + 'static>(root_handler: Arc<Url<R>>, middlewares: AsyncMiddlewareChain<R>) -> Self {
        ProtocolRegistryKind::Single(Arc::new(ProtocolHandler::new(root_handler, middlewares)))
    } 

    /// Construct a `Multi` variant from an existing registry.
    pub fn multi(registry: ProtocolRegistry) -> Self {
        ProtocolRegistryKind::Multi(registry)
    } 

    /// Entry point: dispatch the connection according to the selected mode.
    ///
    /// - `Single` mode directly invokes the stored `handler`.
    /// - `Multi` mode calls `run_multi` on the inner registry.
    pub async fn run(&self, app: Arc<App>, conn: Connection) {
        match self {
            ProtocolRegistryKind::Single(handler) => {
                let (read_half, write_half) = conn.split();
                let reader = BufReader::new(read_half);
                let writer = BufWriter::new(write_half);
                handler.handle(app, reader, writer).await;
            } 
            ProtocolRegistryKind::Multi(registry) => {
                // Use detection logic for multiple protocols.
                registry.run_multi(app, conn).await;
            }
        }
    } 

    /// Retrieve the root Url<R> for a given protocol type `R`.
    /// Returns `Some(Arc<Url<R>>)` if a handler of type `R` is present.
    pub fn url<R: Rx + 'static>(&self) -> Option<Arc<Url<R>>> {
        match self {
            ProtocolRegistryKind::Single(handler) => {
                handler
                    .as_any()
                    .downcast_ref::<ProtocolHandler<R>>()
                    .map(|ph| ph.root_handler.clone())
            }
            ProtocolRegistryKind::Multi(registry) => {
                for handler in &registry.handlers {
                    if let Some(ph) = handler.as_any().downcast_ref::<ProtocolHandler<R>>() {
                        return Some(ph.root_handler.clone());
                    }
                }
                None
            }
        }
    } 

    /// Retrieve the Middleware<R> for a given protocol type `R`.
    /// Returns `Some(AsymcMiddlewareChain<R>)` if a handler of type `R` is present.
    pub fn middlewares<R: Rx + 'static>(&self) -> Option<AsyncMiddlewareChain<R>> {
        match self {
            ProtocolRegistryKind::Single(handler) => {
                handler
                    .as_any()
                    .downcast_ref::<ProtocolHandler<R>>()
                    .map(|ph| ph.middlewares.clone())
            }
            ProtocolRegistryKind::Multi(registry) => {
                for handler in &registry.handlers {
                    if let Some(ph) = handler.as_any().downcast_ref::<ProtocolHandler<R>>() {
                        return Some(ph.middlewares.clone());
                    }
                }
                None
            }
        }
    } 

    /// This function add a new url to the app. It will be added to the root url 
    /// # Arguments 
    /// * `url` - The url to add. It should be a string.
    pub fn lit_url<R: Rx + 'static, T: Into<String>>(
        &self, 
        url: T, 
    ) -> Result<Arc<Url<R>>, String> { 
        let url = url.into(); 
        println!("Adding url: {}", url); 
        match self.url::<R>() 
            .map(|root| {  
                root.clone()
                .literal_url(
                    &url, 
                    None, 
                    self.middlewares::<R>().unwrap_or(vec![]), 
                    ParamsClone::default()
                )
            }) 
        {
            Some(Ok(url)) => Ok(url),
            Some(Err(e)) => Err(e),
            None => Err("Protocol Not Found".to_string()), 
        }
    } 

    pub fn reg_from<R: Rx + 'static>(
        &self,
        segments: &[PathPattern]
    ) -> Result<Arc<Url<R>>, String> { 
        match self.url::<R>()
            .map(|root| { 
                let mut current = root.clone(); 
                for seg in segments { 
                    current = current.get_child_or_create(seg.clone())?; 
                    current.set_middlewares(self.middlewares::<R>().unwrap_or(vec![])); 
                } 
                Ok::<Arc<Url<R>>, String>(current) 
            }) {  
                Some(Ok(url)) => Ok(url), 
                Some(Err(e)) => Err(e), 
                None => Err("Protocol Not Found".to_string()) 

        }
        // for seg in segments { 
        //     current = current.get_child_or_create(seg.clone())?; 
        //     current.set_middlewares((*self.middlewares).clone()); 
        // }
        // Ok(current)
    }
} 
