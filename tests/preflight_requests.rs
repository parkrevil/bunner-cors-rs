mod common;

use bunner_cors_rs::constants::{header, method};
use bunner_cors_rs::{AllowedHeaders, CorsDecision, Origin, OriginDecision};
use common::asserts::assert_preflight;
use common::builders::{policy, preflight_request};
use common::headers::{has_header, header_value, vary_values};
use std::collections::BTreeSet;

#[test]
fn default_preflight_reflects_request_headers() {
    let policy = policy().build();
    let (headers, status, halt) = assert_preflight(
        preflight_request()
            .origin("https://foo.bar")
            .request_method(method::GET)
            .request_headers("X-Test, Content-Type")
            .evaluate(&policy),
    );

    assert_eq!(status, 204);
    assert!(
        halt,
        "preflight should halt when preflight_continue is false"
    );
    assert_eq!(
        header_value(&headers, header::ACCESS_CONTROL_ALLOW_ORIGIN),
        Some("*")
    );
    assert_eq!(
        header_value(&headers, header::ACCESS_CONTROL_ALLOW_HEADERS),
        Some("X-Test, Content-Type")
    );
    assert_eq!(
        vary_values(&headers),
        BTreeSet::from([header::ACCESS_CONTROL_REQUEST_HEADERS.into()])
    );
}

#[test]
fn preflight_without_request_method_still_uses_defaults() {
    let policy = policy().build();
    let (headers, status, _halt) = assert_preflight(
        preflight_request()
            .origin("https://foo.bar")
            .evaluate(&policy),
    );

    assert_eq!(status, 204);
    assert_eq!(
        header_value(&headers, header::ACCESS_CONTROL_ALLOW_METHODS),
        Some("GET,HEAD,PUT,PATCH,POST,DELETE")
    );
}

#[test]
fn preflight_with_disallowed_method_still_returns_configured_methods() {
    let (headers, status, _halt) = assert_preflight(
        preflight_request()
            .origin("https://foo.bar")
            .request_method(method::DELETE)
            .evaluate(&policy().methods([method::GET, method::POST]).build()),
    );

    assert_eq!(status, 204);
    assert_eq!(
        header_value(&headers, header::ACCESS_CONTROL_ALLOW_METHODS),
        Some("GET,POST")
    );
}

#[test]
fn preflight_with_disallowed_header_returns_configured_list() {
    let (headers, status, _halt) = assert_preflight(
        preflight_request()
            .origin("https://foo.bar")
            .request_method(method::GET)
            .request_headers("X-Disallowed")
            .evaluate(
                &policy()
                    .allowed_headers(AllowedHeaders::list(["X-Allowed"]))
                    .build(),
            ),
    );

    assert_eq!(status, 204);
    assert_eq!(
        header_value(&headers, header::ACCESS_CONTROL_ALLOW_HEADERS),
        Some("X-Allowed")
    );
}

#[test]
fn preflight_without_request_method_still_reflects_request_headers() {
    let (headers, status, _halt) = assert_preflight(
        preflight_request()
            .origin("https://foo.bar")
            .request_headers("X-Reflect")
            .evaluate(&policy().build()),
    );

    assert_eq!(status, 204);
    assert_eq!(
        header_value(&headers, header::ACCESS_CONTROL_ALLOW_METHODS),
        Some("GET,HEAD,PUT,PATCH,POST,DELETE")
    );
    assert_eq!(
        header_value(&headers, header::ACCESS_CONTROL_ALLOW_HEADERS),
        Some("X-Reflect")
    );
}

#[test]
fn preflight_mirror_headers_without_request_headers_omits_allow_headers() {
    let policy = policy()
        .allowed_headers(AllowedHeaders::MirrorRequest)
        .build();

    let (headers, _status, _halt) = assert_preflight(
        preflight_request()
            .origin("https://foo.bar")
            .request_method(method::POST)
            .evaluate(&policy),
    );

    assert!(!has_header(&headers, header::ACCESS_CONTROL_ALLOW_HEADERS));
    assert_eq!(
        vary_values(&headers),
        BTreeSet::from([header::ACCESS_CONTROL_REQUEST_HEADERS.into()])
    );
}

#[test]
fn preflight_with_disabled_origin_returns_not_applicable() {
    let policy = policy().origin(Origin::disabled()).build();

    let decision = preflight_request()
        .origin("https://skip.dev")
        .request_method(method::GET)
        .evaluate(&policy);

    assert!(matches!(decision, CorsDecision::NotApplicable));
}

#[test]
fn preflight_with_custom_origin_skip_returns_not_applicable() {
    let policy = policy()
        .origin(Origin::custom(|origin, _ctx| match origin {
            Some("https://skip.dev") => OriginDecision::Skip,
            _ => OriginDecision::Mirror,
        }))
        .build();

    let decision = preflight_request()
        .origin("https://skip.dev")
        .request_method(method::POST)
        .evaluate(&policy);

    assert!(matches!(decision, CorsDecision::NotApplicable));
}

#[test]
fn preflight_custom_origin_requires_request_method() {
    let policy = policy()
        .origin(Origin::custom(|_, ctx| {
            if ctx.access_control_request_method.is_some() {
                OriginDecision::Any
            } else {
                OriginDecision::Skip
            }
        }))
        .build();

    let missing_method = preflight_request()
        .origin("https://ctx.dev")
        .evaluate(&policy);

    assert!(matches!(missing_method, CorsDecision::NotApplicable));

    let (headers, _status, _halt) = assert_preflight(
        preflight_request()
            .origin("https://ctx.dev")
            .request_method(method::PUT)
            .evaluate(&policy),
    );

    assert_eq!(
        header_value(&headers, header::ACCESS_CONTROL_ALLOW_ORIGIN),
        Some("*")
    );
}

#[test]
fn preflight_custom_origin_checks_request_headers() {
    let policy = policy()
        .origin(Origin::custom(|_, ctx| {
            match ctx.access_control_request_headers {
                Some(headers) if headers.to_ascii_lowercase().contains("x-allow") => {
                    OriginDecision::Mirror
                }
                _ => OriginDecision::Skip,
            }
        }))
        .build();

    let decision = preflight_request()
        .origin("https://headers.dev")
        .request_method(method::POST)
        .evaluate(&policy);

    assert!(matches!(decision, CorsDecision::NotApplicable));

    let (headers, _status, _halt) = assert_preflight(
        preflight_request()
            .origin("https://headers.dev")
            .request_method(method::POST)
            .request_headers("X-Allow, X-Trace")
            .evaluate(&policy),
    );

    assert_eq!(
        header_value(&headers, header::ACCESS_CONTROL_ALLOW_ORIGIN),
        Some("https://headers.dev")
    );
}
