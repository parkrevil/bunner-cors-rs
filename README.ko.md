<h1 align="center">bunner-cors-rs</h1>

<p align="center">
    <a href="https://crates.io/crates/bunner_cors_rs"><img src="https://img.shields.io/crates/v/bunner_cors_rs.svg" alt="Crates.io"></a>
    <a href="https://github.com/parkrevil/bunner-cors-rs/releases"><img src="https://img.shields.io/github/v/release/parkrevil/bunner-cors-rs?sort=semver" alt="version"></a>
    <a href="https://github.com/parkrevil/bunner-cors-rs/actions/workflows/tests.yml"><img src="https://github.com/parkrevil/bunner-cors-rs/actions/workflows/tests.yml/badge.svg?branch=main" alt="tests"></a>
    <a href="https://codecov.io/gh/parkrevil/bunner-cors-rs"><img src="https://codecov.io/gh/parkrevil/bunner-cors-rs/branch/main/graph/badge.svg" alt="coverage"></a>
    <a href="LICENSE.md"><img src="https://img.shields.io/badge/license-MIT-blue.svg" alt="License"></a>
</p>

<p align="center">
  <a href="README.md">English</a> | <strong>한국어</strong>
</p>

---

<a id="소개"></a>
## ✨ 소개

`bunner-cors-rs`는 CORS 판정과 헤더 생성을 제공하는 라이브러리입니다.

- **표준 준수**: WHATWG Fetch 표준 및 CORS 명세 준수
- **설정 검증**: 생성 시점에 잘못된 옵션 조합 차단
- **Origin 매칭**: 정확한 문자열, 목록, 정규식, 사용자 정의 로직 지원
- **Private Network Access**: Preflight 요청에 대한 PNA 헤더 지원
- **Thread-safe**: `Cors` 인스턴스 공유 가능
- **프레임워크 중립**: HTTP 요청/응답 타입에 의존하지 않음


> [!IMPORTANT]
> 이 라이브러리는 HTTP 서버나 미들웨어 기능은 제공하지 않으므로 사용 중인 프레임워크에 맞춰 통합 코드를 작성해야 합니다.

---

## 📚 목차
*   [**소개**](#소개)
*   [**시작하기**](#시작하기)
    *   [설치](#설치)
    *   [빠른 시작](#빠른-시작)
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
*   [**오류**](#오류)
    *   [검증 오류](#검증-오류)
    *   [런타임 오류](#런타임-오류)
*   [**요청 판정 및 결과 처리**](#요청-판정-및-결과-처리)
    *   [요청 컨텍스트 준비](#요청-컨텍스트-준비)
    *   [판정 결과 처리](#판정-결과-처리)
*   [**예제**](#예제)
*   [**기여하기**](#기여하기)
*   [**라이선스**](#라이선스)

---

<a id="시작하기"></a>
## 🚀 시작하기

<a id="설치"></a>
### 설치

`cargo add`를 사용하여 라이브러리를 추가하세요:

```bash
cargo add bunner_cors_rs
```

또는 `Cargo.toml`에 직접 추가할 수 있습니다:

```toml
[dependencies]
bunner_cors_rs = "0.1.0"
```

<a id="빠른-시작"></a>
### 빠른 시작

아래 예제는 [`http`](https://docs.rs/http/latest/http/) 크레이트를 사용해 응답을 구성하며, `Cors::check()`에서 반환되는 결과를 실제 HTTP 응답으로 변환하는 흐름을 보여 줍니다.


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

let cors = Cors::new(CorsOptions::new().expect("valid configuration");

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
> `Cors` 인스턴스는 애플리케이션 시작 시 한 번 생성하고 재사용하세요.

---

<a id="corsoptions"></a>
## ⚙️ CorsOptions

애플리케이션 사양에 맞게 CorsOptions 을 설정하세요. 다음은 `CorsOptions`과 `CorsOptions::default()`를 사용시 설정되는 기본값입니다.

| 옵션 | 기본값 | 설명 |
|------|--------|------|
| `origin` | `Origin::Any` | 모든 Origin 허용 |
| `methods` | `["GET", "HEAD", "PUT", "PATCH", "POST", "DELETE"]` | 일반적인 HTTP 메서드 |
| `allowed_headers` | `AllowedHeaders::List()` | 명시적으로 허용된 헤더만 |
| `exposed_headers` | `ExposedHeaders::default()` | 노출 헤더 없음 |
| `credentials` | `false` | 자격증명 불허 |
| `max_age` | `None` | Preflight 캐시 미설정 |
| `allow_null_origin` | `false` | null Origin 불허 |
| `allow_private_network` | `false` | 사설망 접근 불허 |
| `timing_allow_origin` | `None` | 타이밍 정보 미노출 |

<a id="origin"></a>
### `origin`
허용할 출처를 지정합니다.

#### `Origin::Any`

모든 출처를 허용합니다.


```rust
use bunner_cors_rs::{CorsOptions, Origin};

let options = CorsOptions::new();
```
```http
Access-Control-Allow-Origin: *
Vary: Origin
```

> [!IMPORTANT]
> `credentials: true`일 때 `Origin::Any`는 사용할 수 없습니다.

#### `Origin::exact`

단일 도메인만 허용할 때 사용합니다.

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

여러 도메인을 명시적으로 허용합니다.

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

정규식을 사용한 유연한 매칭입니다.

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
> 패턴 길이는 최대 50,000자, 컴파일 시간은 100ms로 제한됩니다. 초과 시 `PatternError`가 발생합니다.

#### `Origin::predicate`

사용자가 직접 판정 조건을 설정합니다. `true` 반환 시 요청 Origin을 그대로 반영하고, false 반환 시 거부합니다.

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

CORS 판정을 비활성화합니다. `OriginDecision::Skip`을 반환하므로 `CorsDecision::NotApplicable`이 반환되고 CORS 헤더가 생성되지 않습니다.

```rust
let options = CorsOptions::new().origin(Origin::disabled());

let decision = cors.check(&request_context)?;
assert!(matches!(decision, CorsDecision::NotApplicable));
```

#### `Origin::custom`

`OriginDecision`을 직접 제어하여 복잡한 로직을 구현할 수 있습니다:

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
> `credentials: true`인 상황에서 사용자 콜백이 `OriginDecision::Any`를 반환하면 런타임 오류가 발생합니다. CORS 표준상 자격증명과 와일드카드 Origin은 함께 사용할 수 없습니다.

---

<a id="methods"></a>
### `methods`

Preflight 요청과 Simple 요청에서 허용할 HTTP 메서드를 지정합니다.

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

Preflight 요청에서 클라이언트가 보낼 수 있는 헤더를 지정합니다.


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
> - `credentials: true`일 때 `AllowedHeaders::Any`는 사용할 수 없습니다.
> - 허용 헤더 목록에 `"*"`를 포함할 수 없습니다. 와일드카드가 필요하다면 `AllowedHeaders::Any`를 사용하세요.


<a id="exposed_headers"></a>
### `exposed_headers`

Simple 요청에서 클라이언트에게 노출할 응답 헤더를 지정합니다.


```rust
use bunner_cors_rs::{CorsOptions, ExposedHeaders, Origin};

let options = CorsOptions::new()
    .exposed_headers(ExposedHeaders::list(["X-Total-Count", "X-Page-Number"]));
```

```http
Access-Control-Expose-Headers: X-Total-Count,X-Page-Number
```

> [!IMPORTANT]
> - `credentials: true`일 때 `ExposedHeaders::Any`는 사용할 수 없습니다.
> - `"*"`를 다른 헤더명과 혼합해 사용할 수 없습니다.

---

<a id="credentials"></a>
### `credentials`

자격증명을 포함한 요청 허용 여부를 지정합니다.


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
> `credentials: true`일 때 다음 설정은 사용할 수 없습니다: `Origin::Any`, `AllowedHeaders::Any`, `ExposedHeaders::Any`, `TimingAllowOrigin::Any`.

---

<a id="max_age"></a>
### `max_age`

Preflight 응답 캐시 시간(초)을 지정합니다.

```rust
let options = CorsOptions::new()
    .max_age(3600);
```

```http
Access-Control-Max-Age: 3600
```

> [!NOTE]
> `Some(0)`은 `Access-Control-Max-Age: 0` 헤더를 전송합니다. `None`은 헤더를 전송하지 않습니다.

---

<a id="allow_null_origin"></a>
### `allow_null_origin`

Origin 헤더 값이 `"null"`인 요청 허용 여부를 지정합니다.

```rust
let options = CorsOptions::new()
    .allow_null_origin(true);
```
```http
Access-Control-Allow-Origin: null
Vary: Origin
```

---

<a id="allow_private_network"></a>
### `allow_private_network`

Private Network Access 요청을 허용합니다.

```rust
let options = CorsOptions::new()
    .origin(Origin::exact("https://app.example.com"))
    .credentials(true)
    .allow_private_network(true);
```
```http
Access-Control-Allow-Origin: https://app.example.com
Access-Control-Allow-Credentials: true
Access-Control-Allow-Private-Network: true
Vary: Origin
```

> [!IMPORTANT]
> 이 옵션을 사용하려면 `credentials: true`와 특정 Origin 설정이 필수입니다.

---

<a id="timing_allow_origin"></a>
### `timing_allow_origin`

`Timing-Allow-Origin` 헤더를 지정합니다.

```rust
use bunner_cors_rs::{CorsOptions, Origin, TimingAllowOrigin};

let options = CorsOptions::new()
    .timing_allow_origin(TimingAllowOrigin::list([
        "https://analytics.example.com",
    ]));
```

```http
Timing-Allow-Origin: https://analytics.example.com
```

> [!IMPORTANT]
> `credentials: true`일 때 `TimingAllowOrigin::Any`는 사용할 수 없습니다.

---

<a id="오류"></a>
## 🚨 오류

<a id="검증-오류"></a>
### 검증 오류

`Cors::new()`는 잘못된 설정 조합이 있을 경우 `ValidationError`를 반환합니다. 주요 검증 오류는 다음과 같습니다.

| 오류 | 설명 |
|------|------|
| `CredentialsRequireSpecificOrigin` | `credentials: true`일 때 `Origin::Any` 사용 불가 |
| `AllowedHeadersAnyNotAllowedWithCredentials` | `credentials: true`일 때 `AllowedHeaders::Any` 사용 불가 |
| `AllowedHeadersListCannotContainWildcard` | 허용 헤더 목록에 `"*"` 포함 불가 (대신 `AllowedHeaders::Any` 사용) |
| `ExposeHeadersWildcardRequiresCredentialsDisabled` | 노출 헤더에 `"*"`를 쓰려면 `credentials: false` 필요 |
| `ExposeHeadersWildcardCannotBeCombined` | 노출 헤더에 `"*"`와 다른 헤더를 함께 지정 불가 |
| `PrivateNetworkRequiresCredentials` | `allow_private_network: true`일 때 `credentials: true` 필수 |
| `PrivateNetworkRequiresSpecificOrigin` | `allow_private_network: true`일 때 `Origin::Any` 사용 불가 |
| `TimingAllowOriginWildcardNotAllowedWithCredentials` | `credentials: true`일 때 `TimingAllowOrigin::Any` 사용 불가 |
| `AllowedMethodsCannotContainWildcard` | 허용 메서드 목록에 `"*"` 포함 불가 |
| `AllowedMethodsListContainsInvalidToken` | 허용 메서드가 유효한 HTTP 메서드 토큰이 아님 |
| `AllowedHeadersListContainsInvalidToken` | 허용 헤더가 유효한 HTTP 헤더 이름이 아님 |
| `ExposeHeadersListContainsInvalidToken` | 노출 헤더가 유효한 HTTP 헤더 이름이 아님 |

<a id="런타임-오류"></a>
### 런타임 오류

`Cors::check()`는 `CorsError`를 반환할 수 있습니다.

| 오류 | 설명 |
|------|------|
| `InvalidOriginAnyWithCredentials` | `Origin::custom` 콜백이 `credentials: true` 상황에서 `OriginDecision::Any`를 반환한 경우 (CORS 표준 위반) |

---

<a id="요청-판정-및-결과-처리"></a>
## 📋 요청 판정 및 결과 처리

<a id="요청-컨텍스트-준비"></a>
### 요청 컨텍스트 준비

CORS 판정을 위해 HTTP 요청 정보를 `RequestContext`로 변환해야 합니다.

| 필드 | 타입 | HTTP 헤더 | 설명 |
|------|------|-----------|------|
| `method` | `&'a str` | 요청 메서드 | 실제 HTTP 메서드 문자열 (`"GET"`, `"POST"`, `"OPTIONS"` 등) |
| `origin` | `Option<&'a str>` | `Origin` | 요청의 출처. 헤더가 없으면 `None` |
| `access_control_request_method` | `Option<&'a str>` | `Access-Control-Request-Method` | Preflight 요청에서 실행할 메서드. 값이 없으면 `None` |
| `access_control_request_headers` | `Option<&'a str>` | `Access-Control-Request-Headers` | Preflight 요청에서 사용할 헤더 목록(쉼표 구분). 값이 없으면 `None` |
| `access_control_request_private_network` | `bool` | `Access-Control-Request-Private-Network` | 헤더 존재 여부 (`true`/`false`) |

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

<a id="판정-결과-처리"></a>
### 판정 결과 처리

`cors.check()`는 요청 유형과 옵션 조합에 따라 다음 네 가지 결과 중 하나를 반환합니다.

| 변형 | 반환 조건 | 추가 설명 |
|------|-----------|-----------|
| `PreflightAccepted` | `OPTIONS` 요청이며 Origin, 메서드, 헤더가 모두 허용될 때 | Preflight 응답에 필요한 모든 CORS 헤더가 포함 |
| `PreflightRejected` | `OPTIONS` 요청이지만 Origin 또는 요청된 메서드/헤더가 허용되지 않을 때 | `PreflightRejectionReason`으로 거부 원인을 확인 |
| `SimpleAccepted` | 비-`OPTIONS` 요청이며 Origin 검사가 허용되고 요청 메서드가 허용 목록에 포함될 때 | Origin 허용 시 `Access-Control-Allow-Origin` 등 필요한 헤더가 포함 |
| `SimpleRejected` | 비-`OPTIONS` 요청이며 Origin 검사가 Disallow일 때 | `Vary` 헤더 등이 포함된 거부용 헤더 반환 |
| `NotApplicable` | CORS 처리가 필요 없거나 판단을 건너뛰어야 할 때 | Origin 헤더가 없거나, 허용 메서드 목록에 포함되지 않거나, `Origin::disabled()`을 사용한 경우 |

#### `PreflightAccepted`

OPTIONS 요청이 성공한 경우입니다. 반환된 헤더를 응답에 추가하세요.

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

Origin이 허용되지 않거나 요청된 메서드/헤더가 정책을 위반하면 이 변형을 반환합니다. `PreflightRejection.reason`에는 `OriginNotAllowed`, `MethodNotAllowed`, `HeadersNotAllowed`, `MissingAccessControlRequestMethod` 중 하나가 포함됩니다.

```rust
CorsDecision::PreflightRejected(rejection) => {
    eprintln!("CORS Preflight Rejected: {:?}", rejection.reason);

    return Response::builder().status(403).body(().into()).unwrap();
}
```

#### `SimpleAccepted`

단순 요청입니다. 반환된 헤더를 응답에 그대로 추가하세요.

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

Origin이 허용되지 않은 단순 요청입니다. 반환된 헤더(예: `Vary: Origin`)를 거부 응답과 함께 사용하세요.

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

CORS 처리가 필요하지 않습니다. CORS 헤더를 추가하지 마세요.

<a id="예제"></a>
## 📝 예제

프레임워크별 적용 예제는 `/examples` 디렉토리에 있습니다.

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

### 테스트

이 라이브러리는 유닛 테스트, 통합 테스트, property-based 테스트, snapshot 테스트를 포함합니다:

```bash
# 일반 테스트
make test

# 커버리지
make coverage

# 벤치마크
make bench
```

<a id="기여하기"></a>
## ❤️ 기여하기

기여는 일정 기간동안 받지 않습니다. 준비되는대로 업데이트 하겠습니다.

문제 혹은 요청사항이 있을 경우 이슈를 등록해주세요.

<a id="라이선스"></a>
## 📜 라이선스

MIT License. 자세한 내용은 [LICENSE.md](LICENSE.md) 파일을 참조하세요.
