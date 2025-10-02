use super::*;
use crate::constants::header;

mod new {
    use super::*;

    #[test]
    fn should_start_with_empty_headers_when_new_called_then_initialize_collection() {
        let collection = HeaderCollection::new();

        assert!(collection.into_headers().is_empty());
    }
}

mod push {
    use super::*;

    #[test]
    fn should_store_header_once_when_header_regular_then_persist_value() {
        let mut collection = HeaderCollection::new();

        collection.push("Access-Control-Expose-Headers".into(), "X-Trace".into());

        let headers = collection.into_headers();
        assert_eq!(
            headers.get("Access-Control-Expose-Headers"),
            Some(&"X-Trace".to_string())
        );
    }

    #[test]
    fn should_use_deduplicated_value_when_header_vary_then_preserve_first_entry() {
        let mut collection = HeaderCollection::new();

        collection.push(header::VARY.to_string(), "Origin".into());
        collection.push(header::VARY.to_string(), "origin".into());

        let headers = collection.into_headers();
        assert_eq!(headers.get(header::VARY), Some(&"Origin".to_string()));
    }
}

mod add_vary {
    use super::*;

    #[test]
    fn should_store_unique_entries_when_values_have_mixed_case_then_deduplicate_case_insensitively() {
        let mut collection = HeaderCollection::new();

        collection.add_vary("Origin");
        collection.add_vary("Access-Control-Request-Headers");
        collection.add_vary("origin");

        let headers = collection.into_headers();
        assert_eq!(
            headers.get(header::VARY),
            Some(&"Origin, Access-Control-Request-Headers".to_string())
        );
    }

    #[test]
    fn should_remove_vary_header_when_value_whitespace_then_skip_entry() {
        let mut collection = HeaderCollection::new();

        collection.add_vary("   ");

        let headers = collection.into_headers();
        assert!(!headers.contains_key(header::VARY));
    }

    #[test]
    fn should_preserve_existing_entries_when_value_whitespace_then_ignore_addition() {
        let mut collection = HeaderCollection::new();
        collection.add_vary("Origin");

        collection.add_vary("   ");

        let headers = collection.into_headers();
        assert_eq!(headers.get(header::VARY), Some(&"Origin".to_string()));
    }
}

mod extend {
    use super::*;

    #[test]
    fn should_combine_and_deduplicate_when_extending_collections_then_merge_headers() {
        let mut base = HeaderCollection::new();
        base.push("Access-Control-Allow-Credentials".into(), "true".into());
        base.add_vary("Origin");
        let mut other = HeaderCollection::new();
        other.push(header::VARY.to_string(), "origin".into());
        other.push("Access-Control-Expose-Headers".into(), "X-Trace".into());

        base.extend(other);

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
    fn should_remove_vary_header_when_extending_with_whitespace_then_skip_entry() {
        let mut base = HeaderCollection::new();
        let mut other = HeaderCollection::new();
        other.push(header::VARY.to_string(), "   ".into());

        base.extend(other);

        let headers = base.into_headers();
        assert!(!headers.contains_key(header::VARY));
    }

    #[test]
    fn should_preserve_vary_value_when_extending_with_whitespace_then_retain_existing_entry() {
        let mut base = HeaderCollection::new();
        base.add_vary("Origin");
        let mut other = HeaderCollection::new();
        other.push(header::VARY.to_string(), "   ".into());

        base.extend(other);

        let headers = base.into_headers();
        assert_eq!(headers.get(header::VARY), Some(&"Origin".to_string()));
    }

    #[test]
    fn should_merge_vary_header_when_extending_with_other_collection_then_combine_entries() {
        let mut base = HeaderCollection::new();
        base.add_vary("Access-Control-Request-Method");
        let mut other = HeaderCollection::new();
        other.add_vary("Origin");

        base.extend(other);

        let headers = base.into_headers();
        assert_eq!(
            headers.get(header::VARY),
            Some(&"Access-Control-Request-Method, Origin".to_string())
        );
    }
}

mod into_headers {
    use super::*;

    #[test]
    fn should_consume_collection_and_return_map_when_into_headers_called_then_produce_map() {
        let mut collection = HeaderCollection::new();
        collection.push("Access-Control-Allow-Methods".into(), "GET".into());

        let headers = collection.into_headers();

        assert_eq!(
            headers.get("Access-Control-Allow-Methods"),
            Some(&"GET".to_string())
        );
    }

    #[test]
    fn should_emit_vary_header_first_when_into_headers_called_then_preserve_ordering() {
        let mut collection = HeaderCollection::new();
        collection.add_vary("Origin");
        collection.push("Access-Control-Allow-Methods".into(), "GET".into());

        let headers = collection.into_headers();
        let mut keys = headers.keys();

        assert_eq!(keys.next(), Some(&header::VARY.to_string()));
        assert_eq!(
            keys.next(),
            Some(&"Access-Control-Allow-Methods".to_string())
        );
    }
}
