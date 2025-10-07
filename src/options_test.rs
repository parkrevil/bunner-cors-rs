use super::*;
use crate::allowed_headers::AllowedHeaders;
use crate::allowed_methods::AllowedMethods;
use crate::exposed_headers::ExposedHeaders;
use crate::origin::Origin;
use crate::timing_allow_origin::TimingAllowOrigin;

mod default {
    use super::*;

    #[test]
    fn given_new_options_when_constructed_then_uses_baseline_defaults() {
        // Arrange & Act
        let options = CorsOptions::new();

        // Assert
        assert!(matches!(options.origin, Origin::Any));
        assert_eq!(options.methods, AllowedMethods::default());
        assert!(options.allowed_headers == AllowedHeaders::default());
        assert!(matches!(options.exposed_headers, ExposedHeaders::None));
        assert!(!options.credentials);
        assert!(options.max_age.is_none());
        assert!(!options.allow_null_origin);
        assert!(!options.allow_private_network);
        assert!(options.timing_allow_origin.is_none());
    }

    #[test]
    fn given_distinct_instances_when_values_change_then_preserve_isolation() {
        // Arrange
        let first = CorsOptions::new().credentials(true);
        let second = CorsOptions::new();

        // Act
        let has_same_credentials = first.credentials == second.credentials;

        // Assert
        assert!(!has_same_credentials);
    }
}

mod display {
    use super::*;

    #[test]
    fn given_validation_errors_when_display_called_then_mentions_context() {
        // Arrange
        let cases: [(ValidationError, &str); 16] = [
            (
                ValidationError::CredentialsRequireSpecificOrigin,
                "specific allowed origin",
            ),
            (
                ValidationError::AllowedHeadersAnyNotAllowedWithCredentials,
                "AllowedHeaders::Any",
            ),
            (
                ValidationError::AllowedHeadersListCannotContainWildcard,
                "cannot include \"*\"",
            ),
            (
                ValidationError::AllowedHeadersListContainsInvalidToken,
                "valid HTTP header",
            ),
            (
                ValidationError::ExposeHeadersWildcardRequiresCredentialsDisabled,
                "credentials are disabled",
            ),
            (
                ValidationError::ExposeHeadersWildcardCannotBeCombined,
                "cannot be combined",
            ),
            (
                ValidationError::ExposeHeadersListContainsInvalidToken,
                "valid HTTP header",
            ),
            (
                ValidationError::PrivateNetworkRequiresCredentials,
                "requires enabling credentials",
            ),
            (
                ValidationError::PrivateNetworkRequiresSpecificOrigin,
                "specific allowed origin",
            ),
            (
                ValidationError::AllowedMethodsCannotContainEmptyToken,
                "cannot contain empty",
            ),
            (
                ValidationError::AllowedMethodsCannotContainWildcard,
                "cannot include the wildcard",
            ),
            (
                ValidationError::AllowedMethodsListContainsInvalidToken,
                "valid HTTP method",
            ),
            (
                ValidationError::AllowedHeadersCannotContainEmptyToken,
                "cannot contain empty",
            ),
            (
                ValidationError::ExposeHeadersCannotContainEmptyValue,
                "cannot contain empty",
            ),
            (
                ValidationError::TimingAllowOriginWildcardNotAllowedWithCredentials,
                "cannot be a wildcard",
            ),
            (
                ValidationError::TimingAllowOriginCannotContainEmptyValue,
                "cannot contain empty",
            ),
        ];

        // Act & Assert
        for (error, phrase) in cases {
            assert!(error.to_string().contains(phrase));
        }
    }
}

mod validate {
    use super::*;

    mod credentials {
        use super::*;

        #[test]
        fn given_credentials_with_any_origin_when_validate_called_then_returns_specific_origin_error()
         {
            let options = CorsOptions::new().credentials(true);
            let result = options.validate();

            assert!(matches!(
                result,
                Err(ValidationError::CredentialsRequireSpecificOrigin)
            ));
        }

        #[test]
        fn given_credentials_with_allowed_headers_any_when_validate_called_then_returns_header_error()
         {
            let options = CorsOptions::new()
                .credentials(true)
                .origin(Origin::list(["https://api.test"]))
                .allowed_headers(AllowedHeaders::Any);
            let result = options.validate();

            assert!(matches!(
                result,
                Err(ValidationError::AllowedHeadersAnyNotAllowedWithCredentials)
            ));
        }
    }

    mod allowed_headers_rules {
        use super::*;

        #[test]
        fn given_header_list_with_wildcard_when_validate_called_then_returns_list_wildcard_error() {
            let options = CorsOptions::new().allowed_headers(AllowedHeaders::list(["*", "X-Test"]));
            let result = options.validate();

            assert!(matches!(
                result,
                Err(ValidationError::AllowedHeadersListCannotContainWildcard)
            ));
        }

        #[test]
        fn given_header_list_with_invalid_token_when_validate_called_then_returns_invalid_token_error()
         {
            let options =
                CorsOptions::new().allowed_headers(AllowedHeaders::list(["X-Trace", "X Header"]));
            let result = options.validate();

            assert!(matches!(
                result,
                Err(ValidationError::AllowedHeadersListContainsInvalidToken)
            ));
        }

        #[test]
        fn given_header_list_with_empty_value_when_validate_called_then_returns_empty_token_error()
        {
            let options =
                CorsOptions::new().allowed_headers(AllowedHeaders::list(["X-Test", "  "]));
            let result = options.validate();

            assert!(matches!(
                result,
                Err(ValidationError::AllowedHeadersCannotContainEmptyToken)
            ));
        }
    }

    mod allowed_methods_rules {
        use super::*;

        #[test]
        fn given_methods_with_wildcard_when_validate_called_then_returns_wildcard_error() {
            let options = CorsOptions::new().methods(AllowedMethods::list(["GET", "*"]));
            let result = options.validate();

            assert!(matches!(
                result,
                Err(ValidationError::AllowedMethodsCannotContainWildcard)
            ));
        }

        #[test]
        fn given_methods_with_empty_entry_when_validate_called_then_returns_empty_token_error() {
            let options = CorsOptions::new().methods(AllowedMethods::list(["GET", "  ", "POST"]));
            let result = options.validate();

            assert!(matches!(
                result,
                Err(ValidationError::AllowedMethodsCannotContainEmptyToken)
            ));
        }

        #[test]
        fn given_methods_with_invalid_token_when_validate_called_then_returns_invalid_token_error()
        {
            let options = CorsOptions::new().methods(AllowedMethods::list(["GET", "PO ST"]));
            let result = options.validate();

            assert!(matches!(
                result,
                Err(ValidationError::AllowedMethodsListContainsInvalidToken)
            ));
        }
    }

    mod exposed_headers_rules {
        use super::*;

        #[test]
        fn given_wildcard_with_credentials_when_validate_called_then_returns_credentials_error() {
            let options = CorsOptions::new()
                .credentials(true)
                .origin(Origin::list(["https://api.test"]))
                .exposed_headers(ExposedHeaders::Any);
            let result = options.validate();

            assert!(matches!(
                result,
                Err(ValidationError::ExposeHeadersWildcardRequiresCredentialsDisabled)
            ));
        }

        #[test]
        fn given_headers_with_invalid_token_when_validate_called_then_returns_invalid_token_error()
        {
            let options =
                CorsOptions::new().exposed_headers(ExposedHeaders::list(["X-Trace", "X Header"]));
            let result = options.validate();

            assert!(matches!(
                result,
                Err(ValidationError::ExposeHeadersListContainsInvalidToken)
            ));
        }

        #[test]
        fn given_headers_with_empty_entry_when_validate_called_then_returns_empty_value_error() {
            let options =
                CorsOptions::new().exposed_headers(ExposedHeaders::list(["  ", "X-Trace"]));
            let result = options.validate();

            assert!(matches!(
                result,
                Err(ValidationError::ExposeHeadersCannotContainEmptyValue)
            ));
        }

        #[test]
        fn given_wildcard_without_credentials_when_validate_called_then_returns_ok() {
            let options = CorsOptions::new().exposed_headers(ExposedHeaders::Any);
            let result = options.validate();

            assert!(result.is_ok());
        }

        #[test]
        fn given_wildcard_combined_with_headers_when_validate_called_then_returns_combination_error()
         {
            let options = CorsOptions::new().exposed_headers(ExposedHeaders::list(["*", "X-Test"]));
            let result = options.validate();

            assert!(matches!(
                result,
                Err(ValidationError::ExposeHeadersWildcardCannotBeCombined)
            ));
        }
    }

    mod private_network_rules {
        use super::*;

        #[test]
        fn given_private_network_without_credentials_when_validate_called_then_returns_credentials_error()
         {
            let options = CorsOptions::new().allow_private_network(true);
            let result = options.validate();

            assert!(matches!(
                result,
                Err(ValidationError::PrivateNetworkRequiresCredentials)
            ));
        }

        #[test]
        fn given_private_network_with_wildcard_origin_when_validate_called_then_returns_origin_error()
         {
            let options = CorsOptions::new()
                .allow_private_network(true)
                .credentials(true);
            let result = options.validate();

            assert!(matches!(
                result,
                Err(ValidationError::PrivateNetworkRequiresSpecificOrigin)
            ));
        }

        #[test]
        fn given_private_network_with_specific_origin_when_validate_called_then_returns_ok() {
            let options = CorsOptions::new()
                .allow_private_network(true)
                .credentials(true)
                .origin(Origin::list(["https://intranet.test"]));
            let result = options.validate();

            assert!(result.is_ok());
        }
    }

    mod timing_rules {
        use super::*;

        #[test]
        fn given_timing_allow_origin_any_with_credentials_when_validate_called_then_returns_wildcard_error()
         {
            let options = CorsOptions::new()
                .credentials(true)
                .timing_allow_origin(TimingAllowOrigin::Any)
                .origin(Origin::list(["https://api.test"]));
            let result = options.validate();

            assert!(matches!(
                result,
                Err(ValidationError::TimingAllowOriginWildcardNotAllowedWithCredentials)
            ));
        }

        #[test]
        fn given_timing_allow_origin_with_empty_entry_when_validate_called_then_returns_empty_value_error()
         {
            let options = CorsOptions::new()
                .timing_allow_origin(TimingAllowOrigin::list([" ", "https://metrics.test"]));
            let result = options.validate();

            assert!(matches!(
                result,
                Err(ValidationError::TimingAllowOriginCannotContainEmptyValue)
            ));
        }
    }

    mod composite_rules {
        use super::*;

        #[test]
        fn given_specific_configuration_when_validate_called_then_returns_ok() {
            let options = CorsOptions::new()
                .origin(Origin::list(["https://api.test"]))
                .allowed_headers(AllowedHeaders::list(["X-Test"]))
                .exposed_headers(ExposedHeaders::list(["X-Expose"]))
                .credentials(true)
                .timing_allow_origin(TimingAllowOrigin::list(["https://metrics.test"]));
            let result = options.validate();

            assert!(result.is_ok());
        }

        #[test]
        fn given_max_age_configuration_when_validate_called_then_returns_ok() {
            let options = CorsOptions::new().max_age(600);
            let result = options.validate();

            assert!(result.is_ok());
        }

        #[test]
        fn given_private_network_and_wildcard_origin_conflicts_when_validate_called_then_returns_specific_origin_error()
         {
            let options = CorsOptions::new()
                .credentials(true)
                .allow_private_network(true)
                .allowed_headers(AllowedHeaders::list(["*", "X-Test"]))
                .timing_allow_origin(TimingAllowOrigin::Any);
            let result = options.validate();

            assert!(matches!(
                result,
                Err(ValidationError::PrivateNetworkRequiresSpecificOrigin)
            ));
        }

        #[test]
        fn given_wildcard_header_and_timing_conflicts_when_validate_called_then_returns_header_error()
         {
            let options = CorsOptions::new()
                .origin(Origin::list(["https://api.test"]))
                .credentials(true)
                .allowed_headers(AllowedHeaders::list(["*", "X-Test"]))
                .exposed_headers(ExposedHeaders::list(["  ", "X-Expose"]))
                .timing_allow_origin(TimingAllowOrigin::Any);
            let result = options.validate();

            assert!(matches!(
                result,
                Err(ValidationError::AllowedHeadersListCannotContainWildcard)
            ));
        }
    }
}
