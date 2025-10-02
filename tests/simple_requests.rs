mod common;

use bunner_cors_rs::constants::{header, method};
use bunner_cors_rs::{CorsDecision, Origin};
use common::asserts::assert_simple;
use common::builders::{cors, simple_request};
use common::headers::{has_header, header_value};

mod check {
    use super::*;

    #[test]
    fn should_allow_any_origin_when_default_simple_request_then_return_wildcard() {
        let cors = cors().build();

        let headers = assert_simple(simple_request().origin("https://example.com").check(&cors));

        assert_eq!(
            header_value(&headers, header::ACCESS_CONTROL_ALLOW_ORIGIN),
            Some("*"),
        );
        assert!(!has_header(&headers, header::VARY));
    }

    #[test]
    fn should_return_not_applicable_when_simple_request_without_origin_then_skip() {
        let cors = cors().build();

        let decision = simple_request().check(&cors);

        assert!(matches!(decision, CorsDecision::NotApplicable));
    }

    #[test]
    fn should_emit_expose_headers_when_simple_request_configures_expose_headers_then_return_value()
    {
        let cors = cors().exposed_headers(["X-Trace", "X-Auth"]).build();

        let headers = assert_simple(simple_request().origin("https://example.com").check(&cors));

        assert_eq!(
            header_value(&headers, header::ACCESS_CONTROL_EXPOSE_HEADERS),
            Some("X-Trace,X-Auth"),
        );
    }

    #[test]
    fn should_omit_expose_headers_when_not_configured_then_keep_absent() {
        let cors = cors().build();

        let headers = assert_simple(simple_request().origin("https://example.com").check(&cors));

        assert!(
            !has_header(&headers, header::ACCESS_CONTROL_EXPOSE_HEADERS),
            "expose headers should be absent when not configured",
        );
    }

    #[test]
    fn should_omit_private_network_header_when_simple_request_has_private_network_support_then_skip_header()
     {
        let cors = cors()
            .origin(Origin::exact("https://example.com"))
            .credentials(true)
            .private_network(true)
            .build();

        let headers = assert_simple(simple_request().origin("https://example.com").check(&cors));

        assert!(
            !has_header(&headers, header::ACCESS_CONTROL_ALLOW_PRIVATE_NETWORK),
            "private network header should remain absent on simple responses",
        );
    }

    #[test]
    fn should_omit_sensitive_headers_when_simple_request_origin_disallowed_then_exclude_sensitive()
    {
        let cors = cors()
            .origin(Origin::list(["https://allowed.example"]))
            .credentials(true)
            .exposed_headers(["X-Trace"])
            .build();

        let headers = assert_simple(simple_request().origin("https://deny.example").check(&cors));

        assert!(!has_header(&headers, header::ACCESS_CONTROL_ALLOW_ORIGIN));
        assert!(!has_header(
            &headers,
            header::ACCESS_CONTROL_ALLOW_CREDENTIALS
        ));
        assert!(!has_header(&headers, header::ACCESS_CONTROL_EXPOSE_HEADERS));
        assert!(has_header(&headers, header::VARY));
    }

    #[test]
    fn should_return_not_applicable_when_simple_request_method_disallowed_then_skip() {
        let cors = cors().methods([method::POST]).build();

        let decision = simple_request()
            .method(method::DELETE)
            .origin("https://methods.example")
            .check(&cors);

        assert!(matches!(decision, CorsDecision::NotApplicable));
    }
}
