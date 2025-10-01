use super::*;
use crate::Headers;
use crate::allowed_headers::AllowedHeaders;
use crate::allowed_methods::AllowedMethods;
use crate::constants::header;
use crate::context::RequestContext;
use crate::normalized_request::NormalizedRequest;
use crate::options::{CorsOptions, ValidationError};
use crate::origin::{Origin, OriginDecision};
use crate::result::{CorsDecision, CorsError, PreflightRejection, PreflightRejectionReason};
use crate::timing_allow_origin::TimingAllowOrigin;

fn build_request(
    method: &'static str,
    origin: &'static str,
    acrm: &'static str,
    acrh: &'static str,
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
    origin: &'static str,
    acrm: &'static str,
    acrh: &'static str,
) -> RequestContext<'static> {
    build_request(method, origin, acrm, acrh, false)
}

fn request_with_private_network(
    method: &'static str,
    origin: &'static str,
    acrm: &'static str,
    acrh: &'static str,
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
    Cors::new(CorsOptions {
        methods: AllowedMethods::list(["GET"]),
        allowed_headers: AllowedHeaders::list(["X-Test"]),
        exposed_headers: Some(vec!["X-Test".into()]),
        ..options
    })
    .expect("valid CORS configuration")
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

fn expect_not_applicable(result: Result<CorsDecision, CorsError>) {
    match result.expect("evaluation should succeed") {
        CorsDecision::NotApplicable => {}
        other => panic!("expected not applicable decision, got {:?}", other),
    }
}

mod new {
    use super::*;

    #[test]
    fn simple_request_should_return_simple_decision() {
        // Arrange
        let options = CorsOptions {
            origin: Origin::any(),
            ..CorsOptions::default()
        };
        let cors = cors_with(options);
        let request = request("GET", "https://allowed.test", "", "");

        // Act
        let decision = cors.check(&request).expect("cors evaluation succeeded");

        // Assert
        match decision {
            CorsDecision::SimpleAccepted { headers } => {
                assert!(headers.contains_key(header::ACCESS_CONTROL_ALLOW_ORIGIN));
            }
            other => panic!("expected simple acceptance, got {:?}", other),
        }
    }

    #[test]
    fn new_should_reject_wildcard_origin_with_credentials() {
        // Arrange
        let options = CorsOptions {
            origin: Origin::any(),
            credentials: true,
            ..CorsOptions::default()
        };

        // Act
        let result = Cors::new(options);

        // Assert
        assert!(matches!(
            result,
            Err(ValidationError::CredentialsRequireSpecificOrigin)
        ));
    }
}

mod check {
    use super::*;

    #[test]
    fn given_any_origin_when_preflight_request_then_return_preflight_decision() {
        // Arrange
        let options = CorsOptions {
            origin: Origin::any(),
            ..CorsOptions::default()
        };
        let cors = cors_with(options);
        let request = request("OPTIONS", "https://allowed.test", "GET", "X-Test");

        // Act
        match cors
            .check(&request)
            .expect("cors evaluation should succeed")
        {
            CorsDecision::PreflightAccepted { headers } => {
                // Assert
                assert!(headers.contains_key(header::ACCESS_CONTROL_ALLOW_ORIGIN));
            }
            other => panic!("expected preflight acceptance, got {:?}", other),
        }
    }

    #[test]
    fn given_any_origin_when_simple_request_then_return_simple_decision() {
        // Arrange
        let options = CorsOptions {
            origin: Origin::any(),
            ..CorsOptions::default()
        };
        let cors = cors_with(options);
        let request = request("GET", "https://allowed.test", "", "");

        // Act
        match cors
            .check(&request)
            .expect("cors evaluation should succeed")
        {
            CorsDecision::SimpleAccepted { headers } => {
                // Assert
                assert!(headers.contains_key(header::ACCESS_CONTROL_ALLOW_ORIGIN));
            }
            other => panic!("expected simple acceptance, got {:?}", other),
        }
    }

    #[test]
    fn should_return_not_applicable_given_origin_skips_processing_when_check() {
        // Arrange
        let options = CorsOptions {
            origin: Origin::custom(|_, _| OriginDecision::Skip),
            ..CorsOptions::default()
        };
        let cors = cors_with(options);
        let request = request("OPTIONS", "https://denied.test", "GET", "X-Test");

        // Act
        let decision = cors
            .check(&request)
            .expect("cors evaluation should succeed");

        // Assert
        assert!(matches!(decision, CorsDecision::NotApplicable));
    }

    #[test]
    fn should_return_not_applicable_given_origin_disabled_when_check() {
        // Arrange
        let options = CorsOptions {
            origin: Origin::disabled(),
            ..CorsOptions::default()
        };
        let cors = cors_with(options);
        let request = request("OPTIONS", "https://denied.test", "GET", "X-Test");

        // Act
        let decision = cors
            .check(&request)
            .expect("cors evaluation should succeed");

        // Assert
        assert!(matches!(decision, CorsDecision::NotApplicable));
    }
}

mod process_preflight {
    use super::*;

    #[test]
    fn should_return_not_applicable_given_origin_skip_when_preflight_request() {
        // Arrange
        let options = CorsOptions {
            origin: Origin::custom(|_, _| OriginDecision::Skip),
            ..CorsOptions::default()
        };
        let cors = cors_with(options);
        let original = request("OPTIONS", "https://denied.test", "GET", "X-Test");

        // Act
        expect_not_applicable(preflight_decision(&cors, &original));
    }

    #[test]
    fn should_report_rejection_reason_given_disallowed_origin_when_preflight_request() {
        // Arrange
        let options = CorsOptions {
            origin: Origin::list(["https://allowed.test"]),
            ..CorsOptions::default()
        };
        let cors = cors_with(options);
        let original = request("OPTIONS", "https://denied.test", "GET", "X-Test");

        // Act
        let rejection = expect_preflight_rejected(preflight_decision(&cors, &original));
        // Assert
        assert_eq!(rejection.reason, PreflightRejectionReason::OriginNotAllowed);
        assert!(rejection.headers.contains_key(header::VARY));
    }

    #[test]
    fn should_fail_given_origin_returns_any_with_credentials_when_preflight_request() {
        // Arrange
        let cors = Cors::new(CorsOptions {
            origin: Origin::custom(|_, _| OriginDecision::Any),
            credentials: true,
            ..CorsOptions::default()
        })
        .expect("valid CORS configuration");
        let original = request("OPTIONS", "https://wild.test", "GET", "");

        // Act
        let error = preflight_decision(&cors, &original)
            .expect_err("preflight should reject any origin when credentials required");

        // Assert
        assert!(matches!(error, CorsError::InvalidOriginAnyWithCredentials));
    }

    #[test]
    fn should_aggregate_expected_headers_given_allowed_origin_when_preflight_request() {
        // Arrange
        let options = CorsOptions {
            origin: Origin::any(),
            max_age: Some("600".into()),
            ..CorsOptions::default()
        };
        let cors = cors_with(options);
        let original = request("OPTIONS", "https://allowed.test", "GET", "X-Test");

        // Act
        let headers = expect_preflight_accepted(preflight_decision(&cors, &original));
        assert!(headers.contains_key(header::ACCESS_CONTROL_ALLOW_METHODS));
        assert!(headers.contains_key(header::ACCESS_CONTROL_ALLOW_HEADERS));
        assert!(!headers.contains_key(header::ACCESS_CONTROL_EXPOSE_HEADERS));
        assert!(headers.contains_key(header::ACCESS_CONTROL_MAX_AGE));
    }

    #[test]
    fn should_return_not_applicable_given_missing_request_method_when_preflight_request() {
        // Arrange
        let cors = Cors::new(CorsOptions::default()).expect("valid CORS configuration");
        let original = request("OPTIONS", "https://allowed.test", "", "X-Test");

        // Act
        expect_not_applicable(preflight_decision(&cors, &original));
    }

    #[test]
    fn should_reject_given_not_allowed_request_headers_when_preflight_request() {
        // Arrange
        let cors = Cors::new(CorsOptions {
            origin: Origin::any(),
            allowed_headers: AllowedHeaders::list(["X-Allowed"]),
            ..CorsOptions::default()
        })
        .expect("valid CORS configuration");
        let original = request("OPTIONS", "https://allowed.test", "GET", "X-Forbidden");

        // Act
        let rejection = expect_preflight_rejected(preflight_decision(&cors, &original));

        // Assert
        assert_eq!(
            rejection.reason,
            PreflightRejectionReason::HeadersNotAllowed {
                requested_headers: "x-forbidden".to_string(),
            }
        );
    }

    #[test]
    fn should_reject_given_not_allowed_request_method_when_preflight_request() {
        // Arrange
        let cors = Cors::new(CorsOptions {
            origin: Origin::any(),
            methods: AllowedMethods::list(["GET", "POST"]),
            ..CorsOptions::default()
        })
        .expect("valid CORS configuration");
        let original = request("OPTIONS", "https://allowed.test", "DELETE", "");

        // Act
        let rejection = expect_preflight_rejected(preflight_decision(&cors, &original));

        // Assert
        assert_eq!(
            rejection.reason,
            PreflightRejectionReason::MethodNotAllowed {
                requested_method: "delete".to_string(),
            }
        );
    }

    #[test]
    fn should_emit_wildcard_header_given_allowed_headers_any_when_preflight_request() {
        // Arrange
        let cors = Cors::new(CorsOptions {
            origin: Origin::any(),
            allowed_headers: AllowedHeaders::any(),
            ..CorsOptions::default()
        })
        .expect("valid CORS configuration");
        let original = request("OPTIONS", "https://allowed.test", "GET", "X-Anything");

        // Act
        let headers = expect_preflight_accepted(preflight_decision(&cors, &original));
        // Assert
        assert_eq!(
            headers.get(header::ACCESS_CONTROL_ALLOW_HEADERS),
            Some(&"*".to_string())
        );
    }

    #[test]
    fn should_omit_header_given_max_age_not_configured_when_preflight_request() {
        // Arrange
        let cors = Cors::new(CorsOptions {
            origin: Origin::any(),
            max_age: None,
            allowed_headers: AllowedHeaders::any(),
            ..CorsOptions::default()
        })
        .expect("valid CORS configuration");
        let original = request("OPTIONS", "https://allowed.test", "GET", "X-Test");

        // Act
        let headers = expect_preflight_accepted(preflight_decision(&cors, &original));
        // Assert
        assert!(!headers.contains_key(header::ACCESS_CONTROL_MAX_AGE));
    }

    #[test]
    fn should_emit_private_network_header_given_private_network_requested_when_preflight_request() {
        // Arrange
        let cors = Cors::new(CorsOptions {
            allow_private_network: true,
            credentials: true,
            origin: Origin::list(["https://intranet.test"]),
            ..CorsOptions::default()
        })
        .expect("valid CORS configuration");
        let original = request_with_private_network("OPTIONS", "https://intranet.test", "GET", "");

        // Act
        let headers = expect_preflight_accepted(preflight_decision(&cors, &original));
        // Assert
        assert_eq!(
            headers.get(header::ACCESS_CONTROL_ALLOW_PRIVATE_NETWORK),
            Some(&"true".to_string())
        );
    }

    #[test]
    fn should_not_emit_header_given_private_network_disabled_when_preflight_request() {
        // Arrange
        let cors = Cors::new(CorsOptions::default()).expect("valid CORS configuration");
        let original = request_with_private_network("OPTIONS", "https://intranet.test", "GET", "");

        // Act
        let headers = expect_preflight_accepted(preflight_decision(&cors, &original));
        // Assert
        assert!(!headers.contains_key(header::ACCESS_CONTROL_ALLOW_PRIVATE_NETWORK));
    }

    #[test]
    fn should_not_emit_header_given_timing_allow_origin_configured_when_preflight_request() {
        // Arrange
        let cors = Cors::new(CorsOptions {
            timing_allow_origin: Some(TimingAllowOrigin::list([
                "https://metrics.test",
                "https://dash.test",
            ])),
            allowed_headers: AllowedHeaders::any(),
            ..CorsOptions::default()
        })
        .expect("valid CORS configuration");
        let original = request("OPTIONS", "https://allowed.test", "GET", "X-Test");

        // Act
        let headers = expect_preflight_accepted(preflight_decision(&cors, &original));
        // Assert
        assert!(
            !headers.contains_key(header::TIMING_ALLOW_ORIGIN),
            "expected Timing-Allow-Origin to be omitted on preflight responses"
        );
    }
}

mod process_simple {
    use super::*;

    #[test]
    fn should_return_not_applicable_given_origin_skip_when_simple_request() {
        // Arrange
        let options = CorsOptions {
            origin: Origin::custom(|_, _| OriginDecision::Skip),
            ..CorsOptions::default()
        };
        let cors = cors_with(options);
        let original = request("GET", "https://denied.test", "", "");

        // Act
        expect_not_applicable(simple_decision(&cors, &original));
    }

    #[test]
    fn should_return_not_applicable_given_method_not_allowed_when_simple_request() {
        // Arrange
        let cors = Cors::new(CorsOptions {
            methods: AllowedMethods::list(["POST"]),
            ..CorsOptions::default()
        })
        .expect("valid CORS configuration");
        let original = request("GET", "https://allowed.test", "", "");

        // Act
        expect_not_applicable(simple_decision(&cors, &original));
    }

    #[test]
    fn should_fail_given_origin_returns_any_with_credentials_when_simple_request() {
        // Arrange
        let cors = Cors::new(CorsOptions {
            origin: Origin::custom(|_, _| OriginDecision::Any),
            credentials: true,
            ..CorsOptions::default()
        })
        .expect("valid CORS configuration");
        let original = request("GET", "https://wild.test", "", "");

        // Act
        let error = simple_decision(&cors, &original)
            .expect_err("simple request should reject any origin when credentials required");

        // Assert
        assert!(matches!(error, CorsError::InvalidOriginAnyWithCredentials));
    }

    #[test]
    fn should_emit_vary_without_allow_origin_given_disallowed_origin_when_simple_request() {
        let cors = Cors::new(CorsOptions {
            origin: Origin::list(["https://allowed.test"]),
            ..CorsOptions::default()
        })
        .expect("valid CORS configuration");
        let original = request("GET", "https://denied.test", "", "");

        let headers = expect_simple_accepted(simple_decision(&cors, &original));

        assert_eq!(headers.get(header::VARY), Some(&"Origin".to_string()));
        assert!(!headers.contains_key(header::ACCESS_CONTROL_ALLOW_ORIGIN));
    }

    #[test]
    fn should_emit_simple_headers_given_allowed_origin_when_simple_request() {
        // Arrange
        let cors = Cors::new(CorsOptions {
            origin: Origin::list(["https://allowed.test"]),
            credentials: true,
            ..CorsOptions::default()
        })
        .expect("valid CORS configuration");
        let original = request("GET", "https://allowed.test", "", "");

        // Act
        let headers = expect_simple_accepted(simple_decision(&cors, &original));
        // Assert
        assert!(headers.contains_key(header::ACCESS_CONTROL_ALLOW_ORIGIN));
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
    fn should_not_emit_credentials_header_given_credentials_disabled_when_simple_request() {
        // Arrange
        let cors = Cors::new(CorsOptions::default()).expect("valid CORS configuration");
        let original = request("GET", "https://allowed.test", "", "");

        // Act
        let headers = expect_simple_accepted(simple_decision(&cors, &original));
        // Assert
        assert!(headers.contains_key(header::ACCESS_CONTROL_ALLOW_ORIGIN));
        assert!(!headers.contains_key(header::ACCESS_CONTROL_ALLOW_CREDENTIALS));
    }

    #[test]
    fn should_mirror_request_origin_given_origin_list_used_when_simple_request() {
        // Arrange
        let cors = Cors::new(CorsOptions {
            origin: Origin::list(["https://allowed.test"]),
            credentials: true,
            ..CorsOptions::default()
        })
        .expect("valid CORS configuration");
        let original = request("GET", "https://allowed.test", "", "");

        // Act
        let headers = expect_simple_accepted(simple_decision(&cors, &original));
        // Assert
        assert_eq!(
            headers.get(header::ACCESS_CONTROL_ALLOW_ORIGIN),
            Some(&"https://allowed.test".to_string())
        );
        assert!(headers.contains_key(header::ACCESS_CONTROL_ALLOW_CREDENTIALS));
    }

    #[test]
    fn should_not_emit_header_on_simple_response_given_private_network_allowed_when_simple_request()
    {
        // Arrange
        let cors = Cors::new(CorsOptions {
            allow_private_network: true,
            credentials: true,
            origin: Origin::list(["https://intranet.test"]),
            ..CorsOptions::default()
        })
        .expect("valid CORS configuration");
        let original = request("GET", "https://intranet.test", "", "");

        // Act
        let headers = expect_simple_accepted(simple_decision(&cors, &original));
        // Assert
        assert!(!headers.contains_key(header::ACCESS_CONTROL_ALLOW_PRIVATE_NETWORK));
    }

    #[test]
    fn should_emit_header_given_timing_allow_origin_configured_when_simple_request() {
        // Arrange
        let cors = Cors::new(CorsOptions {
            timing_allow_origin: Some(TimingAllowOrigin::any()),
            ..CorsOptions::default()
        })
        .expect("valid CORS configuration");
        let original = request("GET", "https://allowed.test", "", "");

        // Act
        let headers = expect_simple_accepted(simple_decision(&cors, &original));
        // Assert
        assert_eq!(
            headers.get(header::TIMING_ALLOW_ORIGIN),
            Some(&"*".to_string())
        );
    }
}
