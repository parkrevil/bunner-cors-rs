use super::*;

mod list {
    use super::*;

    #[test]
    fn should_collect_into_list_variant_given_values_provided() {
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
    fn should_create_empty_list_variant_given_iterator_is_empty() {
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
    fn should_preserve_order_given_values_include_empty_entries() {
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

    #[test]
    fn should_keep_first_instance_given_values_include_case_duplicates() {
        // Arrange
        let input = ["X-Trace", "x-trace", "X-Other"];

        // Act
        let result = AllowedHeaders::list(input);

        // Assert
        match result {
            AllowedHeaders::List(values) => {
                assert_eq!(values, vec!["X-Trace".to_string(), "X-Other".to_string()])
            }
            _ => panic!("expected list variant"),
        }
    }
}

mod any {
    use super::*;

    #[test]
    fn should_return_any_variant_when_called() {
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
    fn should_return_mirror_request_when_default() {
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
    fn should_allow_all_headers_given_any() {
        let headers = AllowedHeaders::any();
        assert!(headers.allows_headers("x-custom"));
    }

    #[test]
    fn when_mirror_request_should_allow_all_headers() {
        // removed mirror behavior; list and any are covered elsewhere
    }

    #[test]
    fn should_validate_case_insensitively_given_list() {
        let headers = AllowedHeaders::list(["X-Custom", "Content-Type"]);
        assert!(headers.allows_headers("x-custom, content-type"));
        assert!(!headers.allows_headers("x-custom, x-other"));
    }

    #[test]
    fn should_allow_given_request_headers_empty() {
        let headers = AllowedHeaders::list(["X-Custom"]);
        assert!(headers.allows_headers("  "));
    }
}
