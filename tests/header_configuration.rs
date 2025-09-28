mod common;

use bunner_cors_rs::constants::{header, method};
use bunner_cors_rs::{AllowedHeaders, CorsOptions, Cors, Origin};
use common::asserts::{assert_preflight, assert_simple};
use common::builders::{cors, preflight_request, simple_request};
use common::headers::{has_header, header_value, vary_values};
use std::collections::BTreeSet;

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
            .evaluate(&cors),
    );

    assert_eq!(
        header_value(&headers, header::ACCESS_CONTROL_ALLOW_HEADERS),
        Some("Content-Type,X-Custom")
    );
    assert!(
        vary_values(&headers).is_empty(),
        "should not add Vary when headers list is explicit"
    );
}

#[test]
fn credentials_and_exposed_headers_are_honored() {
    let cors = cors()
        .credentials(true)
        .exposed_headers(["X-Response-Time", "X-Trace"])
        .build();

    let headers = assert_simple(simple_request().origin("https://foo.bar").evaluate(&cors));

    assert_eq!(
        header_value(&headers, header::ACCESS_CONTROL_ALLOW_CREDENTIALS),
        Some("true")
    );
    assert_eq!(
        header_value(&headers, header::ACCESS_CONTROL_EXPOSE_HEADERS),
        Some("X-Response-Time,X-Trace")
    );
}

#[test]
fn credentials_disabled_omits_allow_credentials_header() {
    let cors = cors().build();

    let headers = assert_simple(simple_request().origin("https://foo.bar").evaluate(&cors));

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
            .evaluate(&cors),
    );
    let vary = vary_values(&headers);

    assert_eq!(
        vary,
        BTreeSet::from([
            header::ACCESS_CONTROL_REQUEST_HEADERS.to_string(),
            header::ORIGIN.to_string()
        ])
    );
}

#[test]
fn headers_alias_configures_allow_headers() {
    let options = CorsOptions {
        headers: Some(AllowedHeaders::list(["X-Alias", "X-Trace"])),
        ..CorsOptions::default()
    };
    let cors = Cors::new(options);

    let (headers, _status, _halt) = assert_preflight(
        preflight_request()
            .origin("https://foo.bar")
            .request_method(method::POST)
            .request_headers("X-Other")
            .evaluate(&cors),
    );

    assert_eq!(
        header_value(&headers, header::ACCESS_CONTROL_ALLOW_HEADERS),
        Some("X-Alias,X-Trace")
    );
    assert!(vary_values(&headers).is_empty());
}

#[test]
fn headers_alias_does_not_override_explicit_allowed_headers() {
    let cors = Cors::new(CorsOptions {
        allowed_headers: AllowedHeaders::list(["X-Primary"]),
        headers: Some(AllowedHeaders::list(["X-Alias"])),
        ..CorsOptions::default()
    });

    let (headers, _status, _halt) = assert_preflight(
        preflight_request()
            .origin("https://foo.bar")
            .request_method(method::GET)
            .request_headers("X-Other")
            .evaluate(&cors),
    );

    assert_eq!(
        header_value(&headers, header::ACCESS_CONTROL_ALLOW_HEADERS),
        Some("X-Primary")
    );
    assert!(vary_values(&headers).is_empty());
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
            .evaluate(&cors),
    );

    assert_eq!(
        header_value(&headers, header::ACCESS_CONTROL_ALLOW_HEADERS),
        Some(requested)
    );
    assert_eq!(
        vary_values(&headers),
        BTreeSet::from([header::ACCESS_CONTROL_REQUEST_HEADERS.into()])
    );
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
            .evaluate(&cors),
    );

    assert!(!has_header(&headers, header::ACCESS_CONTROL_ALLOW_HEADERS));
    assert_eq!(
        vary_values(&headers),
        BTreeSet::from([header::ACCESS_CONTROL_REQUEST_HEADERS.into()])
    );
}

#[test]
fn empty_allowed_headers_list_omits_allow_headers() {
    let cors = {
        let empty: Vec<&str> = Vec::new();
        cors()
            .allowed_headers(AllowedHeaders::list(empty))
            .build()
    };

    let (headers, _status, _halt) = assert_preflight(
        preflight_request()
            .origin("https://foo.bar")
            .request_method(method::GET)
            .request_headers("X-Test")
            .evaluate(&cors),
    );

    assert!(!has_header(&headers, header::ACCESS_CONTROL_ALLOW_HEADERS));
    assert!(vary_values(&headers).is_empty());
}
