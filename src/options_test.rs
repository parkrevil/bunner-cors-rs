use super::*;
use crate::allowed_headers::AllowedHeaders;
use crate::allowed_methods::AllowedMethods;
use crate::origin::Origin;
use crate::timing_allow_origin::TimingAllowOrigin;

mod default {
    use super::*;

    #[test]
    fn when_constructed_should_use_expected_defaults() {
        // Arrange & Act
        let options = CorsOptions::default();

        // Assert
        assert!(matches!(options.origin, Origin::Any));
        assert_eq!(options.methods, AllowedMethods::default());
        assert!(options.allowed_headers == AllowedHeaders::default());
        assert_eq!(options.exposed_headers, None);
        assert!(!options.credentials);
        assert!(options.max_age.is_none());
        assert!(!options.preflight_continue);
        assert_eq!(options.options_success_status, 204);
        assert!(!options.allow_private_network);
        assert!(options.timing_allow_origin.is_none());
    }

    #[test]
    fn when_mutated_instance_should_not_affect_other_defaults() {
        // Arrange
        let mut first = CorsOptions::default();
        let second = CorsOptions::default();

        // Act
        first.credentials = true;

        // Assert
        assert_ne!(first.credentials, second.credentials);
    }
}

mod validate {
    use super::*;

    #[test]
    fn when_credentials_allow_any_origin_should_return_error() {
        // Arrange
        let options = CorsOptions {
            origin: Origin::any(),
            credentials: true,
            ..CorsOptions::default()
        };

        // Act
        let result = options.validate();

        // Assert
        assert!(matches!(
            result,
            Err(ValidationError::CredentialsRequireSpecificOrigin)
        ));
    }

    #[test]
    fn when_allowed_headers_list_contains_wildcard_should_return_error() {
        // Arrange
        let options = CorsOptions {
            allowed_headers: AllowedHeaders::list(["*", "X-Test"]),
            ..CorsOptions::default()
        };

        // Act
        let result = options.validate();

        // Assert
        assert!(matches!(
            result,
            Err(ValidationError::AllowedHeadersListCannotContainWildcard)
        ));
    }

    #[test]
    fn when_expose_headers_contains_wildcard_should_return_error() {
        // Arrange
        let options = CorsOptions {
            exposed_headers: Some(vec!["*".to_string()]),
            ..CorsOptions::default()
        };

        // Act
        let result = options.validate();

        // Assert
        assert!(matches!(
            result,
            Err(ValidationError::ExposeHeadersListCannotContainWildcard)
        ));
    }

    #[test]
    fn when_configuration_is_specific_should_return_ok() {
        // Arrange
        let options = CorsOptions {
            origin: Origin::list(["https://api.test"]),
            allowed_headers: AllowedHeaders::list(["X-Test"]),
            exposed_headers: Some(vec!["X-Expose".to_string()]),
            credentials: true,
            timing_allow_origin: Some(TimingAllowOrigin::any()),
            ..CorsOptions::default()
        };

        // Act
        let result = options.validate();

        // Assert
        assert!(result.is_ok());
    }

    #[test]
    fn when_success_status_out_of_range_should_return_error() {
        // Arrange
        let options = CorsOptions {
            options_success_status: 399,
            ..CorsOptions::default()
        };

        // Act
        let result = options.validate();

        // Assert
        assert!(matches!(
            result,
            Err(ValidationError::InvalidSuccessStatus(399))
        ));
    }

    #[test]
    fn when_max_age_not_numeric_should_return_error() {
        // Arrange
        let options = CorsOptions {
            max_age: Some("ten minutes".into()),
            ..CorsOptions::default()
        };

        // Act
        let result = options.validate();

        // Assert
        assert!(matches!(
            result,
            Err(ValidationError::InvalidMaxAge(value)) if value == "ten minutes"
        ));
    }
}
