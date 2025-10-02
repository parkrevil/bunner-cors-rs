use super::*;
use crate::constants::header;

mod new {
    use super::*;

    #[test]
    fn should_start_with_empty_headers_when_called() {
        let collection = HeaderCollection::new();

        assert!(collection.into_headers().is_empty());
    }
}

mod push {
    use super::*;

    #[test]
    fn should_store_once_given_header_is_regular() {
        let mut collection = HeaderCollection::new();

        collection.push("Access-Control-Expose-Headers".into(), "X-Trace".into());

        let headers = collection.into_headers();
        assert_eq!(
            headers.get("Access-Control-Expose-Headers"),
            Some(&"X-Trace".to_string())
        );
    }

    #[test]
    fn should_use_deduplicated_value_given_header_is_vary() {
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
    fn should_store_unique_entries_given_values_have_mixed_case() {
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
    fn should_remove_vary_header_given_value_is_whitespace() {
        let mut collection = HeaderCollection::new();

        collection.add_vary("   ");

        let headers = collection.into_headers();
        assert!(!headers.contains_key(header::VARY));
    }

    #[test]
    fn should_preserve_them_given_value_is_whitespace_and_existing_entries_present() {
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
    fn should_combine_and_deduplicate_given_merging_collections() {
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
    fn should_remove_header_given_extending_with_whitespace_vary() {
        let mut base = HeaderCollection::new();
        let mut other = HeaderCollection::new();
        other.push(header::VARY.to_string(), "   ".into());

        base.extend(other);

        let headers = base.into_headers();
        assert!(!headers.contains_key(header::VARY));
    }

    #[test]
    fn should_preserve_value_given_extending_existing_vary_with_whitespace() {
        let mut base = HeaderCollection::new();
        base.add_vary("Origin");
        let mut other = HeaderCollection::new();
        other.push(header::VARY.to_string(), "   ".into());

        base.extend(other);

        let headers = base.into_headers();
        assert_eq!(headers.get(header::VARY), Some(&"Origin".to_string()));
    }

    #[test]
    fn should_merge_vary_from_other_collection() {
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
    fn should_consume_collection_and_return_map_when_called() {
        let mut collection = HeaderCollection::new();
        collection.push("Access-Control-Allow-Methods".into(), "GET".into());

        let headers = collection.into_headers();

        assert_eq!(
            headers.get("Access-Control-Allow-Methods"),
            Some(&"GET".to_string())
        );
    }

    #[test]
    fn should_emit_vary_header_before_others() {
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
