# bunner_cors_rs

Rust implementation of Bunner's CORS policy engine, providing fast, predictable cross-origin request handling for edge and proxy deployments.

## Highlights
- Strict validation of `CorsOptions`, including a credential and origin guard that prevents using `*` when credentials are enabled.
- Support for private network access (PNA) preflight headers and `Timing-Allow-Origin` emission.
- Rich origin matching options (wildcard, lists, predicates, and custom resolvers) with request normalization helpers.
- Extensible preflight response hook for adding custom headers or overriding the status/end-response behavior.

## Getting started
Create a `Cors` instance with validated options and feed normalized requests to receive decisions:

```rust
use bunner_cors_rs::{Cors, CorsOptions, Origin};

fn build_cors() -> Cors {
    Cors::try_new(CorsOptions {
        origin: Origin::list(["https://example.com"]),
        credentials: true,
        ..CorsOptions::default()
    }).expect("valid CORS configuration")
}
```

- `Cors::try_new` performs validation and returns an error if the configuration is invalid (for example, credentials with a wildcard origin).
- `Cors::new` is a convenience constructor that panics on invalid configurations and is useful once validation has been covered elsewhere.

### Preflight response hook

Customize preflight responses by registering a hook that has mutable access to the computed `PreflightResult`:

```rust
use bunner_cors_rs::{Cors, CorsOptions, Origin, PreflightResponseHook};
use std::sync::Arc;

fn build_cors_with_hook() -> Cors {
    let hook: PreflightResponseHook = Arc::new(|request, result| {
        if request.origin.ends_with(".internal") {
            result.headers.insert("x-internal-route".into(), "true".into());
            result.end_response = false; // keep chain alive for additional middleware
        }
    });

    Cors::try_new(CorsOptions {
        origin: Origin::list(["https://example.com"]),
        preflight_response_hook: Some(hook),
        ..CorsOptions::default()
    }).expect("valid CORS configuration")
}
```

The hook receives the normalized request context (lowercased components) along with the mutable `PreflightResult`, allowing you to add or remove headers, tweak the status code, or change whether the response short-circuits.

## Development

This repository uses the standard quality gates:

1. `make format` – run `rustfmt` and ensure formatting is clean.
2. `make lint` – execute `cargo clippy` checks under the default feature set.
3. `make test` – run the full `cargo nextest` suite.

All new changes should keep these commands green.
