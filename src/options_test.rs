use super::*;
use crate::allowed_headers::AllowedHeaders;
use crate::allowed_methods::AllowedMethods;
use crate::exposed_headers::ExposedHeaders;
use crate::origin::Origin;
use crate::timing_allow_origin::TimingAllowOrigin;

mod default {
    use super::*;

    #[test]
    fn should_use_expected_defaults_when_constructed_then_match_baseline_values() {
        let options = CorsOptions::new();

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
    fn should_not_affect_other_defaults_when_instance_mutated_then_preserve_isolation() {
        let mut first = CorsOptions::new();
        first.credentials = true;
        let second = CorsOptions::new();

        assert_ne!(first.credentials, second.credentials);
    }
}

mod validate {
    use super::*;

    #[test]
    fn should_include_descriptive_messages_when_validation_errors_display_then_match_expectations()
    {
        let cases: Vec<(ValidationError, &str)> = vec![
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

        for (error, expected) in cases {
            let message = error.to_string();
            assert!(
                message.contains(expected),
                "expected '{message}' to contain '{expected}'"
            );
        }
    }

    #[test]
    fn should_return_error_when_credentials_allow_any_origin_then_require_specific_origin() {
        let options = CorsOptions::new().credentials(true);
        let result = options.validate();

        assert!(matches!(
            result,
            Err(ValidationError::CredentialsRequireSpecificOrigin)
        ));
    }

    #[test]
    fn should_return_error_when_credentials_and_allowed_headers_any_then_require_specific_headers()
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

    #[test]
    fn should_return_error_when_allowed_headers_list_contains_wildcard_then_reject_configuration() {
        let options = CorsOptions::new().allowed_headers(AllowedHeaders::list(["*", "X-Test"]));
        let result = options.validate();

        assert!(matches!(
            result,
            Err(ValidationError::AllowedHeadersListCannotContainWildcard)
        ));
    }

    #[test]
    fn should_return_error_when_allowed_methods_list_contains_wildcard_then_reject_configuration() {
        let options = CorsOptions::new().methods(AllowedMethods::list(["GET", "*"]));
        let result = options.validate();

        assert!(matches!(
            result,
            Err(ValidationError::AllowedMethodsCannotContainWildcard)
        ));
    }

    #[test]
    fn should_return_error_when_allowed_methods_list_contains_empty_token_then_reject_configuration()
     {
        let options = CorsOptions::new().methods(AllowedMethods::list(["GET", "  ", "POST"]));
        let result = options.validate();

        assert!(matches!(
            result,
            Err(ValidationError::AllowedMethodsCannotContainEmptyToken)
        ));
    }

    #[test]
    fn should_return_error_when_allowed_methods_list_contains_invalid_token_then_reject_configuration()
     {
        let options = CorsOptions::new().methods(AllowedMethods::list(["GET", "PO ST"]));
        let result = options.validate();

        assert!(matches!(
            result,
            Err(ValidationError::AllowedMethodsListContainsInvalidToken)
        ));
    }

    #[test]
    fn should_return_error_when_allowed_headers_list_contains_invalid_token_then_reject_configuration()
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
    fn should_return_error_when_allowed_headers_list_contains_empty_token_then_reject_configuration()
     {
        let options = CorsOptions::new().allowed_headers(AllowedHeaders::list(["X-Test", "  "]));
        let result = options.validate();

        assert!(matches!(
            result,
            Err(ValidationError::AllowedHeadersCannotContainEmptyToken)
        ));
    }

    #[test]
    fn should_return_error_when_expose_headers_wildcard_with_credentials_then_require_credentials_disabled()
     {
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
    fn should_return_error_when_expose_headers_contains_invalid_token_then_reject_configuration() {
        let options =
            CorsOptions::new().exposed_headers(ExposedHeaders::list(["X-Trace", "X Header"]));
        let result = options.validate();

        assert!(matches!(
            result,
            Err(ValidationError::ExposeHeadersListContainsInvalidToken)
        ));
    }

    #[test]
    fn should_return_ok_when_expose_headers_wildcard_without_credentials_then_accept_configuration()
    {
        let options = CorsOptions::new().exposed_headers(ExposedHeaders::Any);
        let result = options.validate();

        assert!(result.is_ok());
    }

    #[test]
    fn should_return_error_when_expose_headers_wildcard_combined_with_headers_then_reject_configuration()
     {
        let options = CorsOptions::new().exposed_headers(ExposedHeaders::list(["*", "X-Test"]));
        let result = options.validate();

        assert!(matches!(
            result,
            Err(ValidationError::ExposeHeadersWildcardCannotBeCombined)
        ));
    }

    #[test]
    fn should_return_error_when_expose_headers_contains_empty_value_then_reject_configuration() {
        let options = CorsOptions::new().exposed_headers(ExposedHeaders::list(["  ", "X-Trace"]));
        let result = options.validate();

        assert!(matches!(
            result,
            Err(ValidationError::ExposeHeadersCannotContainEmptyValue)
        ));
    }

    #[test]
    fn should_return_error_when_private_network_enabled_without_credentials_then_require_credentials()
     {
        let options = CorsOptions::new().allow_private_network(true);
        let result = options.validate();

        assert!(matches!(
            result,
            Err(ValidationError::PrivateNetworkRequiresCredentials)
        ));
    }

    #[test]
    fn should_return_error_when_private_network_enabled_with_wildcard_origin_then_require_specific_origin()
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
    fn should_return_ok_when_private_network_enabled_with_specific_origin_then_accept_configuration()
     {
        let options = CorsOptions::new()
            .allow_private_network(true)
            .credentials(true)
            .origin(Origin::list(["https://intranet.test"]));
        let result = options.validate();

        assert!(result.is_ok());
    }

    #[test]
    fn should_return_ok_when_configuration_specific_then_accept_settings() {
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
    fn should_return_error_when_timing_allow_origin_any_with_credentials_then_reject_configuration()
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
    fn should_return_error_when_timing_allow_origin_contains_empty_entry_then_reject_configuration()
    {
        let options = CorsOptions::new()
            .timing_allow_origin(TimingAllowOrigin::list([" ", "https://metrics.test"]));
        let result = options.validate();

        assert!(matches!(
            result,
            Err(ValidationError::TimingAllowOriginCannotContainEmptyValue)
        ));
    }

    #[test]
    fn should_return_ok_when_max_age_configured_then_accept_configuration() {
        let options = CorsOptions::new().max_age(600);
        let result = options.validate();

        assert!(result.is_ok());
    }
}
