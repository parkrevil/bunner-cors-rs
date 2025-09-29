mod common;

use bunner_cors_rs::constants::{header, method};
use bunner_cors_rs::{
    AllowedHeaders, CorsDecision, Origin, OriginDecision, OriginMatcher, RequestContext,
};
use common::asserts::assert_preflight;
use common::builders::{cors, preflight_request};
use common::headers::{has_header, header_value, vary_values};
use std::collections::HashSet;

#[test]
fn default_preflight_reflects_request_headers() {
    let cors = cors().build();
    let (headers, status, halt) = assert_preflight(
        preflight_request()
            .origin("https://foo.bar")
            .request_method(method::GET)
            .request_headers("X-Test, Content-Type")
            .check(&cors),
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
        HashSet::from([header::ACCESS_CONTROL_REQUEST_HEADERS.into()])
    );
}

#[test]
fn preflight_without_request_method_still_uses_defaults() {
    let cors = cors().build();
    let (headers, status, _halt) =
        assert_preflight(preflight_request().origin("https://foo.bar").check(&cors));

    assert_eq!(status, 204);
    assert_eq!(
        header_value(&headers, header::ACCESS_CONTROL_ALLOW_METHODS),
        Some("GET,HEAD,PUT,PATCH,POST,DELETE")
    );
}

#[test]
fn preflight_methods_any_without_request_method_still_sets_wildcard_header() {
    let cors = cors().methods_any().build();

    let (headers, _status, _halt) =
        assert_preflight(preflight_request().origin("https://wild.dev").check(&cors));

    assert_eq!(
        header_value(&headers, header::ACCESS_CONTROL_ALLOW_METHODS),
        Some("*"),
    );
}

#[test]
fn preflight_allowed_headers_any_without_request_headers_still_sets_wildcard() {
    let cors = cors().allowed_headers(AllowedHeaders::any()).build();

    let (headers, _status, _halt) =
        assert_preflight(preflight_request().origin("https://wild.dev").check(&cors));

    assert_eq!(
        header_value(&headers, header::ACCESS_CONTROL_ALLOW_HEADERS),
        Some("*"),
    );
    assert!(vary_values(&headers).is_empty());
}

#[test]
fn preflight_allowed_headers_any_with_credentials_retains_wildcard() {
    let cors = cors()
        .origin(Origin::exact("https://wild.dev"))
        .credentials(true)
        .allowed_headers(AllowedHeaders::any())
        .build();

    let (headers, _status, _halt) = assert_preflight(
        preflight_request()
            .origin("https://wild.dev")
            .request_method(method::POST)
            .request_headers("X-Test")
            .check(&cors),
    );

    assert_eq!(
        header_value(&headers, header::ACCESS_CONTROL_ALLOW_ORIGIN),
        Some("https://wild.dev"),
    );
    assert_eq!(
        header_value(&headers, header::ACCESS_CONTROL_ALLOW_HEADERS),
        Some("*"),
    );
    assert_eq!(
        header_value(&headers, header::ACCESS_CONTROL_ALLOW_CREDENTIALS),
        Some("true"),
    );
    assert_eq!(
        vary_values(&headers),
        HashSet::from([header::ORIGIN.to_string()]),
    );
}

#[test]
fn preflight_custom_methods_preserve_case() {
    let cors = cors().methods(["post", "FETCH"]).build();

    let (headers, _status, _halt) = assert_preflight(
        preflight_request()
            .origin("https://wild.dev")
            .request_method(method::GET)
            .check(&cors),
    );

    assert_eq!(
        header_value(&headers, header::ACCESS_CONTROL_ALLOW_METHODS),
        Some("post,FETCH"),
    );
}
#[test]
fn preflight_with_disallowed_method_still_returns_configured_methods() {
    let (headers, status, _halt) = assert_preflight(
        preflight_request()
            .origin("https://foo.bar")
            .request_method(method::DELETE)
            .check(&cors().methods([method::GET, method::POST]).build()),
    );

    assert_eq!(status, 204);
    assert_eq!(
        header_value(&headers, header::ACCESS_CONTROL_ALLOW_METHODS),
        Some("GET,POST")
    );
}

#[test]
fn preflight_with_disallowed_header_returns_configured_list() {
    let (headers, status, _halt) = assert_preflight(
        preflight_request()
            .origin("https://foo.bar")
            .request_method(method::GET)
            .request_headers("X-Disallowed")
            .check(
                &cors()
                    .allowed_headers(AllowedHeaders::list(["X-Allowed"]))
                    .build(),
            ),
    );

    assert_eq!(status, 204);
    assert_eq!(
        header_value(&headers, header::ACCESS_CONTROL_ALLOW_HEADERS),
        Some("X-Allowed")
    );
}

#[test]
fn preflight_without_request_method_still_reflects_request_headers() {
    let (headers, status, _halt) = assert_preflight(
        preflight_request()
            .origin("https://foo.bar")
            .request_headers("X-Reflect")
            .check(&cors().build()),
    );

    assert_eq!(status, 204);
    assert_eq!(
        header_value(&headers, header::ACCESS_CONTROL_ALLOW_METHODS),
        Some("GET,HEAD,PUT,PATCH,POST,DELETE")
    );
    assert_eq!(
        header_value(&headers, header::ACCESS_CONTROL_ALLOW_HEADERS),
        Some("X-Reflect")
    );
}

#[test]
fn preflight_mirror_headers_without_request_headers_omits_allow_headers() {
    let cors = cors()
        .allowed_headers(AllowedHeaders::MirrorRequest)
        .build();

    let (headers, _status, _halt) = assert_preflight(
        preflight_request()
            .origin("https://foo.bar")
            .request_method(method::POST)
            .check(&cors),
    );

    assert!(!has_header(&headers, header::ACCESS_CONTROL_ALLOW_HEADERS));
    assert_eq!(
        vary_values(&headers),
        HashSet::from([header::ACCESS_CONTROL_REQUEST_HEADERS.into()])
    );
}

#[test]
fn preflight_with_disabled_origin_returns_not_applicable() {
    let cors = cors().origin(Origin::disabled()).build();

    let decision = preflight_request()
        .origin("https://skip.dev")
        .request_method(method::GET)
        .check(&cors);

    assert!(matches!(decision, CorsDecision::NotApplicable));
}

#[test]
fn preflight_with_custom_origin_skip_returns_not_applicable() {
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
fn preflight_custom_origin_requires_request_method() {
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

    let (headers, _status, _halt) = assert_preflight(
        preflight_request()
            .origin("https://ctx.dev")
            .request_method(method::PUT)
            .check(&cors),
    );

    assert_eq!(
        header_value(&headers, header::ACCESS_CONTROL_ALLOW_ORIGIN),
        Some("*")
    );
}

#[test]
fn preflight_custom_origin_checks_request_headers() {
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
        .build();

    let decision = preflight_request()
        .origin("https://headers.dev")
        .request_method(method::POST)
        .check(&cors);

    assert!(matches!(decision, CorsDecision::NotApplicable));

    let (headers, _status, _halt) = assert_preflight(
        preflight_request()
            .origin("https://headers.dev")
            .request_method(method::POST)
            .request_headers("X-Allow, X-Trace")
            .check(&cors),
    );

    assert_eq!(
        header_value(&headers, header::ACCESS_CONTROL_ALLOW_ORIGIN),
        Some("https://headers.dev")
    );
}

#[test]
fn preflight_with_credentials_sets_allow_credentials_header() {
    let cors = cors()
        .origin(Origin::exact("https://cred.dev"))
        .credentials(true)
        .build();

    let (headers, status, halt) = assert_preflight(
        preflight_request()
            .origin("https://cred.dev")
            .request_method(method::POST)
            .check(&cors),
    );

    assert_eq!(status, 204);
    assert!(
        halt,
        "preflight should halt when preflight_continue is false"
    );
    assert_eq!(
        header_value(&headers, header::ACCESS_CONTROL_ALLOW_ORIGIN),
        Some("https://cred.dev"),
    );
    assert_eq!(
        header_value(&headers, header::ACCESS_CONTROL_ALLOW_CREDENTIALS),
        Some("true"),
    );
    assert!(
        vary_values(&headers).contains(header::ORIGIN),
        "origin should be part of vary when it is reflected",
    );
}

#[test]
fn preflight_origin_list_matches_request_origin() {
    let cors = cors()
        .origin(Origin::list([
            OriginMatcher::exact("https://allowed.one"),
            OriginMatcher::pattern_str(r"^https://.*\.allow\.dev$").unwrap(),
        ]))
        .build();

    let (headers, _status, _halt) = assert_preflight(
        preflight_request()
            .origin("https://api.allow.dev")
            .request_method(method::PUT)
            .check(&cors),
    );

    assert_eq!(
        header_value(&headers, header::ACCESS_CONTROL_ALLOW_ORIGIN),
        Some("https://api.allow.dev"),
    );
    assert!(
        vary_values(&headers).contains(header::ORIGIN),
        "origin should be part of vary when origin list is restrictive",
    );
}

#[test]
fn preflight_origin_predicate_observes_normalized_request() {
    let cors = cors()
        .origin(Origin::predicate(|origin, ctx| {
            origin == "https://predicate.dev"
                && ctx.method == "options"
                && ctx.access_control_request_method == "post"
        }))
        .build();

    let (headers, _status, _halt) = assert_preflight(
        preflight_request()
            .origin("https://predicate.dev")
            .request_method(method::POST)
            .check(&cors),
    );

    assert_eq!(
        header_value(&headers, header::ACCESS_CONTROL_ALLOW_ORIGIN),
        Some("https://predicate.dev"),
    );
    assert!(
        vary_values(&headers).contains(header::ORIGIN),
        "predicate-based origins should add vary for Origin",
    );
}

#[test]
fn preflight_disallowed_origin_sets_vary_without_allow_origin() {
    let cors = cors()
        .origin(Origin::list([OriginMatcher::exact("https://allow.dev")]))
        .build();

    let (headers, _status, _halt) = assert_preflight(
        preflight_request()
            .origin("https://deny.dev")
            .request_method(method::GET)
            .check(&cors),
    );

    assert!(
        !has_header(&headers, header::ACCESS_CONTROL_ALLOW_ORIGIN),
        "disallowed origin should not emit allow-origin",
    );
    assert!(
        vary_values(&headers).contains(header::ORIGIN),
        "disallowed origin should still signal vary on Origin",
    );
}

#[test]
fn preflight_accepts_mixed_case_options_and_request_method() {
    let cors = cors().build();
    let method = String::from("oPtIoNs");
    let requested_method = String::from("pOsT");
    let requested_headers = String::from("X-MiXeD, Content-Type");

    let ctx = RequestContext {
        method: &method,
        origin: "https://case.dev",
        access_control_request_method: &requested_method,
        access_control_request_headers: &requested_headers,
    };

    let (headers, status, halt) = assert_preflight(cors.check(&ctx));

    assert_eq!(status, 204);
    assert!(halt);
    assert_eq!(
        header_value(&headers, header::ACCESS_CONTROL_ALLOW_METHODS),
        Some("GET,HEAD,PUT,PATCH,POST,DELETE"),
    );
    assert_eq!(
        header_value(&headers, header::ACCESS_CONTROL_ALLOW_HEADERS),
        Some(requested_headers.as_str()),
    );
}

#[test]
fn preflight_methods_any_sets_wildcard_header() {
    let cors = cors().methods_any().build();

    let (headers, _status, _halt) = assert_preflight(
        preflight_request()
            .origin("https://wild.dev")
            .request_method(method::DELETE)
            .check(&cors),
    );

    assert_eq!(
        header_value(&headers, header::ACCESS_CONTROL_ALLOW_METHODS),
        Some("*"),
    );
}

#[test]
fn preflight_allowed_headers_any_sets_wildcard_header() {
    let cors = cors().allowed_headers(AllowedHeaders::any()).build();

    let (headers, _status, _halt) = assert_preflight(
        preflight_request()
            .origin("https://wild.dev")
            .request_method(method::GET)
            .request_headers("X-Test")
            .check(&cors),
    );

    assert_eq!(
        header_value(&headers, header::ACCESS_CONTROL_ALLOW_HEADERS),
        Some("*"),
    );
    assert!(vary_values(&headers).is_empty());
}
