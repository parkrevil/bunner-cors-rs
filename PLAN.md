# 코드베이스 검토 결과 및 개선 계획

## 1. 오픈소스 품질 기준 검토

### 문서화 부족 ❌
- **README.md 파일이 비어있음**: 프로젝트 소개, 사용법, 예제, 설치 방법 등 필수 문서 누락
- **API 문서 부족**: public API에 대한 rustdoc 주석이 거의 없음
- **CHANGELOG.md 없음**: 버전별 변경사항 추적 불가
- **CONTRIBUTING.md 없음**: 기여 가이드라인 부재
- **예제 코드 부족**: examples/ 디렉토리 없음, 실제 사용 사례 제공 필요

### 메타데이터 불완전 ⚠️
- **Cargo.toml**: description, repository, documentation, homepage, keywords, categories 등 메타데이터 누락

---

## 2. CORS 국제 표준 준수 검토

### 누락된 CORS 기능 ❌
- **preflight 캐싱 지원 부족**: `Access-Control-Max-Age` 헤더는 있으나 실제 캐싱 메커니즘 없음
- **Vary 헤더 자동화**: 수동 관리 중이나 일부 케이스에서 누락 가능성
- **CORS 에러 타입 제한적**: `CorsError` enum이 한 가지 경우만 처리 (Any + Credentials)

### CORS 표준 준수 확인 필요 ⚠️
- **Private Network Access 지원**: 구현되어 있으나 최신 표준 변경사항 재확인 필요
- **null origin 처리**: 현재 구현되어 있으나 표준 명세와 정확히 일치하는지 검증 필요
- **Origin 대소문자 처리**: ASCII와 Unicode를 다르게 처리하는데, 표준 명세 확인 필요

### 표준 명세 문서화 부족 ⚠️
- 어떤 CORS 표준 버전을 따르는지 명시 없음
- W3C Fetch Standard, WHATWG Fetch 등 참조 명세 문서화 필요

---

## 3. 클린 코드 원칙 준수 검토

### 네이밍 일관성 문제 ⚠️
- **약어 혼재**: `acrm`, `acrh` (테스트 헬퍼) vs 풀네임 사용이 혼재됨
- **모듈명 vs 타입명**: `allowed_headers` 모듈 내 `AllowedHeaders` enum과 `AllowedHeaderList` struct 명확성 개선 필요

### 단일 책임 원칙 위반 가능성 ⚠️
- **HeaderBuilder**: 9개의 서로 다른 헤더 빌드 메서드 보유, 책임 분산 고려 필요
- **CorsOptions::validate()**: 250줄 이상의 복잡한 검증 로직, 검증 규칙 분리 고려

### 코드 중복 ⚠️
- **테스트 헬퍼 함수 중복**: `src/*_test.rs`와 `tests/common/builders.rs`에 유사한 request 빌더 존재
- **정규화 로직 분산**: `normalize_lower`, `equals_ignore_case`, `lowercase_unicode_*` 함수들이 util.rs에 분산

### Pool 패턴 과도 사용 주의 ⚠️
- **3곳에서 thread_local pool 사용**: headers.rs, normalized_request.rs, allowed_headers.rs
- 조기 최적화 가능성, 프로파일링 결과 기반 최적화인지 확인 필요
- pool 관리 로직이 각각 중복 구현됨

### 매직 넘버/상수 ⚠️
```rust
const SMALL_LIST_LINEAR_SCAN_THRESHOLD: usize = 4;
const HEADER_BUFFER_POOL_LIMIT: usize = 64;
const NORMALIZATION_BUFFER_POOL_LIMIT: usize = 16;
const MAX_PATTERN_LENGTH: usize = 50_000;
const MAX_ORIGIN_LENGTH: usize = 4_096;
```
- 이러한 상수들의 선택 근거 문서화 필요
- 설정 가능하게 할지 검토 필요

### 에러 처리 개선 필요 ⚠️
- **CorsError enum**: 단일 variant만 존재, 더 다양한 에러 케이스 표현 필요
- **Result 타입 일관성**: 일부 public API는 Result 반환, 일부는 panic 가능성

### 테스트 커버리지 관련 ⚠️
- unit test는 매우 잘 작성되어 있으나, integration test의 edge case 커버리지 확인 필요
- property-based test가 있으나 범위가 제한적

---

## 4. 구조 및 패턴 개선사항

### Public API 표면적 과다 ⚠️
```rust
pub use origin::{
    Origin, OriginCallbackFn, OriginDecision, OriginMatcher, OriginPredicateFn, PatternError,
};
```
- `OriginCallbackFn`, `OriginPredicateFn` 타입 별칭이 public으로 노출됨
- `OriginMatcher`가 public이나 사용자가 직접 생성할 일이 없음
- API 표면적 최소화 필요

### 빌더 패턴 불완전 ⚠️
- `CorsOptions`는 struct with pub fields 방식 사용
- `CorsOptionsBuilder` 패턴 제공하여 ergonomics 개선 고려

### 타입 안전성 개선 가능 ⚠️
- `RequestContext` 필드가 모두 `&'a str`로 되어있어 컴파일 타임 검증 부족
- newtype 패턴 적용 고려 (예: `Origin(&'a str)`, `Method(&'a str)`)

### 비동기 지원 고려 ⚠️
- 현재 동기 API만 제공
- 비동기 런타임(tokio, async-std) 통합 시 어려움
- 향후 async 지원 계획 고려 필요

---

## 5. 성능 관련 검토

### 불필요한 allocation 가능성 ⚠️
```rust
pub fn header_value(&self) -> Option<String> {
    if self.0.is_empty() {
        None
    } else {
        Some(self.0.join(","))
    }
}
```
- 매번 새 String 할당, `Cow<'static, str>` 또는 캐싱 고려

### Regex 캐싱 전략 재검토 필요 ⚠️
- global static `REGEX_CACHE`와 RwLock 사용
- 높은 동시성 환경에서 병목 가능성
- DashMap 등 concurrent data structure 고려

---

## 6. 보안 검토

### 입력 검증 강화 필요 ⚠️
- origin 길이 제한: `MAX_ORIGIN_LENGTH = 4_096` 존재하나 실제 공격 시나리오 기반 검증 필요
- regex pattern DoS 방어: timeout 있으나 `PATTERN_COMPILE_BUDGET = 100ms`의 적절성 검증 필요

### Unsafe 코드 검토 ✅
```rust
unsafe {
    owned.as_mut_vec()[index..].make_ascii_lowercase();
}
```
- normalized_request.rs의 unsafe 블록 존재
- 경계 검사가 제대로 되어있으나 주석으로 SAFETY 명시 필요

---

## 개선 우선순위

### P0 (Critical) - 즉시 수정 필요
1. **README.md 작성**: 프로젝트 설명, 사용법, 예제
2. **Cargo.toml edition 수정**: 2024 → 2021
3. **Public API rustdoc 추가**: 모든 public 함수/타입에 문서 주석
4. **Cargo.toml 메타데이터 추가**: description, repository 등

### P1 (High) - 릴리스 전 필수
1. **CHANGELOG.md 작성**: 버전 관리 시작
2. **API 표면적 최소화**: 불필요한 public export 제거
3. **CORS 표준 준수 검증**: 참조 명세 문서화 및 테스트 강화
4. **CorsOptionsBuilder 제공**: 사용성 개선

### P2 (Medium) - 다음 마이너 버전
1. **에러 타입 확장**: CorsError에 더 다양한 variant 추가
2. **예제 코드 추가**: examples/ 디렉토리에 실제 사용 사례
3. **CONTRIBUTING.md 작성**: 기여 가이드라인
4. **HeaderBuilder 리팩토링**: 책임 분산 검토

### P3 (Low) - 장기 개선
1. **비동기 지원 검토**: async API 제공 계획
2. **성능 벤치마크 확장**: 더 많은 시나리오 추가
3. **Pool 최적화 재검토**: 프로파일링 기반 최적화
4. **타입 안전성 강화**: newtype 패턴 적용 검토
