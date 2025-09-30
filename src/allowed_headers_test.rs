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
            AllowedHeaders::MirrorRequest => {}
            _ => panic!("expected mirror request variant"),
        }
    }
}
