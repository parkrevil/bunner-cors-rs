<h1 align="center">bunner-cors-rs</h1>

<p align="center">
    <a href="https://crates.io/crates/bunner_cors_rs"><img src="https://img.shields.io/crates/v/bunner_cors_rs.svg" alt="Crates.io"></a>
    <a href="https://github.com/parkrevil/bunner-cors-rs/actions/workflows/pr-check.yml"><img src="https://github.com/parkrevil/bunner-cors-rs/actions/workflows/pr-check.yml/badge.svg?branch=main" alt="CI"></a>
    <a href="https://github.com/parkrevil/bunner-cors-rs/actions/workflows/tests.yml"><img src="https://github.com/parkrevil/bunner-cors-rs/actions/workflows/tests.yml/badge.svg?branch=main" alt="Tests"></a>
    <a href="https://github.com/parkrevil/bunner-cors-rs/actions/workflows/coverage.yml"><img src="https://github.com/parkrevil/bunner-cors-rs/actions/workflows/coverage.yml/badge.svg?branch=main" alt="Coverage"></a>
    <a href="https://github.com/parkrevil/bunner-cors-rs/actions/workflows/publish.yml"><img src="https://github.com/parkrevil/bunner-cors-rs/actions/workflows/publish.yml/badge.svg?branch=main" alt="Release"></a>
    <a href="LICENSE.md"><img src="https://img.shields.io/badge/license-MIT-blue.svg" alt="License"></a>
</p>

<p align="center">
  <strong>English</strong> | <a href="README.ko.md">한국어</a>
</p>

---

<a id="introduction"></a>
## ✨ Introduction

`bunner-cors-rs` is a library that provides CORS decision-making and header generation.

- **Standards Compliant**: Adheres to WHATWG Fetch standard and CORS specification
- **Configuration Validation**: Blocks invalid option combinations at creation time
- **Origin Matching**: Supports exact strings, lists, regular expressions, and custom logic
- **Private Network Access**: PNA header support for preflight requests
- **Thread-safe**: `Cors` instances can be shared
- **Framework Neutral**: Does not depend on HTTP request/response types


> [!IMPORTANT]
> This library does not provide HTTP server or middleware functionality, so you must write integration code tailored to your framework.

---

## 📚 Table of Contents
*   [**Introduction**](#introduction)
*   [**Getting Started**](#getting-started)
    *   [Installation](#installation)
    *   [Quick Start](#quick-start)
*   [**CorsOptions**](#corsoptions)
    *   [origin](#origin)
    *   [methods](#methods)
    *   [allowed_headers](#allowed_headers)
    *   [exposed_headers](#exposed_headers)
    *   [credentials](#credentials)
    *   [max_age](#max_age)
    *   [allow_null_origin](#allow_null_origin)
    *   [allow_private_network](#allow_private_network)
    *   [timing_allow_origin](#timing_allow_origin)
*   [**Errors**](#errors)
    *   [Validation Errors](#validation-errors)
    *   [Runtime Errors](#runtime-errors)
*   [**Request Evaluation and Result Handling**](#request-evaluation-and-result-handling)
    *   [Preparing Request Context](#preparing-request-context)
    *   [Processing Decision Results](#processing-decision-results)
*   [**Examples**](#examples)
*   [**Contributing**](#contributing)
*   [**License**](#license)

---

<a id="getting-started"></a>
## 🚀 Getting Started

<a id="installation"></a>
### Installation

Add the library using `cargo add`:

```bash
cargo add bunner_cors_rs
```

Or add it directly to `Cargo.toml`:

```toml
[dependencies]
bunner_cors_rs = "0.1.0"
```

<a id="quick-start"></a>
### Quick Start

The example below uses the [`http`](https://docs.rs/http/latest/http/) crate to construct responses, showing how to convert the result returned by `Cors::check()` into an actual HTTP response.


```rust
use bunner_cors_rs::{
    Cors, CorsDecision, CorsError, CorsOptions, Headers, Origin, RequestContext,
};
use http::{header::HeaderName, HeaderValue, Response, StatusCode};

fn apply_headers(target: &mut http::HeaderMap, headers: Headers) {
    for (name, value) in headers {
        let name = HeaderName::from_bytes(name.as_bytes()).expect("valid header name");
        let value = HeaderValue::from_str(&value).expect("valid header value");
        target.insert(name, value);
    }
}

fn handle_request(cors: &Cors, ctx: RequestContext<'_>) -> Result<Response<String>, CorsError> {
    match cors.check(&ctx)? {
        CorsDecision::PreflightAccepted { headers } => {
            let mut response = Response::builder()
                .status(StatusCode::NO_CONTENT)
                .body(String::new())
                .unwrap();
            apply_headers(response.headers_mut(), headers);
            Ok(response)
        }
        CorsDecision::PreflightRejected(rejection) => {
            let mut response = Response::builder()
                .status(StatusCode::FORBIDDEN)
                .body(String::new())
                .unwrap();
            apply_headers(response.headers_mut(), rejection.headers);
            Ok(response)
        }
        CorsDecision::SimpleAccepted { headers } => {
            let mut response = Response::builder()
                .status(StatusCode::OK)
                .body("application response".into())
                .unwrap();
            apply_headers(response.headers_mut(), headers);
            Ok(response)
        }
        CorsDecision::SimpleRejected(rejection) => {
            let mut response = Response::builder()
                .status(StatusCode::FORBIDDEN)
                .body(String::new())
                .unwrap();
            apply_headers(response.headers_mut(), rejection.headers);
            Ok(response)
        }
        CorsDecision::NotApplicable => Ok(Response::builder()
            .status(StatusCode::OK)
            .body("non-CORS response".into())
            .unwrap()),
    }
}

let cors = Cors::new(CorsOptions::new()).expect("valid configuration");

let request = RequestContext {
    method: "GET",
    origin: Some("https://example.com"),
    access_control_request_method: None,
    access_control_request_headers: None,
    access_control_request_private_network: false,
};

match handle_request(&cors, request) {
    Ok(response) => {
        println!("status: {}", response.status());
    }
    Err(error) => {
        eprintln!("CORS error: {error}");
    }
}
```

> [!TIP]
> Create the `Cors` instance once at application startup and reuse it.

---

<a id="corsoptions"></a>
## ⚙️ CorsOptions

Configure CorsOptions according to your application requirements. The following shows `CorsOptions` and the default values when using `CorsOptions::default()`.

| Option | Default | Description |
|--------|---------|-------------|
| `origin` | `Origin::Any` | Allow all origins |
| `methods` | `["GET", "HEAD", "PUT", "PATCH", "POST", "DELETE"]` | Common HTTP methods |
| `allowed_headers` | `AllowedHeaders::List()` | Only explicitly allowed headers |
| `exposed_headers` | `ExposedHeaders::default()` | No exposed headers |
| `credentials` | `false` | Credentials not allowed |
| `max_age` | `None` | Preflight cache not configured |
| `allow_null_origin` | `false` | null origin not allowed |
| `allow_private_network` | `false` | Private network access not allowed |
| `timing_allow_origin` | `None` | Timing information not exposed |

<a id="origin"></a>
### `origin`
Specifies which origins to allow.

#### `Origin::Any`

Allows all origins.


```rust
use bunner_cors_rs::{CorsOptions, Origin};

let options = CorsOptions::new();
```
```http
Access-Control-Allow-Origin: *
Vary: Origin
```

> [!IMPORTANT]
> `Origin::Any` cannot be used when `credentials: true`.

#### `Origin::exact`

Use when allowing only a single domain.

```rust
let options = CorsOptions::new()
    .origin(Origin::exact("https://app.example.com"))
    .credentials(true);
```
```http
Access-Control-Allow-Origin: https://app.example.com
Access-Control-Allow-Credentials: true
Vary: Origin
```

#### `Origin::list`

Explicitly allows multiple domains.

```rust
use bunner_cors_rs::{CorsOptions, OriginMatcher};

let options = CorsOptions::new()
    .origin(Origin::list(vec![
        OriginMatcher::exact("https://app.example.com"),
        OriginMatcher::exact("https://admin.example.com"),
    ]));
```
```http
Access-Control-Allow-Origin: https://app.example.com
Vary: Origin
```

#### `OriginMatcher::pattern_str`

Flexible matching using regular expressions.

```rust
let options = CorsOptions::new()
    .origin(Origin::list(vec![
        OriginMatcher::pattern_str(r"https://.*\\.example\\.com")
            .expect("valid pattern"),
    ]));
```
```http
Access-Control-Allow-Origin: https://api.example.com
Vary: Origin
```

> [!CAUTION]
> Pattern length is limited to 50,000 characters and compile time to 100ms. Exceeding these limits will raise a `PatternError`.

#### `Origin::predicate`

Allows you to set custom validation logic. Returns the request Origin as-is when returning `true`, rejects when returning false.

```rust
let options = CorsOptions::new()
    .origin(Origin::predicate(|origin, _ctx| {
        origin.ends_with(".trusted.com") || origin == "https://partner.io"
    }));
```

```http
Access-Control-Allow-Origin: https://api.trusted.com
Vary: Origin
```

#### `Origin::disabled`

Disables CORS evaluation. Returns `OriginDecision::Skip`, so `CorsDecision::NotApplicable` is returned and no CORS headers are generated.

```rust
let options = CorsOptions::new().origin(Origin::disabled());

let decision = cors.check(&request_context)?;
assert!(matches!(decision, CorsDecision::NotApplicable));
```

#### `Origin::custom`

Directly controls `OriginDecision` to implement complex logic:

```rust
use bunner_cors_rs::OriginDecision;

let options = CorsOptions::new()
    .origin(Origin::custom(|maybe_origin, ctx| {
        match maybe_origin {
            Some(origin) if origin.starts_with("https://") => {
                if origin.ends_with(".trusted.com") {
                    OriginDecision::Mirror
                } else if origin == "https://special.partner.io" {
                    OriginDecision::Exact("https://partner.io".into())
                } else {
                    OriginDecision::Disallow
                }
            }
            Some(_) => OriginDecision::Disallow,
            None => OriginDecision::Skip,
        }
    }));
```

> [!WARNING]
> If a user callback returns `OriginDecision::Any` when `credentials: true`, a runtime error occurs. According to the CORS standard, credentials and wildcard origins cannot be used together.

---

<a id="methods"></a>
### `methods`

Specifies HTTP methods to allow in preflight and simple requests.

```rust
use bunner_cors_rs::{AllowedMethods, CorsOptions, Origin};

let options = CorsOptions::new()
    .methods(AllowedMethods::list(["GET", "POST", "DELETE"]));
```
```http
Access-Control-Allow-Methods: GET,POST,DELETE
```

---

<a id="allowed_headers"></a>
### `allowed_headers`

Specifies headers the client can send in preflight requests.


```rust
use bunner_cors_rs::{AllowedHeaders, CorsOptions, Origin};

let options = CorsOptions::new()
    .allowed_headers(AllowedHeaders::list([
        "Content-Type",
        "Authorization",
        "X-Api-Key",
    ]));
```

```http
Access-Control-Allow-Headers: Content-Type,Authorization,X-Api-Key
```

> [!IMPORTANT]
> - `AllowedHeaders::Any` cannot be used when `credentials: true`.
> - `"*"` cannot be included in the allowed headers list. Use `AllowedHeaders::Any` if you need a wildcard.


<a id="exposed_headers"></a>
### `exposed_headers`

Specifies response headers to expose to the client in simple requests.


```rust
use bunner_cors_rs::{CorsOptions, ExposedHeaders, Origin};

let options = CorsOptions::new()
    .exposed_headers(ExposedHeaders::list(["X-Total-Count", "X-Page-Number"]));
```

```http
Access-Control-Expose-Headers: X-Total-Count,X-Page-Number
```

> [!IMPORTANT]
> - `ExposedHeaders::Any` cannot be used when `credentials: true`.
> - `"*"` cannot be mixed with other header names.

---

<a id="credentials"></a>
### `credentials`

Specifies whether to allow requests with credentials.


```rust
let options = CorsOptions::new()
    .origin(Origin::exact("https://app.example.com"))
    .credentials(true);
```
```http
Access-Control-Allow-Origin: https://app.example.com
Access-Control-Allow-Credentials: true
Vary: Origin
```

> [!IMPORTANT]
> When `credentials: true`, the following configurations cannot be used: `Origin::Any`, `AllowedHeaders::Any`, `ExposedHeaders::Any`, `TimingAllowOrigin::Any`.

---

<a id="max_age"></a>
### `max_age`

Specifies the preflight response cache time in seconds.

```rust
let options = CorsOptions::new()
    .max_age(3600);
```

```http
Access-Control-Max-Age: 3600
```

> [!NOTE]
> `Some(0)` sends the `Access-Control-Max-Age: 0` header. `None` does not send the header.

---

<a id="allow_null_origin"></a>
### `allow_null_origin`

Specifies whether to allow requests with Origin header value `"null"`.

```rust
let options = CorsOptionsBuilder::new()
    .allow_null_origin(true)
    .build()
    .expect("valid configuration");
```
```http
Access-Control-Allow-Origin: null
Vary: Origin
```

---

<a id="allow_private_network"></a>
### `allow_private_network`

Allows Private Network Access requests.

```rust
let options = CorsOptionsBuilder::new()
    .origin(Origin::exact("https://app.example.com"))
    .credentials(true)
    .allow_private_network(true)
    .build()
    .expect("valid configuration");
```
```http
Access-Control-Allow-Origin: https://app.example.com
Access-Control-Allow-Credentials: true
Access-Control-Allow-Private-Network: true
Vary: Origin
```

> [!IMPORTANT]
> To use this option, `credentials: true` and a specific origin configuration are required.

---

<a id="timing_allow_origin"></a>
### `timing_allow_origin`

Specifies the `Timing-Allow-Origin` header.

```rust
use bunner_cors_rs::{CorsOptions, Origin, TimingAllowOrigin};

let options = CorsOptionsBuilder::new()
    .timing_allow_origin(TimingAllowOrigin::list([
        "https://analytics.example.com",
    ]))
    .build()
    .expect("valid configuration");
```

```http
Timing-Allow-Origin: https://analytics.example.com
```

> [!IMPORTANT]
> `TimingAllowOrigin::Any` cannot be used when `credentials: true`.

---

<a id="errors"></a>
## 🚨 Errors

<a id="validation-errors"></a>
### Validation Errors

`Cors::new()` returns a `ValidationError` if there are invalid configuration combinations. The main validation errors are:

| Error | Description |
|-------|-------------|
| `CredentialsRequireSpecificOrigin` | Cannot use `Origin::Any` when `credentials: true` |
| `AllowedHeadersAnyNotAllowedWithCredentials` | Cannot use `AllowedHeaders::Any` when `credentials: true` |
| `AllowedHeadersListCannotContainWildcard` | Cannot include `"*"` in allowed headers list (use `AllowedHeaders::Any` instead) |
| `ExposeHeadersWildcardRequiresCredentialsDisabled` | Need `credentials: false` to use `"*"` in exposed headers |
| `ExposeHeadersWildcardCannotBeCombined` | Cannot specify `"*"` with other headers in exposed headers |
| `PrivateNetworkRequiresCredentials` | `credentials: true` required when `allow_private_network: true` |
| `PrivateNetworkRequiresSpecificOrigin` | Cannot use `Origin::Any` when `allow_private_network: true` |
| `TimingAllowOriginWildcardNotAllowedWithCredentials` | Cannot use `TimingAllowOrigin::Any` when `credentials: true` |
| `AllowedMethodsCannotContainWildcard` | Cannot include `"*"` in allowed methods list |
| `AllowedMethodsListContainsInvalidToken` | Allowed method is not a valid HTTP method token |
| `AllowedHeadersListContainsInvalidToken` | Allowed header is not a valid HTTP header name |
| `ExposeHeadersListContainsInvalidToken` | Exposed header is not a valid HTTP header name |

<a id="runtime-errors"></a>
### Runtime Errors

`Cors::check()` can return a `CorsError`.

| Error | Description |
|-------|-------------|
| `InvalidOriginAnyWithCredentials` | When `Origin::custom` callback returns `OriginDecision::Any` in a `credentials: true` situation (violates CORS standard) |

---

<a id="request-evaluation-and-result-handling"></a>
## 📋 Request Evaluation and Result Handling

<a id="preparing-request-context"></a>
### Preparing Request Context

HTTP request information must be converted to `RequestContext` for CORS evaluation.

| Field | Type | HTTP Header | Description |
|-------|------|-------------|-------------|
| `method` | `&'a str` | Request method | Actual HTTP method string (`"GET"`, `"POST"`, `"OPTIONS"`, etc.) |
| `origin` | `Option<&'a str>` | `Origin` | Request origin. Use `None` when the header is absent. |
| `access_control_request_method` | `Option<&'a str>` | `Access-Control-Request-Method` | Method to execute in preflight request. `None` if absent |
| `access_control_request_headers` | `Option<&'a str>` | `Access-Control-Request-Headers` | Comma-separated list of headers to use in preflight request. `None` if absent |
| `access_control_request_private_network` | `bool` | `Access-Control-Request-Private-Network` | Header presence (`true`/`false`). |

```rust
use bunner_cors_rs::RequestContext;

let context = RequestContext {
    method: "POST",
    origin: Some("https://app.example.com"),
    access_control_request_method: Some("POST"),
    access_control_request_headers: Some("content-type"),
    access_control_request_private_network: false,
};

let decision = cors.check(&context)?;
```

<a id="processing-decision-results"></a>
### Processing Decision Results

`cors.check()` returns one of the following four results depending on request type and option combination.

| Variant | Return Condition | Additional Description |
|---------|------------------|------------------------|
| `PreflightAccepted` | `OPTIONS` request with allowed Origin, method, and headers | Includes all CORS headers needed for preflight response |
| `PreflightRejected` | `OPTIONS` request but Origin or requested method/headers not allowed | Can check rejection reason via `PreflightRejectionReason` |
| `SimpleAccepted` | Non-`OPTIONS` request with allowed Origin check and request method in allowed list | Includes `Access-Control-Allow-Origin` and other necessary headers when origin is allowed |
| `SimpleRejected` | Non-`OPTIONS` request with Disallow Origin check | Returns rejection headers including `Vary` header |
| `NotApplicable` | CORS processing not needed or should be skipped | Cases like no Origin header, method not in allowed list, or using `Origin::disabled()` |

#### `PreflightAccepted`

OPTIONS request succeeded. Add the returned headers to the response.

```rust
use bunner_cors_rs::CorsDecision;

match cors.check(&context)? {
    CorsDecision::PreflightAccepted { headers } => {
        let mut response = Response::builder().status(204).body(().into()).unwrap();

        for (name, value) in headers {
            response.headers_mut().insert(
                name.parse().unwrap(),
                value.parse().unwrap(),
            );
        }

        return response;
    }
    _ => {}
}
```

#### `PreflightRejected`

Returns this variant when origin is not allowed or requested method/headers violate policy. `PreflightRejection.reason` contains one of: `OriginNotAllowed`, `MethodNotAllowed`, `HeadersNotAllowed`, `MissingAccessControlRequestMethod`.

```rust
CorsDecision::PreflightRejected(rejection) => {
    eprintln!("CORS Preflight Rejected: {:?}", rejection.reason);

    return Response::builder().status(403).body(().into()).unwrap();
}
```

#### `SimpleAccepted`

Simple request. Add the returned headers directly to the response.

```rust
CorsDecision::SimpleAccepted { headers } => {
    let mut response = HttpResponse::Ok();

    for (name, value) in headers {
        response.append_header((name, value));
    }

    return response.body(your_content);
}
```

#### `SimpleRejected`

Simple request with disallowed origin. Use the returned headers (e.g., `Vary: Origin`) with a rejection response.

```rust
CorsDecision::SimpleRejected(rejection) => {
    let mut response = HttpResponse::Forbidden();

    for (name, value) in rejection.headers {
        response.append_header((name, value));
    }

    return response.finish();
}
```

#### `NotApplicable`

CORS processing not needed. Do not add CORS headers.

<a id="examples"></a>
## 📝 Examples

Framework-specific examples are in the `/examples` directory.

### axum
```bash
cargo run --example axum
curl -X GET -H "Origin: http://api.example.com" -I http://127.0.0.1:5001/greet
```

### Actix Web
```bash
cargo run --example actix
curl -X GET -H "Origin: http://api.example.com" -I http://127.0.0.1:5002/greet
```

### hyper
```bash
cargo run --example hyper
curl -X GET -H "Origin: http://api.example.com" -I http://127.0.0.1:5003/greet
```

### Testing

This library includes unit tests, integration tests, property-based tests, and snapshot tests:

```bash
# Run tests
make test

# Coverage
make coverage

# Benchmarks
make bench
```

<a id="contributing"></a>
## ❤️ Contributing

Contributions are not being accepted for a period of time. Updates will be provided when ready.

Please file an issue if you have problems or requests.

<a id="license"></a>
## 📜 License

MIT License. See [LICENSE.md](LICENSE.md) for details.
