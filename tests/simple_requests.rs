mod common;

use bunner_cors_rs::constants::{header, method};
use bunner_cors_rs::{CorsDecision, Origin};
use common::asserts::assert_simple;
use common::builders::{cors, simple_request};
use common::headers::{has_header, header_value};

#[test]
fn should_allow_any_origin_in_default_simple_request() {
    let cors = cors().build();
    let headers = assert_simple(simple_request().origin("https://example.com").check(&cors));

    assert_eq!(
        header_value(&headers, header::ACCESS_CONTROL_ALLOW_ORIGIN),
        Some("*")
    );
    assert!(!has_header(&headers, header::VARY));
}

#[test]
fn should_be_not_applicable_given_simple_request_without_origin() {
    let cors = cors().build();

    let decision = simple_request().check(&cors);

    assert!(matches!(decision, CorsDecision::NotApplicable));
}

#[test]
fn should_emit_expose_header_given_simple_request_with_expose_headers() {
    let cors = cors().exposed_headers(["X-Trace", "X-Auth"]).build();

    let headers = assert_simple(simple_request().origin("https://example.com").check(&cors));

    assert_eq!(
        header_value(&headers, header::ACCESS_CONTROL_EXPOSE_HEADERS),
        Some("X-Trace,X-Auth"),
    );
}

#[test]
fn should_not_emit_expose_header_given_simple_request_without_expose_headers() {
    let cors = cors().build();

    let headers = assert_simple(simple_request().origin("https://example.com").check(&cors));

    assert!(
        !has_header(&headers, header::ACCESS_CONTROL_EXPOSE_HEADERS),
        "expose headers should be absent when not configured",
    );
}

#[test]
fn simple_request_with_private_network_support_does_not_emit_header() {
    let cors = cors()
        .origin(Origin::exact("https://example.com"))
        .credentials(true)
        .private_network(true)
        .build();

    let headers = assert_simple(simple_request().origin("https://example.com").check(&cors));

    assert!(
        !has_header(&headers, header::ACCESS_CONTROL_ALLOW_PRIVATE_NETWORK),
        "private network header should remain absent on simple responses"
    );
}

#[test]
fn simple_request_with_disallowed_origin_omits_sensitive_headers() {
    let cors = cors()
        .origin(Origin::list(["https://allowed.example"]))
        .credentials(true)
        .exposed_headers(["X-Trace"])
        .build();

    let headers = assert_simple(simple_request().origin("https://deny.example").check(&cors));

    assert!(!has_header(&headers, header::ACCESS_CONTROL_ALLOW_ORIGIN));
    assert!(!has_header(
        &headers,
        header::ACCESS_CONTROL_ALLOW_CREDENTIALS
    ));
    assert!(!has_header(&headers, header::ACCESS_CONTROL_EXPOSE_HEADERS));
    assert!(has_header(&headers, header::VARY));
}

#[test]
fn simple_request_with_disallowed_method_is_rejected() {
    let cors = cors().methods([method::POST]).build();

    let decision = simple_request()
        .method(method::DELETE)
        .origin("https://methods.example")
        .check(&cors);

    assert!(matches!(decision, CorsDecision::NotApplicable));
}
