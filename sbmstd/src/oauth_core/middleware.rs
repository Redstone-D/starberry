use std::{sync::Arc, any::Any, pin::Pin, future::Future};
use starberry_core::context::Rc;
use starberry_core::app::middleware::AsyncMiddleware;
use super::oauth_provider::{ClientStore, TokenManager, Authorizer};
use super::memory::{InMemoryClientStore, InMemoryTokenManager, InMemoryAuthorizer};

/// OAuth2 middleware layer with configurable stores and endpoints.
#[derive(Clone)]
pub struct OAuthLayer {
    client_store: Arc<dyn ClientStore>,
    token_manager: Arc<dyn TokenManager>,
    authorizer: Arc<dyn Authorizer>,
    authorize_endpoint: String,
    token_endpoint: String,
}

impl OAuthLayer {
    /// Creates a new OAuthLayer with in-memory defaults and standard endpoints.
    pub fn new() -> Self {
        OAuthLayer {
            client_store: Arc::new(InMemoryClientStore::new(Vec::new())),
            token_manager: Arc::new(InMemoryTokenManager::new()),
            authorizer: Arc::new(InMemoryAuthorizer::new()),
            authorize_endpoint: "/oauth/authorize".into(),
            token_endpoint: "/oauth/token".into(),
        }
    }

    /// Sets a custom client store.
    pub fn client_store(mut self, store: Arc<dyn ClientStore>) -> Self {
        self.client_store = store;
        self
    }

    /// Sets a custom token manager.
    pub fn token_manager(mut self, manager: Arc<dyn TokenManager>) -> Self {
        self.token_manager = manager;
        self
    }

    /// Sets a custom authorizer.
    pub fn authorizer(mut self, authorizer: Arc<dyn Authorizer>) -> Self {
        self.authorizer = authorizer;
        self
    }

    /// Overrides the authorization endpoint path.
    pub fn authorize_endpoint<S: Into<String>>(mut self, path: S) -> Self {
        self.authorize_endpoint = path.into();
        self
    }

    /// Overrides the token endpoint path.
    pub fn token_endpoint<S: Into<String>>(mut self, path: S) -> Self {
        self.token_endpoint = path.into();
        self
    }
}

impl AsyncMiddleware for OAuthLayer {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn return_self() -> Self {
        OAuthLayer::new()
    }

    fn handle<'a>(
        &'a self,
        mut ctx: Rc,
        next: Box<dyn Fn(Rc) -> Pin<Box<dyn Future<Output = Rc> + Send>> + Send + Sync + 'static>,
    ) -> Pin<Box<dyn Future<Output = Rc> + Send + 'static>> {
        let authorize_path = self.authorize_endpoint.clone();
        let token_path = self.token_endpoint.clone();
        let client_store = self.client_store.clone();
        let token_manager = self.token_manager.clone();
        let authorizer = self.authorizer.clone();

        Box::pin(async move {
            let path = ctx.path();
            if path == authorize_path {
                // TODO: implement authorization code endpoint logic
            } else if path == token_path {
                // TODO: implement token endpoint logic
            } else {
                // TODO: validate bearer token and inject OAuth context for protected routes
            }
            next(ctx).await
        })
    }
} 