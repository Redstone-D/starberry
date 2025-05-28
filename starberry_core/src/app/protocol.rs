use std::{
    future::Future,
    pin::Pin,
    sync::Arc,
};
use tokio::io::{
    AsyncBufReadExt,
    AsyncWriteExt,
    BufReader,
    BufWriter,
    ReadHalf,
    WriteHalf,
};
use crate::{connection::Connection, context::Rx};
use super::application::App;

/// A function pointer to inspect the first bytes of a connection
/// and decide whether a protocol should handle it.
/// Returns `true` if the given buffer matches the protocol signature.
type TestFn = fn(&[u8]) -> bool;

/// A function pointer that, given the `App` and split I/O halves wrapped
/// in buffered reader/writer, returns a boxed `Future` that drives the
/// protocol handler to completion.
type HandlerFn =
    fn(Arc<App>, BufReader<ReadHalf<Connection>>, BufWriter<WriteHalf<Connection>>)
        -> Pin<Box<dyn Future<Output = ()> + Send>>;

/// Internal struct tying a single protocol's detection function (`test`)
/// to its processing function (`handle`).
struct ProtocolHandler {
    /// Peeks at the first bytes of the connection to see if this protocol applies.
    test: TestFn,
    /// Drives the protocol to completion once selected.
    handle: HandlerFn,
}

/// Registry for multiple protocol handlers
/// using a simple `Vec<ProtocolHandler>` for O(n) dispatch.
/// This avoids hash lookups and TypeId overhead, trading for a small
/// linear scan over handlers in registration order.
pub struct ProtocolRegistry {
    /// Ordered list of protocol handlers (test + handle).
    handlers: Vec<ProtocolHandler>,
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
    pub fn register<P>(&mut self)
    where
        P: Rx + 'static,
    {
        self.handlers.push(ProtocolHandler {
            test: P::test_protocol,
            handle: |app, reader, writer| {
                // Wrap the async process call in a pinned, boxed future.
                Box::pin(async move {
                    P::process(app, reader, writer).await;
                })
            },
        });
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
            if (handler.test)(&buf[..n]) {
                // 4) if test passes, dispatch to this protocol's handler
                (handler.handle)(app.clone(), reader, writer).await;
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
    Single {
        /// The function pointer that drives the chosen protocol.
        handler: HandlerFn,
    },
    /// Multi‐protocol mode. Contains a full `ProtocolRegistry`.
    Multi(ProtocolRegistry),
}

impl ProtocolRegistryKind {
    /// Construct a `Single` variant for protocol `P`, avoiding any
    /// loops or lookups. This is the fastest path when you know at
    /// compile time which protocol to run.
    pub fn single<P>() -> Self
    where
        P: Rx + 'static,
    {
        // Build the handler fn directly from P::process.
        let handler: HandlerFn = |app, reader, writer| {
            Box::pin(async move {
                P::process(app, reader, writer).await;
            })
        };
        ProtocolRegistryKind::Single { handler }
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
            ProtocolRegistryKind::Single { handler } => {
                // Directly drive the single protocol handler.
                let (read_half, write_half) = conn.split();
                let reader = BufReader::new(read_half);
                let writer = BufWriter::new(write_half);
                (handler)(app, reader, writer).await;
            }
            ProtocolRegistryKind::Multi(registry) => {
                // Use detection logic for multiple protocols.
                registry.run_multi(app, conn).await;
            }
        }
    }
} 
