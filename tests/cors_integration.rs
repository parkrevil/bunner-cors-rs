use bunner_cors_rs::constants::{header, method};
use bunner_cors_rs::{
    AllowedHeaders, CorsDecision, CorsPolicy, Header, Origin, OriginDecision, OriginMatcher,
};
use std::collections::BTreeSet;

mod support;

use support::{PreflightRequestBuilder, policy, preflight_request, simple_request};

fn header_value<'a>(headers: &'a [Header], name: &str) -> Option<&'a str> {
    headers
        .iter()
        .find(|header| header.name.eq_ignore_ascii_case(name))
        .map(|header| header.value.as_str())
}

fn has_header(headers: &[Header], name: &str) -> bool {
    header_value(headers, name).is_some()
}

fn vary_values(headers: &[Header]) -> BTreeSet<String> {
    header_value(headers, header::VARY)
        .map(|value| {
            value
                .split(',')
                .map(|part| part.trim().to_string())
                .filter(|part| !part.is_empty())
                .collect::<BTreeSet<_>>()
        })
        .unwrap_or_default()
}

fn assert_simple(decision: CorsDecision) -> Vec<Header> {
    match decision {
        CorsDecision::Simple(result) => result.headers,
        other => panic!("expected simple decision, got {:?}", other),
    }
}

fn assert_preflight(decision: CorsDecision) -> (Vec<Header>, u16, bool) {
    match decision {
        CorsDecision::Preflight(result) => (result.headers, result.status, result.halt_response),
        other => panic!("expected preflight decision, got {:?}", other),
    }
}

mod simple_requests {
    use super::*;
    use super::{header, policy, simple_request};

    #[test]
    fn default_simple_request_allows_any_origin() {
        let policy = policy().build();
        let headers = assert_simple(
            simple_request()
                .origin("https://example.com")
                .evaluate(&policy),
        );

        assert_eq!(
            header_value(&headers, header::ACCESS_CONTROL_ALLOW_ORIGIN),
            Some("*")
        );
        assert!(!has_header(&headers, header::VARY));
    }

    #[test]
    fn default_simple_request_without_origin_still_allows_any() {
        let policy = policy().build();
        let headers = assert_simple(simple_request().evaluate(&policy));

        assert_eq!(
            header_value(&headers, header::ACCESS_CONTROL_ALLOW_ORIGIN),
            Some("*")
        );
    }
}

mod origin_configuration {
    use super::*;
    use super::{header, method, policy, simple_request};
    use regex::Regex;

    #[test]
    fn exact_origin_is_reflected_with_vary() {
        let policy = policy()
            .origin(Origin::exact("https://allowed.dev"))
            .build();

        let headers = assert_simple(
            simple_request()
                .method(method::POST)
                .origin("https://other.dev")
                .evaluate(&policy),
        );

        assert_eq!(
            header_value(&headers, header::ACCESS_CONTROL_ALLOW_ORIGIN),
            Some("https://allowed.dev")
        );
        assert_eq!(
            vary_values(&headers),
            BTreeSet::from([header::ORIGIN.to_string()])
        );
    }

    #[test]
    fn origin_list_supports_exact_and_patterns() {
        let policy = policy()
            .origin(Origin::list([
                OriginMatcher::exact("https://exact.example"),
                OriginMatcher::pattern(Regex::new(r"^https://.*\.allowed\.org$").unwrap()),
            ]))
            .build();

        let headers = assert_simple(
            simple_request()
                .origin("https://service.allowed.org")
                .evaluate(&policy),
        );

        assert_eq!(
            header_value(&headers, header::ACCESS_CONTROL_ALLOW_ORIGIN),
            Some("https://service.allowed.org")
        );
        assert_eq!(
            vary_values(&headers),
            BTreeSet::from([header::ORIGIN.to_string()])
        );

        let headers = assert_simple(
            simple_request()
                .origin("https://deny.dev")
                .evaluate(&policy),
        );

        assert!(!has_header(&headers, header::ACCESS_CONTROL_ALLOW_ORIGIN));
        assert_eq!(
            vary_values(&headers),
            BTreeSet::from([header::ORIGIN.to_string()])
        );
    }

    #[test]
    fn predicate_origin_allows_custom_logic() {
        let policy = policy()
            .origin(Origin::predicate(|origin, _ctx| {
                origin.ends_with(".trusted")
            }))
            .build();

        let allowed_headers = assert_simple(
            simple_request()
                .origin("https://service.trusted")
                .evaluate(&policy),
        );
        assert_eq!(
            header_value(&allowed_headers, header::ACCESS_CONTROL_ALLOW_ORIGIN),
            Some("https://service.trusted")
        );

        let denied_headers = assert_simple(
            simple_request()
                .origin("https://service.untrusted")
                .evaluate(&policy),
        );
        assert!(!has_header(
            &denied_headers,
            header::ACCESS_CONTROL_ALLOW_ORIGIN
        ));
    }

    #[test]
    fn custom_origin_can_skip_processing() {
        let policy = policy()
            .origin(Origin::custom(|origin, _ctx| match origin {
                Some("https://allow.me") => OriginDecision::Mirror,
                _ => OriginDecision::Skip,
            }))
            .build();

        let allowed_headers = assert_simple(
            simple_request()
                .origin("https://allow.me")
                .evaluate(&policy),
        );
        assert_eq!(
            header_value(&allowed_headers, header::ACCESS_CONTROL_ALLOW_ORIGIN),
            Some("https://allow.me")
        );

        assert!(matches!(
            simple_request().origin("https://deny.me").evaluate(&policy),
            CorsDecision::NotApplicable
        ));
    }

    #[test]
    fn disallowed_origin_returns_headers_without_allow_origin() {
        let policy = policy()
            .origin(Origin::list([OriginMatcher::exact("https://allow.one")]))
            .build();

        let headers = assert_simple(
            simple_request()
                .origin("https://deny.one")
                .evaluate(&policy),
        );

        assert!(!has_header(&headers, header::ACCESS_CONTROL_ALLOW_ORIGIN));
        assert_eq!(
            vary_values(&headers),
            BTreeSet::from([header::ORIGIN.to_string()])
        );
    }
}

mod preflight_requests {
    use super::*;
    use super::{header, method, policy, preflight_request};

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
    fn preflight_with_disallowed_method_is_not_applicable() {
        assert!(matches!(
            preflight_request()
                .origin("https://foo.bar")
                .request_method(method::DELETE)
                .evaluate(&policy().methods([method::GET, method::POST]).build()),
            CorsDecision::NotApplicable
        ));
    }

    #[test]
    fn preflight_with_disallowed_header_is_not_applicable() {
        assert!(matches!(
            preflight_request()
                .origin("https://foo.bar")
                .request_method(method::GET)
                .request_headers("X-Disallowed")
                .evaluate(
                    &policy()
                        .allowed_headers(AllowedHeaders::list(["X-Allowed"]))
                        .build()
                ),
            CorsDecision::NotApplicable
        ));
    }
}

mod header_configuration {
    use super::*;
    use super::{header, method, policy, preflight_request, simple_request};

    #[test]
    fn preflight_with_explicit_headers_does_not_reflect_request() {
        let policy = policy()
            .allowed_headers(AllowedHeaders::list(["Content-Type", "X-Custom"]))
            .build();

        let (headers, _status, _halt) = assert_preflight(
            preflight_request()
                .origin("https://foo.bar")
                .request_method(method::POST)
                .request_headers("X-Custom")
                .evaluate(&policy),
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
        let policy = policy()
            .credentials(true)
            .exposed_headers(["X-Response-Time", "X-Trace"])
            .build();

        let headers = assert_simple(simple_request().origin("https://foo.bar").evaluate(&policy));

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
    fn vary_headers_are_deduplicated_and_sorted() {
        let policy = policy()
            .origin(Origin::exact("https://allowed.dev"))
            .allowed_headers(AllowedHeaders::MirrorRequest)
            .build();

        let (headers, _status, _halt) = assert_preflight(
            preflight_request()
                .origin("https://allowed.dev")
                .request_method(method::PUT)
                .request_headers("X-Test")
                .evaluate(&policy),
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
}

mod misc_configuration {
    use super::*;
    use super::{header, method, policy, preflight_request};

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
}

mod property_based {
    use super::*;
    use super::{header, method, policy, preflight_request, simple_request};
    use proptest::prelude::*;

    fn staggered_case(input: &str) -> String {
        input
            .chars()
            .enumerate()
            .map(|(idx, ch)| {
                if idx % 2 == 0 {
                    ch.to_ascii_lowercase()
                } else {
                    ch.to_ascii_uppercase()
                }
            })
            .collect()
    }

    fn subdomain_strategy() -> impl Strategy<Value = String> {
        proptest::string::string_regex("[a-z0-9]{1,16}").unwrap()
    }

    fn header_name_strategy() -> impl Strategy<Value = String> {
        proptest::string::string_regex("[A-Za-z]{1,16}").unwrap()
    }

    proptest! {
        #[test]
        fn exact_origin_reflects_arbitrary_https_subdomain(subdomain in subdomain_strategy()) {
            let origin = format!("https://{}.example.com", subdomain);

            let headers = assert_simple(
                simple_request()
                    .origin(origin.as_str())
                    .evaluate(
                        &policy()
                            .origin(Origin::exact(origin.clone()))
                            .build()
                    ),
            );

            prop_assert_eq!(
                header_value(&headers, header::ACCESS_CONTROL_ALLOW_ORIGIN),
                Some(origin.as_str())
            );
        }

        #[test]
        fn allowed_headers_matching_is_case_insensitive(header in header_name_strategy()) {
            let allowed = header.to_uppercase();
            let request_variant = staggered_case(&header);

            let decision = preflight_request()
                .origin("https://prop.test")
                .request_method(method::GET)
                .request_headers(request_variant)
                .evaluate(
                    &policy()
                        .allowed_headers(AllowedHeaders::list([allowed.clone()]))
                        .build()
                );

            prop_assert!(matches!(decision, CorsDecision::Preflight(_)));
        }
    }
}

mod snapshot_validations {
    use super::*;
    use super::{PreflightRequestBuilder, method, policy, preflight_request};
    use insta::assert_yaml_snapshot;
    use serde::Serialize;

    #[derive(Serialize)]
    struct HeaderSnapshot {
        name: String,
        value: String,
    }

    #[derive(Serialize)]
    struct PreflightSnapshot {
        status: u16,
        halt: bool,
        headers: Vec<HeaderSnapshot>,
    }

    fn capture_preflight(
        policy: &CorsPolicy,
        request: PreflightRequestBuilder,
    ) -> PreflightSnapshot {
        let (headers, status, halt) = assert_preflight(request.evaluate(policy));
        PreflightSnapshot {
            status,
            halt,
            headers: headers
                .into_iter()
                .map(|header| HeaderSnapshot {
                    name: header.name,
                    value: header.value,
                })
                .collect(),
        }
    }

    #[test]
    fn default_preflight_snapshot() {
        let snapshot = capture_preflight(
            &policy().build(),
            preflight_request()
                .origin("https://snapshot.dev")
                .request_method(method::GET)
                .request_headers("X-Debug, Content-Type"),
        );

        assert_yaml_snapshot!("default_preflight_snapshot", snapshot);
    }

    #[test]
    fn mirror_origin_preflight_snapshot() {
        let policy = policy()
            .origin(Origin::list([OriginMatcher::exact("https://mirror.dev")]))
            .credentials(true)
            .allowed_headers(AllowedHeaders::MirrorRequest)
            .max_age("3600")
            .build();

        let snapshot = capture_preflight(
            &policy,
            preflight_request()
                .origin("https://mirror.dev")
                .request_method(method::POST)
                .request_headers("X-Trace-Id"),
        );

        assert_yaml_snapshot!("mirror_origin_preflight_snapshot", snapshot);
    }
}
