mod common;

use bunner_cors_rs::constants::{header, method};
use bunner_cors_rs::{CorsDecision, Origin, OriginDecision, OriginMatcher, PatternError};
use common::asserts::{assert_simple, assert_vary_eq};
use common::builders::{cors, simple_request};
use common::headers::{has_header, header_value};
use regex_automata::meta::Regex;

#[test]
fn should_reflect_exact_origin_with_vary() {
    let cors = cors().origin(Origin::exact("https://allowed.dev")).build();

    let headers = assert_simple(
        simple_request()
            .method(method::POST)
            .origin("https://allowed.dev")
            .check(&cors),
    );

    assert_eq!(
        header_value(&headers, header::ACCESS_CONTROL_ALLOW_ORIGIN),
        Some("https://allowed.dev")
    );
    assert_vary_eq(&headers, [header::ORIGIN]);
}

#[test]
fn should_support_exact_and_patterns_in_origin_list() {
    let cors = cors()
        .origin(Origin::list([
            OriginMatcher::exact("https://exact.example"),
            OriginMatcher::pattern_str(r"^https://.*\.allowed\.org$").unwrap(),
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
    assert_vary_eq(&headers, [header::ORIGIN]);

    let headers = assert_simple(simple_request().origin("https://deny.dev").check(&cors));

    assert!(!has_header(&headers, header::ACCESS_CONTROL_ALLOW_ORIGIN));
    assert_vary_eq(&headers, [header::ORIGIN]);
}

#[test]
fn should_match_case_insensitively_in_origin_list() {
    let cors = cors()
        .origin(Origin::list([OriginMatcher::exact("https://Case.Match")]))
        .build();

    let headers = assert_simple(simple_request().origin("https://case.match").check(&cors));

    assert_eq!(
        header_value(&headers, header::ACCESS_CONTROL_ALLOW_ORIGIN),
        Some("https://case.match"),
    );
    assert_vary_eq(&headers, [header::ORIGIN]);
}

#[test]
fn should_be_case_insensitive_in_exact_origin_matching() {
    let cors = cors()
        .origin(Origin::exact("https://Allowed.Service"))
        .build();

    let headers = assert_simple(
        simple_request()
            .origin("https://allowed.service")
            .check(&cors),
    );

    assert_eq!(
        header_value(&headers, header::ACCESS_CONTROL_ALLOW_ORIGIN),
        Some("https://Allowed.Service"),
    );
    assert_vary_eq(&headers, [header::ORIGIN]);
}

#[test]
fn should_be_case_insensitive_in_origin_pattern_matching() {
    let cors = cors()
        .origin(Origin::list([OriginMatcher::pattern_str(
            r"^https://svc\.[a-z]+\.domain$",
        )
        .unwrap()]))
        .build();

    let headers = assert_simple(
        simple_request()
            .origin("https://SVC.metrics.domain")
            .check(&cors),
    );

    assert_eq!(
        header_value(&headers, header::ACCESS_CONTROL_ALLOW_ORIGIN),
        Some("https://SVC.metrics.domain"),
    );
    assert_vary_eq(&headers, [header::ORIGIN]);
}

#[test]
fn should_disallow_exact_origin_mismatch() {
    let cors = cors().origin(Origin::exact("https://allowed.dev")).build();

    let headers = assert_simple(simple_request().origin("https://denied.dev").check(&cors));

    assert!(!has_header(&headers, header::ACCESS_CONTROL_ALLOW_ORIGIN));
    assert_vary_eq(&headers, [header::ORIGIN]);
}

#[test]
fn should_combine_bool_regex_and_exact_matchers_in_origin_list() {
    let cors = cors()
        .origin(Origin::list([
            OriginMatcher::from(false),
            OriginMatcher::pattern_str(r"^https://.*\.hybrid\.dev$").unwrap(),
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
    assert_vary_eq(&headers, [header::ORIGIN]);

    let headers = assert_simple(
        simple_request()
            .origin("https://explicit.hybrid")
            .check(&cors),
    );

    assert_eq!(
        header_value(&headers, header::ACCESS_CONTROL_ALLOW_ORIGIN),
        Some("https://explicit.hybrid")
    );

    let headers = assert_simple(simple_request().origin("https://deny.hybrid").check(&cors));

    assert!(!has_header(&headers, header::ACCESS_CONTROL_ALLOW_ORIGIN));
    assert_vary_eq(&headers, [header::ORIGIN]);
}

#[test]
fn should_support_boolean_entries_in_origin_list() {
    let cors = cors().origin(Origin::list([false, true])).build();

    let headers = assert_simple(simple_request().origin("https://boolean.dev").check(&cors));

    assert_eq!(
        header_value(&headers, header::ACCESS_CONTROL_ALLOW_ORIGIN),
        Some("https://boolean.dev")
    );
}

#[test]
fn should_disallow_given_origin_list_all_false_entries() {
    let cors = cors().origin(Origin::list([false])).build();

    let headers = assert_simple(simple_request().origin("https://deny.boole").check(&cors));

    assert!(!has_header(&headers, header::ACCESS_CONTROL_ALLOW_ORIGIN));
    assert_vary_eq(&headers, [header::ORIGIN]);
}

#[test]
fn should_allow_custom_logic_in_predicate_origin() {
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
fn should_inspect_request_method_in_predicate_origin() {
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
fn should_skip_processing_in_custom_origin() {
    let cors = cors()
        .origin(Origin::custom(|origin, _ctx| match origin {
            Some("https://allow.me") => OriginDecision::Mirror,
            _ => OriginDecision::Skip,
        }))
        .build();

    let allowed_headers = assert_simple(simple_request().origin("https://allow.me").check(&cors));
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
fn should_return_exact_value_in_custom_origin() {
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
    assert_vary_eq(&headers, [header::ORIGIN]);
}

#[test]
fn should_add_vary_given_custom_origin_returning_disallow() {
    let cors = cors()
        .origin(Origin::custom(|origin, _| match origin {
            Some("https://allow.me") => OriginDecision::Mirror,
            _ => OriginDecision::Disallow,
        }))
        .build();

    let headers = assert_simple(simple_request().origin("https://deny.me").check(&cors));

    assert!(!has_header(&headers, header::ACCESS_CONTROL_ALLOW_ORIGIN));
    assert_vary_eq(&headers, [header::ORIGIN]);
}

#[test]
fn should_handle_requests_without_origin_header_in_custom_origin() {
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
    assert_vary_eq(&headers, [header::ORIGIN]);
}

#[test]
fn should_not_emit_vary_given_custom_origin_returning_any() {
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
fn should_return_headers_without_allow_origin_given_disallowed_origin() {
    let cors = cors()
        .origin(Origin::list([OriginMatcher::exact("https://allow.one")]))
        .build();

    let headers = assert_simple(simple_request().origin("https://deny.one").check(&cors));

    assert!(!has_header(&headers, header::ACCESS_CONTROL_ALLOW_ORIGIN));
    assert_vary_eq(&headers, [header::ORIGIN]);
}

#[test]
fn should_short_circuit_cors_processing_given_disabled_origin() {
    let cors = cors().origin(Origin::disabled()).build();

    assert!(matches!(
        simple_request().origin("https://any.dev").check(&cors),
        CorsDecision::NotApplicable
    ));
}

#[test]
fn should_short_circuit_cors_processing_given_missing_origin_header() {
    let cors = cors()
        .origin(Origin::list([OriginMatcher::exact("https://allow.dev")]))
        .build();

    let decision = simple_request().check(&cors);

    assert!(matches!(decision, CorsDecision::NotApplicable));
}

#[test]
fn should_omit_allow_origin_given_custom_origin_mirror_with_missing_origin() {
    let cors = cors()
        .origin(Origin::custom(|_, _| OriginDecision::Mirror))
        .build();

    let headers = assert_simple(simple_request().check(&cors));

    assert!(!has_header(&headers, header::ACCESS_CONTROL_ALLOW_ORIGIN));
    assert_vary_eq(&headers, [header::ORIGIN]);
}

#[test]
fn should_behave_like_regex_automata_in_regex_pattern_compilation() {
    let matcher = OriginMatcher::pattern_str(r"^https://.*\.test\.com$").unwrap();
    assert!(matcher.matches("https://sub.test.com"));
    assert!(!matcher.matches("https://sub.other.com"));

    // Large patterns now hit the safety guard instead of compiling indefinitely.
    let large_pattern = format!(r"^https://{}\.example\.com$", "a".repeat(100_000));
    match OriginMatcher::pattern_str(&large_pattern) {
        Err(PatternError::TooLong { length, max }) => {
            assert!(
                length > max,
                "length guard should trigger for oversized patterns"
            );
        }
        Err(other) => panic!("unexpected pattern error: {other:?}"),
        Ok(_) => panic!("expected length guard to trigger"),
    }

    // Invalid syntax should still surface an error from regex-automata.
    assert!(OriginMatcher::pattern_str("(").is_err());
}

#[test]
fn should_support_precompiled_regex_matcher_in_origin_list() {
    let cors = cors()
        .origin(Origin::list([OriginMatcher::pattern(
            Regex::new(r"^https://precompiled\..*\.dev$").unwrap(),
        )]))
        .build();

    let headers = assert_simple(
        simple_request()
            .origin("https://precompiled.api.dev")
            .check(&cors),
    );

    assert_eq!(
        header_value(&headers, header::ACCESS_CONTROL_ALLOW_ORIGIN),
        Some("https://precompiled.api.dev"),
    );
}
