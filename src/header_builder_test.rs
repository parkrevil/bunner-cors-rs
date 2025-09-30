use super::*;
use crate::allowed_headers::AllowedHeaders;
use crate::allowed_methods::AllowedMethods;
use crate::constants::header;
use crate::context::RequestContext;
use crate::options::CorsOptions;
use crate::origin::{Origin, OriginDecision};

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

fn options_with_origin(origin: Origin) -> CorsOptions {
    CorsOptions {
        origin,
        ..CorsOptions::default()
    }
}

mod new {
    use super::*;

    #[test]
    fn when_constructed_should_use_provided_options_reference() {
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
    fn when_origin_is_any_should_emit_wildcard_without_vary() {
        // Arrange
        let options = options_with_origin(Origin::any());
        let builder = HeaderBuilder::new(&options);
        let ctx = request("GET", "https://api.test", "", "");

        // Act
        let (headers, skip) = builder.build_origin_headers(&ctx, &ctx);
        let map = headers.into_headers();

        // Assert
        assert!(!skip);
        assert_eq!(
            map.get(header::ACCESS_CONTROL_ALLOW_ORIGIN),
            Some(&"*".to_string())
        );
        assert!(!map.contains_key(header::VARY));
    }

    #[test]
    fn when_origin_matches_list_should_mirror_request_origin() {
        // Arrange
        let options = options_with_origin(Origin::list(["https://app.test"]));
        let builder = HeaderBuilder::new(&options);
        let original = request("GET", "https://app.test", "", "");
        let normalized = request("get", "https://app.test", "", "");

        // Act
        let (headers, skip) = builder.build_origin_headers(&original, &normalized);
        let map = headers.into_headers();

        // Assert
        assert!(!skip);
        assert_eq!(
            map.get(header::ACCESS_CONTROL_ALLOW_ORIGIN),
            Some(&"https://app.test".to_string())
        );
        assert_eq!(map.get(header::VARY), Some(&"Origin".to_string()));
    }

    #[test]
    fn when_origin_is_custom_skip_should_short_circuit() {
        // Arrange
        let options = options_with_origin(Origin::custom(|_, _| OriginDecision::Skip));
        let builder = HeaderBuilder::new(&options);
        let ctx = request("OPTIONS", "https://skip.test", "", "");

        // Act
        let (headers, skip) = builder.build_origin_headers(&ctx, &ctx);

        // Assert
        assert!(skip);
        assert!(headers.into_headers().is_empty());
    }

    #[test]
    fn when_origin_is_disallowed_should_only_emit_vary() {
        // Arrange
        let options = options_with_origin(Origin::list(["https://allowed.test"]));
        let builder = HeaderBuilder::new(&options);
        let ctx = request("GET", "https://denied.test", "", "");

        // Act
        let (headers, skip) = builder.build_origin_headers(&ctx, &ctx);
        let map = headers.into_headers();

        // Assert
        assert!(!skip);
        assert_eq!(map.get(header::VARY), Some(&"Origin".to_string()));
        assert!(!map.contains_key(header::ACCESS_CONTROL_ALLOW_ORIGIN));
    }
}

mod build_methods_header {
    use super::*;

    #[test]
    fn when_methods_have_values_should_emit_header() {
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
    fn when_methods_header_value_is_none_should_leave_collection_empty() {
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
    fn when_credentials_enabled_should_emit_true_header() {
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
    fn when_credentials_disabled_should_return_empty_collection() {
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
    fn when_configured_list_should_emit_joined_value() {
        // Arrange
        let options = CorsOptions {
            allowed_headers: AllowedHeaders::list(["X-Trace", "X-Auth"]),
            ..CorsOptions::default()
        };
        let builder = HeaderBuilder::new(&options);
        let ctx = request("OPTIONS", "https://api.test", "", "");

        // Act
        let map = builder.build_allowed_headers(&ctx).into_headers();

        // Assert
        assert_eq!(
            map.get(header::ACCESS_CONTROL_ALLOW_HEADERS),
            Some(&"X-Trace,X-Auth".to_string())
        );
        assert!(!map.contains_key(header::VARY));
    }

    #[test]
    fn when_mirroring_request_headers_should_reflect_original() {
        // Arrange
        let options = CorsOptions {
            allowed_headers: AllowedHeaders::MirrorRequest,
            ..CorsOptions::default()
        };
        let builder = HeaderBuilder::new(&options);
        let ctx = request("OPTIONS", "https://api.test", "", "X-Trace, X-Auth");

        // Act
        let map = builder.build_allowed_headers(&ctx).into_headers();

        // Assert
        assert_eq!(
            map.get(header::ACCESS_CONTROL_ALLOW_HEADERS),
            Some(&"X-Trace, X-Auth".to_string())
        );
        assert_eq!(
            map.get(header::VARY),
            Some(&"Access-Control-Request-Headers".to_string())
        );
    }

    #[test]
    fn when_request_headers_absent_should_only_set_vary() {
        // Arrange
        let options = CorsOptions {
            allowed_headers: AllowedHeaders::MirrorRequest,
            ..CorsOptions::default()
        };
        let builder = HeaderBuilder::new(&options);
        let ctx = request("OPTIONS", "https://api.test", "", "");

        // Act
        let map = builder.build_allowed_headers(&ctx).into_headers();

        // Assert
        assert!(!map.contains_key(header::ACCESS_CONTROL_ALLOW_HEADERS));
        assert_eq!(
            map.get(header::VARY),
            Some(&"Access-Control-Request-Headers".to_string())
        );
    }

    #[test]
    fn when_any_should_emit_wildcard() {
        // Arrange
        let options = CorsOptions {
            allowed_headers: AllowedHeaders::Any,
            ..CorsOptions::default()
        };
        let builder = HeaderBuilder::new(&options);
        let ctx = request("OPTIONS", "https://api.test", "", "");

        // Act
        let map = builder.build_allowed_headers(&ctx).into_headers();

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
    fn when_values_present_should_emit_comma_separated_header() {
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
    fn when_values_absent_should_return_empty_collection() {
        // Arrange
        let options = CorsOptions::default();
        let builder = HeaderBuilder::new(&options);

        // Act
        let map = builder.build_exposed_headers().into_headers();

        // Assert
        assert!(map.is_empty());
    }
}

mod build_max_age_header {
    use super::*;

    #[test]
    fn when_max_age_specified_should_emit_header() {
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
    fn when_max_age_missing_should_return_empty_collection() {
        // Arrange
        let options = CorsOptions::default();
        let builder = HeaderBuilder::new(&options);

        // Act
        let map = builder.build_max_age_header().into_headers();

        // Assert
        assert!(map.is_empty());
    }

    #[test]
    fn when_max_age_blank_should_not_emit_header() {
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
