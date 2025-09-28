mod common;

use bunner_cors_rs::constants::{header, method};
use bunner_cors_rs::{Cors, CorsOptions};
use common::asserts::assert_preflight;
use common::builders::{cors, preflight_request};
use common::headers::{has_header, header_value};

#[test]
fn max_age_and_preflight_continue_affect_preflight_response() {
    let cors = cors().max_age("600").preflight_continue(true).build();

    let (headers, status, halt) = assert_preflight(
        preflight_request()
            .origin("https://foo.bar")
            .request_method(method::GET)
            .check(&cors),
    );

    assert_eq!(status, 204);
    assert!(
        !halt,
        "halt flag should be false when preflight_continue is true"
    );
    assert_eq!(
        header_value(&headers, header::ACCESS_CONTROL_MAX_AGE),
        Some("600")
    );
}

#[test]
fn empty_max_age_does_not_emit_header() {
    let cors = cors().max_age("").build();

    let (headers, _status, _halt) = assert_preflight(
        preflight_request()
            .origin("https://foo.bar")
            .request_method(method::GET)
            .check(&cors),
    );

    assert!(!has_header(&headers, header::ACCESS_CONTROL_MAX_AGE));
}

#[test]
fn default_max_age_is_absent() {
    let cors = cors().build();

    let (headers, _status, _halt) = assert_preflight(
        preflight_request()
            .origin("https://foo.bar")
            .request_method(method::GET)
            .check(&cors),
    );

    assert!(!has_header(&headers, header::ACCESS_CONTROL_MAX_AGE));
}

#[test]
fn zero_max_age_is_emitted() {
    let cors = cors().max_age("0").build();

    let (headers, _status, _halt) = assert_preflight(
        preflight_request()
            .origin("https://foo.bar")
            .request_method(method::GET)
            .check(&cors),
    );

    assert_eq!(
        header_value(&headers, header::ACCESS_CONTROL_MAX_AGE),
        Some("0")
    );
}

#[test]
fn custom_success_status_is_reflected() {
    let cors = Cors::new(CorsOptions {
        options_success_status: 299,
        ..CorsOptions::default()
    });

    let (_headers, status, halt) = assert_preflight(
        preflight_request()
            .origin("https://foo.bar")
            .request_method(method::GET)
            .check(&cors),
    );

    assert_eq!(status, 299);
    assert!(halt);
}

#[test]
fn empty_methods_list_omits_allow_methods_header() {
    let cors = cors().methods(Vec::<String>::new()).build();

    let (headers, _status, _halt) = assert_preflight(
        preflight_request()
            .origin("https://foo.bar")
            .request_method(method::PATCH)
            .check(&cors),
    );

    assert!(!has_header(&headers, header::ACCESS_CONTROL_ALLOW_METHODS));
}
