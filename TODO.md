# TODO - bunner-cors-rs Production Ready Checklist

## 🔴 Critical Priority (1주일 내 완료 필요)

### 1. Cargo.toml 메타데이터 수정
- [ ] 패키지명 `bunner_cors_rs` → `bunner-cors-rs` 변경 검토 (crates.io 컨벤션)
- [ ] 필수 메타데이터 추가:
  ```toml
  authors = ["Junhyung Park <email@example.com>"]
  description = "Fast, predictable CORS policy engine for Rust applications"
  documentation = "https://docs.rs/bunner-cors-rs"
  homepage = "https://github.com/parkrevil/bunner-cors-rs"
  repository = "https://github.com/parkrevil/bunner-cors-rs"
  readme = "README.md"
  keywords = ["cors", "http", "web", "security", "edge"]
  categories = ["web-programming", "network-programming"]
  rust-version = "1.70"
  ```
- [ ] docs.rs 설정 추가:
  ```toml
  [package.metadata.docs.rs]
  all-features = true
  rustdoc-args = ["--cfg", "docsrs"]
  ```

### 2. CHANGELOG.md 생성
- [ ] SemVer 가이드라인에 따른 CHANGELOG 작성
- [ ] 현재 버전 (0.1.0) 변경사항 문서화
- [ ] Unreleased 섹션 추가
- [ ] Keep a Changelog 형식 준수

### 3. API 문서화 (50% 이상)
- [ ] `src/cors.rs` - Cors 구조체 및 메서드 문서화
- [ ] `src/origin.rs` - Origin, OriginDecision, OriginMatcher 문서화
- [ ] `src/options.rs` - CorsOptions, ValidationError 문서화
- [ ] `src/allowed_headers.rs` - AllowedHeaders 문서화
- [ ] `src/allowed_methods.rs` - AllowedMethods 문서화
- [ ] `src/result.rs` - CorsDecision, CorsError 문서화
- [ ] 각 public 함수에 예제 코드 추가
- [ ] WHATWG Fetch Standard 참조 링크 추가

### 4. README.md 개선
- [ ] Features 섹션 추가
- [ ] 사용 예제 5개 이상 추가:
  - [ ] Basic Setup
  - [ ] Exact Origin Matching
  - [ ] Regex Pattern Matching
  - [ ] Custom Origin Logic
  - [ ] Error Handling
- [ ] Comparison 테이블 (tower-http, actix-cors 비교)
- [ ] Installation 가이드
- [ ] Quick Start 가이드

### 5. GitHub Actions CI/CD 구성
- [ ] `.github/workflows/ci.yml` 생성
- [ ] 자동 테스트 (cargo test)
- [ ] 자동 lint (cargo clippy)
- [ ] 자동 format 체크 (cargo fmt --check)
- [ ] 여러 Rust 버전 테스트 (MSRV, stable, nightly)
- [ ] 플랫폼별 테스트 (Linux, macOS, Windows)

---

## 🟡 High Priority (2주일 내 완료)

### 6. CONTRIBUTING.md 생성
- [ ] Development Setup 가이드
- [ ] Running Tests 절차
- [ ] Code Style 가이드라인
- [ ] Pull Request 프로세스
- [ ] 커밋 메시지 컨벤션
- [ ] 이슈 보고 가이드

### 7. SECURITY.md 생성
- [ ] 보안 이슈 보고 방법
- [ ] 지원되는 버전 명시
- [ ] 보안 정책 설명
- [ ] 책임 있는 공개 절차

### 9. CORS 표준 준수 강화
- [ ] CORS-safelisted headers 자동 허용 검토
  - Accept
  - Accept-Language
  - Content-Language
  - Content-Type (특정 값만)
- [ ] CORS-safelisted methods 자동 허용 검토
  - GET, HEAD, POST
- [ ] Preflight max-age 권장값 문서화 (1-86400초)

---

## 🟢 Medium Priority (1개월 내 완료)

### 11. API 문서화 완성 (100%)
- [ ] 모든 public 타입에 문서 추가
- [ ] 모든 public 함수에 문서 추가
- [ ] 각 모듈에 module-level 문서 추가
- [ ] 복잡한 개념에 대한 가이드 추가
- [ ] 표준 참조 링크 추가
  - [ ] WHATWG Fetch Standard
  - [ ] W3C CORS Recommendation
  - [ ] Private Network Access Draft

### 13. 성능 최적화
- [ ] 헤더 할당 최적화 (pool 패턴 검토)
- [ ] 문자열 비교 최적화
- [ ] 불필요한 clone() 제거
- [ ] Zero-copy 처리 확대
- [ ] 프로파일링 및 핫스팟 최적화

### 14. 통합 예제
- [ ] `examples/` 디렉토리 생성
- [ ] Axum 통합 예제
- [ ] Actix-web 통합 예제
- [ ] Rocket 통합 예제
- [ ] Hyper 통합 예제
- [ ] Standalone 사용 예제

### 15. 코드 커버리지
- [ ] codecov 또는 coveralls 통합
- [ ] 커버리지 배지 추가
- [ ] 90% 이상 커버리지 목표
- [ ] 누락된 테스트 케이스 추가

---

## 🔵 Low Priority (Future Enhancements)

### 17. Edge Cases 처리
  - [ ] 비정상적인 헤더 값 처리
  - [ ] IDN (Internationalized Domain Names) 지원

### 18. 문서 추가
- [ ] Architecture Decision Records (ADR)
- [ ] Design Philosophy 문서
- [ ] Migration 가이드 (다른 라이브러리에서)
- [ ] FAQ 섹션
- [ ] Troubleshooting 가이드

### 20. 안정성
- [ ] Fuzzing 테스트 추가
- [ ] Mutation 테스트
  - [ ] Property-based 테스트 확대
  - [ ] Integration 테스트 확대
  - [x] Unicode/초과 길이 Origin 회귀 테스트 추가
- [ ] Stress 테스트

---

## 📋 Pre-Release Checklist (v1.0.0 전)

### 필수 항목
- [ ] 모든 Critical Priority 항목 완료
- [ ] 모든 High Priority 항목 완료
- [ ] API 안정성 보장
- [ ] Breaking changes 문서화
- [ ] 마이그레이션 가이드 작성
- [ ] 보안 감사 완료
- [ ] 성능 벤치마크 공개
- [ ] 커뮤니티 피드백 수집 및 반영

### 권장 항목
- [ ] 최소 3개 프로덕션 배포 사례
- [ ] 외부 코드 리뷰
- [ ] 주요 프레임워크와의 통합 검증
- [ ] 크로스 플랫폼 테스트 완료
- [ ] 상세한 성능 특성 문서화

---

## 📝 Notes

### 현재 상태 평가
- **코드 품질**: 8.5/10 (매우 우수)
- **CORS 구현**: 8.0/10 (핵심 기능 충실)
- **오픈소스 준비도**: 5.0/10 (문서화 부족)
- **Production Ready**: 6.0/10 (기능 준비, 인프라 부족)

### 강점
- ✅ 클린 코드 및 단일 책임 원칙 준수
- ✅ 강력한 타입 안전성
- ✅ 174개의 포괄적인 테스트
- ✅ BDD 스타일 테스트 네이밍
- ✅ Private Network Access 지원
- ✅ Timing-Allow-Origin 지원

### 주요 약점
- ⚠️ API 문서 매우 부족 (25개 doc comment만)
- ⚠️ Cargo.toml 메타데이터 누락
- ⚠️ 사용 예제 부족 (1개만)
- ⚠️ CI/CD 파이프라인 없음
- ⚠️ CHANGELOG 없음
- ⚠️ CONTRIBUTING 가이드 없음

---

## 🎯 Milestone Timeline

### Week 1 (Critical)
- Cargo.toml 수정
- CHANGELOG.md 생성
- API 문서 50% 완료
- GitHub Actions 기본 구성

### Week 2 (High Priority)
- CONTRIBUTING.md 작성
- SECURITY.md 작성
- README 개선
- 테스트 코드 개선 시작

### Week 3-4 (Complete High Priority)
- CORS 표준 준수 강화
- 벤치마크 추가
- API 문서 100% 완료
- 예제 코드 5개 이상

### Month 2 (Polish & Release)
- 모든 Medium Priority 완료
- 커뮤니티 피드백 수집
- Pre-release 버전 배포 (0.9.x)
- 성능 최적화

### v1.0.0 Release
- 모든 필수 항목 완료
- API 안정화
- 프로덕션 사용 준비 완료

---

*Last Updated: 2025-10-02*
*Status: In Progress*
