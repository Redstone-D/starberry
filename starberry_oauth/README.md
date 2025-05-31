# starberry_oauth

`starberry_oauth` is a flexible OAuth2 server and client library built on `starberry_core`.

## Features

- Fully async, pluggable stores (in-memory, database, JWT, custom)
- PKCE (S256) enforcement and CSRF protection
- JWT (HS256/RS256) issuance and validation with JWKS caching
- Structured `tracing` instrumentation
- Robust error handling with JSON responses

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
starberry_oauth = { path = "../starberry_oauth" }
``` 

## Configuration (`starberry.toml`)

See [starberry.toml](../starberry.toml) for a complete example.  It covers:

- Default token TTLs
- OAuth2 provider entries
- Store backends (in-memory, database, redis)
- JWKS cache settings
- Rate-limiter settings

## Quick Start

In your `main.rs`:

```rust
use starberry_core::App;
use starberry_oauth::oauth_core::{OAuthLayer, JWTTokenManager, JwksCache};
use starberry_oauth::oauth_core::http_client::CoreHttpClient;

#[tokio::main]
async fn main() {
    // Load config from `starberry.toml`...

    // HTTP client and JWKS cache
    let http = CoreHttpClient::new(50, 1024 * 1024);
    let jwks = JwksCache::new(http.clone(), jwks_url, Duration::from_secs(600)).await.unwrap();

    // JWT manager
    let jwt_mgr = JWTTokenManager::new_rs256(&priv_pem, &pub_pem, 3600)
        .with_claims(issuer, audience)
        .with_jwks(jwks);

    // Build OAuth middleware
    let oauth = OAuthLayer::new()
        .client_store(in_memory_client_store)
        .token_manager(Arc::new(jwt_mgr))
        .authorize_endpoint("/oauth/authorize")
        .token_endpoint("/oauth/token");

    // App pipeline
    App::new()
        .with_middleware(oauth)
        .run("0.0.0.0:8080")
        .await;
}
```

## Testing

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