mod common;

use bunner_cors_rs::constants::{header, method};
use bunner_cors_rs::{CorsDecision, Origin, OriginDecision, OriginMatcher};
use common::asserts::assert_simple;
use common::builders::{cors, simple_request};
use common::headers::{has_header, header_value, vary_values};
use regex::Regex;
use std::collections::HashSet;

#[test]
fn exact_origin_is_reflected_with_vary() {
    let cors = cors().origin(Origin::exact("https://allowed.dev")).build();

    let headers = assert_simple(
        simple_request()
            .method(method::POST)
            .origin("https://other.dev")
            .check(&cors),
    );

    assert_eq!(
        header_value(&headers, header::ACCESS_CONTROL_ALLOW_ORIGIN),
        Some("https://allowed.dev")
    );
    assert_eq!(
        vary_values(&headers),
        HashSet::from([header::ORIGIN.to_string()])
    );
}

#[test]
fn origin_list_supports_exact_and_patterns() {
    let cors = cors()
        .origin(Origin::list([
            OriginMatcher::exact("https://exact.example"),
            OriginMatcher::pattern(Regex::new(r"^https://.*\.allowed\.org$").unwrap()),
        ]))
        .build();

    let headers = assert_simple(
        simple_request()
            .origin("https://service.allowed.org")
            .check(&cors),
    );

    assert_eq!(
        header_value(&headers, header::ACCESS_CONTROL_ALLOW_ORIGIN),
        Some("https://service.allowed.org")
    );
    assert_eq!(
        vary_values(&headers),
        HashSet::from([header::ORIGIN.to_string()])
    );

    let headers = assert_simple(simple_request().origin("https://deny.dev").check(&cors));

    assert!(!has_header(&headers, header::ACCESS_CONTROL_ALLOW_ORIGIN));
    assert_eq!(
        vary_values(&headers),
        HashSet::from([header::ORIGIN.to_string()])
    );
}

#[test]
fn origin_list_combines_bool_regex_and_exact_matchers() {
    let cors = cors()
        .origin(Origin::list([
            OriginMatcher::from(false),
            OriginMatcher::pattern(Regex::new(r"^https://.*\.hybrid\.dev$").unwrap()),
            OriginMatcher::exact("https://explicit.hybrid"),
        ]))
        .build();

    let headers = assert_simple(
        simple_request()
            .origin("https://api.hybrid.dev")
            .check(&cors),
    );

    assert_eq!(
        header_value(&headers, header::ACCESS_CONTROL_ALLOW_ORIGIN),
        Some("https://api.hybrid.dev")
    );
    assert_eq!(
        vary_values(&headers),
        HashSet::from([header::ORIGIN.to_string()])
    );

    let headers = assert_simple(
        simple_request()
            .origin("https://explicit.hybrid")
            .check(&cors),
    );

    assert_eq!(
        header_value(&headers, header::ACCESS_CONTROL_ALLOW_ORIGIN),
        Some("https://explicit.hybrid")
    );

    let headers = assert_simple(
        simple_request()
            .origin("https://deny.hybrid")
            .check(&cors),
    );

    assert!(!has_header(&headers, header::ACCESS_CONTROL_ALLOW_ORIGIN));
    assert_eq!(
        vary_values(&headers),
        HashSet::from([header::ORIGIN.to_string()])
    );
}

#[test]
fn origin_list_supports_boolean_entries() {
    let cors = cors().origin(Origin::list([false, true])).build();

    let headers = assert_simple(
        simple_request()
            .origin("https://boolean.dev")
            .check(&cors),
    );

    assert_eq!(
        header_value(&headers, header::ACCESS_CONTROL_ALLOW_ORIGIN),
        Some("https://boolean.dev")
    );
}

#[test]
fn origin_list_all_false_entries_disallow() {
    let cors = cors().origin(Origin::list([false])).build();

    let headers = assert_simple(
        simple_request()
            .origin("https://deny.boole")
            .check(&cors),
    );

    assert!(!has_header(&headers, header::ACCESS_CONTROL_ALLOW_ORIGIN));
    assert_eq!(
        vary_values(&headers),
        HashSet::from([header::ORIGIN.to_string()])
    );
}

#[test]
fn predicate_origin_allows_custom_logic() {
    let cors = cors()
        .origin(Origin::predicate(|origin, _ctx| {
            origin.ends_with(".trusted")
        }))
        .build();

    let allowed_headers = assert_simple(
        simple_request()
            .origin("https://service.trusted")
            .check(&cors),
    );
    assert_eq!(
        header_value(&allowed_headers, header::ACCESS_CONTROL_ALLOW_ORIGIN),
        Some("https://service.trusted")
    );

    let denied_headers = assert_simple(
        simple_request()
            .origin("https://service.untrusted")
            .check(&cors),
    );
    assert!(!has_header(
        &denied_headers,
        header::ACCESS_CONTROL_ALLOW_ORIGIN
    ));
}

#[test]
fn predicate_origin_can_inspect_request_method() {
    let cors = cors()
        .origin(Origin::predicate(|origin, ctx| {
            origin == "https://method.dev" && ctx.method.eq_ignore_ascii_case(method::POST)
        }))
        .build();

    let headers = assert_simple(
        simple_request()
            .origin("https://method.dev")
            .method(method::POST)
            .check(&cors),
    );

    assert_eq!(
        header_value(&headers, header::ACCESS_CONTROL_ALLOW_ORIGIN),
        Some("https://method.dev")
    );

    let headers = assert_simple(
        simple_request()
            .origin("https://method.dev")
            .method(method::GET)
            .check(&cors),
    );

    assert!(!has_header(&headers, header::ACCESS_CONTROL_ALLOW_ORIGIN));
}

#[test]
fn custom_origin_can_skip_processing() {
    let cors = cors()
        .origin(Origin::custom(|origin, _ctx| match origin {
            Some("https://allow.me") => OriginDecision::Mirror,
            _ => OriginDecision::Skip,
        }))
        .build();

    let allowed_headers =
        assert_simple(simple_request().origin("https://allow.me").check(&cors));
    assert_eq!(
        header_value(&allowed_headers, header::ACCESS_CONTROL_ALLOW_ORIGIN),
        Some("https://allow.me")
    );

    assert!(matches!(
        simple_request().origin("https://deny.me").check(&cors),
        CorsDecision::NotApplicable
    ));
}

#[test]
fn custom_origin_can_return_exact_value() {
    let cors = cors()
        .origin(Origin::custom(|_, _| {
            OriginDecision::Exact("https://override.dev".into())
        }))
        .build();

    let headers = assert_simple(
        simple_request()
            .origin("https://irrelevant.dev")
            .check(&cors),
    );

    assert_eq!(
        header_value(&headers, header::ACCESS_CONTROL_ALLOW_ORIGIN),
        Some("https://override.dev")
    );
    assert_eq!(
        vary_values(&headers),
        HashSet::from([header::ORIGIN.to_string()])
    );
}

#[test]
fn custom_origin_returning_disallow_adds_vary() {
    let cors = cors()
        .origin(Origin::custom(|origin, _| match origin {
            Some("https://allow.me") => OriginDecision::Mirror,
            _ => OriginDecision::Disallow,
        }))
        .build();

    let headers = assert_simple(simple_request().origin("https://deny.me").check(&cors));

    assert!(!has_header(&headers, header::ACCESS_CONTROL_ALLOW_ORIGIN));
    assert_eq!(
        vary_values(&headers),
        HashSet::from([header::ORIGIN.to_string()])
    );
}

#[test]
fn custom_origin_handles_requests_without_origin_header() {
    let cors = cors()
        .origin(Origin::custom(|origin, _| {
            assert!(origin.is_none());
            OriginDecision::Exact("https://fallback.dev".into())
        }))
        .build();

    let headers = assert_simple(simple_request().check(&cors));

    assert_eq!(
        header_value(&headers, header::ACCESS_CONTROL_ALLOW_ORIGIN),
        Some("https://fallback.dev")
    );
    assert_eq!(
        vary_values(&headers),
        HashSet::from([header::ORIGIN.to_string()])
    );
}

#[test]
fn custom_origin_returning_any_does_not_emit_vary() {
    let cors = cors()
        .origin(Origin::custom(|_, _| OriginDecision::Any))
        .build();

    let headers = assert_simple(simple_request().origin("https://any.dev").check(&cors));

    assert_eq!(
        header_value(&headers, header::ACCESS_CONTROL_ALLOW_ORIGIN),
        Some("*")
    );
    assert!(!has_header(&headers, header::VARY));
}

#[test]
fn disallowed_origin_returns_headers_without_allow_origin() {
    let cors = cors()
        .origin(Origin::list([OriginMatcher::exact("https://allow.one")]))
        .build();

    let headers = assert_simple(simple_request().origin("https://deny.one").check(&cors));

    assert!(!has_header(&headers, header::ACCESS_CONTROL_ALLOW_ORIGIN));
    assert_eq!(
        vary_values(&headers),
        HashSet::from([header::ORIGIN.to_string()])
    );
}

#[test]
fn disabled_origin_short_circuits_cors_processing() {
    let cors = cors().origin(Origin::disabled()).build();

    assert!(matches!(
        simple_request().origin("https://any.dev").check(&cors),
        CorsDecision::NotApplicable
    ));
}

#[test]
fn missing_origin_header_with_restrictive_cors_only_sets_vary() {
    let cors = cors()
        .origin(Origin::list([OriginMatcher::exact("https://allow.dev")]))
        .build();

    let headers = assert_simple(simple_request().check(&cors));

    assert!(!has_header(&headers, header::ACCESS_CONTROL_ALLOW_ORIGIN));
    assert_eq!(
        vary_values(&headers),
        HashSet::from([header::ORIGIN.to_string()])
    );
}

#[test]
fn custom_origin_mirror_with_missing_origin_omits_allow_origin() {
    let cors = cors()
        .origin(Origin::custom(|_, _| OriginDecision::Mirror))
        .build();

    let headers = assert_simple(simple_request().check(&cors));

    assert!(!has_header(&headers, header::ACCESS_CONTROL_ALLOW_ORIGIN));
    assert_eq!(
        vary_values(&headers),
        HashSet::from([header::ORIGIN.to_string()])
    );
}
