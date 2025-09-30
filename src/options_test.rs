use super::*;
use crate::allowed_headers::AllowedHeaders;
use crate::allowed_methods::AllowedMethods;
use crate::origin::Origin;

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
