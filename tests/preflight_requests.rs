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

#[test]
fn should_reject_default_preflight_with_requested_headers() {
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
fn should_be_not_applicable_given_preflight_without_request_method() {
    let cors = cors().build();
    let decision = preflight_request().origin("https://foo.bar").check(&cors);
    assert!(matches!(decision, CorsDecision::NotApplicable));
}

#[test]
fn should_set_wildcard_given_allowed_headers_any_without_request_headers() {
    let cors = cors().allowed_headers(AllowedHeaders::any()).build();

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
fn should_reject_given_allowed_headers_any_with_credentials() {
    let result = Cors::new(CorsOptions {
        origin: Origin::exact("https://wild.dev"),
        credentials: true,
        allowed_headers: AllowedHeaders::any(),
        ..CorsOptions::default()
    });

    assert!(matches!(
        result,
        Err(ValidationError::AllowedHeadersAnyNotAllowedWithCredentials)
    ));
}

#[test]
fn should_preserve_case_given_custom_methods() {
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
fn should_reject_given_disallowed_method() {
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
fn should_reject_given_disallowed_header() {
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
fn should_not_reflect_request_headers_given_no_request_method() {
    let decision = preflight_request()
        .origin("https://foo.bar")
        .request_headers("X-Reflect")
        .check(&cors().build());

    assert!(matches!(decision, CorsDecision::NotApplicable));
}

#[test]
fn should_emit_configured_list_given_no_request_headers() {
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
fn should_return_not_applicable_given_disabled_origin() {
    let cors = cors().origin(Origin::disabled()).build();

    let decision = preflight_request()
        .origin("https://skip.dev")
        .request_method(method::GET)
        .check(&cors);

    assert!(matches!(decision, CorsDecision::NotApplicable));
}

#[test]
fn should_return_not_applicable_given_custom_origin_skip() {
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
fn should_require_request_method_given_custom_origin() {
    let cors = cors()
        .origin(Origin::custom(|_, ctx| {
            if !ctx.access_control_request_method.is_empty() {
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
fn should_check_request_headers_given_custom_origin() {
    let cors = cors()
        .origin(Origin::custom(|_, ctx| {
            if ctx
                .access_control_request_headers
                .to_ascii_lowercase()
                .contains("x-allow")
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
fn should_set_allow_credentials_header_given_credentials() {
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
fn should_match_request_origin_given_origin_list() {
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
fn should_observe_normalized_request_given_origin_predicate() {
    let cors = cors()
        .origin(Origin::predicate(|origin, ctx| {
            origin == "https://predicate.dev"
                && ctx.method == "options"
                && ctx.access_control_request_method == "post"
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
fn should_set_vary_without_allow_origin_given_disallowed_origin() {
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
fn should_omit_sensitive_headers_given_disallowed_origin() {
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
fn should_accept_mixed_case_given_options_and_request_method() {
    let cors = cors()
        .allowed_headers(AllowedHeaders::list(["X-MiXeD", "Content-Type"]))
        .build();
    let method = String::from("oPtIoNs");
    let requested_method = String::from("pOsT");
    let requested_headers = String::from("X-MiXeD, Content-Type");

    let ctx = RequestContext {
        method: &method,
        origin: "https://case.dev",
        access_control_request_method: &requested_method,
        access_control_request_headers: &requested_headers,
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
fn should_set_wildcard_header_given_allowed_headers_any() {
    let cors = cors().allowed_headers(AllowedHeaders::any()).build();

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
fn should_emit_allow_header_given_private_network_request() {
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
fn should_omit_allow_header_given_no_private_network_request() {
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
fn should_emit_configured_list_given_multiple_allowed_headers() {
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
