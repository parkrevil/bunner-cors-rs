use super::AllowedMethods;
use crate::constants::method;

mod list {
    use super::*;

    #[test]
    fn should_collect_into_list_variant_given_values_provided() {
        let methods = ["GET", "POST"];

        let result = AllowedMethods::list(methods);

        assert_eq!(result.into_inner(), vec!["GET", "POST"]);
    }

    #[test]
    fn should_create_empty_list_variant_given_iterator_is_empty() {
        let methods: [&str; 0] = [];

        let result = AllowedMethods::list(methods);

        assert!(result.into_inner().is_empty());
    }

    #[test]
    fn should_preserve_order_given_values_include_empty_entries() {
        let methods = ["", "GET"];

        let result = AllowedMethods::list(methods);

        assert_eq!(result.into_inner(), vec![String::new(), "GET".to_string()]);
    }

    #[test]
    fn should_keep_first_instance_given_values_include_case_duplicates() {
        let methods = ["GET", "get", "POST"];

        let result = AllowedMethods::list(methods);

        assert_eq!(
            result.into_inner(),
            vec!["GET".to_string(), "POST".to_string()]
        );
    }
}

mod header_value {
    use super::*;

    #[test]
    fn should_return_none_given_list_is_empty() {
        let methods = AllowedMethods::list(Vec::<String>::new());

        let result = methods.header_value();

        assert!(result.is_none());
    }

    #[test]
    fn should_join_with_commas_given_list_has_values() {
        let methods = AllowedMethods::list(["GET", "PATCH"]);

        let result = methods.header_value();

        assert_eq!(result.as_deref(), Some("GET,PATCH"));
    }

    #[test]
    fn should_include_separator_given_list_contains_empty_and_value() {
        let methods = AllowedMethods::list(["", "GET"]);

        let result = methods.header_value();

        assert_eq!(result.as_deref(), Some(",GET"));
    }
}

mod allows_method {
    use super::*;

    #[test]
    fn should_compare_case_insensitively_given_list() {
        let methods = AllowedMethods::list(["POST", "PATCH"]);
        assert!(methods.allows_method("post"));
        assert!(!methods.allows_method("DELETE"));
    }

    #[test]
    fn should_reject_given_method_missing() {
        let methods = AllowedMethods::list(["GET"]);
        assert!(!methods.allows_method(""));
        assert!(!methods.allows_method("  "));
    }
}

mod default {
    use super::*;

    #[test]
    fn should_return_standard_method_list_when_called() {
        let methods = AllowedMethods::default();

        let expected = vec![
            method::GET.to_string(),
            method::HEAD.to_string(),
            method::PUT.to_string(),
            method::PATCH.to_string(),
            method::POST.to_string(),
            method::DELETE.to_string(),
        ];
        assert_eq!(methods.into_inner(), expected);
    }
}

mod accessors {
    use super::*;

    #[test]
    fn should_return_methods_in_insertion_order_given_iter() {
        let methods = AllowedMethods::list(["OPTIONS", "PATCH", "GET"]);

        let collected: Vec<&str> = methods.iter().map(String::as_str).collect();

        assert_eq!(collected, vec!["OPTIONS", "PATCH", "GET"]);
    }

    #[test]
    fn should_return_methods_without_consuming_given_iter_reference() {
        let methods = AllowedMethods::list(["DELETE", "TRACE"]);

        let collected: Vec<&str> = (&methods).into_iter().map(String::as_str).collect();

        assert_eq!(collected, vec!["DELETE", "TRACE"]);
        assert_eq!(
            methods.as_slice(),
            &["DELETE".to_string(), "TRACE".to_string()]
        );
    }

    #[test]
    fn should_expose_inner_slice_given_as_slice() {
        let methods = AllowedMethods::list(["DELETE", "HEAD"]);

        assert_eq!(
            methods.as_slice(),
            &["DELETE".to_string(), "HEAD".to_string()]
        );
    }
}

mod deref_support {
    use super::*;
    use std::ops::DerefMut;

    #[test]
    fn should_allow_slice_coercion_given_deref() {
        let methods = AllowedMethods::list(["POST", "PATCH"]);

        let slice: &[String] = &methods;

        assert_eq!(slice, &["POST".to_string(), "PATCH".to_string()]);
    }

    #[test]
    fn should_allow_in_place_mutation_given_deref_mut() {
        let mut methods = AllowedMethods::list(["POST"]);

        methods.deref_mut()[0].make_ascii_lowercase();

        assert_eq!(methods.into_inner(), vec!["post".to_string()]);
    }
}

mod conversions {
    use super::*;

    #[test]
    fn should_consume_and_return_inner_vec_given_into_iter() {
        let methods = AllowedMethods::list(["PUT", "PATCH"]);

        let collected: Vec<String> = methods.into_iter().collect();

        assert_eq!(collected, vec!["PUT".to_string(), "PATCH".to_string()]);
    }

    #[test]
    fn should_wrap_values_without_modification_given_from_vec() {
        let original = vec!["CUSTOM".to_string(), "METHOD".to_string()];

        let methods = AllowedMethods::from(original.clone());

        assert_eq!(methods.into_inner(), original);
    }

    #[test]
    fn should_consume_and_return_values_given_into_vec() {
        let methods = AllowedMethods::list(["GET", "POST"]);

        let values: Vec<String> = methods.into();

        assert_eq!(values, vec!["GET".to_string(), "POST".to_string()]);
    }
}
