use super::*;

mod default {
    use super::*;

    #[test]
    fn given_default_when_constructed_then_represents_empty_list_variant() {
        let headers = ExposedHeaders::default();

        assert!(matches!(&headers, ExposedHeaders::List(list) if list.is_empty()));
        assert!(headers.header_value().is_none());
    }
}

mod list {
    use super::*;

    #[test]
    fn given_whitespace_values_when_list_called_then_trims_and_preserves_order() {
        let input = ["  X-Trace  ", "x-trace", "X-Span"];
        let headers = ExposedHeaders::list(input);

        let collected: Vec<_> = headers.iter().cloned().collect();
        assert!(matches!(&headers, ExposedHeaders::List(_)));
        assert_eq!(collected, vec!["X-Trace".to_string(), "X-Span".to_string()]);
    }

    #[test]
    fn given_single_wildcard_when_list_called_then_returns_any_variant() {
        let input = ["*"];
        let headers = ExposedHeaders::list(input);

        assert!(matches!(headers, ExposedHeaders::Any));
        assert_eq!(headers.header_value().as_deref(), Some("*"));
    }

    #[test]
    fn given_empty_iterator_when_list_called_then_returns_empty_list() {
        let input = std::iter::empty::<&str>();
        let headers = ExposedHeaders::list(input);

        assert!(matches!(&headers, ExposedHeaders::List(list) if list.is_empty()));
        assert!(headers.header_value().is_none());
    }

    #[test]
    fn given_empty_string_when_list_called_then_keeps_single_empty_entry() {
        let input = ["  ", ""];
        let headers = ExposedHeaders::list(input);

        if let ExposedHeaders::List(list) = headers {
            assert_eq!(list.values(), &["".to_string()]);
        } else {
            panic!("expected list variant");
        }
    }
}

mod header_value {
    use super::*;

    #[test]
    fn given_list_without_entries_when_header_value_requested_then_returns_none() {
        let headers = ExposedHeaders::list(std::iter::empty::<&str>());
        let value = headers.header_value();

        assert!(value.is_none());
    }

    #[test]
    fn given_list_with_values_when_header_value_requested_then_returns_csv() {
        let headers = ExposedHeaders::list(["X-Trace", "X-Span"]);
        let value = headers.header_value();

        assert_eq!(value.as_deref(), Some("X-Trace,X-Span"));
    }

    #[test]
    fn given_any_variant_when_header_value_requested_then_returns_wildcard() {
        let headers = ExposedHeaders::Any;
        let value = headers.header_value();

        assert_eq!(value.as_deref(), Some("*"));
    }
}

mod iter {
    use super::*;

    #[test]
    fn given_list_variant_when_iter_called_then_yields_insertion_order() {
        let headers = ExposedHeaders::list(["X-Trace", "X-Span"]);
        let collected: Vec<_> = headers.iter().cloned().collect();

        assert_eq!(collected, vec!["X-Trace".to_string(), "X-Span".to_string()]);
    }

    #[test]
    fn given_non_list_variant_when_iter_called_then_returns_empty_iterator() {
        let headers = ExposedHeaders::Any;
        let collected: Vec<_> = headers.iter().collect();

        assert!(collected.is_empty());
    }
}

mod exposed_header_list {
    use super::*;

    #[test]
    fn given_list_variant_when_values_called_then_returns_inner_slice() {
        let headers = ExposedHeaders::list(["X-Trace"]);

        if let ExposedHeaders::List(list) = headers {
            assert_eq!(list.values(), &["X-Trace".to_string()]);
        } else {
            panic!("expected list variant");
        }
    }

    #[test]
    fn given_empty_list_variant_when_is_empty_called_then_returns_true() {
        let headers = ExposedHeaders::list(std::iter::empty::<&str>());

        if let ExposedHeaders::List(list) = headers {
            assert!(list.is_empty());
        } else {
            panic!("expected list variant");
        }
    }
}
