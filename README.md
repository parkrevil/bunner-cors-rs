# bunner_cors_rs

Rust implementation of Bunner's CORS policy engine, providing fast, predictable cross-origin request handling for edge and proxy deployments.

## Highlights
- Strict validation of `CorsOptions`, including a credential and origin guard that prevents using `*` when credentials are enabled.
- Support for private network access (PNA) preflight headers and `Timing-Allow-Origin` emission.
- Rich origin matching options (wildcard, lists, predicates, and custom resolvers) with request normalization helpers.
- Friendly, descriptive validation errors when configurations are invalid.

## Getting started
Create a `Cors` instance with validated options and feed normalized requests to receive decisions:

```rust
use bunner_cors_rs::{Cors, CorsOptions, Origin};

fn build_cors() -> Result<Cors, bunner_cors_rs::ValidationError> {
    Cors::new(CorsOptions {
        origin: Origin::list(["https://example.com"]),
        credentials: true,
        ..CorsOptions::default()
    })
}
```

`Cors::new` validates the configuration and returns a `ValidationError` with a descriptive message when the configuration is invalid (for example, credentials with a wildcard origin).

## Development

This repository uses the standard quality gates:

1. `make format` – run `rustfmt` and ensure formatting is clean.
2. `make lint` – execute `cargo clippy` checks under the default feature set.
3. `make test` – run the full `cargo nextest` suite.

All new changes should keep these commands green.
