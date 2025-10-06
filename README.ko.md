<h1 align="center">bunner-cors-rs</h1>

<p align="center">
  <a href="https://crates.io/crates/bunner_cors_rs"><img src="https://img.shields.io/crates/v/bunner_cors_rs.svg" alt="Crates.io"></a>
  <a href="https://docs.rs/bunner_cors_rs"><img src="https://docs.rs/bunner_cors_rs/badge.svg" alt="Documentation"></a>
  <a href="https://github.com/parkrevil/bunner-cors-rs/actions"><img src="https://github.com/parkrevil/bunner-cors-rs/workflows/CI/badge.svg" alt="CI Status"></a>
  <a href="LICENSE.md"><img src="https://img.shields.io/badge/license-MIT-blue.svg" alt="License: MIT"></a>
</p>

<p align="center">
  <a href="README.md">English</a> | <strong>한국어</strong>
</p>

---

`bunner-cors-rs`는 요청에 대한 CORS 검증과 헤더 생성 로직에 집중한 라이브러리로써 어떤 HTTP 프레임워크와도 쉽게 통합할 수 있도록 설계되었습니다.

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
*   [**검증 및 결과 사용법**](#검증-및-결과-사용법)
    *   [요청 검증 준비](#요청-검증-준비)
    *   [판정 결과 처리](#판정-결과-처리)
*   [**예제**](#예제)
*   [**기여하기**](#기여하기)
*   [**라이선스**](#라이선스)

---
## 소개

### 특징

- ✅ **WHATWG Fetch 표준 준수**: 최신 CORS 명세를 정확히 따릅니다
- 🔒 **안전한 검증**: 잘못된 설정 조합을 생성 시점에 차단합니다
- 🌐 **유연한 Origin 매칭**: 정확한 문자열, 패턴, 커스텀 로직 등 다양한 방식 지원
- 🎯 **Private Network Access 지원**: 사설 네트워크 요청 처리
- 🧵 **Thread-safe**: 동시성 환경에서 안전하게 사용 가능
- 🪶 **경량**: 순수 CORS 로직만 제공, 프레임워크 독립적
- 📦 **프레임워크 중립**: Axum, Actix-web, Hyper 등 어디서나 사용 가능

### 주의사항

이 라이브러리는 **CORS 판정 로직**에만 집중합니다. HTTP 서버나 미들웨어 기능은 제공하지 않으므로, 사용 중인 프레임워크에 맞춰 통합 코드를 작성해야 합니다.

## 시작하기

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

### 빠른 시작

가장 간단한 CORS 설정부터 시작해보겠습니다.


```rust
use bunner_cors_rs::{Cors, CorsOptions, Origin, RequestContext};

let cors = Cors::new(CorsOptions {
    origin: Origin::Any,
    credentials: false,
    ..Default::default()
}).expect("valid configuration");

let request = RequestContext {
    method: "GET",
    origin: "https://example.com",
    access_control_request_method: "",
    access_control_request_headers: "",
    access_control_request_private_network: false,
};

match cors.check(&request) {
    Ok(decision) => println!("CORS decision: {:?}", decision),
    Err(e) => eprintln!("CORS error: {}", e),
}
```

> [!TIP]
> `Cors` 인스턴스는 애플리케이션 시작 시 한 번만 생성하고 요청마다 재사용하세요. 요청마다 새로 생성하면 불필요한 성능 부담이 생깁니다.

---

## CorsOptions

애플리케이션 사양에 맞게 CorsOptions 을 설정하세요. 다음은 `CorsOptions`과 `CorsOptions::default()`를 사용시 설정되는 기본값입니다.

| 옵션 | 기본값 | 설명 |
|------|--------|------|
| `origin` | `Origin::Any` | 모든 Origin 허용 |
| `methods` | `["GET", "HEAD", "PUT", "PATCH", "POST", "DELETE"]` | 일반적인 HTTP 메서드 |
| `allowed_headers` | `AllowedHeaders::List()` | 명시적으로 허용된 헤더만 |
| `exposed_headers` | `ExposedHeaders::None` | 노출 헤더 없음 |
| `credentials` | `false` | 자격증명 불허 |
| `max_age` | `None` | Preflight 캐시 미설정 |
| `allow_null_origin` | `false` | null Origin 불허 |
| `allow_private_network` | `false` | 사설망 접근 불허 |
| `timing_allow_origin` | `None` | 타이밍 정보 미노출 |

### `origin`
어떤 출처의 요청을 허용할지 결정합니다. CORS의 가장 핵심적인 설정으로, 다양한 매칭 전략을 제공합니다.

#### `Origin::Any`

개발 환경이나 공개 API에서 사용합니다.


```rust
use bunner_cors_rs::{CorsOptions, Origin};

let options = CorsOptions {
    origin: Origin::Any,
    credentials: false,
    ..Default::default()
};
```
```
Access-Control-Allow-Origin: *
Vary: Origin
```

> [!IMPORTANT]
> `credentials: true`일 때 `Origin::Any`는 사용할 수 없습니다.
> 
> [!NOTE]
> 이 라이브러리는 상황에 따라 `Vary: Origin`을 자동으로 추가합니다. 응답 헤더는 그대로 사용하면 됩니다.

#### `Origin::exact`

단일 도메인만 허용할 때 사용합니다. 자격증명과 함께 사용 가능합니다.

```rust
let options = CorsOptions {
    origin: Origin::exact("https://app.example.com"),
    credentials: true,
    ..Default::default()
};
```
```
Access-Control-Allow-Origin: https://app.example.com
Access-Control-Allow-Credentials: true
Vary: Origin
```

#### `Origin::list`

여러 도메인을 명시적으로 허용합니다.

```rust
use bunner_cors_rs::OriginMatcher;

let options = CorsOptions {
    origin: Origin::list(vec![
        OriginMatcher::exact("https://app.example.com"),
        OriginMatcher::exact("https://admin.example.com"),
    ]),
    ..Default::default()
};
```
```
Access-Control-Allow-Origin: https://app.example.com
Vary: Origin
```

#### `OriginMatcher::pattern_str`

정규식을 사용한 유연한 매칭입니다. 하위 도메인 전체를 허용할 때 유용합니다.

```rust
let options = CorsOptions {
    origin: Origin::list(vec![
        OriginMatcher::pattern_str(r"https://.*\.example\.com")
            .expect("valid pattern"),
    ]),
    ..Default::default()
};
```
```
Access-Control-Allow-Origin: https://api.example.com
Vary: Origin
```

> [!CAUTION]
> 패턴 길이는 최대 50,000자, 컴파일 시간은 100ms로 제한됩니다. 초과 시 `PatternError`가 발생합니다.
>
> [!TIP]
> - 가능한 경우 `exact`/`list`를 우선 사용하고, 정규식은 최소화하세요.
> - 앵커(`^`, `$`)를 사용해 과도한 매칭을 방지하세요.
> - 점(`.`), 물음표(`?`) 등은 의도대로 이스케이프하세요.

#### `Origin::predicate`

사용자가 직접 검증 조건을 설정합니다. `true` 반환 시 요청 Origin을 그대로 반영하고, false 반환 시 거부합니다.

```rust
let options = CorsOptions {
    origin: Origin::predicate(|origin, _ctx| {
        origin.ends_with(".trusted.com") || origin == "https://partner.io"
    }),
    ..Default::default()
};
```

```
Access-Control-Allow-Origin: https://api.trusted.com
Vary: Origin
```

> [!TIP]
> 더 세밀한 제어가 필요하면 `Origin::custom`을 사용하여 `OriginDecision`을 직접 반환할 수 있습니다.

#### `Origin::custom`

`OriginDecision`을 직접 제어하여 복잡한 로직을 구현할 수 있습니다:

```rust
use bunner_cors_rs::OriginDecision;

let options = CorsOptions {
    origin: Origin::custom(|maybe_origin, ctx| {
        match maybe_origin {
            Some(origin) if origin.starts_with("https://") => {
                if origin.ends_with(".trusted.com") {
                    OriginDecision::Mirror  // Request origin reflected
                } else if origin == "https://special.partner.io" {
                    OriginDecision::Exact("https://partner.io".into())  // Override origin
                } else {
                    OriginDecision::Disallow  // Reject
                }
            }
            Some(_) => OriginDecision::Disallow,  // Non-HTTPS rejected
            None => OriginDecision::Skip,  // No origin header, skip CORS
        }
    }),
    ..Default::default()
};
```

> [!WARNING]
> `credentials: true`인 상황에서 사용자 콜백이 `OriginDecision::Any`를 반환하면 런타임 오류가 발생합니다. CORS 표준상 자격증명과 와일드카드 Origin은 함께 사용할 수 없습니다.

---

### `methods`

Preflight 요청과 Simple 요청에서 허용할 HTTP 메서드를 지정합니다.

```rust
use bunner_cors_rs::AllowedMethods;

let options = CorsOptions {
    origin: Origin::Any,
    methods: AllowedMethods::list(["GET", "POST", "DELETE"]),
    ..Default::default()
};
```
```
Access-Control-Allow-Methods: GET,POST,DELETE
```

> [!TIP]
> 유효한 HTTP 메서드 토큰만 허용됩니다. 표준에 맞춰 대문자 메서드명을 사용하는 것을 권장합니다.

---

### `allowed_headers`

Preflight 요청에서 클라이언트가 보낼 수 있는 헤더를 지정합니다.


```rust
use bunner_cors_rs::AllowedHeaders;

let options = CorsOptions {
    origin: Origin::Any,
    allowed_headers: AllowedHeaders::list(["Content-Type", "Authorization", "X-Api-Key"]),
    ..Default::default()
};
```

```
Access-Control-Allow-Headers: Content-Type,Authorization,X-Api-Key
```

> [!IMPORTANT]
> - `credentials: true`일 때 `AllowedHeaders::Any`는 사용할 수 없습니다.
> - 허용 헤더 목록에 `"*"`를 포함할 수 없습니다. 와일드카드가 필요하다면 `AllowedHeaders::Any`를 사용하세요.


### `exposed_headers`

Simple 요청에서 클라이언트에게 노출할 응답 헤더를 지정합니다.


```rust
use bunner_cors_rs::ExposedHeaders;

let options = CorsOptions {
    origin: Origin::Any,
    exposed_headers: ExposedHeaders::list(["X-Total-Count", "X-Page-Number"]),
    ..Default::default()
};
```

```
Access-Control-Expose-Headers: X-Total-Count,X-Page-Number
```

> [!IMPORTANT]
> - `credentials: true`일 때 `ExposedHeaders::Any`는 사용할 수 없습니다.
> - `"*"`를 다른 헤더명과 혼합해 사용할 수 없습니다.

---

### `credentials`

쿠키, Authorization 헤더, TLS 클라이언트 인증서 등 자격증명을 포함한 요청을 허용할지 결정합니다.


```rust
let options = CorsOptions {
    origin: Origin::exact("https://app.example.com"),
    credentials: true,
    ..Default::default()
};
```
```
Access-Control-Allow-Origin: https://app.example.com
Access-Control-Allow-Credentials: true
Vary: Origin
```

> [!IMPORTANT]
> `credentials: true`일 때 다음 설정은 사용할 수 없습니다: `Origin::Any`, `AllowedHeaders::Any`, `ExposedHeaders::Any`, `TimingAllowOrigin::Any`.

> [!TIP]
> 쿠키 기반 인증을 사용하는 경우, 크로스 사이트 요청에는 쿠키에 `SameSite=None; Secure` 속성을 설정해야 브라우저가 전송합니다.

---

### `max_age`

브라우저가 Preflight 응답을 캐시할 시간(초)을 지정합니다. 이를 통해 반복적인 Preflight 요청을 줄여 성능을 향상시킬 수 있습니다.

```rust
let options = CorsOptions {
    origin: Origin::Any,
    max_age: Some(3600),
    ..Default::default()
};
```

```
Access-Control-Max-Age: 3600
```

> [!NOTE]
> `Some(0)`은 "캐시하지 않음"을 의미합니다. `None`과 달리 헤더가 `Access-Control-Max-Age: 0`으로 전송되어 브라우저에 캐시 금지를 명시합니다.

> 일부 브라우저는 `Access-Control-Max-Age`에 자체 상한을 둘 수 있습니다. 너무 큰 값을 설정해도 실제 캐시 지속시간은 브라우저 정책에 의해 단축될 수 있습니다.



---

### `allow_null_origin`

요청 헤더의 Origin이 문자열 `"null"`일 때 허용할지 여부를 결정합니다.

```rust
let options = CorsOptions {
    origin: Origin::Any,
    allow_null_origin: true,
    ..Default::default()
};
```
```
Access-Control-Allow-Origin: null
Vary: Origin
```

> [!WARNING]
> 보안상 민감하므로 신뢰된 환경에서만 활성화하세요.

> [!NOTE]
> `Origin` 헤더가 아예 없는 경우와 값이 `"null"`인 경우는 다릅니다. 이 라이브러리는 두 경우를 구분하여 처리합니다.

---

### `allow_private_network`

Private Network Access 요청을 허용합니다.

```rust
let options = CorsOptions {
    origin: Origin::exact("https://app.example.com"),
    credentials: true,
    allow_private_network: true,
    ..Default::default()
};
```
```
Access-Control-Allow-Origin: https://app.example.com
Access-Control-Allow-Credentials: true
Access-Control-Allow-Private-Network: true
Vary: Origin
```

> [!IMPORTANT]
> 이 옵션을 사용하려면 `credentials: true`와 특정 Origin 설정이 필수입니다.

> [!NOTE]
> 이 라이브러리는 Preflight 응답에 `Access-Control-Allow-Private-Network: true` 헤더를 설정해 줍니다. 이 헤더의 해석과 실제 동작은 브라우저가 결정합니다.

> 브라우저 지원 범위가 제한적일 수 있습니다. 최신 지원 현황을 확인하고 폴백 전략을 고려하세요.

---

### `timing_allow_origin`

`Timing-Allow-Origin` 헤더를 설정하여, 특정 Origin이 리소스 타이밍 정보에 접근할 수 있도록 허용합니다. 성능 분석 도구나 모니터링 서비스에서 유용합니다.

```rust
use bunner_cors_rs::TimingAllowOrigin;

let options = CorsOptions {
    origin: Origin::Any,
    timing_allow_origin: Some(TimingAllowOrigin::list([
        "https://analytics.example.com",
    ])),
    ..Default::default()
};
```

```
Timing-Allow-Origin: https://analytics.example.com
```

> [!IMPORTANT]
> `credentials: true`일 때 `TimingAllowOrigin::Any`는 사용할 수 없습니다.

> [!CAUTION]
> `Timing-Allow-Origin: *`는 리소스 타이밍 정보를 광범위하게 노출할 수 있습니다. 프라이버시와 정보 노출 위험을 검토하고 필요한 출처만 명시적으로 허용하세요.

---

## 오류

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

### 런타임 오류

`Cors::check()`는 `CorsError`를 반환할 수 있습니다.

| 오류 | 설명 |
|------|------|
| `InvalidOriginAnyWithCredentials` | `Origin::custom` 콜백이 `credentials: true` 상황에서 `OriginDecision::Any`를 반환한 경우 (CORS 표준 위반) |

---

## 검증 및 결과 사용법

### 요청 검증 준비

CORS 판정을 위해 HTTP 요청 정보를 `RequestContext`로 변환해야 합니다.

| 필드 | HTTP 헤더 | 설명 |
|------|-----------|------|
| `method` | 요청 메서드 | `"GET"`, `"POST"`, `"OPTIONS"` 등 실제 HTTP 메서드 |
| `origin` | `Origin` | 요청의 출처. 없으면 빈 문자열 `""` |
| `access_control_request_method` | `Access-Control-Request-Method` | Preflight 요청에서 실제로 사용할 메서드. 없으면 `""` |
| `access_control_request_headers` | `Access-Control-Request-Headers` | Preflight 요청에서 사용할 헤더 목록 (쉼표 구분). 없으면 `""` |
| `access_control_request_private_network` | `Access-Control-Request-Private-Network` | PNA 헤더 존재 여부 (`true`/`false`) |

```rust
use bunner_cors_rs::RequestContext;

let context = RequestContext {
    method: "POST",
    origin: "https://app.example.com",
    access_control_request_method: "POST",
    access_control_request_headers: "content-type",
    access_control_request_private_network: false,
};

let decision = cors.check(&context)?;
```

> [!TIP]
> HTTP 헤더명은 대소문자를 구분하지 않으므로, 대부분의 프레임워크에서 `headers.get("origin")`처럼 소문자로 접근합니다. Axum/Actix 모두 소문자 헤더명을 권장합니다.

### 판정 결과 처리

`cors.check()`는 `CorsDecision`을 반환하며 4가지 결과로 나뉩니다.

#### `PreflightAccepted`

OPTIONS 요청이 성공한 경우입니다. 반환된 헤더를 응답에 추가하세요. `204 No Content` 응답을 반환하는 것이 좋습니다.

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

Origin, 메서드 또는 헤더가 허용되지 않은 경우입니다. 보안을 위해 `403 Forbidden` 응답을 반환하는 것이 좋습니다. 거부 이유는 디버깅 목적으로 확인할 수 있습니다.

```rust
CorsDecision::PreflightRejected(rejection) => {
    eprintln!("CORS Preflight Rejected: {:?}", rejection.reason);

    return Response::builder().status(403).body(().into()).unwrap();
}
```

> [!CAUTION]
> 거부 사유는 서버 로그에서만 확인하고, 운영 환경에서는 상세 오류 정보를 클라이언트에 노출하지 않는 것이 좋습니다.

#### `SimpleAccepted`

일반적인 GET/POST 등의 요청입니다. 반환된 헤더를 실제 응답에 추가하세요.

```rust
CorsDecision::SimpleAccepted { headers } => {
    let mut response = HttpResponse::Ok();

    for (name, value) in headers {
        response.append_header((name, value));
    }

    return response.body(your_content);
}
```

#### `NotApplicable`

Origin 헤더가 없거나 CORS가 필요하지 않은 요청입니다. 응답에 CORS 헤더를 추가하지 말고 요청을 처리하시면 됩니다.

## 예제

`/examples/frameworks` 디렉토리에 프레임워크별 통합 예제가 제공됩니다. 인스턴스는 애플리케이션 시작 시 한 번 생성하여 상태로 공유하세요.

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

## 기여하기

기여는 일정 기간동안 받지 않습니다. 준비되는대로 업데이트 하겠습니다.

문제 혹은 요청사항이 있을 경우 이슈를 등록해주세요.

## 라이선스

MIT License. 자세한 내용은 [LICENSE.md](LICENSE.md) 파일을 참조하세요.
