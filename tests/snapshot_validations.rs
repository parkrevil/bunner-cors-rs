mod common;

use bunner_cors_rs::constants::method;
use bunner_cors_rs::{AllowedHeaders, CorsPolicy, Origin, OriginMatcher};
use common::asserts::assert_preflight;
use common::builders::{PreflightRequestBuilder, policy, preflight_request};
use insta::assert_yaml_snapshot;
use regex::Regex;
use serde::Serialize;

#[derive(Serialize)]
struct HeaderSnapshot {
    name: String,
    value: String,
}

#[derive(Serialize)]
struct PreflightSnapshot {
    status: u16,
    halt: bool,
    headers: Vec<HeaderSnapshot>,
}

fn capture_preflight(policy: &CorsPolicy, request: PreflightRequestBuilder) -> PreflightSnapshot {
    let (headers, status, halt) = assert_preflight(request.evaluate(policy));
    PreflightSnapshot {
        status,
        halt,
        headers: headers
            .into_iter()
            .map(|header| HeaderSnapshot {
                name: header.name,
                value: header.value,
            })
            .collect(),
    }
}

#[test]
fn default_preflight_snapshot() {
    let snapshot = capture_preflight(
        &policy().build(),
        preflight_request()
            .origin("https://snapshot.dev")
            .request_method(method::GET)
            .request_headers("X-Debug, Content-Type"),
    );

    assert_yaml_snapshot!("default_preflight_snapshot", snapshot);
}

#[test]
fn mirror_origin_preflight_snapshot() {
    let policy = policy()
        .origin(Origin::list([OriginMatcher::exact("https://mirror.dev")]))
        .credentials(true)
        .allowed_headers(AllowedHeaders::MirrorRequest)
        .max_age("3600")
        .build();

    let snapshot = capture_preflight(
        &policy,
        preflight_request()
            .origin("https://mirror.dev")
            .request_method(method::POST)
            .request_headers("X-Trace-Id"),
    );

    assert_yaml_snapshot!("mirror_origin_preflight_snapshot", snapshot);
}

#[test]
fn strict_origin_preflight_snapshot() {
    let policy = policy()
        .origin(Origin::list([OriginMatcher::pattern(
            Regex::new(r"^https://.*\.strict\.dev$").unwrap(),
        )]))
        .methods([method::GET, method::POST])
        .credentials(true)
        .allowed_headers(AllowedHeaders::list(["X-Strict", "X-Trace"]))
        .exposed_headers(["X-Result"])
        .max_age("0")
        .build();

    let snapshot = capture_preflight(
        &policy,
        preflight_request()
            .origin("https://api.strict.dev")
            .request_method(method::POST)
            .request_headers("X-Strict, X-Trace"),
    );

    assert_yaml_snapshot!("strict_origin_preflight_snapshot", snapshot);
}
