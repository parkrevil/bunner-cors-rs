use super::*;
use crate::constants::method;

mod list {
    use super::*;

    #[test]
    fn when_values_provided_should_collect_into_list_variant() {
        // Arrange
        let methods = ["GET", "POST"];

        // Act
        let result = AllowedMethods::list(methods);

        // Assert
        match result {
            AllowedMethods::List(values) => assert_eq!(values, vec!["GET", "POST"]),
            _ => panic!("expected list variant"),
        }
    }

    #[test]
    fn when_iterator_is_empty_should_create_empty_list_variant() {
        // Arrange
        let methods: [&str; 0] = [];

        // Act
        let result = AllowedMethods::list(methods);

        // Assert
        match result {
            AllowedMethods::List(values) => assert!(values.is_empty()),
            _ => panic!("expected list variant"),
        }
    }

    #[test]
    fn when_values_include_empty_entries_should_preserve_order() {
        // Arrange
        let methods = ["", "GET"];

        // Act
        let result = AllowedMethods::list(methods);

        // Assert
        match result {
            AllowedMethods::List(values) => {
                assert_eq!(values, vec![String::new(), "GET".to_string()])
            }
            _ => panic!("expected list variant"),
        }
    }
}

mod any {
    use super::*;

    #[test]
    fn when_called_should_return_any_variant() {
        // Arrange & Act
        let result = AllowedMethods::any();

        // Assert
        match result {
            AllowedMethods::Any => {}
            _ => panic!("expected any variant"),
        }
    }
}

mod header_value {
    use super::*;

    #[test]
    fn when_variant_is_any_should_return_wildcard() {
        // Arrange
        let methods = AllowedMethods::any();

        // Act
        let result = methods.header_value();

        // Assert
        assert_eq!(result.as_deref(), Some("*"));
    }

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

mod default {
    use super::*;

    #[test]
    fn when_called_should_return_standard_method_list() {
        // Arrange & Act
        let methods = AllowedMethods::default();

        // Assert
        match methods {
            AllowedMethods::List(values) => {
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
            _ => panic!("expected list variant"),
        }
    }
}
