use async_trait::async_trait;
use starberry_core::connection::{Rx, Tx};
use starberry_core::http::context::{HttpReqCtx, HttpResCtx};
use super::http_client::{OAuthHttpClient, HttpRequest, HttpResponse, HttpClientError};

/// Adapter to use an `OAuthHttpClient` as a `Tx` for outgoing HTTP requests.
pub struct ClientTx<C: OAuthHttpClient> {
    inner: C,
    last_response: Option<HttpResponse>,
}

impl<C: OAuthHttpClient> ClientTx<C> {
    /// Wraps an OAuthHttpClient into a Tx adapter.
    pub fn new(client: C) -> Self {
        ClientTx { inner: client, last_response: None }
    }
}

#[async_trait]
impl<C> Tx for ClientTx<C>
where
    C: OAuthHttpClient + Send + Sync + Clone + 'static,
{
    type Request = HttpRequest;
    type Response = HttpResponse;
    type Error = HttpClientError;

    /// Sends the request via the inner OAuthHttpClient and stores the response.
    async fn process(&mut self, request: Self::Request) -> Result<&mut Self::Response, Self::Error> {
        let resp = self.inner.execute(request).await?;
        self.last_response = Some(resp);
        Ok(self.last_response.as_mut().unwrap())
    }

    /// Optional cleanup for the Tx adapter. Here we just drop the inner client.
    async fn shutdown(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}
