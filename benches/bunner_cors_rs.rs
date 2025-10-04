use bunner_cors_rs::{
    AllowedHeaders, AllowedMethods, Cors, CorsDecision, CorsOptions, ExposedHeaders,
    NormalizedRequest, Origin, OriginDecision, OriginMatcher, RequestContext, TimingAllowOrigin,
    equals_ignore_case, normalize_lower,
};
use criterion::{
    BenchmarkId, Criterion, SamplingMode, Throughput, black_box, criterion_group, criterion_main,
};
use once_cell::sync::Lazy;
use pprof::criterion::{Output, PProfProfiler};
use std::alloc::{GlobalAlloc, Layout, System};
use std::env;
use std::sync::atomic::{AtomicU64, Ordering};

const HEAVY_METHOD: &str = "POsT";
const HEAVY_ACCESS_METHOD: &str = "PuT";
static HEAVY_ORIGIN: &str = "HTTPS://EDGE.BENCH.ALLOWED";
static HEAVY_SIMPLE_ORIGIN: &str = "HTTPS://SIMPLE.BENCH.ALLOWED";

static HEAVY_HEADER_LINE: Lazy<&'static str> = Lazy::new(|| {
    let headers = (0..64)
        .map(|idx| format!("X-BENCH-HEADER-{idx:03}"))
        .collect::<Vec<_>>()
        .join(",");
    Box::leak(headers.into_boxed_str())
});

static LARGE_HEADER_LINE: Lazy<&'static str> = Lazy::new(|| {
    let headers = (0..256)
        .map(|idx| format!("X-ALLOW-{idx:03}"))
        .collect::<Vec<_>>()
        .join(",");
    Box::leak(headers.into_boxed_str())
});

static LARGE_ORIGIN_PATTERNS: Lazy<Vec<OriginMatcher>> = Lazy::new(|| {
    (0..256)
        .map(|idx| {
            let pattern = format!("^https://svc{idx:03}\\.bench\\.allowed$");
            OriginMatcher::pattern_str(&pattern).expect("valid benchmark regex")
        })
        .collect()
});

#[derive(Default)]
struct CountingAllocator {
    total_bytes: AtomicU64,
    allocations: AtomicU64,
}

impl CountingAllocator {
    const fn new() -> Self {
        Self {
            total_bytes: AtomicU64::new(0),
            allocations: AtomicU64::new(0),
        }
    }

    fn reset(&self) {
        self.total_bytes.store(0, Ordering::Relaxed);
        self.allocations.store(0, Ordering::Relaxed);
    }

    fn snapshot(&self) -> AllocationSnapshot {
        AllocationSnapshot {
            bytes: self.total_bytes.load(Ordering::Relaxed),
            allocations: self.allocations.load(Ordering::Relaxed),
        }
    }
}

unsafe impl GlobalAlloc for CountingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let ptr = unsafe { System.alloc(layout) };
        if !ptr.is_null() {
            self.total_bytes
                .fetch_add(layout.size() as u64, Ordering::Relaxed);
            self.allocations.fetch_add(1, Ordering::Relaxed);
        }
        ptr
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        let ptr = unsafe { System.alloc_zeroed(layout) };
        if !ptr.is_null() {
            self.total_bytes
                .fetch_add(layout.size() as u64, Ordering::Relaxed);
            self.allocations.fetch_add(1, Ordering::Relaxed);
        }
        ptr
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        let result = unsafe { System.realloc(ptr, layout, new_size) };
        if !result.is_null() {
            let delta = new_size.saturating_sub(layout.size()) as u64;
            self.total_bytes.fetch_add(delta, Ordering::Relaxed);
            self.allocations.fetch_add(1, Ordering::Relaxed);
        }
        result
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        unsafe { System.dealloc(ptr, layout) };
    }
}

#[derive(Clone, Copy, Debug)]
struct AllocationSnapshot {
    bytes: u64,
    allocations: u64,
}

#[global_allocator]
static GLOBAL_ALLOCATOR: CountingAllocator = CountingAllocator::new();

fn reset_allocation_counters() {
    GLOBAL_ALLOCATOR.reset();
}

fn allocation_snapshot() -> AllocationSnapshot {
    GLOBAL_ALLOCATOR.snapshot()
}

fn build_cors() -> Cors {
    Cors::new(CorsOptions {
        origin: Origin::list([
            OriginMatcher::Exact("https://bench.allowed".to_string()),
            OriginMatcher::pattern_str(r"^https://.*\.bench\.allowed$").unwrap(),
        ]),
        methods: AllowedMethods::list(["GET", "POST", "OPTIONS"]),
        allowed_headers: AllowedHeaders::list(["X-Custom-One", "X-Custom-Two", "Content-Type"]),
        exposed_headers: ExposedHeaders::list(["X-Expose-One", "X-Expose-Two"]),
        credentials: true,
        max_age: Some(600),
        allow_null_origin: false,
        allow_private_network: true,
        timing_allow_origin: None,
    })
    .expect("valid benchmark configuration")
}

fn build_cors_options_base() -> CorsOptions {
    CorsOptions {
        origin: Origin::list([
            OriginMatcher::Exact("https://bench.allowed".into()),
            OriginMatcher::pattern_str(r"^https://.*\\.bench\\.allowed$").unwrap(),
        ]),
        methods: AllowedMethods::list(["GET", "POST", "OPTIONS", "PUT"]),
        allowed_headers: AllowedHeaders::list([
            "X-Custom-One",
            "X-Custom-Two",
            "Content-Type",
            "X-Profiling",
        ]),
        exposed_headers: ExposedHeaders::list(["X-Expose-One", "X-Expose-Two"]),
        credentials: true,
        max_age: Some(600),
        allow_null_origin: false,
        allow_private_network: true,
        timing_allow_origin: Some(TimingAllowOrigin::list(vec![
            String::from("https://metrics.bench.allowed"),
            String::from("https://insights.bench.allowed"),
        ])),
    }
}

fn build_cors_wildcard() -> Cors {
    Cors::new(CorsOptions {
        origin: Origin::any(),
        methods: AllowedMethods::list(["GET", "POST", "OPTIONS"]),
        allowed_headers: AllowedHeaders::default(),
        exposed_headers: ExposedHeaders::None,
        credentials: false,
        max_age: None,
        allow_null_origin: false,
        allow_private_network: false,
        timing_allow_origin: None,
    })
    .expect("valid wildcard configuration")
}

fn build_cors_null_origin_allowed() -> Cors {
    let mut options = build_cors_options_base();
    options.allow_null_origin = true;
    options.credentials = false;
    options.allow_private_network = false;
    options.origin = Origin::list([
        OriginMatcher::Exact("https://bench.allowed".into()),
        OriginMatcher::Exact("null".into()),
    ]);
    Cors::new(options).expect("valid null origin configuration")
}

fn build_cors_no_private_network() -> Cors {
    Cors::new(CorsOptions {
        allow_private_network: false,
        ..build_cors_options_base()
    })
    .expect("valid configuration without private network")
}

fn build_cors_timing_enabled() -> Cors {
    Cors::new(CorsOptions {
        timing_allow_origin: Some(TimingAllowOrigin::list(vec![
            String::from("https://metrics.bench.allowed"),
            String::from("https://dashboard.bench.allowed"),
        ])),
        ..build_cors_options_base()
    })
    .expect("valid timing allow configuration")
}

fn build_cors_exposed_headers_disabled() -> Cors {
    Cors::new(CorsOptions {
        exposed_headers: ExposedHeaders::None,
        ..build_cors_options_base()
    })
    .expect("valid exposed headers off configuration")
}

fn build_cors_with_large_lists(size: usize) -> Cors {
    let origin_matchers = LARGE_ORIGIN_PATTERNS
        .iter()
        .take(size)
        .cloned()
        .collect::<Vec<_>>();
    let methods = (0..size)
        .map(|idx| format!("METHOD_{idx:03}"))
        .collect::<Vec<_>>();
    let headers = generate_large_headers(size.max(1));

    Cors::new(CorsOptions {
        origin: Origin::list(origin_matchers),
        methods: AllowedMethods::list(methods),
        allowed_headers: AllowedHeaders::list(headers),
        exposed_headers: ExposedHeaders::None,
        credentials: true,
        max_age: Some(120),
        allow_null_origin: false,
        allow_private_network: true,
        timing_allow_origin: None,
    })
    .expect("valid large configuration")
}

fn build_preflight_request<'a>() -> RequestContext<'a> {
    RequestContext {
        method: "OPTIONS",
        origin: "https://bench.allowed",
        access_control_request_method: "POST",
        access_control_request_headers: "X-Custom-One, content-type",
        access_control_request_private_network: true,
    }
}

fn build_null_origin_request<'a>() -> RequestContext<'a> {
    RequestContext {
        method: "OPTIONS",
        origin: "null",
        access_control_request_method: "POST",
        access_control_request_headers: "x-custom-one",
        access_control_request_private_network: true,
    }
}

fn build_simple_request<'a>() -> RequestContext<'a> {
    RequestContext {
        method: "GET",
        origin: "https://bench.allowed",
        access_control_request_method: "",
        access_control_request_headers: "",
        access_control_request_private_network: false,
    }
}

fn build_simple_request_disallowed_method<'a>() -> RequestContext<'a> {
    RequestContext {
        method: "DELETE",
        origin: "https://bench.allowed",
        access_control_request_method: "",
        access_control_request_headers: "",
        access_control_request_private_network: false,
    }
}

fn build_simple_request_uppercase() -> RequestContext<'static> {
    RequestContext {
        method: HEAVY_METHOD,
        origin: HEAVY_SIMPLE_ORIGIN,
        access_control_request_method: "",
        access_control_request_headers: HEAVY_HEADER_LINE.as_ref(),
        access_control_request_private_network: false,
    }
}

fn build_heavy_preflight_request() -> RequestContext<'static> {
    RequestContext {
        method: "OPTIONS",
        origin: HEAVY_ORIGIN,
        access_control_request_method: HEAVY_ACCESS_METHOD,
        access_control_request_headers: HEAVY_HEADER_LINE.as_ref(),
        access_control_request_private_network: true,
    }
}

fn build_large_preflight_request(size: usize) -> RequestContext<'static> {
    let index = size.saturating_sub(1);
    let origin = format!("https://svc{index:03}.bench.allowed");
    let method = format!("METHOD_{index:03}");
    let headers = generate_large_headers(size.max(1))
        .into_iter()
        .collect::<Vec<_>>()
        .join(",");
    let leaked_origin: &'static str = Box::leak(origin.into_boxed_str());
    let leaked_method: &'static str = Box::leak(method.into_boxed_str());
    let leaked_headers: &'static str = Box::leak(headers.into_boxed_str());
    RequestContext {
        method: "OPTIONS",
        origin: leaked_origin,
        access_control_request_method: leaked_method,
        access_control_request_headers: leaked_headers,
        access_control_request_private_network: true,
    }
}

fn bench_preflight_processing(c: &mut Criterion) {
    let cors = build_cors();
    let mut group = c.benchmark_group("preflight_processing");

    group.bench_function("accept_allowed_preflight", |b| {
        let request = build_preflight_request();
        b.iter(|| {
            let decision = cors.check(&request).expect("evaluation succeeds");
            match decision {
                CorsDecision::PreflightAccepted { .. } => {}
                other => panic!("unexpected decision: {other:?}"),
            }
        })
    });

    let rejecting_cors = Cors::new(CorsOptions {
        origin: Origin::exact("https://other.host"),
        ..CorsOptions::default()
    })
    .expect("valid rejecting configuration");

    group.bench_function("reject_disallowed_preflight", |b| {
        let request = build_preflight_request();
        b.iter(|| {
            let decision = rejecting_cors.check(&request).expect("evaluation succeeds");
            match decision {
                CorsDecision::PreflightRejected(_) => {}
                other => panic!("unexpected decision: {other:?}"),
            }
        })
    });

    group.finish();
}

fn bench_simple_processing(c: &mut Criterion) {
    let cors = build_cors();
    let mut group = c.benchmark_group("simple_processing");

    group.bench_function("accept_allowed_simple", |b| {
        let request = build_simple_request();
        b.iter(|| {
            let decision = cors.check(&request).expect("evaluation succeeds");
            match decision {
                CorsDecision::SimpleAccepted { .. } => {}
                other => panic!("unexpected decision: {other:?}"),
            }
        })
    });

    group.bench_function("skip_disallowed_simple", |b| {
        let request = build_simple_request_disallowed_method();
        b.iter(|| {
            let decision = cors.check(&request).expect("evaluation succeeds");
            match decision {
                CorsDecision::NotApplicable => {}
                other => panic!("unexpected decision: {other:?}"),
            }
        })
    });

    group.finish();
}

fn bench_configuration_variants(c: &mut Criterion) {
    let mut group = c.benchmark_group("configuration_variants");
    group.sample_size(40);

    let wildcard_cors = build_cors_wildcard();
    group.bench_function("wildcard_origin_any", |b| {
        let request = build_simple_request();
        b.iter(|| {
            let decision = wildcard_cors.check(&request).expect("evaluation succeeds");
            match decision {
                CorsDecision::SimpleAccepted { .. } => {}
                other => panic!("unexpected decision: {other:?}"),
            }
        })
    });

    let null_origin_cors = build_cors_null_origin_allowed();
    group.bench_function("allow_null_origin_preflight", |b| {
        let request = build_null_origin_request();
        b.iter(|| {
            let decision = null_origin_cors
                .check(&request)
                .expect("evaluation succeeds");
            match decision {
                CorsDecision::PreflightAccepted { .. } => {}
                other => panic!("unexpected decision: {other:?}"),
            }
        })
    });

    let no_private_network_cors = build_cors_no_private_network();
    group.bench_function("private_network_disabled", |b| {
        let request = build_preflight_request();
        b.iter(|| {
            let decision = no_private_network_cors
                .check(&request)
                .expect("evaluation succeeds");
            match decision {
                CorsDecision::PreflightAccepted { .. } => {}
                other => panic!("unexpected decision: {other:?}"),
            }
        })
    });

    group.finish();
}

fn bench_origin_matching(c: &mut Criterion) {
    let exact_origin = Origin::exact("https://bench.allowed");
    let list_origin = Origin::list([
        OriginMatcher::Exact("https://bench.allowed".into()),
        OriginMatcher::pattern_str(r"^https://.*\.bench\.allowed$").unwrap(),
    ]);
    let predicate_origin = Origin::custom(|origin, _| match origin {
        Some(value) if value.ends_with("bench.allowed") => OriginDecision::Mirror,
        _ => OriginDecision::Disallow,
    });

    let mut group = c.benchmark_group("origin_matching");
    let ctx = build_simple_request();

    group.bench_function("exact_origin_match", |b| {
        b.iter(|| {
            let decision = exact_origin.resolve(Some("https://bench.allowed"), &ctx);
            match decision {
                OriginDecision::Exact(_) | OriginDecision::Mirror => {}
                other => panic!("unexpected decision: {other:?}"),
            }
        })
    });

    group.bench_function("list_origin_regex_match", |b| {
        b.iter(|| {
            let decision = list_origin.resolve(Some("https://api.bench.allowed"), &ctx);
            match decision {
                OriginDecision::Mirror => {}
                other => panic!("unexpected decision: {other:?}"),
            }
        })
    });

    group.bench_function("predicate_origin_match", |b| {
        b.iter(|| {
            let decision = predicate_origin.resolve(Some("https://edge.bench.allowed"), &ctx);
            match decision {
                OriginDecision::Mirror => {}
                other => panic!("unexpected decision: {other:?}"),
            }
        })
    });

    group.finish();
}

fn bench_scaling_inputs(c: &mut Criterion) {
    let mut group = c.benchmark_group("scaling_inputs");
    group.sampling_mode(SamplingMode::Flat);

    for &size in &[16_usize, 64, 128, 256] {
        let cors = build_cors_with_large_lists(size);
        let request = build_large_preflight_request(size);

        group.bench_with_input(
            BenchmarkId::new("preflight_large", size),
            &cors,
            |b, cors| {
                b.iter(|| {
                    let decision = cors.check(&request).expect("evaluation succeeds");
                    match decision {
                        CorsDecision::PreflightAccepted { .. } => {}
                        other => panic!("unexpected decision: {other:?}"),
                    }
                })
            },
        );
    }

    group.finish();
}

fn generate_large_headers(count: usize) -> Vec<String> {
    (0..count).map(|idx| format!("X-Bench-{idx:03}")).collect()
}

fn bench_header_evaluation(c: &mut Criterion) {
    let allowed = AllowedHeaders::list(generate_large_headers(128));
    let request_header = generate_large_headers(64)
        .into_iter()
        .collect::<Vec<_>>()
        .join(",");

    let mut group = c.benchmark_group("header_evaluation");
    group.throughput(Throughput::Elements(64));

    group.bench_function("allows_headers_large", |b| {
        b.iter(|| {
            assert!(allowed.allows_headers(&request_header));
        })
    });

    let disallowed_request = format!("{request_header},X-Forbidden-Bench");
    group.bench_function("rejects_headers_large", |b| {
        b.iter(|| {
            assert!(!allowed.allows_headers(&disallowed_request));
        })
    });

    group.finish();
}

fn bench_header_feature_toggles(c: &mut Criterion) {
    let mut group = c.benchmark_group("header_feature_toggles");

    let timing_enabled = build_cors_timing_enabled();
    group.bench_function("timing_allow_origin_enabled", |b| {
        let request = build_simple_request();
        b.iter(|| {
            let decision = timing_enabled.check(&request).expect("evaluation succeeds");
            match decision {
                CorsDecision::SimpleAccepted { .. } => {}
                other => panic!("unexpected decision: {other:?}"),
            }
        })
    });

    let exposed_headers_disabled = build_cors_exposed_headers_disabled();
    group.bench_function("exposed_headers_disabled", |b| {
        let request = build_simple_request();
        b.iter(|| {
            let decision = exposed_headers_disabled
                .check(&request)
                .expect("evaluation succeeds");
            match decision {
                CorsDecision::SimpleAccepted { .. } => {}
                other => panic!("unexpected decision: {other:?}"),
            }
        })
    });

    group.finish();
}

fn bench_string_comparisons(c: &mut Criterion) {
    let mut group = c.benchmark_group("string_comparisons");
    group.throughput(Throughput::Elements(1));

    let ascii_lower = "https://edge.bench.allowed";
    let ascii_mixed = "HTTPS://EDGE.BENCH.ALLOWED";
    let ascii_other = "https://other.bench.allowed";
    let unicode_mixed = "https://DÉV.BENCH.ALLOWED";
    let unicode_lower = normalize_lower(unicode_mixed);
    let hybrid_case = "https://DÉV.bench.ALLOWED";
    let hybrid_lower = normalize_lower(hybrid_case);
    let unicode_ascii_mix = "https://dév.BENCH.allowed";
    let unicode_ascii_mix_lower = normalize_lower(unicode_ascii_mix);

    group.bench_function("equals_ascii_same_case", |b| {
        b.iter(|| {
            black_box(equals_ignore_case(
                black_box(ascii_lower),
                black_box(ascii_lower),
            ));
        })
    });

    group.bench_function("equals_ascii_mixed_case", |b| {
        b.iter(|| {
            black_box(equals_ignore_case(
                black_box(ascii_mixed),
                black_box(ascii_lower),
            ));
        })
    });

    group.bench_function("equals_ascii_mismatch", |b| {
        b.iter(|| {
            black_box(equals_ignore_case(
                black_box(ascii_lower),
                black_box(ascii_other),
            ));
        })
    });

    group.bench_function("equals_unicode_mixed_case", |b| {
        b.iter(|| {
            black_box(equals_ignore_case(
                black_box(unicode_mixed),
                black_box(&unicode_lower),
            ));
        })
    });

    group.bench_function("equals_unicode_hybrid_case", |b| {
        b.iter(|| {
            black_box(equals_ignore_case(
                black_box(hybrid_case),
                black_box(&hybrid_lower),
            ));
        })
    });

    group.bench_function("equals_unicode_ascii_mix", |b| {
        b.iter(|| {
            black_box(equals_ignore_case(
                black_box(unicode_ascii_mix),
                black_box(&unicode_ascii_mix_lower),
            ));
        })
    });

    group.bench_function("normalize_lower_ascii", |b| {
        b.iter(|| {
            black_box(normalize_lower(black_box(ascii_mixed)));
        })
    });

    group.bench_function("normalize_lower_unicode", |b| {
        b.iter(|| {
            black_box(normalize_lower(black_box(unicode_mixed)));
        })
    });

    group.finish();
}

fn bench_request_normalization(c: &mut Criterion) {
    let mut group = c.benchmark_group("request_normalization");

    let heavy_simple_request = build_simple_request_uppercase();
    group.bench_function("simple_request_normalization", |b| {
        b.iter(|| {
            let normalized = NormalizedRequest::new(&heavy_simple_request);
            black_box(normalized);
        })
    });

    let heavy_preflight_request = build_heavy_preflight_request();
    group.bench_function("preflight_request_normalization", |b| {
        b.iter(|| {
            let normalized = NormalizedRequest::new(&heavy_preflight_request);
            black_box(normalized);
        })
    });

    let mixed_unicode_request = RequestContext {
        method: "OpTiOns",
        origin: "https://DÉV.edge.BENCH.allowed",
        access_control_request_method: "PuT",
        access_control_request_headers: "X-Trace, X-DÉBUG",
        access_control_request_private_network: true,
    };

    group.bench_function("mixed_request_normalization", |b| {
        b.iter(|| {
            let normalized = NormalizedRequest::new(&mixed_unicode_request);
            black_box(normalized);
        })
    });

    let large_headers_request = RequestContext {
        method: HEAVY_METHOD,
        origin: HEAVY_ORIGIN,
        access_control_request_method: HEAVY_ACCESS_METHOD,
        access_control_request_headers: LARGE_HEADER_LINE.as_ref(),
        access_control_request_private_network: true,
    };

    group.bench_function("large_header_normalization", |b| {
        b.iter(|| {
            let normalized = NormalizedRequest::new(&large_headers_request);
            black_box(normalized);
        })
    });

    group.finish();
}

fn bench_allocation_profile(c: &mut Criterion) {
    let mut group = c.benchmark_group("allocation_profile");
    group.sample_size(30);

    let cors = build_cors();
    let request = build_preflight_request();
    group.bench_function("preflight_allocations", |b| {
        b.iter(|| {
            reset_allocation_counters();
            let decision = cors.check(&request).expect("evaluation succeeds");
            assert!(matches!(decision, CorsDecision::PreflightAccepted { .. }));
            let counts = allocation_snapshot();
            black_box((counts.bytes, counts.allocations));
        })
    });

    let simple_request = build_simple_request_disallowed_method();
    group.bench_function("simple_skip_allocations", |b| {
        b.iter(|| {
            reset_allocation_counters();
            let decision = cors.check(&simple_request).expect("evaluation succeeds");
            assert!(matches!(decision, CorsDecision::NotApplicable));
            let counts = allocation_snapshot();
            black_box((counts.bytes, counts.allocations));
        })
    });

    group.finish();
}

fn bench_cors(c: &mut Criterion) {
    bench_preflight_processing(c);
    bench_simple_processing(c);
    bench_configuration_variants(c);
    bench_origin_matching(c);
    bench_scaling_inputs(c);
    bench_header_evaluation(c);
    bench_header_feature_toggles(c);
    bench_string_comparisons(c);
    bench_request_normalization(c);
    bench_allocation_profile(c);
}

fn configure_criterion() -> Criterion {
    if env::var_os("BUNNER_PROFILE_FLAMEGRAPH").is_some() {
        Criterion::default().with_profiler(PProfProfiler::new(1000, Output::Flamegraph(None)))
    } else {
        Criterion::default()
    }
}

criterion_group!(
    name = bunner_cors_rs_benches;
    config = configure_criterion();
    targets = bench_cors
);
criterion_main!(bunner_cors_rs_benches);
