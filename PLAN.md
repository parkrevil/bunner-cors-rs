# Performance Optimization Plan (Phase 2)

## Current Status
- Profiling infrastructure is active (`BUNNER_PROFILE_FLAMEGRAPH=1 cargo bench --bench bunner_cors_rs <group>`) and flamegraphs are archived under `target/criterion/**/profile/flamegraph.svg` for each hot benchmark.
- Origin list matching regressions have been resolved by hybridizing the cached `HashSet` path with a linear scan fallback for small matcher sets (≤4 entries).
- Recent `make bench` runs (2025-10-03) show improvements across `configuration_variants`, `simple_processing`, `request_normalization`, and allocation micro-benches; no functional regressions were observed (`cargo test` fully green).

## Completed Work (2025-10-03)
- Reproduced the reported regressions via full `make bench`, then captured targeted flamegraphs for `request_normalization` and `origin_matching` to isolate hotspots.
- Implemented the `OriginList` hybrid dispatch to eliminate `HashSet` overhead for short lists, yielding ~45% speedup in `origin_matching/list_origin_regex_match` and restoring scaling benchmarks.
- Re-ran focused benches (`origin_matching`, `scaling_inputs`, `header_evaluation`, `request_normalization`) plus a final full suite to confirm the improvements and ensure statistical stability.
- Validated changes with `cargo test`, covering 216 unit and integration cases, along with scenario suites under `tests/`.

## Next Actions (Owner: backend perf)
1. **NormalizeRequest fast-path**  
   - Profile `NormalizedRequest::normalize_component` to reduce `String::from_iter` allocations (consider scratch buffer reuse or in-place lowercase via `Vec<u8>` guard).  
   - Add microbench to compare ASCII-only vs. Unicode-heavy payloads after the change.
2. **Unicode comparison tuning**  
   - Investigate `string_comparisons/equals_unicode_mixed_case` noise; explore memoized `normalize_lower` buffers or SIMD-assisted lowering.  
   - Document findings in `docs/perf/2025-10-XX-*` once stabilized.
3. **Documentation & automation**  
   - Publish a short write-up on the hybrid origin-list strategy (docs/perf).  
   - Wire `make bench` into CI nightlies with threshold alerts once variance envelopes are finalized.

## Monitoring & Benchmarks
- Track the following suites after each significant change:
  - `request_normalization/*`
  - `origin_matching/list_origin_regex_match`
  - `scaling_inputs/preflight_large/{16,64,128}`
  - `string_comparisons/equals_unicode_mixed_case`
- Capture fresh flamegraphs whenever a benchmark shows >5% regression relative to the 2025-10-03 baseline.

## Risks & Mitigations
- **Benchmark variance**: Criterion noise can mask small regressions; mitigate by running targeted benches twice and consulting the stored `report/change/*.svg` artifacts.
- **Allocation tracking**: The global counting allocator in `benches/bunner_cors_rs.rs` may skew results if new allocations are introduced; reset counters inside each iteration when experimenting.

## Coordination Notes
- Keep `TODO.md` and this plan in sync after each performance push.  
- Coordinate with the docs owner before publishing new perf reports to avoid conflicting narratives.
