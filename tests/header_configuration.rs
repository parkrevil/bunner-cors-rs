mod common;

use bunner_cors_rs::constants::{header, method};
use bunner_cors_rs::{AllowedHeaders, CorsDecision, Origin, TimingAllowOrigin};
use common::asserts::{
    assert_header_eq, assert_preflight, assert_simple, assert_vary_eq, assert_vary_is_empty,
};
use common::builders::{cors, preflight_request, simple_request};
use common::headers::{has_header, header_value};
use std::collections::HashSet;

#[test]
fn preflight_with_explicit_headers_does_not_reflect_request() {
    let cors = cors()
        .allowed_headers(AllowedHeaders::list(["Content-Type", "X-Custom"]))
        .build();

    let (headers, _status, _halt) = assert_preflight(
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
fn credentials_and_exposed_headers_are_honored() {
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
fn credentials_disabled_omits_allow_credentials_header() {
    let cors = cors().build();

    let headers = assert_simple(simple_request().origin("https://foo.bar").check(&cors));

    assert!(!has_header(
        &headers,
        header::ACCESS_CONTROL_ALLOW_CREDENTIALS
    ));
}

#[test]
fn vary_headers_are_deduplicated_and_sorted() {
    let cors = cors()
        .origin(Origin::exact("https://allowed.dev"))
        .allowed_headers(AllowedHeaders::MirrorRequest)
        .build();

    let (headers, _status, _halt) = assert_preflight(
        preflight_request()
            .origin("https://allowed.dev")
            .request_method(method::PUT)
            .request_headers("X-Test")
            .check(&cors),
    );
    assert_vary_eq(
        &headers,
        [header::ACCESS_CONTROL_REQUEST_HEADERS, header::ORIGIN],
    );
}

#[test]
fn vary_header_contains_unique_entries() {
    let cors = cors()
        .origin(Origin::exact("https://allowed.dev"))
        .allowed_headers(AllowedHeaders::MirrorRequest)
        .build();

    let (headers, _status, _halt) = assert_preflight(
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
fn mirror_request_headers_preserves_formatting() {
    let cors = cors()
        .allowed_headers(AllowedHeaders::MirrorRequest)
        .build();
    let requested = "X-Test , x-next";

    let (headers, _status, _halt) = assert_preflight(
        preflight_request()
            .origin("https://foo.bar")
            .request_method(method::PATCH)
            .request_headers(requested)
            .check(&cors),
    );

    assert_header_eq(&headers, header::ACCESS_CONTROL_ALLOW_HEADERS, requested);
    assert_vary_eq(&headers, [header::ACCESS_CONTROL_REQUEST_HEADERS]);
}

#[test]
fn mirror_request_headers_skip_allow_headers_when_request_value_empty() {
    let cors = cors()
        .allowed_headers(AllowedHeaders::MirrorRequest)
        .build();

    let (headers, _status, _halt) = assert_preflight(
        preflight_request()
            .origin("https://foo.bar")
            .request_method(method::GET)
            .request_headers("")
            .check(&cors),
    );

    assert!(!has_header(&headers, header::ACCESS_CONTROL_ALLOW_HEADERS));
    assert_vary_eq(&headers, [header::ACCESS_CONTROL_REQUEST_HEADERS]);
}

#[test]
fn empty_allowed_headers_list_omits_allow_headers() {
    let cors = {
        let empty: Vec<&str> = Vec::new();
        cors().allowed_headers(AllowedHeaders::list(empty)).build()
    };

    let decision = preflight_request()
        .origin("https://foo.bar")
        .request_method(method::GET)
        .request_headers("X-Test")
        .check(&cors);

    assert!(matches!(decision, CorsDecision::NotApplicable));
}

#[test]
fn many_exposed_headers_work_correctly() {
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
fn timing_allow_origin_wildcard_emits_on_simple_response() {
    let cors = cors().timing_allow_origin(TimingAllowOrigin::any()).build();

    let headers = assert_simple(simple_request().origin("https://foo.bar").check(&cors));

    assert_header_eq(&headers, header::TIMING_ALLOW_ORIGIN, "*");
}

#[test]
fn timing_allow_origin_list_emits_on_preflight() {
    let cors = cors()
        .timing_allow_origin(TimingAllowOrigin::list([
            "https://metrics.foo",
            "https://dash.foo",
        ]))
        .build();

    let (headers, _status, _halt) = assert_preflight(
        preflight_request()
            .origin("https://foo.bar")
            .request_method(method::GET)
            .check(&cors),
    );

    assert_header_eq(
        &headers,
        header::TIMING_ALLOW_ORIGIN,
        "https://metrics.foo https://dash.foo",
    );
}
