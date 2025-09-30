mod common;

use bunner_cors_rs::Origin;
use bunner_cors_rs::constants::header;
use common::asserts::assert_simple;
use common::builders::{cors, simple_request};
use common::headers::{has_header, header_value};

#[test]
fn default_simple_request_allows_any_origin() {
    let cors = cors().build();
    let headers = assert_simple(simple_request().origin("https://example.com").check(&cors));

    assert_eq!(
        header_value(&headers, header::ACCESS_CONTROL_ALLOW_ORIGIN),
        Some("*")
    );
    assert!(!has_header(&headers, header::VARY));
}

#[test]
fn default_simple_request_without_origin_still_allows_any() {
    let cors = cors().build();
    let headers = assert_simple(simple_request().check(&cors));

    assert_eq!(
        header_value(&headers, header::ACCESS_CONTROL_ALLOW_ORIGIN),
        Some("*")
    );
}

#[test]
fn simple_request_with_expose_headers_should_emit_expose_header() {
    let cors = cors().exposed_headers(["X-Trace", "X-Auth"]).build();

    let headers = assert_simple(simple_request().origin("https://example.com").check(&cors));

    assert_eq!(
        header_value(&headers, header::ACCESS_CONTROL_EXPOSE_HEADERS),
        Some("X-Trace,X-Auth"),
    );
}

#[test]
fn simple_request_without_expose_headers_should_not_emit_expose_header() {
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
