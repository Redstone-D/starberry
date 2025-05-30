use std::collections::VecDeque;
use std::sync::Arc;
use tokio::sync::Mutex;
use crate::context::Tx;
use async_trait::async_trait;

/// A simple asynchronous pool for objects implementing the `Tx` trait.
#[derive(Clone, Debug)]
pub struct TxPool<T: Tx> {
    items: Arc<Mutex<VecDeque<T>>>,
}

impl<T: Tx> TxPool<T> {
    /// Creates a new, empty pool.
    pub fn new() -> Self {
        TxPool {
            items: Arc::new(Mutex::new(VecDeque::new())),
        }
    }

    /// Adds a `Tx` instance to the pool.
    pub async fn add(&self, tx: T) {
        let mut items = self.items.lock().await;
        items.push_back(tx);
    }

    /// Attempts to retrieve a `Tx` instance from the pool.
    /// Returns `Some(item)` if available, otherwise `None`.
    pub async fn get(&self) -> Option<T> {
        let mut items = self.items.lock().await;
        items.pop_front()
    }

    /// Returns a `Tx` instance back to the pool.
    pub async fn release(&self, tx: T) {
        let mut items = self.items.lock().await;
        items.push_back(tx);
    }
}

/// Generic asynchronous pool trait for reusing resources.
#[async_trait]
pub trait Pool {
    /// The type of resource managed by this pool.
    type Item;
    /// Error type when acquiring a resource.
    type Error;
    /// Acquire a resource from the pool, waiting if necessary.
    async fn get(&self) -> Result<Self::Item, Self::Error>;
    /// Return a resource back to the pool.
    async fn release(&self, item: Self::Item);
}

#[async_trait]
impl<T: Tx + Send + Sync + 'static> Pool for TxPool<T> {
    type Item = T;
    type Error = ();

    async fn get(&self) -> Result<Self::Item, Self::Error> {
        self.get().await.ok_or(())
    }

    async fn release(&self, item: Self::Item) {
        self.release(item).await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use crate::context::Tx;
    use crate::pool::Pool;

    #[derive(Clone)]
    struct DummyTx {
        data: String,
    }

    #[async_trait]
    impl Tx for DummyTx {
        type Request = String;
        type Response = String;
        type Error = ();

        async fn process(&mut self, request: Self::Request) -> Result<&mut Self::Response, Self::Error> {
            self.data = request;
            Ok(&mut self.data)
        }

        async fn shutdown(&mut self) -> Result<(), Self::Error> {
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_txpool_add_get_release() {
        let pool = TxPool::new();
        pool.add(DummyTx { data: "init".to_string() }).await;
        let mut item = pool.get().await.expect("Should get a DummyTx");
        let resp = item.process("hello".to_string()).await.expect("process failed");
        assert_eq!(resp, &"hello".to_string());
        pool.release(item).await;
        let item2 = pool.get().await.expect("Should get again");
        assert_eq!(item2.data, "hello".to_string());
    }

    #[tokio::test]
    async fn test_txpool_pool_trait() {
        let pool = TxPool::new();
        pool.add(DummyTx { data: "".to_string() }).await;
        let item = <TxPool<DummyTx> as Pool>::get(&pool).await.expect("get via Pool");
        <TxPool<DummyTx> as Pool>::release(&pool, item).await;
    }
} 