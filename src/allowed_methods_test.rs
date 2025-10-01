use crate::allowed_methods::AllowedMethods;
use crate::constants::method;

mod list {
    use super::AllowedMethods;

    #[test]
    fn should_collect_into_list_variant_given_values_provided() {
        // Arrange
        let methods = ["GET", "POST"];

        // Act
        let result = AllowedMethods::list(methods);

        // Assert
        let AllowedMethods::List(values) = result;
        assert_eq!(values, vec!["GET", "POST"]);
    }

    #[test]
    fn should_create_empty_list_variant_given_iterator_is_empty() {
        // Arrange
        let methods: [&str; 0] = [];

        // Act
        let result = AllowedMethods::list(methods);

        // Assert
        let AllowedMethods::List(values) = result;
        assert!(values.is_empty());
    }

    #[test]
    fn should_preserve_order_given_values_include_empty_entries() {
        // Arrange
        let methods = ["", "GET"];

        // Act
        let result = AllowedMethods::list(methods);

        // Assert
        let AllowedMethods::List(values) = result;
        assert_eq!(values, vec![String::new(), "GET".to_string()]);
    }

    #[test]
    fn should_keep_first_instance_given_values_include_case_duplicates() {
        // Arrange
        let methods = ["GET", "get", "POST"];

        // Act
        let result = AllowedMethods::list(methods);

        // Assert
        let AllowedMethods::List(values) = result;
        assert_eq!(values, vec!["GET".to_string(), "POST".to_string()]);
    }
}

mod header_value {
    use super::AllowedMethods;

    #[test]
    fn should_return_none_given_list_is_empty() {
        // Arrange
        let methods = AllowedMethods::list(Vec::<String>::new());

        // Act
        let result = methods.header_value();

        // Assert
        assert!(result.is_none());
    }

    #[test]
    fn should_join_with_commas_given_list_has_values() {
        // Arrange
        let methods = AllowedMethods::list(["GET", "PATCH"]);

        // Act
        let result = methods.header_value();

        // Assert
        assert_eq!(result.as_deref(), Some("GET,PATCH"));
    }

    #[test]
    fn should_include_separator_given_list_contains_empty_and_value() {
        // Arrange
        let methods = AllowedMethods::list(["", "GET"]);

        // Act
        let result = methods.header_value();

        // Assert
        assert_eq!(result.as_deref(), Some(",GET"));
    }
}

mod allows_method {
    use super::AllowedMethods;

    #[test]
    fn should_compare_case_insensitively_given_list() {
        let methods = AllowedMethods::list(["POST", "PATCH"]);
        assert!(methods.allows_method("post"));
        assert!(!methods.allows_method("DELETE"));
    }

    #[test]
    fn should_reject_given_method_missing() {
        let methods = AllowedMethods::list(["GET"]);
        assert!(!methods.allows_method(""));
        assert!(!methods.allows_method("  "));
    }
}

mod default {
    use super::{AllowedMethods, method};

    #[test]
    fn should_return_standard_method_list_when_called() {
        // Arrange & Act
        let methods = AllowedMethods::default();

        // Assert
        let AllowedMethods::List(values) = methods;

        let expected = vec![
            method::GET.to_string(),
            method::HEAD.to_string(),
            method::PUT.to_string(),
            method::PATCH.to_string(),
            method::POST.to_string(),
            method::DELETE.to_string(),
        ];
        assert_eq!(values, expected);
    }
}
