mod common;

use bunner_cors_rs::constants::{header, method};
use bunner_cors_rs::{CorsOptions, CorsPolicy};
use common::asserts::assert_preflight;
use common::builders::{policy, preflight_request};
use common::headers::{has_header, header_value};

#[test]
fn max_age_and_preflight_continue_affect_preflight_response() {
    let policy = policy().max_age("600").preflight_continue(true).build();

    let (headers, status, halt) = assert_preflight(
        preflight_request()
            .origin("https://foo.bar")
            .request_method(method::GET)
            .evaluate(&policy),
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
    let policy = policy().max_age("").build();

    let (headers, _status, _halt) = assert_preflight(
        preflight_request()
            .origin("https://foo.bar")
            .request_method(method::GET)
            .evaluate(&policy),
    );

    assert!(!has_header(&headers, header::ACCESS_CONTROL_MAX_AGE));
}

#[test]
fn default_max_age_is_absent() {
    let policy = policy().build();

    let (headers, _status, _halt) = assert_preflight(
        preflight_request()
            .origin("https://foo.bar")
            .request_method(method::GET)
            .evaluate(&policy),
    );

    assert!(!has_header(&headers, header::ACCESS_CONTROL_MAX_AGE));
}

#[test]
fn zero_max_age_is_emitted() {
    let policy = policy().max_age("0").build();

    let (headers, _status, _halt) = assert_preflight(
        preflight_request()
            .origin("https://foo.bar")
            .request_method(method::GET)
            .evaluate(&policy),
    );

    assert_eq!(
        header_value(&headers, header::ACCESS_CONTROL_MAX_AGE),
        Some("0")
    );
}

#[test]
fn custom_success_status_is_reflected() {
    let policy = CorsPolicy::new(CorsOptions {
        options_success_status: 299,
        ..CorsOptions::default()
    });

    let (_headers, status, halt) = assert_preflight(
        preflight_request()
            .origin("https://foo.bar")
            .request_method(method::GET)
            .evaluate(&policy),
    );

    assert_eq!(status, 299);
    assert!(halt);
}

#[test]
fn empty_methods_list_omits_allow_methods_header() {
    let policy = policy().methods(Vec::<String>::new()).build();

    let (headers, _status, _halt) = assert_preflight(
        preflight_request()
            .origin("https://foo.bar")
            .request_method(method::PATCH)
            .evaluate(&policy),
    );

    assert!(!has_header(&headers, header::ACCESS_CONTROL_ALLOW_METHODS));
}
