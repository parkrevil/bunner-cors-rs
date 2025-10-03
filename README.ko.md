# bunner-cors-rs

<p align="center">
  <a href="https://crates.io/crates/bunner_cors_rs"><img src="https://img.shields.io/crates/v/bunner_cors_rs.svg" alt="Crates.io"></a>
  <a href="https://docs.rs/bunner_cors_rs"><img src="https://docs.rs/bunner_cors_rs/badge.svg" alt="Documentation"></a>
  <a href="https://github.com/parkrevil/bunner-cors-rs/actions"><img src="https://github.com/parkrevil/bunner-cors-rs/workflows/CI/badge.svg" alt="CI Status"></a>
  <a href="LICENSE.md"><img src="https://img.shields.io/badge/license-MIT-blue.svg" alt="License: MIT"></a>
</p>

<p align="center">
  <strong>한국어</strong> | <a href="README.md">English</a>
</p>

<p align="center">
  <em>Minimum Supported Rust Version: 1.75+</em>
</p>

---

`bunner-cors-rs`는 웹 서버나 미들웨어가 아닌, CORS 판정과 헤더 생성 로직에만 집중하여 어떤 HTTP 프레임워크와도 쉽게 통합할 수 있도록 설계된 핵심 라이브러리입니다.

## 📚 목차
*   [**소개**](#소개)
*   [**시작하기**](#시작하기)
    *   [설치](#설치)
    *   [빠른 시작](#빠른-시작)
*   [**옵션**](#옵션)
    *   [Origin 제어](#origin-제어)
    *   [메서드 제어](#메서드-제어)
    *   [헤더 제어](#헤더-제어)
    *   [자격증명 (Credentials)](#자격증명-credentials)
    *   [Preflight 캐싱](#preflight-캐싱)
    *   [Null Origin 허용](#null-origin-허용)
    *   [사설망 접근 (PNA)](#사설망-접근-pna)
    *   [리소스 타이밍](#리소스-타이밍)
    *   [설정 검증 오류](#설정-검증-오류)
*   [**판정 결과 처리**](#판정-결과-처리)
*   [**프레임워크 통합 예시**](#프레임워크-통합-예시)
    *   [Axum](#axum)
    *   [Actix-web](#actix-web)
*   [**성능 및 안전성**](#성능-및-안전성)
*   [**FAQ 및 문제 해결**](#faq-및-문제-해결)
*   [**API 문서**](#api-문서)
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

```sh
cargo add bunner_cors_rs
```

또는 `Cargo.toml`에 직접 추가할 수 있습니다:

```toml
[dependencies]
bunner_cors_rs = "0.1.0"
```

### 빠른 시작

가장 간단한 CORS 설정부터 시작해보겠습니다.

> **💡 중요**: `Cors` 인스턴스는 애플리케이션 시작 시 한 번만 생성하고 요청마다 재사용하세요. 매 요청마다 새로 생성하면 검증 비용이 반복됩니다.

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

이제 각 옵션을 상세히 알아보겠습니다.

#### 기본값 참고

`CorsOptions::default()`를 사용하면 다음 기본값이 적용됩니다:

| 옵션 | 기본값 | 설명 |
|------|--------|------|
| `origin` | `Origin::Any` | 모든 Origin 허용 |
| `methods` | `["GET", "HEAD", "PUT", "PATCH", "POST", "DELETE"]` | 일반적인 HTTP 메서드 |
| `allowed_headers` | `AllowedHeaders::List(빈 목록)` | 명시적으로 허용된 헤더만 |
| `exposed_headers` | `None` | 노출 헤더 없음 |
| `credentials` | `false` | 자격증명 불허 |
| `max_age` | `None` | Preflight 캐시 미설정 |
| `allow_null_origin` | `false` | null Origin 불허 |
| `allow_private_network` | `false` | 사설망 접근 불허 |
| `timing_allow_origin` | `None` | 타이밍 정보 미노출 |

---

## 옵션

각 옵션의 역할, 사용법, 그리고 실제 동작 결과를 예제와 함께 제공합니다.

### Origin 제어

#### 설명

`origin` 옵션은 어떤 출처(Origin)의 요청을 허용할지 결정합니다. CORS의 가장 핵심적인 설정으로, 다양한 매칭 전략을 제공합니다.

#### 사용 예제

**`Origin::Any` - 모든 Origin 허용**

개발 환경이나 공개 API에서 사용합니다. 단, `credentials: true`와 함께 사용할 수 없습니다.

```rust
use bunner_cors_rs::{CorsOptions, Origin};

let options = CorsOptions {
    origin: Origin::Any,
    credentials: false,
    ..Default::default()
};
```

**결과:**
```
Access-Control-Allow-Origin: *
Vary: Origin
```

> **💡 `Vary: Origin` 헤더란?**
> 이 헤더는 동일한 URL이라도 `Origin` 헤더 값에 따라 응답이 달라질 수 있음을 캐싱 프록시(예: CDN)에 알립니다. 이를 통해 특정 Origin을 위해 생성된 응답이 다른 Origin 사용자에게 잘못 캐시되어 제공되는 문제를 방지합니다. `bunner-cors-rs`는 CORS 안전성을 위해 필요 시 이 헤더를 자동으로 추가합니다.
> 
> 추가 팁: `Vary: Origin`은 캐시 단편화(fragmentation)를 늘릴 수 있습니다. CDN을 사용한다면 CORS 응답을 과도하게 캐시하지 않도록 주의하고, Preflight는 가능한 한 빠르게 처리(예: OPTIONS 조기 반환)해 서버 비용을 줄이세요.

**`Origin::exact` - 특정 Origin만 허용**

단일 도메인만 허용할 때 사용합니다. 자격증명과 함께 사용 가능합니다.

```rust
let options = CorsOptions {
    origin: Origin::exact("https://app.example.com"),
    credentials: true,
    ..Default::default()
};
```

**결과:**
```
Access-Control-Allow-Origin: https://app.example.com
Access-Control-Allow-Credentials: true
Vary: Origin
```

**`Origin::list` - 여러 Origin 목록**

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

**결과:**
```
Access-Control-Allow-Origin: https://app.example.com  (요청 Origin이 목록에 있으면 그대로 반영)
Vary: Origin
```

**`OriginMatcher::pattern_str` - 패턴 매칭**

정규식을 사용한 유연한 매칭입니다. 하위 도메인 전체를 허용할 때 유용합니다.

> **제약사항**: 패턴 길이는 최대 50,000자, 컴파일 시간은 100ms로 제한됩니다. 초과 시 `PatternError`가 발생합니다.

```rust
let options = CorsOptions {
    origin: Origin::list(vec![
        OriginMatcher::pattern_str(r"https://.*\.example\.com")
            .expect("valid pattern"),
    ]),
    ..Default::default()
};
```

**결과:**
```
Access-Control-Allow-Origin: https://api.example.com  (패턴 매칭 성공 시 요청 Origin 반영)
Vary: Origin
```

> 모범사례:
> - 가능한 경우 `exact`/`list`를 우선 사용하고, 정규식은 최소화하세요.
> - 앵커(`^`, `$`)를 사용해 과도한 매칭을 방지하세요.
> - 점(`.`), 물음표(`?`) 등은 의도대로 이스케이프하세요.
>
> 예시(권장 형태):
>
> ```rust
> // https://example.com 또는 https://{sub}.example.com 만 허용
> OriginMatcher::pattern_str(r"^https://([a-z0-9-]+\.)?example\.com$")?;
> ```

**`Origin::predicate` / `Origin::custom` - 동적 검증**

복잡한 비즈니스 로직을 적용할 때 사용합니다.

```rust
let options = CorsOptions {
    origin: Origin::predicate(|origin, _ctx| {
        origin.ends_with(".trusted.com") || origin == "https://partner.io"
    }),
    ..Default::default()
};
```

**결과:**
```
Access-Control-Allow-Origin: https://api.trusted.com  (검증 성공 시 요청 Origin 반영)
Vary: Origin
```

> **참고**: `Origin::predicate`는 true 반환 시 요청 Origin을 그대로 반영(Mirror)하고, false 반환 시 거부(Disallow)합니다. 더 세밀한 제어가 필요하면 `Origin::custom`을 사용하여 `OriginDecision`을 직접 반환할 수 있습니다.

**`Origin::custom` 고급 예제**

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

---

### 메서드 제어

#### 설명

`methods` 옵션은 Preflight 요청에서 허용할 HTTP 메서드를 지정합니다. 기본값은 `GET`, `HEAD`, `PUT`, `PATCH`, `POST`, `DELETE`입니다.

#### 사용 예제

```rust
use bunner_cors_rs::AllowedMethods;

let options = CorsOptions {
    origin: Origin::Any,
    methods: AllowedMethods::list(["GET", "POST", "DELETE"]),
    ..Default::default()
};
```

**결과:**
```
Access-Control-Allow-Methods: GET,POST,DELETE
```

---

### 헤더 제어

#### 설명

- **`allowed_headers`**: Preflight 요청에서 클라이언트가 보낼 수 있는 헤더를 지정합니다
- **`exposed_headers`**: Simple 요청에서 클라이언트에게 노출할 응답 헤더를 지정합니다

#### 사용 예제

**`allowed_headers` - 허용 요청 헤더**

```rust
use bunner_cors_rs::AllowedHeaders;

let options = CorsOptions {
    origin: Origin::Any,
    allowed_headers: AllowedHeaders::list(["Content-Type", "Authorization", "X-Api-Key"]),
    ..Default::default()
};
```

**결과:**
```
Access-Control-Allow-Headers: Content-Type,Authorization,X-Api-Key
```

> 모든 헤더 허용하기:
>
> ```rust
> // AllowedHeaders::any()로 모든 헤더를 허용할 수 있습니다.
> // 단, credentials: true와는 함께 사용할 수 없습니다.
> let options = CorsOptions {
>     origin: Origin::Any,
>     credentials: false,
>     allowed_headers: AllowedHeaders::any(),
>     ..Default::default()
> };
> ```
>
> 이 경우 `Access-Control-Allow-Headers` 헤더에 요청된 헤더가 그대로 반영됩니다.

**`exposed_headers` - 노출 응답 헤더**

```rust
let options = CorsOptions {
    origin: Origin::Any,
    exposed_headers: Some(vec!["X-Total-Count".into(), "X-Page-Number".into()]),
    ..Default::default()
};
```

**결과:**
```
Access-Control-Expose-Headers: X-Total-Count,X-Page-Number
```

> 와일드카드 사용하기:
>
> ```rust
> // '*' 사용은 credentials: false 일 때만 허용됩니다.
> let options = CorsOptions {
>     origin: Origin::Any,
>     credentials: false,
>     exposed_headers: Some(vec!["*".into()]),
>     ..Default::default()
> };
> ```
>
> 주의: `"*"`를 다른 헤더명과 혼합해 사용할 수 없습니다.

---

### 자격증명 (Credentials)

#### 설명

`credentials` 옵션은 쿠키, Authorization 헤더, TLS 클라이언트 인증서 등 자격증명을 포함한 요청을 허용할지 결정합니다. `true`로 설정하면 반드시 특정 Origin을 지정해야 합니다 (`Origin::Any` 사용 불가).

#### 사용 예제

```rust
let options = CorsOptions {
    origin: Origin::exact("https://app.example.com"),
    credentials: true,
    ..Default::default()
};
```

**결과:**
```
Access-Control-Allow-Origin: https://app.example.com
Access-Control-Allow-Credentials: true
Vary: Origin
```

---

### Preflight 캐싱

#### 설명

`max_age` 옵션은 브라우저가 Preflight 응답을 캐시할 시간(초)을 지정합니다. 이를 통해 반복적인 Preflight 요청을 줄여 성능을 향상시킬 수 있습니다.

> **형식**: 반드시 음이 아닌 정수를 나타내는 문자열이어야 합니다 (예: `"3600"`). 공백이나 비정수 값은 `InvalidMaxAge` 오류를 발생시킵니다.

#### 사용 예제

```rust
let options = CorsOptions {
    origin: Origin::Any,
    max_age: Some("3600".into()),
    ..Default::default()
};
```

**결과:**
```
Access-Control-Max-Age: 3600
```

> 브라우저 제한: 일부 브라우저는 `Access-Control-Max-Age`의 상한을 내부 정책으로 더 낮게 적용할 수 있습니다(특히 Safari 계열). 설정값이 크더라도 실제 캐싱 시간은 브라우저마다 다를 수 있습니다.

---

### Null Origin 허용

#### 설명

`allow_null_origin` 옵션은 `null` Origin을 허용할지 결정합니다. 파일 프로토콜(`file://`), 샌드박스 iframe, 리다이렉트된 요청 등에서 Origin이 `null`로 표시될 수 있습니다. 보안상 주의가 필요하며, 신뢰할 수 있는 환경에서만 활성화하세요.

#### 사용 예제

```rust
let options = CorsOptions {
    origin: Origin::Any,
    allow_null_origin: true,
    ..Default::default()
};
```

**결과:**
```
Access-Control-Allow-Origin: null  (요청 Origin이 "null"인 경우)
Vary: Origin
```

> **⚠️ 보안 경고**: `null` Origin은 악의적인 공격자가 쉽게 위조할 수 있으므로, 프로덕션 환경에서는 매우 신중하게 사용해야 합니다.

> 참고: `Origin` 헤더가 "없음"(미포함)인 경우와 값이 문자열 `"null"`인 경우는 다릅니다.
> - 헤더가 없으면 대개 CORS가 적용되지 않는 요청으로 간주되어 `NotApplicable` 판정이 날 수 있습니다.
> - 값이 `"null"`이면 CORS 대상이며, `allow_null_origin: true`일 때만 허용됩니다.
> - 특히 `credentials: true`와 `null` 조합은 보안상 매우 위험할 수 있으니 지양하세요.

---

### 사설망 접근 (PNA)

#### 설명

`allow_private_network` 옵션은 Private Network Access를 허용합니다. 공개 웹사이트에서 로컬 네트워크(localhost, 192.168.x.x 등)의 리소스에 접근할 때 필요합니다. 이 옵션을 사용하려면 `credentials: true`와 특정 Origin 설정이 필수입니다.

> **브라우저 호환성**: Private Network Access는 현재 실험적 기능으로, Chromium 기반 브라우저(Chrome 94+)에서 지원됩니다. Firefox와 Safari는 아직 지원하지 않습니다. 표준화가 진행 중이므로 프로덕션 환경에서는 브라우저 호환성을 확인하세요.

> 동작 범위 주의: `Access-Control-Allow-Private-Network: true`는 주로 Preflight(OPTIONS) 교환에서 의미를 갖습니다. 실제 동작은 브라우저/버전에 따라 달라질 수 있으니 대상 브라우저에서 반드시 검증하세요.

#### 사용 예제

```rust
let options = CorsOptions {
    origin: Origin::exact("https://app.example.com"),
    credentials: true,
    allow_private_network: true,
    ..Default::default()
};
```

**결과:**
```
Access-Control-Allow-Origin: https://app.example.com
Access-Control-Allow-Credentials: true
Access-Control-Allow-Private-Network: true
Vary: Origin
```

---

### 리소스 타이밍

#### 설명

`timing_allow_origin` 옵션은 `Timing-Allow-Origin` 헤더를 설정하여, 특정 Origin이 리소스 타이밍 정보에 접근할 수 있도록 허용합니다. 성능 분석 도구나 모니터링 서비스에서 유용합니다.

> **제약사항**: `credentials: true`인 경우 `TimingAllowOrigin::any()` 와일드카드를 사용할 수 없습니다.

#### 사용 예제

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

**결과:**
```
Timing-Allow-Origin: https://analytics.example.com
```

---

### 설정 검증 오류

#### 설명

`Cors::new()`는 잘못된 설정 조합이 있을 경우 `ValidationError`를 반환합니다. 주요 검증 오류는 다음과 같습니다:

| 오류 | 설명 |
|------|------|
| `CredentialsRequireSpecificOrigin` | `credentials: true`일 때 `Origin::Any` 사용 불가 |
| `AllowedHeadersAnyNotAllowedWithCredentials` | `credentials: true`일 때 `AllowedHeaders::any()` 사용 불가 |
| `AllowedHeadersListCannotContainWildcard` | 허용 헤더 목록에 `"*"` 포함 불가 (대신 `AllowedHeaders::any()` 사용) |
| `ExposeHeadersWildcardRequiresCredentialsDisabled` | 노출 헤더에 `"*"`를 쓰려면 `credentials: false` 필요 |
| `ExposeHeadersWildcardCannotBeCombined` | 노출 헤더에 `"*"`와 다른 헤더를 함께 지정 불가 |
| `InvalidMaxAge` | `max_age`가 유효한 숫자 형식이 아님 |
| `PrivateNetworkRequiresCredentials` | `allow_private_network: true`일 때 `credentials: true` 필수 |
| `PrivateNetworkRequiresSpecificOrigin` | `allow_private_network: true`일 때 `Origin::Any` 사용 불가 |
| `TimingAllowOriginWildcardNotAllowedWithCredentials` | `credentials: true`일 때 `TimingAllowOrigin::any()` 사용 불가 |
| `AllowedMethodsCannotContainWildcard` | 허용 메서드 목록에 `"*"` 포함 불가 |
| `AllowedMethodsListContainsInvalidToken` | 허용 메서드가 유효한 HTTP 메서드 토큰이 아님 |
| `AllowedHeadersListContainsInvalidToken` | 허용 헤더가 유효한 HTTP 헤더 이름이 아님 |
| `ExposeHeadersListContainsInvalidToken` | 노출 헤더가 유효한 HTTP 헤더 이름이 아님 |

#### 런타임 오류

`Cors::check()`는 `CorsError`를 반환할 수 있습니다:

- **`InvalidOriginAnyWithCredentials`**: `Origin::custom` 콜백이 `credentials: true` 상황에서 `OriginDecision::Any`를 반환한 경우 (CORS 표준 위반)

---

## 판정 결과 처리

### `RequestContext` 준비

CORS 판정을 위해 HTTP 요청 정보를 `RequestContext`로 변환해야 합니다.

#### 필드 설명

| 필드 | HTTP 헤더 | 설명 |
|------|-----------|------|
| `method` | 요청 메서드 | `"GET"`, `"POST"`, `"OPTIONS"` 등 실제 HTTP 메서드 |
| `origin` | `Origin` | 요청의 출처. 없으면 빈 문자열 `""` |
| `access_control_request_method` | `Access-Control-Request-Method` | Preflight 요청에서 실제로 사용할 메서드. 없으면 `""` |
| `access_control_request_headers` | `Access-Control-Request-Headers` | Preflight 요청에서 사용할 헤더 목록 (쉼표 구분). 없으면 `""` |
| `access_control_request_private_network` | `Access-Control-Request-Private-Network` | PNA 헤더 존재 여부 (`true`/`false`) |

#### 예제

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

> **프레임워크별 매핑 팁**: HTTP 헤더명은 대소문자를 구분하지 않으므로, 대부분의 프레임워크에서 `headers.get("origin")`처럼 소문자로 접근합니다. Axum/Actix 모두 소문자 헤더명을 권장합니다.

### `CorsDecision` 결과 이해 및 처리

`cors.check()`는 4가지 판정 결과를 반환합니다:

**`PreflightAccepted` - Preflight 요청 승인**

OPTIONS 요청이 성공한 경우입니다. 반환된 헤더를 204 응답에 추가하세요.

```rust
use bunner_cors_rs::CorsDecision;

match cors.check(&context)? {
    CorsDecision::PreflightAccepted { headers } => {
        // 예: Axum에서
        let mut response = Response::builder().status(204).body(().into()).unwrap();
        for (name, value) in headers {
            response.headers_mut().insert(
                name.parse().unwrap(),
                value.parse().unwrap(),
            );
        }
        // return response;
    }
    _ => {}
}
```

**`PreflightRejected` - Preflight 요청 거부**

Origin, 메서드, 또는 헤더가 허용되지 않은 경우입니다. 보안을 위해 **HTTP 403 Forbidden** 응답을 반환하는 것이 좋습니다. 거부 이유는 디버깅 목적으로 확인할 수 있습니다.

```rust
CorsDecision::PreflightRejected(rejection) => {
    // rejection.reason으로 상세 원인 파악:
    // - OriginNotAllowed
    // - MethodNotAllowed { requested_method }
    // - HeadersNotAllowed { requested_headers }
    // - MissingAccessControlRequestMethod
    
    // rejection.headers도 있으므로 필요시 Vary 등을 응답에 추가 가능
    eprintln!("CORS Preflight Rejected: {:?}", rejection.reason);

    // HTTP 403 Forbidden 응답을 반환합니다.
    // 예: return Response::builder().status(403).body(().into()).unwrap();
}
```

> 관측성 팁(권장): 운영 환경에서는 구조화 로그를 남겨 원인 분석을 쉽게 하세요.
>
> ```rust
> use tracing::warn;
> warn!(
>     event = "cors_preflight_rejected",
>     reason = ?rejection.reason,
>     requested_method = %context.access_control_request_method,
>     requested_headers = %context.access_control_request_headers,
>     origin = %context.origin,
>     "preflight rejected",
> );
> ```

**`SimpleAccepted` - Simple 요청 승인**

일반적인 GET/POST 등의 요청입니다. 반환된 헤더를 실제 응답에 추가하세요.

```rust
CorsDecision::SimpleAccepted { headers } => {
    // 예: Actix에서
    let mut response = HttpResponse::Ok();
    for (name, value) in headers {
        response.append_header((name, value));
    }
    // return response.body(your_content);
}
```

**`NotApplicable` - CORS 대상 아님**

Origin 헤더가 없거나 CORS가 필요하지 않은 요청입니다. CORS 헤더 없이 정상 처리하세요.

```rust
CorsDecision::NotApplicable => {
    // Process normally without CORS headers
}
```

## 프레임워크 통합 예시

> **💡 중요**: `Cors` 인스턴스는 비싸므로 애플리케이션 시작 시 한 번만 생성하고, `Arc` 또는 프레임워크가 제공하는 상태 공유 메커니즘(예: `axum::Extension`, `actix_web::web::Data`)을 통해 핸들러에서 공유해야 합니다.

> 체인/미들웨어 순서 팁: Preflight(OPTIONS)는 라우팅 초기에 빠르게 처리하고, CORS 판정을 인증·비즈니스 로직보다 앞단에서 수행하면 불필요한 리소스 사용을 줄일 수 있습니다.

### Axum

```rust
use axum::{
    extract::Extension,
    http::{HeaderMap, Method},
    response::Response,
};
use bunner_cors_rs::{Cors, CorsDecision, RequestContext};
use std::sync::Arc;

// main 함수에서 `Extension` 레이어로 `Arc<Cors>`를 추가했다고 가정합니다.
// let cors = Arc::new(Cors::new(...).unwrap());
// let app = Router::new().route("/", get(handler)).layer(Extension(cors));

async fn handler(
    Extension(cors): Extension<Arc<Cors>>,
    method: Method,
    headers: HeaderMap,
) -> Response {
    let context = RequestContext {
        method: method.as_str(),
        origin: headers.get("origin").and_then(|v| v.to_str().ok()).unwrap_or(""),
        access_control_request_method: headers
            .get("access-control-request-method")
            .and_then(|v| v.to_str().ok())
            .unwrap_or(""),
        access_control_request_headers: headers
            .get("access-control-request-headers")
            .and_then(|v| v.to_str().ok())
            .unwrap_or(""),
        access_control_request_private_network: headers
            .contains_key("access-control-request-private-network"),
    };

    match cors.check(&context) {
        Ok(CorsDecision::PreflightAccepted { headers }) => {
            let mut response = Response::builder().status(204).body(().into()).unwrap();
            for (name, value) in headers {
                response
                    .headers_mut()
                    .insert(name.parse().unwrap(), value.parse().unwrap());
            }
            response
        }
        Ok(CorsDecision::SimpleAccepted { headers }) => {
            // 실제 응답에 CORS 헤더를 추가합니다.
            // 예: let mut response = Response::new(...);
            // for (name, value) in headers { ... }
            Response::new(().into())
        }
        _ => Response::builder().status(403).body(().into()).unwrap(),
    }
}
```

### Actix-web

```rust
use actix_web::{web, HttpRequest, HttpResponse};
use bunner_cors_rs::{Cors, CorsDecision, RequestContext};

// App 빌드 시 `web::Data`로 `Cors` 인스턴스를 등록했다고 가정합니다.
// let cors = web::Data::new(Cors::new(...).unwrap());
// App::new().app_data(cors.clone()).route("/", web::to(handler))

async fn handler(cors: web::Data<Cors>, req: HttpRequest) -> HttpResponse {
    let headers = req.headers();
    let context = RequestContext {
        method: req.method().as_str(),
        origin: headers.get("origin").and_then(|v| v.to_str().ok()).unwrap_or(""),
        access_control_request_method: headers
            .get("access-control-request-method")
            .and_then(|v| v.to_str().ok())
            .unwrap_or(""),
        access_control_request_headers: headers
            .get("access-control-request-headers")
            .and_then(|v| v.to_str().ok())
            .unwrap_or(""),
        access_control_request_private_network: headers
            .contains_key("access-control-request-private-network"),
    };

    match cors.check(&context) {
        Ok(CorsDecision::PreflightAccepted { headers }) => {
            let mut response = HttpResponse::NoContent();
            for (name, value) in headers {
                response.append_header((name, value));
            }
            response.finish()
        }
        Ok(CorsDecision::SimpleAccepted { headers }) => {
            let mut response = HttpResponse::Ok();
            for (name, value) in headers {
                response.append_header((name, value));
            }
            // 실제 응답 컨텐츠와 함께 반환
            response.body("Hello from Actix!")
        }
        _ => HttpResponse::Forbidden().finish(),
    }
}
```

## 성능 및 안전성

### 성능 최적화

- **메모리 풀링**: 내부적으로 버퍼를 재사용하여 할당 오버헤드를 최소화합니다
- **정규식 캐싱**: 컴파일된 정규식 패턴을 캐시하여 반복 사용 시 성능을 향상시킵니다
- **스마트 매칭**: 작은 목록은 선형 탐색, 큰 목록은 해시맵을 사용하여 최적화합니다

### 안전성 보장

- **컴파일 타임 검증**: 잘못된 설정 조합(`Origin::Any` + `credentials: true` 등)을 생성 시점에 차단합니다
- **표준 준수**: WHATWG Fetch 명세를 정확히 따라 예상치 못한 동작을 방지합니다
- **Thread-safe**: 모든 타입이 `Send + Sync`를 구현하여 동시성 환경에서 안전합니다

> **동시성 주의사항**: `Cors` 인스턴스는 불변(immutable)이므로 여러 스레드/태스크에서 공유해도 안전합니다. 단, 설정을 변경하려면 새로운 `Cors` 인스턴스를 생성해야 합니다.

### 보안 체크리스트

프로덕션 환경에 배포하기 전에 다음 사항을 확인하세요:

- ✅ `credentials: true` + `Origin::Any` 조합 금지 (라이브러리가 차단)
- ✅ 패턴 매칭은 최소화하고 앵커(`^`, `$`)와 이스케이프 적용
- ✅ `allow_null_origin`은 신뢰 가능한 컨텍스트에서만 사용
- ✅ PNA는 대상 브라우저에서 실제 동작 검증 필요
- ✅ Preflight 거부 시 일관된 403 정책 유지, 로깅 활성화
- ✅ 정규식 패턴의 성능 영향 고려 (캐싱되지만 초기 컴파일 비용 존재)
- ✅ `max_age` 값은 브라우저 정책 고려하여 합리적으로 설정

### 테스트 실행

이 라이브러리는 유닛 테스트, 통합 테스트, property-based 테스트, snapshot 테스트를 포함합니다:

```bash
# 일반 테스트 실행
cargo test

# nextest 사용 (더 빠름)
cargo nextest run

# 커버리지 확인
cargo llvm-cov --html

# 벤치마크 실행
cargo bench
```

## FAQ 및 문제 해결

**Q: `Origin::Any`와 `credentials: true`를 함께 사용하고 싶어요**

A: CORS 표준상 불가능합니다. 자격증명을 허용하려면 반드시 특정 Origin을 명시해야 합니다.

**Q: 모든 헤더를 허용하고 싶어요**

A: `AllowedHeaders::any()`를 사용하세요. 단, `credentials: true`와는 함께 사용할 수 없습니다.

```rust
allowed_headers: AllowedHeaders::any(),
```

**Q: 하위 도메인 전체를 허용하고 싶어요**

A: 정규식 패턴을 사용하세요:

```rust
OriginMatcher::pattern_str(r"https://.*\.example\.com").unwrap()
```

**Q: Preflight 요청이 계속 실패해요**

A: `PreflightRejected`의 `reason` 필드를 확인하세요. 거부된 메서드나 헤더를 알 수 있습니다.

**Q: 비동기 런타임에서 사용해도 되나요?**

A: 네, 이 라이브러리는 동기 API지만 CPU 연산 위주이므로 Tokio 등에서 바로 사용 가능합니다.

## API 문서

전체 API 문서는 [docs.rs](https://docs.rs/bunner_cors_rs)에서 확인하세요.

### 주요 타입

- **`Cors`**: CORS 판정을 수행하는 메인 구조체
- **`CorsOptions`**: CORS 정책 설정
- **`Origin`**: Origin 매칭 전략 (Any, Exact, List, Predicate, Custom)
- **`CorsDecision`**: 판정 결과 (PreflightAccepted, SimpleAccepted, NotApplicable, PreflightRejected)
- **`RequestContext`**: HTTP 요청 정보를 담는 컨텍스트
- **`AllowedMethods`**: 허용 메서드 목록
- **`AllowedHeaders`**: 허용 헤더 목록
- **`TimingAllowOrigin`**: 타이밍 정보 노출 설정

### 버전 및 안정성

- 현재 메이저 버전 0.x 단계에서는 공개 API가 변경될 수 있습니다.
- [Semantic Versioning](https://semver.org/) 규칙을 따르며, 변경 사항은 릴리스 노트에 명시합니다.
- v1.0.0 이후에는 API 안정성이 보장됩니다.

## 기여하기

버그 리포트, 기능 제안, Pull Request를 환영합니다!

1. 이슈를 먼저 등록해 주세요
2. Fork 후 브랜치 생성
3. 변경사항 커밋 (테스트 포함)
4. Pull Request 제출

자세한 내용은 [CONTRIBUTING.md](CONTRIBUTING.md)를 참조하세요.

## 라이선스

MIT License. 자세한 내용은 [LICENSE.md](LICENSE.md) 파일을 참조하세요.

---

## 관련 프로젝트

- [tower-http](https://github.com/tower-rs/tower-http) - Tower/Axum CORS 미들웨어
- [actix-cors](https://github.com/actix/actix-extras) - Actix-web CORS 미들웨어
- [rocket_cors](https://github.com/lawliet89/rocket_cors) - Rocket CORS 미들웨어

이 라이브러리는 위 미들웨어들과 달리 프레임워크 독립적인 코어 로직만 제공합니다.