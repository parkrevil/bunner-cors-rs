mod common;

use bunner_cors_rs::constants::{header, method};
use bunner_cors_rs::{CorsDecision, Origin, PreflightRejectionReason};
use common::asserts::{assert_preflight, assert_vary_contains, assert_vary_not_contains};
use common::builders::{cors, preflight_request};
use common::headers::{has_header, header_value};

mod check {
    use super::*;

    #[test]
    fn should_emit_max_age_when_configured_then_include_in_preflight_response() {
        let cors = cors().max_age(600).build();

        let headers = assert_preflight(
            preflight_request()
                .origin("https://foo.bar")
                .request_method(method::GET)
                .check(&cors),
        );

        assert_eq!(
            header_value(&headers, header::ACCESS_CONTROL_MAX_AGE),
            Some("600"),
        );
    }

    #[test]
    fn should_omit_max_age_when_not_configured_then_skip_header() {
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
    fn should_emit_zero_max_age_when_configured_then_include_header() {
        let cors = cors().max_age(0).build();

        let headers = assert_preflight(
            preflight_request()
                .origin("https://foo.bar")
                .request_method(method::GET)
                .check(&cors),
        );

        assert_eq!(
            header_value(&headers, header::ACCESS_CONTROL_MAX_AGE),
            Some("0"),
        );
    }

    #[test]
    fn should_reject_preflight_when_methods_list_empty_then_return_rejection() {
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
    fn should_emit_vary_origin_when_origin_list_configured_then_include_header() {
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
    fn should_omit_vary_origin_when_origin_allows_any_then_skip_header() {
        let cors = cors().origin(Origin::any()).build();

        let headers = assert_preflight(
            preflight_request()
                .origin("https://foo.bar")
                .request_method(method::GET)
                .check(&cors),
        );

        assert_vary_not_contains(&headers, header::ORIGIN);
    }
}
