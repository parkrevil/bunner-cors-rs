use super::*;
use crate::allowed_headers::AllowedHeaders;
use crate::allowed_methods::AllowedMethods;
use crate::constants::header;
use crate::context::RequestContext;
use crate::normalized_request::NormalizedRequest;
use crate::options::CorsOptions;
use crate::origin::{Origin, OriginDecision};
use crate::result::{CorsDecision, PreflightResult, SimpleResult};
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

fn preflight_result(cors: &Cors, request: &RequestContext<'static>) -> Option<PreflightResult> {
    let normalized_request = NormalizedRequest::new(request);
    let normalized = normalized_request.as_context();
    cors.process_preflight(request, &normalized)
}

fn simple_result(cors: &Cors, request: &RequestContext<'static>) -> Option<SimpleResult> {
    let normalized_request = NormalizedRequest::new(request);
    let normalized = normalized_request.as_context();
    cors.process_simple(request, &normalized)
}

fn cors_with(options: CorsOptions) -> Cors {
    Cors::new(CorsOptions {
        methods: AllowedMethods::list(["GET"]),
        allowed_headers: AllowedHeaders::list(["X-Test"]),
        exposed_headers: Some(vec!["X-Test".into()]),
        credentials: true,
        ..options
    })
}

mod new {

    #[test]
    fn when_preflight_continue_enabled_simple_request_should_return_simple_decision() {
        // Arrange
        let options = CorsOptions {
            origin: Origin::any(),
            preflight_continue: true,
            ..CorsOptions::default()
        };
        let cors = cors_with(options);
        let request = request("GET", "https://allowed.test", "", "");

        // Act
        let decision = cors.check(&request);

        // Assert
        match decision {
            CorsDecision::Simple(result) => {
                assert!(
                    result
                        .headers
                        .contains_key(header::ACCESS_CONTROL_ALLOW_ORIGIN)
                );
            }
            _ => panic!("expected simple decision"),
        }
    }
    use super::*;

    #[test]
    fn when_constructed_with_custom_status_should_use_it() {
        // Arrange
        let options = CorsOptions {
            options_success_status: 208,
            ..CorsOptions::default()
        };
        let cors = Cors::new(options);
        let request = request("OPTIONS", "https://allowed.test", "GET", "X-Test");

        // Act
        let decision = cors.check(&request);

        // Assert
        match decision {
            CorsDecision::Preflight(result) => assert_eq!(result.status, 208),
            _ => panic!("expected preflight decision"),
        }
    }
}

mod check {
    use super::*;

    #[test]
    fn when_preflight_request_should_return_preflight_decision() {
        // Arrange
        let options = CorsOptions {
            origin: Origin::any(),
            ..CorsOptions::default()
        };
        let cors = cors_with(options);
        let request = request("OPTIONS", "https://allowed.test", "GET", "X-Test");

        // Act
        let decision = cors.check(&request);

        // Assert
        match decision {
            CorsDecision::Preflight(result) => {
                assert_eq!(result.status, 204);
                assert!(
                    result
                        .headers
                        .contains_key(header::ACCESS_CONTROL_ALLOW_ORIGIN)
                );
            }
            _ => panic!("expected preflight decision"),
        }
    }

    #[test]
    fn when_simple_request_should_return_simple_decision() {
        // Arrange
        let options = CorsOptions {
            origin: Origin::any(),
            ..CorsOptions::default()
        };
        let cors = cors_with(options);
        let request = request("GET", "https://allowed.test", "", "");

        // Act
        let decision = cors.check(&request);

        // Assert
        match decision {
            CorsDecision::Simple(result) => {
                assert!(
                    result
                        .headers
                        .contains_key(header::ACCESS_CONTROL_ALLOW_ORIGIN)
                );
            }
            _ => panic!("expected simple decision"),
        }
    }

    #[test]
    fn when_origin_skips_processing_should_return_not_applicable() {
        // Arrange
        let options = CorsOptions {
            origin: Origin::custom(|_, _| OriginDecision::Skip),
            ..CorsOptions::default()
        };
        let cors = cors_with(options);
        let request = request("OPTIONS", "https://denied.test", "GET", "X-Test");

        // Act
        let decision = cors.check(&request);

        // Assert
        assert!(matches!(decision, CorsDecision::NotApplicable));
    }

    #[test]
    fn when_origin_disabled_should_return_not_applicable() {
        // Arrange
        let options = CorsOptions {
            origin: Origin::disabled(),
            ..CorsOptions::default()
        };
        let cors = cors_with(options);
        let request = request("OPTIONS", "https://denied.test", "GET", "X-Test");

        // Act
        let decision = cors.check(&request);

        // Assert
        assert!(matches!(decision, CorsDecision::NotApplicable));
    }
}

mod process_preflight {
    use super::*;

    #[test]
    fn when_origin_skip_should_return_none() {
        // Arrange
        let options = CorsOptions {
            origin: Origin::custom(|_, _| OriginDecision::Skip),
            ..CorsOptions::default()
        };
        let cors = cors_with(options);
        let original = request("OPTIONS", "https://denied.test", "GET", "X-Test");

        // Act
        let result = preflight_result(&cors, &original);

        // Assert
        assert!(result.is_none());
    }

    #[test]
    fn when_origin_allowed_should_aggregate_expected_headers() {
        // Arrange
        let options = CorsOptions {
            origin: Origin::any(),
            max_age: Some("600".into()),
            preflight_continue: true,
            ..CorsOptions::default()
        };
        let cors = cors_with(options);
        let original = request("OPTIONS", "https://allowed.test", "GET", "X-Test");

        // Act
        let result = preflight_result(&cors, &original).expect("expected preflight result");

        // Assert
        assert_eq!(result.status, 204);
        assert!(!result.end_response);
        assert!(
            result
                .headers
                .contains_key(header::ACCESS_CONTROL_ALLOW_METHODS)
        );
        assert!(
            result
                .headers
                .contains_key(header::ACCESS_CONTROL_ALLOW_HEADERS)
        );
        assert!(
            result
                .headers
                .contains_key(header::ACCESS_CONTROL_EXPOSE_HEADERS)
        );
        assert!(result.headers.contains_key(header::ACCESS_CONTROL_MAX_AGE));
    }

    #[test]
    fn when_request_method_missing_should_still_emit_default_methods() {
        // Arrange
        let cors = Cors::new(CorsOptions::default());
        let original = request("OPTIONS", "https://allowed.test", "", "X-Test");

        // Act
        let result = preflight_result(&cors, &original).expect("expected preflight result");

        // Assert
        assert_eq!(
            result.headers.get(header::ACCESS_CONTROL_ALLOW_METHODS),
            Some(&"GET,HEAD,PUT,PATCH,POST,DELETE".to_string())
        );
    }

    #[test]
    fn when_request_headers_not_allowed_should_return_none() {
        // Arrange
        let cors = Cors::new(CorsOptions {
            origin: Origin::any(),
            allowed_headers: AllowedHeaders::list(["X-Allowed"]),
            ..CorsOptions::default()
        });
        let original = request("OPTIONS", "https://allowed.test", "GET", "X-Forbidden");

        // Act
        let result = preflight_result(&cors, &original);

        // Assert
        assert!(result.is_none());
    }

    #[test]
    fn when_preflight_continue_disabled_should_end_response() {
        // Arrange
        let cors = Cors::new(CorsOptions::default());
        let original = request("OPTIONS", "https://allowed.test", "GET", "X-Test");

        // Act
        let result = preflight_result(&cors, &original).expect("expected preflight result");

        // Assert
        assert!(result.end_response);
    }

    #[test]
    fn when_allowed_methods_any_should_emit_wildcard_header() {
        // Arrange
        let cors = Cors::new(CorsOptions {
            origin: Origin::any(),
            methods: AllowedMethods::any(),
            ..CorsOptions::default()
        });
        let original = request("OPTIONS", "https://allowed.test", "DELETE", "");

        // Act
        let result = preflight_result(&cors, &original).expect("expected preflight result");

        // Assert
        assert_eq!(
            result.headers.get(header::ACCESS_CONTROL_ALLOW_METHODS),
            Some(&"*".to_string())
        );
    }

    #[test]
    fn when_request_method_not_allowed_should_return_none() {
        // Arrange
        let cors = Cors::new(CorsOptions {
            origin: Origin::any(),
            methods: AllowedMethods::list(["GET", "POST"]),
            ..CorsOptions::default()
        });
        let original = request("OPTIONS", "https://allowed.test", "DELETE", "");

        // Act
        let result = preflight_result(&cors, &original);

        // Assert
        assert!(result.is_none());
    }

    #[test]
    fn when_allowed_headers_any_should_emit_wildcard_header() {
        // Arrange
        let cors = Cors::new(CorsOptions {
            origin: Origin::any(),
            allowed_headers: AllowedHeaders::any(),
            ..CorsOptions::default()
        });
        let original = request("OPTIONS", "https://allowed.test", "GET", "X-Anything");

        // Act
        let result = preflight_result(&cors, &original).expect("expected preflight result");

        // Assert
        assert_eq!(
            result.headers.get(header::ACCESS_CONTROL_ALLOW_HEADERS),
            Some(&"*".to_string())
        );
    }

    #[test]
    fn when_max_age_not_configured_should_omit_header() {
        // Arrange
        let cors = Cors::new(CorsOptions {
            origin: Origin::any(),
            max_age: None,
            ..CorsOptions::default()
        });
        let original = request("OPTIONS", "https://allowed.test", "GET", "X-Test");

        // Act
        let result = preflight_result(&cors, &original).expect("expected preflight result");

        // Assert
        assert!(!result.headers.contains_key(header::ACCESS_CONTROL_MAX_AGE));
    }

    #[test]
    fn when_private_network_requested_should_emit_private_network_header() {
        // Arrange
        let cors = Cors::new(CorsOptions {
            allow_private_network: true,
            ..CorsOptions::default()
        });
        let original = request_with_private_network("OPTIONS", "https://intranet.test", "GET", "");

        // Act
        let result = preflight_result(&cors, &original).expect("expected preflight result");

        // Assert
        assert_eq!(
            result
                .headers
                .get(header::ACCESS_CONTROL_ALLOW_PRIVATE_NETWORK),
            Some(&"true".to_string())
        );
    }

    #[test]
    fn when_private_network_disabled_should_not_emit_header() {
        // Arrange
        let cors = Cors::new(CorsOptions::default());
        let original = request_with_private_network("OPTIONS", "https://intranet.test", "GET", "");

        // Act
        let result = preflight_result(&cors, &original).expect("expected preflight result");

        // Assert
        assert!(
            !result
                .headers
                .contains_key(header::ACCESS_CONTROL_ALLOW_PRIVATE_NETWORK)
        );
    }

    #[test]
    fn when_timing_allow_origin_configured_should_emit_header() {
        // Arrange
        let cors = Cors::new(CorsOptions {
            timing_allow_origin: Some(TimingAllowOrigin::list([
                "https://metrics.test",
                "https://dash.test",
            ])),
            ..CorsOptions::default()
        });
        let original = request("OPTIONS", "https://allowed.test", "GET", "X-Test");

        // Act
        let result = preflight_result(&cors, &original).expect("expected preflight result");

        // Assert
        assert_eq!(
            result.headers.get(header::TIMING_ALLOW_ORIGIN),
            Some(&"https://metrics.test https://dash.test".to_string())
        );
    }
}

mod process_simple {
    use super::*;

    #[test]
    fn when_origin_skip_should_return_none() {
        // Arrange
        let options = CorsOptions {
            origin: Origin::custom(|_, _| OriginDecision::Skip),
            ..CorsOptions::default()
        };
        let cors = cors_with(options);
        let original = request("GET", "https://denied.test", "", "");

        // Act
        let result = simple_result(&cors, &original);

        // Assert
        assert!(result.is_none());
    }

    #[test]
    fn when_origin_allowed_should_emit_simple_headers() {
        // Arrange
        let options = CorsOptions {
            origin: Origin::any(),
            ..CorsOptions::default()
        };
        let cors = cors_with(options);
        let original = request("GET", "https://allowed.test", "", "");

        // Act
        let result = simple_result(&cors, &original).expect("expected simple result");

        // Assert
        assert!(
            result
                .headers
                .contains_key(header::ACCESS_CONTROL_ALLOW_ORIGIN)
        );
        assert!(
            result
                .headers
                .contains_key(header::ACCESS_CONTROL_ALLOW_CREDENTIALS)
        );
    }

    #[test]
    fn when_credentials_disabled_should_not_emit_credentials_header() {
        // Arrange
        let cors = Cors::new(CorsOptions::default());
        let original = request("GET", "https://allowed.test", "", "");

        // Act
        let result = simple_result(&cors, &original).expect("expected simple result");

        // Assert
        assert!(
            result
                .headers
                .contains_key(header::ACCESS_CONTROL_ALLOW_ORIGIN)
        );
        assert!(
            !result
                .headers
                .contains_key(header::ACCESS_CONTROL_ALLOW_CREDENTIALS)
        );
    }

    #[test]
    fn when_origin_list_used_should_mirror_request_origin() {
        // Arrange
        let cors = Cors::new(CorsOptions {
            origin: Origin::list(["https://allowed.test"]),
            credentials: true,
            ..CorsOptions::default()
        });
        let original = request("GET", "https://allowed.test", "", "");

        // Act
        let result = simple_result(&cors, &original).expect("expected simple result");

        // Assert
        assert_eq!(
            result.headers.get(header::ACCESS_CONTROL_ALLOW_ORIGIN),
            Some(&"https://allowed.test".to_string())
        );
        assert!(
            result
                .headers
                .contains_key(header::ACCESS_CONTROL_ALLOW_CREDENTIALS)
        );
    }

    #[test]
    fn when_private_network_allowed_should_emit_header() {
        // Arrange
        let cors = Cors::new(CorsOptions {
            allow_private_network: true,
            ..CorsOptions::default()
        });
        let original = request("GET", "https://intranet.test", "", "");

        // Act
        let result = simple_result(&cors, &original).expect("expected simple result");

        // Assert
        assert_eq!(
            result
                .headers
                .get(header::ACCESS_CONTROL_ALLOW_PRIVATE_NETWORK),
            Some(&"true".to_string())
        );
    }

    #[test]
    fn when_timing_allow_origin_configured_should_emit_header() {
        // Arrange
        let cors = Cors::new(CorsOptions {
            timing_allow_origin: Some(TimingAllowOrigin::any()),
            ..CorsOptions::default()
        });
        let original = request("GET", "https://allowed.test", "", "");

        // Act
        let result = simple_result(&cors, &original).expect("expected simple result");

        // Assert
        assert_eq!(
            result.headers.get(header::TIMING_ALLOW_ORIGIN),
            Some(&"*".to_string())
        );
    }
}
