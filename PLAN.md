# 코드베이스 검토 결과 및 개선 계획

## 1. 오픈소스 품질 기준 검토

### 문서화 부족 ❌
- **README.md 파일이 비어있음**: 프로젝트 소개, 사용법, 예제, 설치 방법 등 필수 문서 누락
- **API 문서 부족**: public API에 대한 rustdoc 주석이 거의 없음
- **CHANGELOG.md 없음**: 버전별 변경사항 추적 불가
- **CONTRIBUTING.md 없음**: 기여 가이드라인 부재
- **예제 코드 부족**: examples/ 디렉토리 없음, 실제 사용 사례 제공 필요
  - **작업 계획**: `README.md`에 소개·사용법·설치 절차를 작성하고, 핵심 public API에 rustdoc을 추가하며, CHANGELOG/CONTRIBUTING 템플릿과 최소 예제 코드를 마련한다.

### 메타데이터 불완전 ⚠️
- **Cargo.toml**: description, repository, documentation, homepage, keywords, categories 등 메타데이터 누락
  - **작업 계획**: `Cargo.toml`에 description, repository, documentation, homepage, keywords, categories 항목을 채워 공개 메타데이터를 완성한다.

---

## 2. CORS 국제 표준 준수 검토

### 누락된 CORS 기능 ❌
- **Vary 헤더 자동화**: 수동 관리 중이나 일부 케이스에서 누락 가능성
- **CORS 에러 타입 제한적**: `CorsError` enum이 한 가지 경우만 처리 (Any + Credentials)
  - **작업 계획**: Vary 헤더 추가 경로를 점검해 공통 헬퍼 또는 자동 삽입 로직을 도입하고, `CorsError`에 추가 variant와 상세 컨텍스트를 정의한다.

### CORS 표준 준수 확인 필요 ⚠️
- **Private Network Access 지원**: 구현되어 있으나 최신 표준 변경사항 재확인 필요
- **null origin 처리**: 현재 구현되어 있으나 표준 명세와 정확히 일치하는지 검증 필요
- **Origin 대소문자 처리**: ASCII와 Unicode를 다르게 처리하는데, 표준 명세 확인 필요
  - **작업 계획**: 최신 Fetch/W3C 명세를 재검토하고, PNA·null origin·대소문자 처리에 대한 회귀 테스트와 문서 근거를 정리한다.

### 표준 명세 문서화 부족 ⚠️
- 어떤 CORS 표준 버전을 따르는지 명시 없음
- W3C Fetch Standard, WHATWG Fetch 등 참조 명세 문서화 필요
  - **작업 계획**: README와 API 문서에 준거 표준 문서 링크와 지원 범위를 명시한다.

---

## 3. 클린 코드 원칙 준수 검토

### 네이밍 일관성 문제 ⚠️
- **약어 혼재**: `acrm`, `acrh` (테스트 헬퍼) vs 풀네임 사용이 혼재됨
- **모듈명 vs 타입명**: `allowed_headers` 모듈 내 `AllowedHeaders` enum과 `AllowedHeaderList` struct 명확성 개선 필요
  - **작업 계획**: 테스트 헬퍼 약어를 풀네임으로 통일하고, `AllowedHeaders` 관련 타입명을 명확한 도메인 용어로 리네이밍한다.

### 단일 책임 원칙 위반 가능성 ⚠️
- **HeaderBuilder**: 9개의 서로 다른 헤더 빌드 메서드 보유, 책임 분산 고려 필요
- **CorsOptions::validate()**: 250줄 이상의 복잡한 검증 로직, 검증 규칙 분리 고려
  - **작업 계획**: 헤더 빌드를 책임별 모듈로 분리하고, `CorsOptions::validate`를 검증 단계별 헬퍼로 나눠 가독성과 테스트 용이성을 높인다.

### 코드 중복 ⚠️
- **테스트 헬퍼 함수 중복**: `src/*_test.rs`와 `tests/common/builders.rs`에 유사한 request 빌더 존재
- **정규화 로직 분산**: `normalize_lower`, `equals_ignore_case`, `lowercase_unicode_*` 함수들이 util.rs에 분산
  - **작업 계획**: 테스트 빌더를 공용 헬퍼로 통합하고, 문자열 정규화 유틸을 단일 모듈로 재구성해 재사용성을 높인다.

### Pool 패턴 과도 사용 주의 ⚠️
- **3곳에서 thread_local pool 사용**: headers.rs, normalized_request.rs, allowed_headers.rs
- 조기 최적화 가능성, 프로파일링 결과 기반 최적화인지 확인 필요
- pool 관리 로직이 각각 중복 구현됨
  - **작업 계획**: thread-local 풀 사용 지점을 전수 조사하고, 공통 추상화 혹은 단일 관리자로 통합한 뒤 프로파일링 결과에 따라 유지 여부를 결정한다.

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
  - **작업 계획**: 주요 상수에 근거 주석을 추가하고, 필요 시 설정 값으로 노출하거나 환경 설정을 통해 오버라이드할 수 있도록 리팩터링한다.

### 에러 처리 개선 필요 ⚠️
- **CorsError enum**: 단일 variant만 존재, 더 다양한 에러 케이스 표현 필요
- **Result 타입 일관성**: 일부 public API는 Result 반환, 일부는 panic 가능성
  - **작업 계획**: `CorsError`를 다양한 오류 케이스로 확장하고, panic 가능 지점을 Result 기반 오류 처리로 통일한다.

### 테스트 커버리지 관련 ⚠️
- unit test는 매우 잘 작성되어 있으나, integration test의 edge case 커버리지 확인 필요
- property-based test가 있으나 범위가 제한적
  - **작업 계획**: property 기반 경계값을 확장하고, 통합 테스트 필요성은 문서로 별도 계층(미들웨어/서버)에서 수행하도록 안내한다.

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
  - **작업 계획**: `lib.rs` re-export 목록을 검토해 `OriginMatcher`와 콜백 타입 별칭을 비공개로 전환하거나 #[doc(hidden)] 처리한다.

### 빌더 패턴 불완전 ⚠️
- `CorsOptions`는 struct with pub fields 방식 사용
- `CorsOptionsBuilder` 패턴 제공하여 ergonomics 개선 고려
  - **작업 계획**: `CorsOptions`의 pub 필드를 단계별 빌더/스마트 생성자로 대체하고, 마이그레이션 가이드를 문서화한다.

### 타입 안전성 개선 가능 ⚠️
- `RequestContext` 필드가 모두 `&'a str`로 되어있어 컴파일 타임 검증 부족
- newtype 패턴 적용 고려 (예: `Origin(&'a str)`, `Method(&'a str)`)
  - **작업 계획**: RequestContext에 newtype을 도입해 생성 시 입력을 검증하고, 기존 호출부를 점진적으로 마이그레이션한다.

### 비동기 지원 고려 ⚠️
- 현재 동기 API만 제공
- 비동기 런타임(tokio, async-std) 통합 시 어려움
- 향후 async 지원 계획 고려 필요
  - **작업 계획**: README/API 문서에 동기 API 전제와 tokio 등에서의 사용 예시(간단한 async 래퍼 포함)를 추가해 사용 가이드를 제공한다.

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
  - **작업 계획**: 반복 생성되는 헤더 문자열을 캐시하거나 `Cow<'static, str>` 반환으로 변경해 할당을 줄인다.

### Regex 캐싱 전략 재검토 필요 ⚠️
- global static `REGEX_CACHE`와 RwLock 사용
- 높은 동시성 환경에서 병목 가능성
- DashMap 등 concurrent data structure 고려
  - **작업 계획**: regex 캐시를 concurrent 자료구조(DashMap 등)로 교체하거나 샤딩 전략을 도입해 락 경합을 줄인다.

---

## 6. 보안 검토

### 입력 검증 강화 필요 ⚠️
- origin 길이 제한: `MAX_ORIGIN_LENGTH = 4_096` 존재하나 실제 공격 시나리오 기반 검증 필요
- regex pattern DoS 방어: timeout 있으나 `PATTERN_COMPILE_BUDGET = 100ms`의 적절성 검증 필요
  - **작업 계획**: 길이·타임아웃 상수를 위협 모델과 운영 데이터에 맞춰 검토하고, 필요 시 값을 조정하거나 구성 가능하도록 문서화한다.
