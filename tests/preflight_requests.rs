mod common;

use bunner_cors_rs::constants::{header, method};
use bunner_cors_rs::{
    AllowedHeaders, Cors, CorsDecision, CorsOptions, Origin, OriginDecision, OriginMatcher,
    PreflightRejectionReason, RequestContext, ValidationError,
};
use common::asserts::{
    assert_header_eq, assert_preflight, assert_vary_contains, assert_vary_is_empty,
};
use common::builders::{cors, preflight_request};
use common::headers::has_header;

mod new {
    use super::*;

    #[test]
    fn should_return_error_when_allowed_headers_any_with_credentials_enabled_then_fail_validation()
    {
        let mut options = CorsOptions::new().origin(Origin::exact("https://wild.dev"));
        options.credentials = true;
        options.allowed_headers = AllowedHeaders::Any;

        let result = Cors::new(options);

        assert!(matches!(
            result,
            Err(ValidationError::AllowedHeadersAnyNotAllowedWithCredentials)
        ));
    }
}

mod check {
    use super::*;

    #[test]
    fn should_reject_preflight_when_requested_headers_not_allowed_then_return_rejection() {
        let cors = cors().build();

        let decision = preflight_request()
            .origin("https://foo.bar")
            .request_method(method::GET)
            .request_headers("X-Test, Content-Type")
            .check(&cors);

        match decision {
            CorsDecision::PreflightRejected(rejection) => {
                assert_eq!(
                    rejection.reason,
                    PreflightRejectionReason::HeadersNotAllowed {
                        requested_headers: "x-test, content-type".to_string(),
                    }
                );
            }
            other => panic!("expected preflight rejection, got {:?}", other),
        }
    }

    #[test]
    fn should_return_not_applicable_when_request_method_missing_then_skip_preflight() {
        let cors = cors().build();

        let decision = preflight_request().origin("https://foo.bar").check(&cors);

        assert!(matches!(decision, CorsDecision::NotApplicable));
    }

    #[test]
    fn should_emit_wildcard_header_when_allowed_headers_any_and_request_headers_missing_then_return_star()
     {
        let cors = cors().allowed_headers(AllowedHeaders::Any).build();

        let headers = assert_preflight(
            preflight_request()
                .origin("https://wild.dev")
                .request_method(method::GET)
                .check(&cors),
        );

        assert_header_eq(&headers, header::ACCESS_CONTROL_ALLOW_HEADERS, "*");
        assert_vary_is_empty(&headers);
    }

    #[test]
    fn should_preserve_case_when_custom_methods_configured_then_emit_original_casing() {
        let cors = cors()
            .methods(["post", "FETCH"])
            .allowed_headers(AllowedHeaders::list(["X-MiXeD", "Content-Type"]))
            .build();

        let headers = assert_preflight(
            preflight_request()
                .origin("https://wild.dev")
                .request_method("FETCH")
                .request_headers("X-MiXeD, Content-Type")
                .check(&cors),
        );

        assert_header_eq(&headers, header::ACCESS_CONTROL_ALLOW_METHODS, "post,FETCH");
        assert_header_eq(
            &headers,
            header::ACCESS_CONTROL_ALLOW_HEADERS,
            "X-MiXeD,Content-Type",
        );
    }

    #[test]
    fn should_reject_preflight_when_request_method_disallowed_then_return_rejection() {
        let decision = preflight_request()
            .origin("https://foo.bar")
            .request_method(method::DELETE)
            .check(&cors().methods([method::GET, method::POST]).build());

        match decision {
            CorsDecision::PreflightRejected(rejection) => {
                assert_eq!(
                    rejection.reason,
                    PreflightRejectionReason::MethodNotAllowed {
                        requested_method: "delete".to_string(),
                    }
                );
            }
            other => panic!("expected preflight rejection, got {:?}", other),
        }
    }

    #[test]
    fn should_reject_preflight_when_request_headers_disallowed_then_return_rejection() {
        let decision = preflight_request()
            .origin("https://foo.bar")
            .request_method(method::GET)
            .request_headers("X-Disallowed")
            .check(
                &cors()
                    .allowed_headers(AllowedHeaders::list(["X-Allowed"]))
                    .build(),
            );

        match decision {
            CorsDecision::PreflightRejected(rejection) => {
                assert_eq!(
                    rejection.reason,
                    PreflightRejectionReason::HeadersNotAllowed {
                        requested_headers: "x-disallowed".to_string(),
                    }
                );
            }
            other => panic!("expected preflight rejection, got {:?}", other),
        }
    }

    #[test]
    fn should_return_not_applicable_when_request_method_absent_then_skip_reflection() {
        let decision = preflight_request()
            .origin("https://foo.bar")
            .request_headers("X-Reflect")
            .check(&cors().build());

        assert!(matches!(decision, CorsDecision::NotApplicable));
    }

    #[test]
    fn should_emit_configured_headers_when_request_headers_missing_then_return_configured_list() {
        let cors = cors()
            .allowed_headers(AllowedHeaders::list(["X-Test"]))
            .build();

        let headers = assert_preflight(
            preflight_request()
                .origin("https://foo.bar")
                .request_method(method::POST)
                .check(&cors),
        );

        assert_header_eq(&headers, header::ACCESS_CONTROL_ALLOW_HEADERS, "X-Test");
        assert_vary_is_empty(&headers);
    }

    #[test]
    fn should_return_not_applicable_when_origin_disabled_then_skip_preflight() {
        let cors = cors().origin(Origin::disabled()).build();

        let decision = preflight_request()
            .origin("https://skip.dev")
            .request_method(method::GET)
            .check(&cors);

        assert!(matches!(decision, CorsDecision::NotApplicable));
    }

    #[test]
    fn should_return_not_applicable_when_custom_origin_requests_skip_then_skip_preflight() {
        let cors = cors()
            .origin(Origin::custom(|origin, _ctx| match origin {
                Some("https://skip.dev") => OriginDecision::Skip,
                _ => OriginDecision::Mirror,
            }))
            .build();

        let decision = preflight_request()
            .origin("https://skip.dev")
            .request_method(method::POST)
            .check(&cors);

        assert!(matches!(decision, CorsDecision::NotApplicable));
    }

    #[test]
    fn should_require_request_method_when_custom_origin_validates_then_enforce_requirement() {
        let cors = cors()
            .origin(Origin::custom(|_, ctx| {
                if ctx
                    .access_control_request_method
                    .is_some_and(|value| !value.is_empty())
                {
                    OriginDecision::Any
                } else {
                    OriginDecision::Skip
                }
            }))
            .build();

        let missing_method = preflight_request().origin("https://ctx.dev").check(&cors);

        assert!(matches!(missing_method, CorsDecision::NotApplicable));

        let headers = assert_preflight(
            preflight_request()
                .origin("https://ctx.dev")
                .request_method(method::PUT)
                .check(&cors),
        );

        assert_header_eq(&headers, header::ACCESS_CONTROL_ALLOW_ORIGIN, "*");
    }

    #[test]
    fn should_validate_request_headers_when_custom_origin_inspects_then_emit_allow_origin() {
        let cors = cors()
            .origin(Origin::custom(|_, ctx| {
                if ctx
                    .access_control_request_headers
                    .is_some_and(|value| value.to_ascii_lowercase().contains("x-allow"))
                {
                    OriginDecision::Mirror
                } else {
                    OriginDecision::Skip
                }
            }))
            .allowed_headers(AllowedHeaders::list(["X-Allow", "X-Trace"]))
            .build();

        let decision = preflight_request()
            .origin("https://headers.dev")
            .request_method(method::POST)
            .check(&cors);

        assert!(matches!(decision, CorsDecision::NotApplicable));

        let headers = assert_preflight(
            preflight_request()
                .origin("https://headers.dev")
                .request_method(method::POST)
                .request_headers("X-Allow, X-Trace")
                .check(&cors),
        );

        assert_header_eq(
            &headers,
            header::ACCESS_CONTROL_ALLOW_ORIGIN,
            "https://headers.dev",
        );
    }

    #[test]
    fn should_emit_allow_credentials_header_when_credentials_enabled_then_include_response() {
        let cors = cors()
            .origin(Origin::exact("https://cred.dev"))
            .credentials(true)
            .build();

        let headers = assert_preflight(
            preflight_request()
                .origin("https://cred.dev")
                .request_method(method::POST)
                .check(&cors),
        );

        assert_header_eq(
            &headers,
            header::ACCESS_CONTROL_ALLOW_ORIGIN,
            "https://cred.dev",
        );
        assert_header_eq(&headers, header::ACCESS_CONTROL_ALLOW_CREDENTIALS, "true");
        assert_vary_contains(&headers, header::ORIGIN);
    }

    #[test]
    fn should_mirror_request_origin_when_origin_list_matches_then_emit_vary() {
        let cors = cors()
            .origin(Origin::list([
                OriginMatcher::exact("https://allowed.one"),
                OriginMatcher::pattern_str(r"^https://.*\.allow\.dev$").unwrap(),
            ]))
            .build();

        let headers = assert_preflight(
            preflight_request()
                .origin("https://api.allow.dev")
                .request_method(method::PUT)
                .check(&cors),
        );

        assert_header_eq(
            &headers,
            header::ACCESS_CONTROL_ALLOW_ORIGIN,
            "https://api.allow.dev",
        );
        assert_vary_contains(&headers, header::ORIGIN);
    }

    #[test]
    fn should_observe_normalized_request_when_origin_predicate_invoked_then_accept_preflight() {
        let cors = cors()
            .origin(Origin::predicate(|origin, ctx| {
                origin == "https://predicate.dev"
                    && ctx.method == "options"
                    && ctx.access_control_request_method == Some("post")
            }))
            .build();

        let headers = assert_preflight(
            preflight_request()
                .origin("https://predicate.dev")
                .request_method(method::POST)
                .check(&cors),
        );

        assert_header_eq(
            &headers,
            header::ACCESS_CONTROL_ALLOW_ORIGIN,
            "https://predicate.dev",
        );
        assert_vary_contains(&headers, header::ORIGIN);
    }

    #[test]
    fn should_emit_vary_without_allow_origin_when_origin_disallowed_then_omit_allow_origin() {
        let cors = cors()
            .origin(Origin::list([OriginMatcher::exact("https://allow.dev")]))
            .build();

        let headers = assert_preflight(
            preflight_request()
                .origin("https://deny.dev")
                .request_method(method::GET)
                .check(&cors),
        );

        assert!(
            !has_header(&headers, header::ACCESS_CONTROL_ALLOW_ORIGIN),
            "disallowed origin should not emit allow-origin",
        );
        assert_vary_contains(&headers, header::ORIGIN);
    }

    #[test]
    fn should_omit_sensitive_headers_when_origin_disallowed_then_exclude_sensitive_headers() {
        let cors = cors()
            .origin(Origin::list([OriginMatcher::exact("https://allow.dev")]))
            .credentials(true)
            .allowed_headers(AllowedHeaders::list(["X-Test"]))
            .build();

        let headers = assert_preflight(
            preflight_request()
                .origin("https://deny.dev")
                .request_method(method::GET)
                .request_headers("X-Test")
                .check(&cors),
        );

        assert!(!has_header(&headers, header::ACCESS_CONTROL_ALLOW_ORIGIN));
        assert!(!has_header(
            &headers,
            header::ACCESS_CONTROL_ALLOW_CREDENTIALS
        ));
        assert!(!has_header(&headers, header::ACCESS_CONTROL_ALLOW_HEADERS));
        assert!(!has_header(&headers, header::ACCESS_CONTROL_ALLOW_METHODS));
        assert_vary_contains(&headers, header::ORIGIN);
    }

    #[test]
    fn should_accept_mixed_case_when_request_and_method_vary_then_emit_standard_headers() {
        let cors = cors()
            .allowed_headers(AllowedHeaders::list(["X-MiXeD", "Content-Type"]))
            .build();
        let method = String::from("oPtIoNs");
        let requested_method = String::from("pOsT");
        let requested_headers = String::from("X-MiXeD, Content-Type");

        let ctx = RequestContext {
            method: &method,
            origin: Some("https://case.dev"),
            access_control_request_method: Some(&requested_method),
            access_control_request_headers: Some(&requested_headers),
            access_control_request_private_network: false,
        };

        let headers = assert_preflight(
            cors.check(&ctx)
                .expect("preflight evaluation should succeed"),
        );

        assert_header_eq(
            &headers,
            header::ACCESS_CONTROL_ALLOW_METHODS,
            "GET,HEAD,PUT,PATCH,POST,DELETE",
        );
        assert_header_eq(
            &headers,
            header::ACCESS_CONTROL_ALLOW_HEADERS,
            "X-MiXeD,Content-Type",
        );
    }

    #[test]
    fn should_emit_wildcard_header_when_allowed_headers_any_then_return_star() {
        let cors = cors().allowed_headers(AllowedHeaders::Any).build();

        let headers = assert_preflight(
            preflight_request()
                .origin("https://wild.dev")
                .request_method(method::GET)
                .request_headers("X-Test")
                .check(&cors),
        );

        assert_header_eq(&headers, header::ACCESS_CONTROL_ALLOW_HEADERS, "*");
        assert_vary_is_empty(&headers);
    }

    #[test]
    fn should_emit_private_network_header_when_request_includes_private_network_then_return_true() {
        let cors = cors()
            .origin(Origin::exact("https://intranet.dev"))
            .credentials(true)
            .private_network(true)
            .build();

        let headers = assert_preflight(
            preflight_request()
                .origin("https://intranet.dev")
                .request_method(method::GET)
                .private_network(true)
                .check(&cors),
        );

        assert_header_eq(
            &headers,
            header::ACCESS_CONTROL_ALLOW_PRIVATE_NETWORK,
            "true",
        );
    }

    #[test]
    fn should_omit_private_network_header_when_request_excludes_private_network_then_skip_header() {
        let cors = cors()
            .origin(Origin::exact("https://intranet.dev"))
            .credentials(true)
            .private_network(true)
            .build();

        let headers = assert_preflight(
            preflight_request()
                .origin("https://intranet.dev")
                .request_method(method::GET)
                .check(&cors),
        );

        assert!(
            !has_header(&headers, header::ACCESS_CONTROL_ALLOW_PRIVATE_NETWORK),
            "header should be absent when request did not opt in"
        );
    }

    #[test]
    fn should_emit_configured_headers_when_multiple_allowed_headers_then_return_list() {
        let cors = cors()
            .allowed_headers(AllowedHeaders::list(["X-Allowed", "X-Trace"]))
            .build();

        let headers = assert_preflight(
            preflight_request()
                .origin("https://foo.bar")
                .request_method(method::GET)
                .request_headers("X-Allowed, X-Trace")
                .check(&cors),
        );

        assert_header_eq(
            &headers,
            header::ACCESS_CONTROL_ALLOW_HEADERS,
            "X-Allowed,X-Trace",
        );
    }
}
