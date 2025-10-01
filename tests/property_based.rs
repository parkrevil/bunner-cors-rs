mod common;

use bunner_cors_rs::constants::{header, method};
use bunner_cors_rs::{AllowedHeaders, CorsDecision, Origin, OriginMatcher};
use common::asserts::assert_simple;
use common::builders::{cors, preflight_request, simple_request};
use common::headers::header_value;
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
    fn should_reflect_arbitrary_https_subdomain_given_exact_origin(subdomain in subdomain_strategy()) {
        let origin = format!("https://{}.example.com", subdomain);

        let headers = assert_simple(
            simple_request()
                .origin(origin.as_str())
                .check(
                    &cors()
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
    fn should_be_case_insensitive_in_allowed_headers_matching(header in header_name_strategy()) {
        let allowed = header.to_uppercase();
        let request_variant = staggered_case(&header);

        let decision = preflight_request()
            .origin("https://prop.test")
            .request_method(method::GET)
            .request_headers(request_variant)
            .check(
                &cors()
                    .allowed_headers(AllowedHeaders::list([allowed.clone()]))
                    .build()
            );

        let is_preflight_accepted = matches!(
            decision,
            CorsDecision::PreflightAccepted { .. }
        );
        prop_assert!(is_preflight_accepted);
    }

    #[test]
    fn should_accept_hybrid_subdomains_given_origin_regex_list(subdomain in subdomain_strategy()) {
        let origin = format!("https://{}.hybrid.dev", subdomain);
        let cors = cors()
            .origin(Origin::list([
                OriginMatcher::from(false),
                OriginMatcher::pattern_str(r"^https://.*\.hybrid\.dev$").unwrap(),
            ]))
            .build();

        let headers = assert_simple(
            simple_request()
                .origin(origin.as_str())
                .check(&cors),
        );

        prop_assert_eq!(
            header_value(&headers, header::ACCESS_CONTROL_ALLOW_ORIGIN),
            Some(origin.as_str())
        );
    }
}
