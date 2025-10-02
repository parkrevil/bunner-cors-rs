# Phase 2 – 문자열 비교 최적화 실행 계획

## 목적
- `origin_matching/*` 벤치마크의 회귀(특히 regex 매칭 경로) 해소
- `preflight_large/*` 벤치에서 문자열 처리에 기인한 CPU 사용률 20% 이상 절감
- 향후 Phase 3, 4의 최적화 전제 조건(캐시/zero-copy)을 마련

## 할 일 목록
1. **관측 확대**
   - Criterion 결과에 대한 flamegraph/trace 확보 (perf 사용 가능한 환경에서 실행).
   - 현재 워크스페이스에서는 `perf` 권한이 없으므로, 별도 머신에서 실행 후 `docs/perf/flamegraphs/<date>/`에 업로드.
   - 벤치 대상: `origin_matching/list_origin_regex_match`, `origin_matching/predicate_origin_match`, `scaling_inputs/preflight_large/*`.
2. **마이크로 벤치 추가**
   - `benches/bunner_cors_rs.rs` 또는 별도 벤치로 `equals_ignore_case`/`normalize_lower` 성능 측정.
   - ASCII fast-path 유효성 및 할당 발생 여부 점검.
3. **정규식 캐싱 전략 실험**
   - `OriginMatcher::pattern_str`에서 컴파일된 정규식을 공유하도록 `RegexSet` 또는 `Arc<Regex>`를 once_cell 기반 저장소로 이동.
   - 벤치에서 캐싱 전/후 비교.
4. **API 영향 평가**
   - 캐싱 구조 도입 시 thread-safety, Send/Sync 제약 검토.
   - Public API 변경 없이 내부 캐시만 추가하는 방향 우선 고려.
5. **성능 측정 및 보고**
   - Phase 0 절차(벤치 재실행, 리포트 보관)를 반복하여 개선 폭 기록.
   - 결과는 `docs/perf/2025-10-XX-phase2-*.md`에 요약(파일은 .gitignore로 인해 Git에는 포함되지 않음).

## 위험 요소
- 정규식 캐싱 시 컴파일 실패 에러 수명 연장 가능성 → 초기화 단계에서 오류 처리 필요.
- Flamegraph 확보를 위해 외부 환경이 필수(권한 문제).

## 대안 관측 전략 (perf 미사용 환경)
- **Criterion 출력**: `string_comparisons` 벤치 그룹이 ASCII/Unicode 경로의 상대적 성능을 측정합니다.
- **할당 계측 연계**: 기존 `allocation_profile` 그룹과 조합하면 문자열 경로 변경이 힙 할당에 미치는 영향을 비교할 수 있습니다.

## 다음 단계
- 외부 환경에서 flamegraph를 먼저 수집하고, 수집까지 시간이 걸리는 동안 마이크로 벤치 추가 작업을 시작한다.
- 벤치 인프라 업데이트 후, 캐싱 실험을 feature flag 형태(예: `cfg(feature = "regex-cache")`)로 구현하여 회귀 위험을 줄인다.
