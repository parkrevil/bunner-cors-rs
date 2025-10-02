use super::*;

mod list {
    use super::*;

    #[test]
    fn should_return_list_variant_when_values_provided_then_collect_values() {
        let input = ["Content-Type", "X-Custom"];

        let result = AllowedHeaders::list(input);

        assert!(matches!(
            result,
            AllowedHeaders::List(values) if values == vec!["Content-Type", "X-Custom"]
        ));
    }

    #[test]
    fn should_return_empty_list_variant_when_iterator_empty_then_initialize_default() {
        let input: [&str; 0] = [];

        let result = AllowedHeaders::list(input);

        assert!(matches!(
            result,
            AllowedHeaders::List(values) if values.is_empty()
        ));
    }

    #[test]
    fn should_preserve_insertion_order_when_values_include_empty_entries_then_keep_positions() {
        let input = ["", "X-Custom"];

        let result = AllowedHeaders::list(input);

        assert!(matches!(
            result,
            AllowedHeaders::List(values)
                if values == vec![String::new(), "X-Custom".to_string()]
        ));
    }

    #[test]
    fn should_ignore_case_duplicates_when_values_include_duplicates_then_keep_first_instance() {
        let input = ["X-Trace", "x-trace", "X-Other"];

        let result = AllowedHeaders::list(input);

        assert!(matches!(
            result,
            AllowedHeaders::List(values)
                if values == vec!["X-Trace".to_string(), "X-Other".to_string()]
        ));
    }
}

mod any {
    use super::*;

    #[test]
    fn should_return_any_variant_when_called_then_provide_wildcard_access() {
        let result = AllowedHeaders::any();

        assert!(matches!(result, AllowedHeaders::Any));
    }
}

mod default {
    use super::*;

    #[test]
    fn should_return_empty_list_variant_when_default_then_match_constructor() {
        let value = AllowedHeaders::default();

        assert!(matches!(
            value,
            AllowedHeaders::List(values) if values.is_empty()
        ));
    }
}

mod allows_headers {
    use super::*;

    #[test]
    fn should_allow_all_headers_when_any_variant_then_accept_request_headers() {
        let headers = AllowedHeaders::any();

        let is_allowed = headers.allows_headers("x-custom");

        assert!(is_allowed);
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
}
