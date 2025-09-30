use crate::allowed_methods::AllowedMethods;
use crate::constants::method;

mod list {
    use super::AllowedMethods;

    #[test]
    fn when_values_provided_should_collect_into_list_variant() {
        // Arrange
        let methods = ["GET", "POST"];

        // Act
        let result = AllowedMethods::list(methods);

        // Assert
        let values = match result {
            AllowedMethods::List(values) => values,
        };
        assert_eq!(values, vec!["GET", "POST"]);
    }

    #[test]
    fn when_iterator_is_empty_should_create_empty_list_variant() {
        // Arrange
        let methods: [&str; 0] = [];

        // Act
        let result = AllowedMethods::list(methods);

        // Assert
        let values = match result {
            AllowedMethods::List(values) => values,
        };
        assert!(values.is_empty());
    }

    #[test]
    fn when_values_include_empty_entries_should_preserve_order() {
        // Arrange
        let methods = ["", "GET"];

        // Act
        let result = AllowedMethods::list(methods);

        // Assert
        let values = match result {
            AllowedMethods::List(values) => values,
        };
        assert_eq!(values, vec![String::new(), "GET".to_string()]);
    }
}

mod header_value {
    use super::AllowedMethods;

    #[test]
    fn when_list_is_empty_should_return_none() {
        // Arrange
        let methods = AllowedMethods::list(Vec::<String>::new());

        // Act
        let result = methods.header_value();

        // Assert
        assert!(result.is_none());
    }

    #[test]
    fn when_list_has_values_should_join_with_commas() {
        // Arrange
        let methods = AllowedMethods::list(["GET", "PATCH"]);

        // Act
        let result = methods.header_value();

        // Assert
        assert_eq!(result.as_deref(), Some("GET,PATCH"));
    }

    #[test]
    fn when_list_contains_empty_and_value_should_include_separator() {
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
    fn when_list_should_compare_case_insensitively() {
        let methods = AllowedMethods::list(["POST", "PATCH"]);
        assert!(methods.allows_method("post"));
        assert!(!methods.allows_method("DELETE"));
    }

    #[test]
    fn when_method_missing_should_reject() {
        let methods = AllowedMethods::list(["GET"]);
        assert!(!methods.allows_method(""));
        assert!(!methods.allows_method("  "));
    }
}

mod default {
    use super::{method, AllowedMethods};

    #[test]
    fn when_called_should_return_standard_method_list() {
        // Arrange & Act
        let methods = AllowedMethods::default();

        // Assert
        let values = match methods {
            AllowedMethods::List(values) => values,
        };

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
