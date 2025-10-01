use super::*;
use crate::allowed_headers::AllowedHeaders;
use crate::allowed_methods::AllowedMethods;
use crate::constants::header;
use crate::context::RequestContext;
use crate::options::CorsOptions;
use crate::origin::{Origin, OriginDecision};
use crate::result::CorsError;
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

fn options_with_origin(origin: Origin) -> CorsOptions {
    CorsOptions {
        origin,
        ..CorsOptions::default()
    }
}

fn expect_allow(
    outcome: Result<(HeaderCollection, OriginDecision), CorsError>,
) -> HeaderCollection {
    match outcome.expect("expected allow outcome") {
        (collection, OriginDecision::Any)
        | (collection, OriginDecision::Mirror)
        | (collection, OriginDecision::Exact(_)) => collection,
        (_, OriginDecision::Disallow) => panic!("expected allow outcome, got disallow"),
        (_, OriginDecision::Skip) => panic!("expected allow outcome, got skip"),
    }
}

fn expect_disallow(
    outcome: Result<(HeaderCollection, OriginDecision), CorsError>,
) -> HeaderCollection {
    match outcome.expect("expected disallow outcome") {
        (collection, OriginDecision::Disallow) => collection,
        (_, OriginDecision::Any) | (_, OriginDecision::Mirror) | (_, OriginDecision::Exact(_)) => {
            panic!("expected disallow outcome, got allow")
        }
        (_, OriginDecision::Skip) => panic!("expected disallow outcome, got skip"),
    }
}

fn expect_skip(outcome: Result<(HeaderCollection, OriginDecision), CorsError>) {
    match outcome.expect("expected skip outcome") {
        (_, OriginDecision::Skip) => {}
        (_, OriginDecision::Any) | (_, OriginDecision::Mirror) | (_, OriginDecision::Exact(_)) => {
            panic!("expected skip outcome, got allow")
        }
        (_, OriginDecision::Disallow) => panic!("expected skip outcome, got disallow"),
    }
}

mod new {
    use super::*;

    #[test]
    fn should_use_provided_options_reference_when_constructed() {
        // Arrange
        let options = CorsOptions::default();

        // Act
        let builder = HeaderBuilder::new(&options);
        let headers = builder.build_methods_header().into_headers();

        // Assert
        let value = headers.get(header::ACCESS_CONTROL_ALLOW_METHODS);
        assert_eq!(value, Some(&"GET,HEAD,PUT,PATCH,POST,DELETE".to_string()));
    }
}

mod build_origin_headers {
    use super::*;

    #[test]
    fn should_emit_wildcard_without_vary_given_origin_is_any() {
        // Arrange
        let options = options_with_origin(Origin::any());
        let builder = HeaderBuilder::new(&options);
        let ctx = request("GET", "https://api.test", "", "");

        // Act
        let map = expect_allow(builder.build_origin_headers(&ctx, &ctx)).into_headers();

        // Assert
        assert_eq!(
            map.get(header::ACCESS_CONTROL_ALLOW_ORIGIN),
            Some(&"*".to_string())
        );
        assert!(!map.contains_key(header::VARY));
    }

    #[test]
    fn should_mirror_request_origin_given_origin_matches_list() {
        // Arrange
        let options = options_with_origin(Origin::list(["https://app.test"]));
        let builder = HeaderBuilder::new(&options);
        let original = request("GET", "https://app.test", "", "");
        let normalized = request("get", "https://app.test", "", "");

        // Act
        let map = expect_allow(builder.build_origin_headers(&original, &normalized)).into_headers();

        // Assert
        assert_eq!(
            map.get(header::ACCESS_CONTROL_ALLOW_ORIGIN),
            Some(&"https://app.test".to_string())
        );
        assert_eq!(map.get(header::VARY), Some(&"Origin".to_string()));
    }

    #[test]
    fn should_short_circuit_given_origin_is_custom_skip() {
        // Arrange
        let options = options_with_origin(Origin::custom(|_, _| OriginDecision::Skip));
        let builder = HeaderBuilder::new(&options);
        let ctx = request("OPTIONS", "https://skip.test", "", "");

        // Act
        let outcome = builder.build_origin_headers(&ctx, &ctx);

        // Assert
        expect_skip(outcome);
    }

    #[test]
    fn should_error_given_any_origin_with_credentials() {
        // Arrange
        let mut options = options_with_origin(Origin::any());
        options.credentials = true;
        let builder = HeaderBuilder::new(&options);
        let ctx = request("OPTIONS", "https://wild.test", "", "");

        // Act
        let error = builder
            .build_origin_headers(&ctx, &ctx)
            .expect_err("expected invalid origin error");

        // Assert
        assert_eq!(error, CorsError::InvalidOriginAnyWithCredentials);
    }

    #[test]
    fn should_error_given_custom_origin_returns_any_with_credentials() {
        // Arrange
        let base = options_with_origin(Origin::custom(|_, _| OriginDecision::Any));
        let options = CorsOptions {
            credentials: true,
            ..base
        };
        let builder = HeaderBuilder::new(&options);
        let ctx = request("OPTIONS", "https://wild.test", "", "");

        // Act
        let error = builder
            .build_origin_headers(&ctx, &ctx)
            .expect_err("expected invalid origin error");

        // Assert
        assert_eq!(error, CorsError::InvalidOriginAnyWithCredentials);
    }

    #[test]
    fn should_only_emit_vary_given_origin_is_disallowed() {
        // Arrange
        let options = options_with_origin(Origin::list(["https://allowed.test"]));
        let builder = HeaderBuilder::new(&options);
        let ctx = request("GET", "https://denied.test", "", "");

        // Act
        let map = expect_disallow(builder.build_origin_headers(&ctx, &ctx)).into_headers();

        // Assert
        assert_eq!(map.get(header::VARY), Some(&"Origin".to_string()));
        assert!(!map.contains_key(header::ACCESS_CONTROL_ALLOW_ORIGIN));
    }

    #[test]
    fn should_not_emit_header_given_origin_mirror_has_empty_request() {
        // Arrange
        let options = options_with_origin(Origin::list(["https://app.test"]));
        let builder = HeaderBuilder::new(&options);
        let original = request("GET", "", "", "");
        let normalized = request("get", "https://app.test", "", "");

        // Act
        let map =
            expect_disallow(builder.build_origin_headers(&original, &normalized)).into_headers();

        // Assert
        assert_eq!(map.get(header::VARY), Some(&"Origin".to_string()));
        assert!(!map.contains_key(header::ACCESS_CONTROL_ALLOW_ORIGIN));
    }

    #[test]
    fn should_preserve_original_casing_given_origin_mirror() {
        // Arrange
        let options = options_with_origin(Origin::list(["https://app.test"]));
        let builder = HeaderBuilder::new(&options);
        let original = request("GET", "https://API.test", "", "");
        let normalized = request("get", "https://app.test", "", "");

        // Act
        let map = expect_allow(builder.build_origin_headers(&original, &normalized)).into_headers();

        // Assert
        assert_eq!(
            map.get(header::ACCESS_CONTROL_ALLOW_ORIGIN),
            Some(&"https://API.test".to_string())
        );
    }
}

mod build_methods_header {
    use super::*;

    #[test]
    fn should_emit_header_given_methods_have_values() {
        // Arrange
        let options = CorsOptions {
            methods: AllowedMethods::list(["GET", "PATCH"]),
            ..CorsOptions::default()
        };
        let builder = HeaderBuilder::new(&options);

        // Act
        let map = builder.build_methods_header().into_headers();

        // Assert
        assert_eq!(
            map.get(header::ACCESS_CONTROL_ALLOW_METHODS),
            Some(&"GET,PATCH".to_string())
        );
    }

    #[test]
    fn should_leave_collection_empty_given_methods_header_value_is_none() {
        // Arrange
        let options = CorsOptions {
            methods: AllowedMethods::list(Vec::<String>::new()),
            ..CorsOptions::default()
        };
        let builder = HeaderBuilder::new(&options);

        // Act
        let map = builder.build_methods_header().into_headers();

        // Assert
        assert!(map.is_empty());
    }
}

mod build_credentials_header {
    use super::*;

    #[test]
    fn should_emit_true_header_given_credentials_enabled() {
        // Arrange
        let options = CorsOptions {
            credentials: true,
            ..CorsOptions::default()
        };
        let builder = HeaderBuilder::new(&options);

        // Act
        let map = builder.build_credentials_header().into_headers();

        // Assert
        assert_eq!(
            map.get(header::ACCESS_CONTROL_ALLOW_CREDENTIALS),
            Some(&"true".to_string())
        );
    }

    #[test]
    fn should_return_empty_collection_given_credentials_disabled() {
        // Arrange
        let options = CorsOptions::default();
        let builder = HeaderBuilder::new(&options);

        // Act
        let map = builder.build_credentials_header().into_headers();

        // Assert
        assert!(map.is_empty());
    }
}

mod build_allowed_headers {
    use super::*;

    #[test]
    fn should_emit_joined_value_given_configured_list() {
        // Arrange
        let options = CorsOptions {
            allowed_headers: AllowedHeaders::list(["X-Trace", "X-Auth"]),
            ..CorsOptions::default()
        };
        let builder = HeaderBuilder::new(&options);

        // Act
        let map = builder.build_allowed_headers().into_headers();

        // Assert
        assert_eq!(
            map.get(header::ACCESS_CONTROL_ALLOW_HEADERS),
            Some(&"X-Trace,X-Auth".to_string())
        );
        assert!(!map.contains_key(header::VARY));
    }

    #[test]
    fn should_return_empty_collection_given_configured_list_is_empty() {
        // Arrange
        let options = CorsOptions {
            allowed_headers: AllowedHeaders::list(Vec::<String>::new()),
            ..CorsOptions::default()
        };
        let builder = HeaderBuilder::new(&options);

        // Act
        let map = builder.build_allowed_headers().into_headers();

        // Assert
        assert!(map.is_empty());
    }

    #[test]
    fn should_still_emit_configured_list_given_request_has_headers() {
        // Arrange
        let options = CorsOptions {
            allowed_headers: AllowedHeaders::list(["X-Test", "X-Trace"]),
            ..CorsOptions::default()
        };
        let builder = HeaderBuilder::new(&options);

        // Act
        let map = builder.build_allowed_headers().into_headers();

        // Assert
        assert_eq!(
            map.get(header::ACCESS_CONTROL_ALLOW_HEADERS),
            Some(&"X-Test,X-Trace".to_string())
        );
        assert!(!map.contains_key(header::VARY));
    }

    #[test]
    fn should_emit_configured_list_given_request_headers_absent() {
        // Arrange
        let options = CorsOptions {
            allowed_headers: AllowedHeaders::list(["X-Test"]),
            ..CorsOptions::default()
        };
        let builder = HeaderBuilder::new(&options);

        // Act
        let map = builder.build_allowed_headers().into_headers();

        // Assert
        assert_eq!(
            map.get(header::ACCESS_CONTROL_ALLOW_HEADERS),
            Some(&"X-Test".to_string())
        );
        assert!(!map.contains_key(header::VARY));
    }

    #[test]
    fn should_emit_wildcard_given_any() {
        // Arrange
        let options = CorsOptions {
            allowed_headers: AllowedHeaders::Any,
            ..CorsOptions::default()
        };
        let builder = HeaderBuilder::new(&options);

        // Act
        let map = builder.build_allowed_headers().into_headers();

        // Assert
        assert_eq!(
            map.get(header::ACCESS_CONTROL_ALLOW_HEADERS),
            Some(&"*".to_string())
        );
    }
}

mod build_exposed_headers {
    use super::*;

    #[test]
    fn should_emit_comma_separated_header_given_values_present() {
        // Arrange
        let options = CorsOptions {
            exposed_headers: Some(vec!["X-Trace".into(), "X-Auth".into()]),
            ..CorsOptions::default()
        };
        let builder = HeaderBuilder::new(&options);

        // Act
        let map = builder.build_exposed_headers().into_headers();

        // Assert
        assert_eq!(
            map.get(header::ACCESS_CONTROL_EXPOSE_HEADERS),
            Some(&"X-Trace,X-Auth".to_string())
        );
    }

    #[test]
    fn should_return_empty_collection_given_values_absent() {
        // Arrange
        let options = CorsOptions::default();
        let builder = HeaderBuilder::new(&options);

        // Act
        let map = builder.build_exposed_headers().into_headers();

        // Assert
        assert!(map.is_empty());
    }

    #[test]
    fn should_return_empty_collection_given_configured_list_is_empty() {
        let options = CorsOptions {
            exposed_headers: Some(Vec::new()),
            ..CorsOptions::default()
        };
        let builder = HeaderBuilder::new(&options);

        let map = builder.build_exposed_headers().into_headers();

        assert!(map.is_empty());
    }

    #[test]
    fn should_emit_trimmed_value_given_values_have_whitespace_and_wildcard() {
        // Arrange
        let options = CorsOptions {
            exposed_headers: Some(vec!["  *  ".into()]),
            ..CorsOptions::default()
        };
        let builder = HeaderBuilder::new(&options);

        // Act
        let map = builder.build_exposed_headers().into_headers();

        // Assert
        assert_eq!(
            map.get(header::ACCESS_CONTROL_EXPOSE_HEADERS),
            Some(&"*".to_string())
        );
    }

    #[test]
    fn should_return_empty_collection_given_values_trim_to_empty() {
        let options = CorsOptions {
            exposed_headers: Some(vec!["   ".into(), "\t".into()]),
            ..CorsOptions::default()
        };
        let builder = HeaderBuilder::new(&options);

        let map = builder.build_exposed_headers().into_headers();

        assert!(map.is_empty());
    }
}

mod build_max_age_header {
    use super::*;

    #[test]
    fn should_emit_header_given_max_age_specified() {
        // Arrange
        let options = CorsOptions {
            max_age: Some("600".into()),
            ..CorsOptions::default()
        };
        let builder = HeaderBuilder::new(&options);

        // Act
        let map = builder.build_max_age_header().into_headers();

        // Assert
        assert_eq!(
            map.get(header::ACCESS_CONTROL_MAX_AGE),
            Some(&"600".to_string())
        );
    }

    #[test]
    fn should_return_empty_collection_given_max_age_missing() {
        // Arrange
        let options = CorsOptions::default();
        let builder = HeaderBuilder::new(&options);

        // Act
        let map = builder.build_max_age_header().into_headers();

        // Assert
        assert!(map.is_empty());
    }

    #[test]
    fn should_not_emit_header_given_max_age_blank() {
        // Arrange
        let options = CorsOptions {
            max_age: Some(String::new()),
            ..CorsOptions::default()
        };
        let builder = HeaderBuilder::new(&options);

        // Act
        let map = builder.build_max_age_header().into_headers();

        // Assert
        assert!(map.is_empty());
    }
}

mod build_private_network_header {
    use super::*;

    #[test]
    fn should_emit_allow_header_given_request_includes_private_network() {
        // Arrange
        let options = CorsOptions {
            allow_private_network: true,
            ..CorsOptions::default()
        };
        let builder = HeaderBuilder::new(&options);
        let ctx = request_with_private_network("OPTIONS", "https://api.test", "POST", "X-Test");

        // Act
        let map = builder.build_private_network_header(&ctx).into_headers();

        // Assert
        assert_eq!(
            map.get(header::ACCESS_CONTROL_ALLOW_PRIVATE_NETWORK),
            Some(&"true".to_string())
        );
    }

    #[test]
    fn should_not_emit_header_given_private_network_disabled() {
        // Arrange
        let options = CorsOptions::default();
        let builder = HeaderBuilder::new(&options);
        let ctx = request_with_private_network("OPTIONS", "https://api.test", "POST", "X-Test");

        // Act
        let map = builder.build_private_network_header(&ctx).into_headers();

        // Assert
        assert!(map.is_empty());
    }

    #[test]
    fn should_return_empty_collection_given_request_does_not_include_private_network() {
        // Arrange
        let options = CorsOptions {
            allow_private_network: true,
            ..CorsOptions::default()
        };
        let builder = HeaderBuilder::new(&options);
        let ctx = request("OPTIONS", "https://api.test", "POST", "X-Test");

        // Act
        let map = builder.build_private_network_header(&ctx).into_headers();

        // Assert
        assert!(map.is_empty());
    }

    #[test]
    fn should_not_emit_header_given_request_is_simple() {
        // Arrange
        let options = CorsOptions {
            allow_private_network: true,
            ..CorsOptions::default()
        };
        let builder = HeaderBuilder::new(&options);
        let ctx = request("GET", "https://api.test", "GET", "");

        // Act
        let map = builder.build_private_network_header(&ctx).into_headers();

        // Assert
        assert!(map.is_empty());
    }
}

mod build_timing_allow_origin_header {
    use super::*;

    #[test]
    fn should_emit_empty_collection_given_configuration_absent() {
        // Arrange
        let options = CorsOptions::default();
        let builder = HeaderBuilder::new(&options);

        // Act
        let map = builder.build_timing_allow_origin_header().into_headers();

        // Assert
        assert!(map.is_empty());
    }

    #[test]
    fn should_emit_wildcard_value_given_any() {
        // Arrange
        let options = CorsOptions {
            timing_allow_origin: Some(TimingAllowOrigin::any()),
            ..CorsOptions::default()
        };
        let builder = HeaderBuilder::new(&options);

        // Act
        let map = builder.build_timing_allow_origin_header().into_headers();

        // Assert
        assert_eq!(map.get(header::TIMING_ALLOW_ORIGIN), Some(&"*".to_string()));
    }

    #[test]
    fn should_emit_space_separated_value_given_list() {
        // Arrange
        let options = CorsOptions {
            timing_allow_origin: Some(TimingAllowOrigin::list([
                "https://metrics.test",
                "https://dash.test",
            ])),
            ..CorsOptions::default()
        };
        let builder = HeaderBuilder::new(&options);

        // Act
        let map = builder.build_timing_allow_origin_header().into_headers();

        // Assert
        assert_eq!(
            map.get(header::TIMING_ALLOW_ORIGIN),
            Some(&"https://metrics.test https://dash.test".to_string())
        );
    }
}
