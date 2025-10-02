use super::AllowedMethods;
use crate::constants::method;

mod list {
    use super::*;

    #[test]
    fn should_collect_methods_when_values_provided_then_preserve_insertion_order() {
        let methods = ["GET", "POST"];

        let result = AllowedMethods::list(methods);

        let expected = AllowedMethods::from(vec!["GET".to_string(), "POST".to_string()]);
        assert_eq!(result, expected);
    }

    #[test]
    fn should_return_empty_collection_when_iterator_empty_then_initialize_default() {
        let methods: [&str; 0] = [];

        let result = AllowedMethods::list(methods);

        let expected = AllowedMethods::from(Vec::<String>::new());
        assert_eq!(result, expected);
    }

    #[test]
    fn should_preserve_positions_when_values_include_empty_entries_then_keep_sequence() {
        let methods = ["", "GET"];

        let result = AllowedMethods::list(methods);

        let expected = AllowedMethods::from(vec![String::new(), "GET".to_string()]);
        assert_eq!(result, expected);
    }

    #[test]
    fn should_ignore_case_duplicates_when_values_include_duplicates_then_keep_first_instance() {
        let methods = ["GET", "get", "POST"];

        let result = AllowedMethods::list(methods);

        let expected = AllowedMethods::from(vec!["GET".to_string(), "POST".to_string()]);
        assert_eq!(result, expected);
    }
}

mod header_value {
    use super::*;

    #[test]
    fn should_return_none_when_list_empty_then_skip_header_value() {
        let methods = AllowedMethods::list(Vec::<String>::new());

        let header = methods.header_value();

        assert!(header.is_none());
    }

    #[test]
    fn should_join_methods_when_list_has_entries_then_emit_comma_delimited_value() {
        let methods = AllowedMethods::list(["GET", "PATCH"]);

        let header = methods.header_value();

        assert_eq!(header.as_deref(), Some("GET,PATCH"));
    }

    #[test]
    fn should_include_separator_when_list_contains_empty_entry_then_preserve_positions() {
        let methods = AllowedMethods::list(["", "GET"]);

        let header = methods.header_value();

        assert_eq!(header.as_deref(), Some(",GET"));
    }
}

mod allows_method {
    use super::*;

    #[test]
    fn should_allow_method_when_case_differs_then_accept_request() {
        let methods = AllowedMethods::list(["POST", "PATCH"]);

        let is_allowed = methods.allows_method("post");

        assert!(is_allowed);
    }

    #[test]
    fn should_reject_method_when_not_allowed_then_deny_request() {
        let methods = AllowedMethods::list(["POST", "PATCH"]);

        let is_allowed = methods.allows_method("DELETE");

        assert!(!is_allowed);
    }

    #[test]
    fn should_reject_method_when_value_empty_then_deny_request() {
        let methods = AllowedMethods::list(["GET"]);

        let is_allowed = methods.allows_method("");

        assert!(!is_allowed);
    }

    #[test]
    fn should_reject_method_when_value_whitespace_then_deny_request() {
        let methods = AllowedMethods::list(["GET"]);

        let is_allowed = methods.allows_method("  ");

        assert!(!is_allowed);
    }
}

mod default {
    use super::*;

    #[test]
    fn should_return_standard_methods_when_default_constructed_then_use_predefined_set() {
        let methods = AllowedMethods::default();

        let expected = AllowedMethods::list([
            method::GET,
            method::HEAD,
            method::PUT,
            method::PATCH,
            method::POST,
            method::DELETE,
        ]);
        assert_eq!(methods, expected);
    }
}

mod iter {
    use super::*;

    #[test]
    fn should_iterate_in_insertion_order_when_iter_called_then_preserve_sequence() {
        let methods = AllowedMethods::list(["OPTIONS", "PATCH", "GET"]);

        let collected: Vec<&str> = methods.iter().map(String::as_str).collect();

        assert_eq!(collected, vec!["OPTIONS", "PATCH", "GET"]);
    }
}

mod as_slice {
    use super::*;

    #[test]
    fn should_expose_inner_slice_when_as_slice_called_then_return_backing_storage() {
        let methods = AllowedMethods::list(["DELETE", "HEAD"]);

        let slice = methods.as_slice();

        assert_eq!(slice, &["DELETE".to_string(), "HEAD".to_string()]);
    }
}

mod into_inner {
    use super::*;

    #[test]
    fn should_consume_collection_when_into_inner_called_then_return_values() {
        let methods = AllowedMethods::list(["POST", "PATCH"]);

        let values = methods.into_inner();

        assert_eq!(values, vec!["POST".to_string(), "PATCH".to_string()]);
    }
}

mod into_iter {
    use super::*;

    #[test]
    fn should_iterate_without_consuming_when_into_iter_called_on_reference_then_preserve_collection() {
        let methods = AllowedMethods::list(["DELETE", "TRACE"]);

        let collected: Vec<&str> = (&methods).into_iter().map(String::as_str).collect();

        assert_eq!(collected, vec!["DELETE", "TRACE"]);
        assert_eq!(methods.as_slice(), &["DELETE".to_string(), "TRACE".to_string()]);
    }

    #[test]
    fn should_consume_collection_when_into_iter_called_then_collect_values() {
        let methods = AllowedMethods::list(["PUT", "PATCH"]);

        let collected: Vec<String> = methods.into_iter().collect();

        assert_eq!(collected, vec!["PUT".to_string(), "PATCH".to_string()]);
    }
}

mod from {
    use super::*;

    #[test]
    fn should_wrap_values_without_modification_when_from_vec_used_then_preserve_entries() {
        let original = vec!["CUSTOM".to_string(), "METHOD".to_string()];

        let methods = AllowedMethods::from(original.clone());

        assert_eq!(methods.into_inner(), original);
    }
}

mod into_vec {
    use super::*;

    #[test]
    fn should_consume_and_return_values_when_into_vec_called_then_retrieve_collection() {
        let methods = AllowedMethods::list(["GET", "POST"]);

        let values: Vec<String> = methods.into();

        assert_eq!(values, vec!["GET".to_string(), "POST".to_string()]);
    }
}

mod deref {
    use super::*;

    #[test]
    fn should_allow_slice_coercion_when_deref_used_then_support_index_access() {
        let methods = AllowedMethods::list(["POST", "PATCH"]);

        let slice: &[String] = &methods;

        assert_eq!(slice, &["POST".to_string(), "PATCH".to_string()]);
    }
}

mod deref_mut {
    use super::*;
    use std::ops::DerefMut;

    #[test]
    fn should_allow_in_place_mutation_when_deref_mut_used_then_mutate_entries() {
        let mut methods = AllowedMethods::list(["POST"]);

        methods.deref_mut()[0].make_ascii_lowercase();

        assert_eq!(methods.into_inner(), vec!["post".to_string()]);
    }
}
