mod common;

use bunner_cors_rs::constants::{header, method};
use bunner_cors_rs::{AllowedHeaders, CorsDecision, Origin, OriginMatcher};
use common::asserts::assert_simple;
use common::builders::{policy, preflight_request, simple_request};
use common::headers::header_value;
use proptest::prelude::*;
use regex::Regex;

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

    #[test]
    fn origin_regex_list_accepts_hybrid_subdomains(subdomain in subdomain_strategy()) {
        let origin = format!("https://{}.hybrid.dev", subdomain);
        let policy = policy()
            .origin(Origin::list([
                OriginMatcher::from(false),
                OriginMatcher::pattern(Regex::new(r"^https://.*\.hybrid\.dev$").unwrap()),
            ]))
            .build();

        let headers = assert_simple(
            simple_request()
                .origin(origin.as_str())
                .evaluate(&policy),
        );

        prop_assert_eq!(
            header_value(&headers, header::ACCESS_CONTROL_ALLOW_ORIGIN),
            Some(origin.as_str())
        );
    }
}
