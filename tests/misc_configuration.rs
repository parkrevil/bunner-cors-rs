mod common;

use bunner_cors_rs::constants::{header, method};
use bunner_cors_rs::{Cors, CorsDecision, CorsOptions, Origin, PreflightRejectionReason};
use common::asserts::{assert_preflight, assert_vary_contains, assert_vary_not_contains};
use common::builders::{cors, preflight_request};
use common::headers::{has_header, header_value};

#[test]
fn max_age_affects_preflight_response() {
    let cors = cors().max_age("600").build();

    let headers = assert_preflight(
        preflight_request()
            .origin("https://foo.bar")
            .request_method(method::GET)
            .check(&cors),
    );

    assert_eq!(
        header_value(&headers, header::ACCESS_CONTROL_MAX_AGE),
        Some("600")
    );
}

#[test]
fn empty_max_age_is_rejected() {
    let result = Cors::new(CorsOptions {
        max_age: Some(String::new()),
        ..CorsOptions::default()
    });

    let error = match result {
        Ok(_) => panic!("empty max-age should be rejected"),
        Err(error) => error,
    };
    assert_eq!(
        error.to_string(),
        "The max-age value '' must be a non-negative integer representing seconds."
    );
}

#[test]
fn default_max_age_is_absent() {
    let cors = cors().build();

    let headers = assert_preflight(
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

    let headers = assert_preflight(
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
fn empty_methods_list_omits_allow_methods_header() {
    let cors = cors().methods(Vec::<String>::new()).build();

    let decision = preflight_request()
        .origin("https://foo.bar")
        .request_method(method::PATCH)
        .check(&cors);

    match decision {
        CorsDecision::PreflightRejected(rejection) => assert_eq!(
            rejection.reason,
            PreflightRejectionReason::MethodNotAllowed {
                requested_method: "patch".to_string(),
            }
        ),
        other => panic!("expected preflight rejection, got {:?}", other),
    }
}

#[test]
fn when_origin_list_is_configured_should_emit_vary_origin_header() {
    let cors = cors()
        .origin(Origin::list(["https://foo.bar", "https://bar.baz"]))
        .build();

    let headers = assert_preflight(
        preflight_request()
            .origin("https://foo.bar")
            .request_method(method::GET)
            .check(&cors),
    );

    assert_vary_contains(&headers, header::ORIGIN);
}

#[test]
fn when_origin_allows_any_should_not_emit_vary_header() {
    let cors = cors().origin(Origin::any()).build();

    let headers = assert_preflight(
        preflight_request()
            .origin("https://foo.bar")
            .request_method(method::GET)
            .check(&cors),
    );

    assert_vary_not_contains(&headers, header::ORIGIN);
}
