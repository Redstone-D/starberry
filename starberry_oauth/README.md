# starberry_oauth

`starberry_oauth` is a flexible OAuth2 server and client library built on `starberry_core`.

## Features

- Fully async, pluggable stores (in-memory, database, JWT, custom)
- PKCE (S256) enforcement and CSRF protection
- JWT (HS256/RS256) issuance and validation with JWKS caching
- Structured `tracing` instrumentation
- Robust error handling with JSON responses
- Feature flags to enable optional plugins with zero runtime cost when disabled:
  - `oauth2` (default): pure OAuth2 core
  - `openid`: OpenID Connect server support (discovery, JWKS, id_token, userinfo)
  - `social`: Social login plugin (ExternalLoginProvider for upstream OAuth2/OIDC)

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
starberry_oauth = { version = "0.6.4", features = ["openid", "social"] }
```

Use `--no-default-features` or selective features to enable only what you need:

```bash
cargo build --no-default-features # only core OAuth2
cargo build --features openid      # core + OpenID Connect
cargo build --features social      # core + Social login
cargo build --all-features         # all plugins enabled
```

## Quick Start

In your `main.rs`, configure the OAuth middleware and attach to your `starberry_core` application:

```rust
use std::sync::Arc;
use starberry_core::app::application::App;
use starberry_core::app::protocol::ProtocolHandlerBuilder;
use starberry_core::http::context::HttpReqCtx;
use starberry_oauth::{OAuthLayer, InMemoryClientStore, InMemoryTokenManager};

#[tokio::main]
async fn main() {
    // Build OAuth2 middleware with in-memory stores
    let oauth_layer = OAuthLayer::new()
        .client_store(Arc::new(InMemoryClientStore::new(vec![])))
        .token_manager(Arc::new(InMemoryTokenManager::new()));

    // Attach middleware and run app
    let app = App::new()
        .single_protocol(
            ProtocolHandlerBuilder::<HttpReqCtx>::new()
                .append_middleware::<OAuthLayer>()
        )
        .build();

    app.run().await;
}
```

## Examples

The crate includes example programs under `examples/`:

- `minimal.rs`    — pure OAuth2 server example
- `openid.rs`     — OpenID Connect server example (`--features openid`)
- `social.rs`     — Social login stub example (`--features social`)

Run them with:

```bash
cargo run --example minimal
cargo run --example openid --features openid
cargo run --example social --features social
```

## Testing

Run all tests, including integration, unit, doc, and feature-gated tests:

```bash
cargo test --all-features
```

### OAuth2 Compliance

To validate RFC compliance, run the OAuth2 conformance tests from [oauth.net](https://oauth.net/2/conformance/).

### Integration Tests

Add Rust tests under `starberry_oauth/tests` exercising:

- Expired JWTs
- Bad CSRF tokens
- PKCE mismatches
- Rate-limited scenarios

Use `reqwest` or the in-memory HTTP client stub for simulating flows.

## Fuzz Testing

Use [`cargo fuzz`](https://github.com/rust-fuzz/cargo-fuzz) to catch panics in token parsing and URL decoding:

```bash
cargo install cargo-fuzz
cd starberry_oauth
cargo fuzz init
# create fuzz_targets/token_parser.rs that calls `jsonwebtoken::decode` with random input
cargo fuzz run token_parser
```

## Load Testing

Use [k6](https://k6.io) to simulate realistic auth-code and client-credentials traffic:

```js
// load_tests/auth.js
import http from 'k6/http';
import { check, sleep } from 'k6';

export let options = { vus: 50, duration: '1m' };

export default function() {
  let res = http.post('http://localhost:8080/oauth/token', {
    grant_type: 'client_credentials',
    client_id: __ENV.CLIENT_ID,
    client_secret: __ENV.CLIENT_SECRET,
  });
  check(res, { 'status is 200': (r) => r.status == 200 });
  sleep(1);
}
```

```bash
k6 run load_tests/auth.js
```

## Security Audit Checklist

- TLS enforcement: HTTPS-only endpoints
- HSTS and CSP headers
- Secure, HttpOnly, SameSite for cookies
- Rotate secrets (JWT keys, client credentials)
- Enforce PKCE S256 and constant-time comparisons
- Monitor rate-limit metrics
- Run dependency vulnerability scans (e.g. `cargo audit`)

---

Contributions welcome! Please file issues or PRs on GitHub. 