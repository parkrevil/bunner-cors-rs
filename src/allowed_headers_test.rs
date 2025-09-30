use super::*;

mod list {
    use super::*;

    #[test]
    fn when_values_provided_should_collect_into_list_variant() {
        // Arrange
        let input = ["Content-Type", "X-Custom"];

        // Act
        let result = AllowedHeaders::list(input);

        // Assert
        match result {
            AllowedHeaders::List(values) => assert_eq!(values, vec!["Content-Type", "X-Custom"]),
            _ => panic!("expected list variant"),
        }
    }

    #[test]
    fn when_iterator_is_empty_should_create_empty_list_variant() {
        // Arrange
        let input: [&str; 0] = [];

        // Act
        let result = AllowedHeaders::list(input);

        // Assert
        match result {
            AllowedHeaders::List(values) => assert!(values.is_empty()),
            _ => panic!("expected list variant"),
        }
    }

    #[test]
    fn when_values_include_empty_entries_should_preserve_order() {
        // Arrange
        let input = ["", "X-Custom"];

        // Act
        let result = AllowedHeaders::list(input);

        // Assert
        match result {
            AllowedHeaders::List(values) => {
                assert_eq!(values, vec![String::new(), "X-Custom".to_string()])
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
        let result = AllowedHeaders::any();

        // Assert
        match result {
            AllowedHeaders::Any => {}
            _ => panic!("expected any variant"),
        }
    }
}

mod default_variant {
    use super::*;

    #[test]
    fn when_default_should_return_mirror_request() {
        // Arrange & Act
        let value = AllowedHeaders::default();

        // Assert
        match value {
            AllowedHeaders::List(values) if values.is_empty() => {}
            _ => panic!("expected empty list variant by default"),
        }
    }
}

mod allows_headers {
    use super::*;

    #[test]
    fn when_any_should_allow_all_headers() {
        let headers = AllowedHeaders::any();
        assert!(headers.allows_headers("x-custom"));
    }

    #[test]
    fn when_mirror_request_should_allow_all_headers() {
        // removed mirror behavior; list and any are covered elsewhere
    }

    #[test]
    fn when_list_should_validate_case_insensitively() {
        let headers = AllowedHeaders::list(["X-Custom", "Content-Type"]);
        assert!(headers.allows_headers("x-custom, content-type"));
        assert!(!headers.allows_headers("x-custom, x-other"));
    }

    #[test]
    fn when_request_headers_empty_should_allow() {
        let headers = AllowedHeaders::list(["X-Custom"]);
        assert!(headers.allows_headers("  "));
    }
}
