use super::*;
use crate::constants::header;

mod new {
    use super::*;

    #[test]
    fn when_called_should_start_with_empty_headers() {
        // Arrange & Act
        let collection = HeaderCollection::new();

        // Assert
        assert!(collection.into_headers().is_empty());
    }
}

mod push {
    use super::*;

    #[test]
    fn when_header_is_regular_should_store_once() {
        // Arrange
        let mut collection = HeaderCollection::new();

        // Act
        collection.push("Access-Control-Expose-Headers".into(), "X-Trace".into());

        // Assert
        let headers = collection.into_headers();
        assert_eq!(
            headers.get("Access-Control-Expose-Headers"),
            Some(&"X-Trace".to_string())
        );
    }

    #[test]
    fn when_header_is_vary_should_use_deduplicated_value() {
        // Arrange
        let mut collection = HeaderCollection::new();

        // Act
        collection.push(header::VARY.to_string(), "Origin".into());
        collection.push(header::VARY.to_string(), "origin".into());

        // Assert
        let headers = collection.into_headers();
        assert_eq!(headers.get(header::VARY), Some(&"Origin".to_string()));
    }
}

mod add_vary {
    use super::*;

    #[test]
    fn when_values_have_mixed_case_should_store_unique_entries() {
        // Arrange
        let mut collection = HeaderCollection::new();

        // Act
        collection.add_vary("Origin");
        collection.add_vary("Access-Control-Request-Headers");
        collection.add_vary("origin");

        // Assert
        let headers = collection.into_headers();
        assert_eq!(
            headers.get(header::VARY),
            Some(&"Origin, Access-Control-Request-Headers".to_string())
        );
    }

    #[test]
    fn when_value_is_whitespace_should_remove_vary_header() {
        // Arrange
        let mut collection = HeaderCollection::new();

        // Act
        collection.add_vary("   ");

        // Assert
        let headers = collection.into_headers();
        assert!(!headers.contains_key(header::VARY));
    }

    #[test]
    fn when_value_is_whitespace_and_existing_entries_present_should_preserve_them() {
        // Arrange
        let mut collection = HeaderCollection::new();
        collection.add_vary("Origin");

        // Act
        collection.add_vary("   ");

        // Assert
        let headers = collection.into_headers();
        assert_eq!(headers.get(header::VARY), Some(&"Origin".to_string()));
    }
}

mod extend {
    use super::*;

    #[test]
    fn when_merging_collections_should_combine_and_deduplicate() {
        // Arrange
        let mut base = HeaderCollection::new();
        base.push("Access-Control-Allow-Credentials".into(), "true".into());
        base.add_vary("Origin");
        let mut other = HeaderCollection::new();
        other.push(header::VARY.to_string(), "origin".into());
        other.push("Access-Control-Expose-Headers".into(), "X-Trace".into());

        // Act
        base.extend(other);

        // Assert
        let headers = base.into_headers();
        assert_eq!(headers.get(header::VARY), Some(&"Origin".to_string()));
        assert_eq!(
            headers.get("Access-Control-Allow-Credentials"),
            Some(&"true".to_string())
        );
        assert_eq!(
            headers.get("Access-Control-Expose-Headers"),
            Some(&"X-Trace".to_string())
        );
    }

    #[test]
    fn when_extending_with_whitespace_vary_should_remove_header() {
        // Arrange
        let mut base = HeaderCollection::new();
        let mut other = HeaderCollection::new();
        other.push(header::VARY.to_string(), "   ".into());

        // Act
        base.extend(other);

        // Assert
        let headers = base.into_headers();
        assert!(!headers.contains_key(header::VARY));
    }

    #[test]
    fn when_extending_existing_vary_with_whitespace_should_preserve_value() {
        // Arrange
        let mut base = HeaderCollection::new();
        base.add_vary("Origin");
        let mut other = HeaderCollection::new();
        other.push(header::VARY.to_string(), "   ".into());

        // Act
        base.extend(other);

        // Assert
        let headers = base.into_headers();
        assert_eq!(headers.get(header::VARY), Some(&"Origin".to_string()));
    }
}

mod into_headers {
    use super::*;

    #[test]
    fn when_called_should_consume_collection_and_return_map() {
        // Arrange
        let mut collection = HeaderCollection::new();
        collection.push("Access-Control-Allow-Methods".into(), "GET".into());

        // Act
        let headers = collection.into_headers();

        // Assert
        assert_eq!(
            headers.get("Access-Control-Allow-Methods"),
            Some(&"GET".to_string())
        );
    }
}
