use super::*;

mod list {
    use super::*;

    #[test]
    fn should_return_list_variant_when_values_provided_then_collect_values() {
        let input = ["Content-Type", "X-Custom"];

        let result = AllowedHeaders::list(input);

        match result {
            AllowedHeaders::List(list) => {
                assert_eq!(
                    list.values(),
                    &["Content-Type".to_string(), "X-Custom".to_string()]
                );
            }
            _ => panic!("expected list variant"),
        }
    }

    #[test]
    fn should_return_empty_list_variant_when_iterator_empty_then_initialize_default() {
        let input: [&str; 0] = [];

        let result = AllowedHeaders::list(input);

        match result {
            AllowedHeaders::List(list) => {
                assert!(list.values().is_empty());
            }
            _ => panic!("expected list variant"),
        }
    }

    #[test]
    fn should_preserve_insertion_order_when_values_include_empty_entries_then_keep_positions() {
        let input = ["", "X-Custom"];

        let result = AllowedHeaders::list(input);

        match result {
            AllowedHeaders::List(list) => {
                assert_eq!(list.values(), &[String::new(), "X-Custom".to_string()]);
            }
            _ => panic!("expected list variant"),
        }
    }

    #[test]
    fn should_ignore_case_duplicates_when_values_include_duplicates_then_keep_first_instance() {
        let input = ["X-Trace", "x-trace", "X-Other"];

        let result = AllowedHeaders::list(input);

        match result {
            AllowedHeaders::List(list) => {
                assert_eq!(
                    list.values(),
                    &["X-Trace".to_string(), "X-Other".to_string()]
                );
            }
            _ => panic!("expected list variant"),
        }
    }

    #[test]
    fn should_store_normalized_lookup_set_when_values_mixed_case_then_support_case_insensitive_queries()
     {
        let input = ["X-Trace", "X-Other"];

        let result = AllowedHeaders::list(input);

        match result {
            AllowedHeaders::List(list) => {
                assert!(list.allows_headers("x-trace"));
                assert!(list.allows_headers("X-other"));
                assert!(!list.allows_headers("x-missing"));
            }
            _ => panic!("expected list variant"),
        }
    }
}

mod any {
    use super::*;

    #[test]
    fn should_return_any_variant_when_called_then_provide_wildcard_access() {
        let result = AllowedHeaders::Any;

        assert!(matches!(result, AllowedHeaders::Any));
    }
}

mod default {
    use super::*;

    #[test]
    fn should_return_empty_list_variant_when_default_then_match_constructor() {
        let value = AllowedHeaders::default();

        match value {
            AllowedHeaders::List(list) => {
                assert!(list.values().is_empty());
            }
            _ => panic!("expected list variant"),
        }
    }
}

mod allows_headers {
    use super::*;

    #[test]
    fn should_allow_all_headers_when_any_variant_then_accept_request_headers() {
        let headers = AllowedHeaders::Any;

        let is_allowed = headers.allows_headers("x-custom");

        assert!(is_allowed);
    }

    #[test]
    fn should_allow_all_headers_with_cache_when_any_variant_then_return_true() {
        let headers = AllowedHeaders::Any;
        let mut cache = AllowedHeadersCache::new();

        assert!(headers.allows_headers_with_cache("x-custom", &mut cache));
    }

    #[test]
    fn should_allow_headers_when_case_differs_then_accept_request() {
        let headers = AllowedHeaders::list(["X-Custom", "Content-Type"]);

        let is_allowed = headers.allows_headers("x-custom, content-type");

        assert!(is_allowed);
    }

    #[test]
    fn should_reject_header_when_value_not_allowed_then_deny_request() {
        let headers = AllowedHeaders::list(["X-Custom", "Content-Type"]);

        let is_allowed = headers.allows_headers("x-custom, x-other");

        assert!(!is_allowed);
    }

    #[test]
    fn should_allow_headers_when_request_header_empty_then_default_to_true() {
        let headers = AllowedHeaders::list(["X-Custom"]);

        let is_allowed = headers.allows_headers("  ");

        assert!(is_allowed);
    }

    #[test]
    fn should_ignore_extra_commas_and_whitespace_when_request_headers_sparse_then_validate_each_token()
     {
        let headers = AllowedHeaders::list(["X-Custom", "Content-Type"]);

        let is_allowed = headers.allows_headers(",, x-custom ,  , CONTENT-TYPE ,, ");

        assert!(is_allowed);
    }

    #[test]
    fn should_reject_when_request_contains_disallowed_token_amidst_allowed_headers() {
        let headers = AllowedHeaders::list(["X-Custom", "Content-Type"]);

        let is_allowed = headers.allows_headers("content-type, x-forbidden, x-custom");

        assert!(!is_allowed);
    }

    #[test]
    fn should_allow_headers_when_tokens_filter_to_empty_then_default_to_true() {
        let headers = AllowedHeaders::list(["X-Custom"]);
        let mut cache = AllowedHeadersCache::new();

        let is_allowed = headers.allows_headers_with_cache(", , ,", &mut cache);

        assert!(is_allowed);
    }
}

mod cache_behavior {
    use super::*;

    #[test]
    fn should_reuse_tokens_when_identity_matches_then_skip_normalization() {
        let mut cache = AllowedHeadersCache::new();
        let request = "X-Custom";

        let first = cache.prepare(request);
        assert_eq!(first, &["x-custom".to_string()]);

        cache.normalized_tokens.push("sentinel".to_string());

        let second = cache.prepare(request);
        assert_eq!(second.len(), 2);
        assert_eq!(second[1], "sentinel");
    }

    #[test]
    fn should_reset_cache_when_reset_called_then_clear_state() {
        let mut cache = AllowedHeadersCache::new();
        cache.prepare("X-Custom");

        cache.reset();

        assert_eq!(cache.identity, (0, 0));
        assert!(cache.normalized_tokens.is_empty());
    }
}
