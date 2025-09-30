use super::*;
use crate::allowed_headers::AllowedHeaders;
use crate::allowed_methods::AllowedMethods;
use crate::constants::header;
use crate::context::RequestContext;
use crate::normalized_request::NormalizedRequest;
use crate::options::CorsOptions;
use crate::origin::{Origin, OriginDecision};
use crate::result::CorsDecision;

fn request(
    method: &'static str,
    origin: &'static str,
    acrm: &'static str,
    acrh: &'static str,
) -> RequestContext<'static> {
    RequestContext {
        method,
        origin,
        access_control_request_method: acrm,
        access_control_request_headers: acrh,
    }
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
        let normalized_request = NormalizedRequest::new(&original);
        let normalized = normalized_request.as_context();

        // Act
        let result = cors.process_preflight(&original, &normalized);

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
        let normalized_request = NormalizedRequest::new(&original);
        let normalized = normalized_request.as_context();

        // Act
        let result = cors
            .process_preflight(&original, &normalized)
            .expect("expected preflight result");

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
        let normalized_request = NormalizedRequest::new(&original);
        let normalized = normalized_request.as_context();

        // Act
        let result = cors.process_simple(&original, &normalized);

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
        let normalized_request = NormalizedRequest::new(&original);
        let normalized = normalized_request.as_context();

        // Act
        let result = cors
            .process_simple(&original, &normalized)
            .expect("expected simple result");

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
}
