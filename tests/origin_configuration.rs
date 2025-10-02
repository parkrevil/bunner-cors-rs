mod common;

use bunner_cors_rs::constants::{header, method};
use bunner_cors_rs::{CorsDecision, Origin, OriginDecision, OriginMatcher, PatternError};
use common::asserts::{assert_simple, assert_vary_eq, assert_vary_is_empty};
use common::builders::{cors, simple_request};
use common::headers::{has_header, header_value};

mod check {
    use super::*;
    use regex_automata::meta::Regex;

    #[test]
    fn should_mirror_exact_origin_when_origin_matches_then_emit_vary() {
        let cors = cors().origin(Origin::exact("https://allowed.dev")).build();

        let headers = assert_simple(
            simple_request()
                .method(method::POST)
                .origin("https://allowed.dev")
                .check(&cors),
        );

        assert_eq!(
            header_value(&headers, header::ACCESS_CONTROL_ALLOW_ORIGIN),
            Some("https://allowed.dev"),
        );
        assert_vary_eq(&headers, [header::ORIGIN]);
    }

    #[test]
    fn should_allow_origin_when_list_contains_exact_and_pattern_then_mirror_and_vary() {
        let cors = cors()
            .origin(Origin::list([
                OriginMatcher::exact("https://exact.example"),
                OriginMatcher::pattern_str(r"^https://.*\.allowed\.org$").unwrap(),
            ]))
            .build();

        let allowed_headers = assert_simple(
            simple_request()
                .origin("https://service.allowed.org")
                .check(&cors),
        );

        assert_eq!(
            header_value(&allowed_headers, header::ACCESS_CONTROL_ALLOW_ORIGIN),
            Some("https://service.allowed.org"),
        );
        assert_vary_eq(&allowed_headers, [header::ORIGIN]);

        let denied_headers =
            assert_simple(simple_request().origin("https://deny.dev").check(&cors));

        assert!(!has_header(
            &denied_headers,
            header::ACCESS_CONTROL_ALLOW_ORIGIN
        ));
        assert_vary_eq(&denied_headers, [header::ORIGIN]);
    }

    #[test]
    fn should_match_case_insensitive_when_origin_list_contains_exact_then_mirror() {
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
    fn should_match_case_insensitive_when_exact_origin_configured_then_preserve_original() {
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
    fn should_match_case_insensitive_when_origin_pattern_configured_then_preserve_request_origin() {
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
    fn should_disallow_origin_when_exact_origin_mismatch_then_omit_allow_origin() {
        let cors = cors().origin(Origin::exact("https://allowed.dev")).build();

        let headers = assert_simple(simple_request().origin("https://denied.dev").check(&cors));

        assert!(!has_header(&headers, header::ACCESS_CONTROL_ALLOW_ORIGIN));
        assert_vary_eq(&headers, [header::ORIGIN]);
    }

    #[test]
    fn should_reject_origin_when_null_by_default_then_emit_vary() {
        let cors = cors().build();

        let headers = assert_simple(simple_request().origin("null").check(&cors));

        assert!(!has_header(&headers, header::ACCESS_CONTROL_ALLOW_ORIGIN));
        assert_vary_eq(&headers, [header::ORIGIN]);
    }

    #[test]
    fn should_allow_null_origin_when_enabled_then_emit_wildcard() {
        let cors = cors().allow_null_origin(true).build();

        let headers = assert_simple(simple_request().origin("null").check(&cors));

        assert_eq!(
            header_value(&headers, header::ACCESS_CONTROL_ALLOW_ORIGIN),
            Some("*"),
        );
        assert_vary_is_empty(&headers);
    }

    #[test]
    fn should_mirror_origin_when_list_contains_multiple_matchers_then_respect_each() {
        let cors = cors()
            .origin(Origin::list([
                OriginMatcher::from(false),
                OriginMatcher::pattern_str(r"^https://.*\.hybrid\.dev$").unwrap(),
                OriginMatcher::exact("https://explicit.hybrid"),
            ]))
            .build();

        let hybrid_headers = assert_simple(
            simple_request()
                .origin("https://api.hybrid.dev")
                .check(&cors),
        );

        assert_eq!(
            header_value(&hybrid_headers, header::ACCESS_CONTROL_ALLOW_ORIGIN),
            Some("https://api.hybrid.dev"),
        );
        assert_vary_eq(&hybrid_headers, [header::ORIGIN]);

        let explicit_headers = assert_simple(
            simple_request()
                .origin("https://explicit.hybrid")
                .check(&cors),
        );

        assert_eq!(
            header_value(&explicit_headers, header::ACCESS_CONTROL_ALLOW_ORIGIN),
            Some("https://explicit.hybrid"),
        );

        let denied_headers =
            assert_simple(simple_request().origin("https://deny.hybrid").check(&cors));

        assert!(!has_header(
            &denied_headers,
            header::ACCESS_CONTROL_ALLOW_ORIGIN
        ));
        assert_vary_eq(&denied_headers, [header::ORIGIN]);
    }

    #[test]
    fn should_mirror_origin_when_boolean_true_in_list_then_reflect_origin() {
        let cors = cors().origin(Origin::list([false, true])).build();

        let headers = assert_simple(simple_request().origin("https://boolean.dev").check(&cors));

        assert_eq!(
            header_value(&headers, header::ACCESS_CONTROL_ALLOW_ORIGIN),
            Some("https://boolean.dev"),
        );
    }

    #[test]
    fn should_disallow_origin_when_boolean_list_false_then_emit_vary() {
        let cors = cors().origin(Origin::list([false])).build();

        let headers = assert_simple(simple_request().origin("https://deny.boole").check(&cors));

        assert!(!has_header(&headers, header::ACCESS_CONTROL_ALLOW_ORIGIN));
        assert_vary_eq(&headers, [header::ORIGIN]);
    }

    #[test]
    fn should_use_predicate_logic_when_custom_origin_provided_then_mirror_trusted() {
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
            Some("https://service.trusted"),
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
    fn should_consider_request_method_when_predicate_checks_method_then_conditionally_mirror() {
        let cors = cors()
            .origin(Origin::predicate(|origin, ctx| {
                origin == "https://method.dev" && ctx.method.eq_ignore_ascii_case(method::POST)
            }))
            .build();

        let post_headers = assert_simple(
            simple_request()
                .origin("https://method.dev")
                .method(method::POST)
                .check(&cors),
        );

        assert_eq!(
            header_value(&post_headers, header::ACCESS_CONTROL_ALLOW_ORIGIN),
            Some("https://method.dev"),
        );

        let get_headers = assert_simple(
            simple_request()
                .origin("https://method.dev")
                .method(method::GET)
                .check(&cors),
        );

        assert!(!has_header(
            &get_headers,
            header::ACCESS_CONTROL_ALLOW_ORIGIN
        ));
    }

    #[test]
    fn should_return_not_applicable_when_custom_origin_skips_then_stop_processing() {
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
            Some("https://allow.me"),
        );

        assert!(matches!(
            simple_request().origin("https://deny.me").check(&cors),
            CorsDecision::NotApplicable
        ));
    }

    #[test]
    fn should_override_origin_when_custom_origin_returns_exact_then_emit_override() {
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
            Some("https://override.dev"),
        );
        assert_vary_eq(&headers, [header::ORIGIN]);
    }

    #[test]
    fn should_emit_vary_when_custom_origin_disallows_then_omit_allow_origin() {
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
    fn should_handle_missing_origin_when_custom_origin_requires_none_then_emit_fallback() {
        let cors = cors()
            .origin(Origin::custom(|origin, _| {
                assert!(origin.is_none());
                OriginDecision::Exact("https://fallback.dev".into())
            }))
            .build();

        let headers = assert_simple(simple_request().check(&cors));

        assert_eq!(
            header_value(&headers, header::ACCESS_CONTROL_ALLOW_ORIGIN),
            Some("https://fallback.dev"),
        );
        assert_vary_eq(&headers, [header::ORIGIN]);
    }

    #[test]
    fn should_not_emit_vary_when_custom_origin_returns_any_then_emit_wildcard() {
        let cors = cors()
            .origin(Origin::custom(|_, _| OriginDecision::Any))
            .build();

        let headers = assert_simple(simple_request().origin("https://any.dev").check(&cors));

        assert_eq!(
            header_value(&headers, header::ACCESS_CONTROL_ALLOW_ORIGIN),
            Some("*"),
        );
        assert!(!has_header(&headers, header::VARY));
    }

    #[test]
    fn should_omit_allow_origin_when_list_disallows_origin_then_emit_vary() {
        let cors = cors()
            .origin(Origin::list([OriginMatcher::exact("https://allow.one")]))
            .build();

        let headers = assert_simple(simple_request().origin("https://deny.one").check(&cors));

        assert!(!has_header(&headers, header::ACCESS_CONTROL_ALLOW_ORIGIN));
        assert_vary_eq(&headers, [header::ORIGIN]);
    }

    #[test]
    fn should_return_not_applicable_when_origin_disabled_then_skip_response() {
        let cors = cors().origin(Origin::disabled()).build();

        assert!(matches!(
            simple_request().origin("https://any.dev").check(&cors),
            CorsDecision::NotApplicable
        ));
    }

    #[test]
    fn should_return_not_applicable_when_origin_missing_and_list_configured_then_skip() {
        let cors = cors()
            .origin(Origin::list([OriginMatcher::exact("https://allow.dev")]))
            .build();

        let decision = simple_request().check(&cors);

        assert!(matches!(decision, CorsDecision::NotApplicable));
    }

    #[test]
    fn should_omit_allow_origin_when_custom_mirror_receives_missing_origin_then_emit_vary() {
        let cors = cors()
            .origin(Origin::custom(|_, _| OriginDecision::Mirror))
            .build();

        let headers = assert_simple(simple_request().check(&cors));

        assert!(!has_header(&headers, header::ACCESS_CONTROL_ALLOW_ORIGIN));
        assert_vary_eq(&headers, [header::ORIGIN]);
    }

    #[test]
    fn should_support_precompiled_regex_when_matcher_in_origin_list_then_reflect_origin() {
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
}

mod pattern_str {
    use super::*;

    #[test]
    fn should_validate_pattern_length_when_origin_matcher_compiles_then_error_on_oversized() {
        let matcher = OriginMatcher::pattern_str(r"^https://.*\.test\.com$").unwrap();
        assert!(matcher.matches("https://sub.test.com"));
        assert!(!matcher.matches("https://sub.other.com"));

        let large_pattern = format!(r"^https://{}\.example\.com$", "a".repeat(100_000));
        match OriginMatcher::pattern_str(&large_pattern) {
            Err(PatternError::TooLong { length, max }) => {
                assert!(
                    length > max,
                    "length guard should trigger for oversized patterns",
                );
            }
            Err(other) => panic!("unexpected pattern error: {other:?}"),
            Ok(_) => panic!("expected length guard to trigger"),
        }

        assert!(OriginMatcher::pattern_str("(").is_err());
    }
}
