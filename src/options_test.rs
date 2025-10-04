use super::*;
use crate::allowed_headers::AllowedHeaders;
use crate::allowed_methods::AllowedMethods;
use crate::origin::Origin;
use crate::timing_allow_origin::TimingAllowOrigin;

mod default {
    use super::*;

    #[test]
    fn should_use_expected_defaults_when_constructed_then_match_baseline_values() {
        let options = CorsOptions::default();

        assert!(matches!(options.origin, Origin::Any));
        assert_eq!(options.methods, AllowedMethods::default());
        assert!(options.allowed_headers == AllowedHeaders::default());
        assert_eq!(options.exposed_headers, None);
        assert!(!options.credentials);
        assert!(options.max_age.is_none());
        assert!(!options.allow_null_origin);
        assert!(!options.allow_private_network);
        assert!(options.timing_allow_origin.is_none());
    }

    #[test]
    fn should_not_affect_other_defaults_when_instance_mutated_then_preserve_isolation() {
        let mut first = CorsOptions::default();
        let second = CorsOptions::default();

        first.credentials = true;

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
                ValidationError::InvalidMaxAge("twenty".into()),
                "must be a non-negative integer",
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
        let options = CorsOptions {
            origin: Origin::any(),
            credentials: true,
            ..CorsOptions::default()
        };

        let result = options.validate();

        assert!(matches!(
            result,
            Err(ValidationError::CredentialsRequireSpecificOrigin)
        ));
    }

    #[test]
    fn should_return_error_when_credentials_and_allowed_headers_any_then_require_specific_headers()
    {
        let options = CorsOptions {
            credentials: true,
            origin: Origin::list(["https://api.test"]),
            allowed_headers: AllowedHeaders::Any,
            ..CorsOptions::default()
        };

        let result = options.validate();

        assert!(matches!(
            result,
            Err(ValidationError::AllowedHeadersAnyNotAllowedWithCredentials)
        ));
    }

    #[test]
    fn should_return_error_when_allowed_headers_list_contains_wildcard_then_reject_configuration() {
        let options = CorsOptions {
            allowed_headers: AllowedHeaders::list(["*", "X-Test"]),
            ..CorsOptions::default()
        };

        let result = options.validate();

        assert!(matches!(
            result,
            Err(ValidationError::AllowedHeadersListCannotContainWildcard)
        ));
    }

    #[test]
    fn should_return_error_when_allowed_methods_list_contains_wildcard_then_reject_configuration() {
        let options = CorsOptions {
            methods: AllowedMethods::list(["GET", "*"]),
            ..CorsOptions::default()
        };

        let result = options.validate();

        assert!(matches!(
            result,
            Err(ValidationError::AllowedMethodsCannotContainWildcard)
        ));
    }

    #[test]
    fn should_return_error_when_allowed_methods_list_contains_empty_token_then_reject_configuration()
     {
        let options = CorsOptions {
            methods: AllowedMethods::list(["GET", "  ", "POST"]),
            ..CorsOptions::default()
        };

        assert!(matches!(
            options.validate(),
            Err(ValidationError::AllowedMethodsCannotContainEmptyToken)
        ));
    }

    #[test]
    fn should_return_error_when_allowed_methods_list_contains_invalid_token_then_reject_configuration()
     {
        let options = CorsOptions {
            methods: AllowedMethods::list(["GET", "PO ST"]),
            ..CorsOptions::default()
        };

        let result = options.validate();

        assert!(matches!(
            result,
            Err(ValidationError::AllowedMethodsListContainsInvalidToken)
        ));
    }

    #[test]
    fn should_return_error_when_allowed_headers_list_contains_invalid_token_then_reject_configuration()
     {
        let options = CorsOptions {
            allowed_headers: AllowedHeaders::list(["X-Trace", "X Header"]),
            ..CorsOptions::default()
        };

        let result = options.validate();

        assert!(matches!(
            result,
            Err(ValidationError::AllowedHeadersListContainsInvalidToken)
        ));
    }

    #[test]
    fn should_return_error_when_allowed_headers_list_contains_empty_token_then_reject_configuration()
     {
        let options = CorsOptions {
            allowed_headers: AllowedHeaders::list(["X-Test", "  "]),
            ..CorsOptions::default()
        };

        assert!(matches!(
            options.validate(),
            Err(ValidationError::AllowedHeadersCannotContainEmptyToken)
        ));
    }

    #[test]
    fn should_return_error_when_expose_headers_wildcard_with_credentials_then_require_credentials_disabled()
     {
        let options = CorsOptions {
            exposed_headers: Some(vec!["*".to_string()]),
            credentials: true,
            origin: Origin::list(["https://api.test"]),
            ..CorsOptions::default()
        };

        let result = options.validate();

        assert!(matches!(
            result,
            Err(ValidationError::ExposeHeadersWildcardRequiresCredentialsDisabled)
        ));
    }

    #[test]
    fn should_return_error_when_expose_headers_contains_invalid_token_then_reject_configuration() {
        let options = CorsOptions {
            exposed_headers: Some(vec!["X-Trace".to_string(), "X Header".to_string()]),
            ..CorsOptions::default()
        };

        let result = options.validate();

        assert!(matches!(
            result,
            Err(ValidationError::ExposeHeadersListContainsInvalidToken)
        ));
    }

    #[test]
    fn should_return_ok_when_expose_headers_wildcard_without_credentials_then_accept_configuration()
    {
        let options = CorsOptions {
            exposed_headers: Some(vec!["*".to_string()]),
            ..CorsOptions::default()
        };

        let result = options.validate();

        assert!(result.is_ok());
    }

    #[test]
    fn should_return_error_when_expose_headers_wildcard_combined_with_headers_then_reject_configuration()
     {
        let options = CorsOptions {
            exposed_headers: Some(vec!["*".to_string(), "X-Test".to_string()]),
            ..CorsOptions::default()
        };

        let result = options.validate();

        assert!(matches!(
            result,
            Err(ValidationError::ExposeHeadersWildcardCannotBeCombined)
        ));
    }

    #[test]
    fn should_return_error_when_max_age_negative_then_reject_configuration() {
        let options = CorsOptions {
            max_age: Some("-1".to_string()),
            ..CorsOptions::default()
        };

        let result = options.validate();

        assert!(matches!(result, Err(ValidationError::InvalidMaxAge(_))));
    }

    #[test]
    fn should_return_error_when_expose_headers_contains_empty_value_then_reject_configuration() {
        let options = CorsOptions {
            exposed_headers: Some(vec!["  ".to_string(), "X-Trace".to_string()]),
            ..CorsOptions::default()
        };

        let result = options.validate();

        assert!(matches!(
            result,
            Err(ValidationError::ExposeHeadersCannotContainEmptyValue)
        ));
    }

    #[test]
    fn should_return_error_when_private_network_enabled_without_credentials_then_require_credentials()
     {
        let options = CorsOptions {
            allow_private_network: true,
            ..CorsOptions::default()
        };

        assert!(matches!(
            options.validate(),
            Err(ValidationError::PrivateNetworkRequiresCredentials)
        ));
    }

    #[test]
    fn should_return_error_when_private_network_enabled_with_wildcard_origin_then_require_specific_origin()
     {
        let options = CorsOptions {
            allow_private_network: true,
            credentials: true,
            origin: Origin::any(),
            ..CorsOptions::default()
        };

        assert!(matches!(
            options.validate(),
            Err(ValidationError::PrivateNetworkRequiresSpecificOrigin)
        ));
    }

    #[test]
    fn should_return_ok_when_private_network_enabled_with_specific_origin_then_accept_configuration()
     {
        let options = CorsOptions {
            allow_private_network: true,
            credentials: true,
            origin: Origin::list(["https://intranet.test"]),
            ..CorsOptions::default()
        };

        assert!(options.validate().is_ok());
    }

    #[test]
    fn should_return_ok_when_configuration_specific_then_accept_settings() {
        let options = CorsOptions {
            origin: Origin::list(["https://api.test"]),
            allowed_headers: AllowedHeaders::list(["X-Test"]),
            exposed_headers: Some(vec!["X-Expose".to_string()]),
            credentials: true,
            timing_allow_origin: Some(TimingAllowOrigin::list(["https://metrics.test"])),
            ..CorsOptions::default()
        };

        let result = options.validate();

        assert!(result.is_ok());
    }

    #[test]
    fn should_return_error_when_timing_allow_origin_any_with_credentials_then_reject_configuration()
    {
        let options = CorsOptions {
            credentials: true,
            timing_allow_origin: Some(TimingAllowOrigin::any()),
            origin: Origin::list(["https://api.test"]),
            ..CorsOptions::default()
        };

        let result = options.validate();

        assert!(matches!(
            result,
            Err(ValidationError::TimingAllowOriginWildcardNotAllowedWithCredentials)
        ));
    }

    #[test]
    fn should_return_error_when_timing_allow_origin_contains_empty_entry_then_reject_configuration()
    {
        let options = CorsOptions {
            timing_allow_origin: Some(TimingAllowOrigin::list([" ", "https://metrics.test"])),
            ..CorsOptions::default()
        };

        let result = options.validate();

        assert!(matches!(
            result,
            Err(ValidationError::TimingAllowOriginCannotContainEmptyValue)
        ));
    }

    #[test]
    fn should_return_error_when_max_age_not_numeric_then_reject_configuration() {
        let options = CorsOptions {
            max_age: Some("ten minutes".into()),
            ..CorsOptions::default()
        };

        let result = options.validate();

        assert!(matches!(
            result,
            Err(ValidationError::InvalidMaxAge(value)) if value == "ten minutes"
        ));
    }

    #[test]
    fn should_return_error_when_max_age_blank_then_reject_configuration() {
        let options = CorsOptions {
            max_age: Some("  ".into()),
            ..CorsOptions::default()
        };

        assert!(matches!(
            options.validate(),
            Err(ValidationError::InvalidMaxAge(value)) if value.trim().is_empty()
        ));
    }
}
