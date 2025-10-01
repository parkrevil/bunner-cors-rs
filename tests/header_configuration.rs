mod common;

use bunner_cors_rs::constants::{header, method};
use bunner_cors_rs::{
    AllowedHeaders, CorsDecision, Origin, PreflightRejectionReason, TimingAllowOrigin,
};
use common::asserts::{
    assert_header_eq, assert_preflight, assert_simple, assert_vary_eq, assert_vary_is_empty,
};
use common::builders::{cors, preflight_request, simple_request};
use common::headers::{has_header, header_value};
use std::collections::HashSet;

#[test]
fn should_not_reflect_request_given_preflight_with_explicit_headers() {
    let cors = cors()
        .allowed_headers(AllowedHeaders::list(["Content-Type", "X-Custom"]))
        .build();

    let headers = assert_preflight(
        preflight_request()
            .origin("https://foo.bar")
            .request_method(method::POST)
            .request_headers("X-Custom")
            .check(&cors),
    );

    assert_header_eq(
        &headers,
        header::ACCESS_CONTROL_ALLOW_HEADERS,
        "Content-Type,X-Custom",
    );
    assert_vary_is_empty(&headers);
}

#[test]
fn should_honor_credentials_and_exposed_headers() {
    let cors = cors()
        .credentials(true)
        .exposed_headers(["X-Response-Time", "X-Trace"])
        .build();

    let headers = assert_simple(simple_request().origin("https://foo.bar").check(&cors));

    assert_header_eq(&headers, header::ACCESS_CONTROL_ALLOW_CREDENTIALS, "true");
    assert_header_eq(
        &headers,
        header::ACCESS_CONTROL_EXPOSE_HEADERS,
        "X-Response-Time,X-Trace",
    );
}

#[test]
fn should_omit_allow_credentials_header_given_credentials_disabled() {
    let cors = cors().build();

    let headers = assert_simple(simple_request().origin("https://foo.bar").check(&cors));

    assert!(!has_header(
        &headers,
        header::ACCESS_CONTROL_ALLOW_CREDENTIALS
    ));
}

#[test]
fn should_deduplicate_and_sort_vary_headers() {
    let cors = cors()
        .origin(Origin::exact("https://allowed.dev"))
        .allowed_headers(AllowedHeaders::list(["X-Test"]))
        .build();

    let headers = assert_preflight(
        preflight_request()
            .origin("https://allowed.dev")
            .request_method(method::PUT)
            .request_headers("X-Test")
            .check(&cors),
    );
    assert_vary_eq(&headers, [header::ORIGIN]);
}

#[test]
fn should_contain_unique_entries_in_vary_header() {
    let cors = cors()
        .origin(Origin::exact("https://allowed.dev"))
        .allowed_headers(AllowedHeaders::list(["X-Test"]))
        .build();

    let headers = assert_preflight(
        preflight_request()
            .origin("https://allowed.dev")
            .request_method(method::POST)
            .request_headers("X-Test")
            .check(&cors),
    );

    let vary_header = header_value(&headers, header::VARY).expect("vary header is present");
    let parts: Vec<_> = vary_header
        .split(',')
        .map(|part| part.trim())
        .filter(|part| !part.is_empty())
        .collect();
    let unique: HashSet<_> = parts.iter().map(|part| part.to_ascii_lowercase()).collect();

    assert_eq!(
        parts.len(),
        unique.len(),
        "vary header should not contain duplicates"
    );
}

#[test]
fn should_reject_given_preflight_with_unlisted_request_header() {
    let cors = cors()
        .allowed_headers(AllowedHeaders::list(["X-Test"]))
        .build();
    let requested = "X-Test , x-next";

    let decision = preflight_request()
        .origin("https://foo.bar")
        .request_method(method::PATCH)
        .request_headers(requested)
        .check(&cors);

    match decision {
        CorsDecision::PreflightRejected(rejection) => assert_eq!(
            rejection.reason,
            PreflightRejectionReason::HeadersNotAllowed {
                requested_headers: "x-test , x-next".to_string(),
            }
        ),
        other => panic!("expected preflight rejection, got {:?}", other),
    }
}

#[test]
fn should_emit_configured_list_given_preflight_without_request_headers() {
    let cors = cors()
        .allowed_headers(AllowedHeaders::list(["X-Test"]))
        .build();

    let headers = assert_preflight(
        preflight_request()
            .origin("https://foo.bar")
            .request_method(method::GET)
            .request_headers("")
            .check(&cors),
    );

    assert_header_eq(&headers, header::ACCESS_CONTROL_ALLOW_HEADERS, "X-Test");
    assert_vary_is_empty(&headers);
}

#[test]
fn should_omit_allow_headers_given_empty_allowed_headers_list() {
    let cors = {
        let empty: Vec<&str> = Vec::new();
        cors().allowed_headers(AllowedHeaders::list(empty)).build()
    };

    let decision = preflight_request()
        .origin("https://foo.bar")
        .request_method(method::GET)
        .request_headers("X-Test")
        .check(&cors);

    match decision {
        CorsDecision::PreflightRejected(rejection) => assert_eq!(
            rejection.reason,
            PreflightRejectionReason::HeadersNotAllowed {
                requested_headers: "x-test".to_string(),
            }
        ),
        other => panic!("expected preflight rejection, got {:?}", other),
    }
}

#[test]
fn should_work_correctly_given_many_exposed_headers() {
    let cors = cors()
        .exposed_headers([
            "X-Header-1",
            "X-Header-2",
            "X-Header-3",
            "X-Header-4",
            "X-Header-5",
            "X-Header-6",
            "X-Header-7",
            "X-Header-8",
            "X-Header-9",
            "X-Header-10",
            "X-Header-11",
            "X-Header-12",
            "X-Header-13",
            "X-Header-14",
            "X-Header-15",
            "X-Header-16",
            "X-Header-17",
            "X-Header-18",
            "X-Header-19",
            "X-Header-20",
        ])
        .build();

    let headers = assert_simple(simple_request().origin("https://foo.bar").check(&cors));

    let exposed = header_value(&headers, header::ACCESS_CONTROL_EXPOSE_HEADERS).unwrap();
    assert!(exposed.contains("X-Header-1"));
    assert!(exposed.contains("X-Header-20"));
}

#[test]
fn should_emit_on_simple_response_given_timing_allow_origin_wildcard() {
    let cors = cors().timing_allow_origin(TimingAllowOrigin::any()).build();

    let headers = assert_simple(simple_request().origin("https://foo.bar").check(&cors));

    assert_header_eq(&headers, header::TIMING_ALLOW_ORIGIN, "*");
}

#[test]
fn should_omit_on_preflight_given_timing_allow_origin() {
    let cors = cors()
        .timing_allow_origin(TimingAllowOrigin::list([
            "https://metrics.foo",
            "https://dash.foo",
        ]))
        .build();

    let headers = assert_preflight(
        preflight_request()
            .origin("https://foo.bar")
            .request_method(method::GET)
            .check(&cors),
    );

    assert!(
        header_value(&headers, header::TIMING_ALLOW_ORIGIN).is_none(),
        "expected Timing-Allow-Origin to be omitted from preflight response"
    );
}
