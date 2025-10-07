use super::*;
use crate::ExposedHeaders;
use crate::Headers;
use crate::allowed_headers::AllowedHeaders;
use crate::allowed_methods::AllowedMethods;
use crate::constants::header;
use crate::context::RequestContext;
use crate::normalized_request::NormalizedRequest;
use crate::options::{CorsOptions, ValidationError};
use crate::origin::{Origin, OriginDecision};
use crate::result::{
    CorsDecision, CorsError, PreflightRejection, PreflightRejectionReason, SimpleRejection,
    SimpleRejectionReason,
};
use crate::timing_allow_origin::TimingAllowOrigin;

fn build_request(
    method: &'static str,
    origin: Option<&'static str>,
    acrm: Option<&'static str>,
    acrh: Option<&'static str>,
    private_network: bool,
) -> RequestContext<'static> {
    RequestContext {
        method,
        origin,
        access_control_request_method: acrm,
        access_control_request_headers: acrh,
        access_control_request_private_network: private_network,
    }
}

fn request(
    method: &'static str,
    origin: Option<&'static str>,
    acrm: Option<&'static str>,
    acrh: Option<&'static str>,
) -> RequestContext<'static> {
    build_request(method, origin, acrm, acrh, false)
}

fn request_with_private_network(
    method: &'static str,
    origin: Option<&'static str>,
    acrm: Option<&'static str>,
    acrh: Option<&'static str>,
) -> RequestContext<'static> {
    build_request(method, origin, acrm, acrh, true)
}

fn preflight_decision(
    cors: &Cors,
    request: &RequestContext<'static>,
) -> Result<CorsDecision, CorsError> {
    let normalized_request = NormalizedRequest::new(request);
    let normalized = normalized_request.as_context();
    cors.process_preflight(request, &normalized)
}

fn simple_decision(
    cors: &Cors,
    request: &RequestContext<'static>,
) -> Result<CorsDecision, CorsError> {
    let normalized_request = NormalizedRequest::new(request);
    let normalized = normalized_request.as_context();
    cors.process_simple(request, &normalized)
}

fn cors_with(options: CorsOptions) -> Cors {
    let options = options
        .methods(AllowedMethods::list(["GET"]))
        .allowed_headers(AllowedHeaders::list(["X-Test"]))
        .exposed_headers(ExposedHeaders::list(["X-Test"]));

    Cors::new(options).expect("valid CORS configuration")
}

fn expect_preflight_accepted(result: Result<CorsDecision, CorsError>) -> Headers {
    match result.expect("preflight evaluation should succeed") {
        CorsDecision::PreflightAccepted { headers } => headers,
        other => panic!("expected preflight acceptance, got {:?}", other),
    }
}

fn expect_preflight_rejected(result: Result<CorsDecision, CorsError>) -> PreflightRejection {
    match result.expect("preflight evaluation should succeed") {
        CorsDecision::PreflightRejected(rejection) => rejection,
        other => panic!("expected preflight rejection, got {:?}", other),
    }
}

fn expect_simple_accepted(result: Result<CorsDecision, CorsError>) -> Headers {
    match result.expect("simple evaluation should succeed") {
        CorsDecision::SimpleAccepted { headers } => headers,
        other => panic!("expected simple acceptance, got {:?}", other),
    }
}

fn expect_simple_rejected(result: Result<CorsDecision, CorsError>) -> SimpleRejection {
    match result.expect("simple evaluation should succeed") {
        CorsDecision::SimpleRejected(rejection) => rejection,
        other => panic!("expected simple rejection, got {:?}", other),
    }
}

fn expect_not_applicable(result: Result<CorsDecision, CorsError>) {
    match result.expect("evaluation should succeed") {
        CorsDecision::NotApplicable => {}
        other => panic!("expected not applicable decision, got {:?}", other),
    }
}

mod new {
    use super::*;

    #[test]
    fn should_build_instance_when_options_valid_then_allow_simple_checks() {
        let cors = cors_with(CorsOptions::new());
        let request = request("GET", Some("https://allowed.test"), None, None);

        let decision = cors.check(&request).expect("cors evaluation succeeded");

        assert!(matches!(decision, CorsDecision::SimpleAccepted { .. }));
    }

    #[test]
    fn should_reject_any_origin_with_credentials_when_new_called_then_return_error() {
        let mut options = CorsOptions::new().origin(Origin::any());
        options.credentials = true;

        let result = Cors::new(options);

        assert!(matches!(
            result,
            Err(ValidationError::CredentialsRequireSpecificOrigin)
        ));
    }
}

mod check {
    use super::*;

    #[test]
    fn should_return_preflight_decision_when_request_uses_options_then_emit_preflight_variant() {
        let cors = cors_with(CorsOptions::new());
        let request = request(
            "OPTIONS",
            Some("https://allowed.test"),
            Some("GET"),
            Some("X-Test"),
        );

        let decision = cors
            .check(&request)
            .expect("cors evaluation should succeed");

        assert!(matches!(decision, CorsDecision::PreflightAccepted { .. }));
    }

    #[test]
    fn should_return_simple_decision_when_request_not_options_then_emit_simple_variant() {
        let cors = cors_with(CorsOptions::new());
        let request = request("GET", Some("https://allowed.test"), None, None);

        let decision = cors
            .check(&request)
            .expect("cors evaluation should succeed");

        assert!(matches!(decision, CorsDecision::SimpleAccepted { .. }));
    }

    #[test]
    fn should_return_simple_rejection_when_origin_disallowed_then_emit_rejection_variant() {
        let cors = cors_with(CorsOptions::new().origin(Origin::list(["https://allowed.test"])));
        let request = request("GET", Some("https://denied.test"), None, None);

        let decision = cors
            .check(&request)
            .expect("cors evaluation should succeed");

        assert!(matches!(decision, CorsDecision::SimpleRejected(_)));
    }

    #[test]
    fn should_return_not_applicable_when_origin_handler_skips_then_stop_processing() {
        let cors =
            cors_with(CorsOptions::new().origin(Origin::custom(|_, _| OriginDecision::Skip)));
        let request = request(
            "OPTIONS",
            Some("https://skip.test"),
            Some("GET"),
            Some("X-Test"),
        );

        let decision = cors
            .check(&request)
            .expect("cors evaluation should succeed");

        assert!(matches!(decision, CorsDecision::NotApplicable));
    }
}

mod process_preflight {
    use super::*;

    #[test]
    fn should_return_not_applicable_when_request_method_missing_then_skip_preflight_flow() {
        let cors = Cors::new(CorsOptions::new()).expect("valid CORS configuration");
        let request = request(
            "OPTIONS",
            Some("https://allowed.test"),
            None,
            Some("X-Test"),
        );

        expect_not_applicable(preflight_decision(&cors, &request));
    }

    #[test]
    fn should_return_not_applicable_when_origin_handler_skips_then_stop_evaluation() {
        let cors =
            cors_with(CorsOptions::new().origin(Origin::custom(|_, _| OriginDecision::Skip)));
        let request = request(
            "OPTIONS",
            Some("https://denied.test"),
            Some("GET"),
            Some("X-Test"),
        );

        expect_not_applicable(preflight_decision(&cors, &request));
    }

    #[test]
    fn should_return_origin_not_allowed_when_origin_rejected_then_include_vary_header() {
        let cors = cors_with(CorsOptions::new().origin(Origin::list(["https://allowed.test"])));
        let request = request(
            "OPTIONS",
            Some("https://blocked.test"),
            Some("GET"),
            Some("X-Test"),
        );

        let rejection = expect_preflight_rejected(preflight_decision(&cors, &request));

        assert_eq!(rejection.reason, PreflightRejectionReason::OriginNotAllowed);
        assert!(rejection.headers.contains_key(header::VARY));
    }

    #[test]
    fn should_return_method_not_allowed_when_request_method_disallowed_then_report_method() {
        let cors = Cors::new(
            CorsOptions::new()
                .origin(Origin::any())
                .methods(AllowedMethods::list(["GET", "POST"])),
        )
        .expect("valid CORS configuration");
        let request = request("OPTIONS", Some("https://allowed.test"), Some("PATCH"), None);

        let rejection = expect_preflight_rejected(preflight_decision(&cors, &request));

        assert_eq!(
            rejection.reason,
            PreflightRejectionReason::MethodNotAllowed {
                requested_method: "patch".to_string(),
            }
        );
    }

    #[test]
    fn should_return_headers_not_allowed_when_request_headers_disallowed_then_report_headers() {
        let cors = Cors::new(
            CorsOptions::new()
                .origin(Origin::any())
                .allowed_headers(AllowedHeaders::list(["X-Allowed"])),
        )
        .expect("valid CORS configuration");
        let request = request(
            "OPTIONS",
            Some("https://allowed.test"),
            Some("GET"),
            Some("X-Forbidden"),
        );

        let rejection = expect_preflight_rejected(preflight_decision(&cors, &request));

        assert_eq!(
            rejection.reason,
            PreflightRejectionReason::HeadersNotAllowed {
                requested_headers: "x-forbidden".to_string(),
            }
        );
    }

    #[test]
    fn should_attach_expected_headers_when_origin_allowed_then_accept_preflight_request() {
        let cors = cors_with(CorsOptions::new().origin(Origin::any()).max_age(600));
        let request = request(
            "OPTIONS",
            Some("https://allowed.test"),
            Some("GET"),
            Some("X-Test"),
        );

        let headers = expect_preflight_accepted(preflight_decision(&cors, &request));

        assert!(headers.contains_key(header::ACCESS_CONTROL_ALLOW_METHODS));
        assert!(headers.contains_key(header::ACCESS_CONTROL_ALLOW_HEADERS));
        assert!(headers.contains_key(header::ACCESS_CONTROL_MAX_AGE));
    }

    #[test]
    fn should_emit_private_network_header_when_request_allows_private_network_then_include_flag() {
        let cors = cors_with(
            CorsOptions::new()
                .allow_private_network(true)
                .credentials(true)
                .origin(Origin::list(["https://intranet.test"])),
        );
        let request = request_with_private_network(
            "OPTIONS",
            Some("https://intranet.test"),
            Some("GET"),
            Some("X-Test"),
        );

        let headers = expect_preflight_accepted(preflight_decision(&cors, &request));

        assert_eq!(
            headers.get(header::ACCESS_CONTROL_ALLOW_PRIVATE_NETWORK),
            Some(&"true".to_string())
        );
    }
}

mod process_simple {
    use super::*;

    #[test]
    fn should_return_not_applicable_when_origin_handler_skips_then_stop_simple_flow() {
        let cors =
            cors_with(CorsOptions::new().origin(Origin::custom(|_, _| OriginDecision::Skip)));
        let request = request("GET", Some("https://denied.test"), None, None);

        expect_not_applicable(simple_decision(&cors, &request));
    }

    #[test]
    fn should_return_not_applicable_when_method_not_allowed_then_skip_simple_flow() {
        let cors = Cors::new(CorsOptions::new().methods(AllowedMethods::list(["POST"])))
            .expect("valid CORS configuration");
        let request = request("GET", Some("https://allowed.test"), None, None);

        expect_not_applicable(simple_decision(&cors, &request));
    }

    #[test]
    fn should_return_error_when_origin_any_with_credentials_then_reject_simple_configuration() {
        let cors = Cors::new(
            CorsOptions::new()
                .origin(Origin::custom(|_, _| OriginDecision::Any))
                .credentials(true),
        )
        .expect("valid CORS configuration");
        let request = request("GET", Some("https://wild.test"), None, None);

        let error = simple_decision(&cors, &request)
            .expect_err("simple request should reject any origin when credentials required");

        assert!(matches!(error, CorsError::InvalidOriginAnyWithCredentials));
    }

    #[test]
    fn should_emit_vary_without_allow_origin_when_origin_disallowed_then_return_vary_header() {
        let cors = Cors::new(CorsOptions::new().origin(Origin::list(["https://allowed.test"])))
            .expect("valid CORS configuration");
        let request = request("GET", Some("https://denied.test"), None, None);

        let rejection = expect_simple_rejected(simple_decision(&cors, &request));

        assert_eq!(rejection.reason, SimpleRejectionReason::OriginNotAllowed);
        let headers = rejection.headers;

        assert_eq!(headers.get(header::VARY), Some(&"Origin".to_string()));
        assert!(!headers.contains_key(header::ACCESS_CONTROL_ALLOW_ORIGIN));
    }

    #[test]
    fn should_emit_credentials_header_when_origin_allowed_with_credentials_then_include_header() {
        let cors = Cors::new(
            CorsOptions::new()
                .origin(Origin::list(["https://allowed.test"]))
                .credentials(true),
        )
        .expect("valid CORS configuration");
        let request = request("GET", Some("https://allowed.test"), None, None);

        let headers = expect_simple_accepted(simple_decision(&cors, &request));

        assert_eq!(
            headers.get(header::ACCESS_CONTROL_ALLOW_ORIGIN),
            Some(&"https://allowed.test".to_string())
        );
        assert_eq!(
            headers.get(header::ACCESS_CONTROL_ALLOW_CREDENTIALS),
            Some(&"true".to_string())
        );
    }

    #[test]
    fn should_emit_timing_allow_origin_when_configuration_allows_then_return_wildcard_value() {
        let cors = Cors::new(CorsOptions::new().timing_allow_origin(TimingAllowOrigin::Any))
            .expect("valid CORS configuration");
        let request = request("GET", Some("https://allowed.test"), None, None);

        let headers = expect_simple_accepted(simple_decision(&cors, &request));

        assert_eq!(
            headers.get(header::TIMING_ALLOW_ORIGIN),
            Some(&"*".to_string())
        );
    }
}
