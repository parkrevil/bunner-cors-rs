use super::*;

#[test]
fn should_return_none_when_default_then_no_headers_exposed() {
    let headers = ExposedHeaders::default();

    assert!(matches!(headers, ExposedHeaders::None));
    assert!(headers.header_value().is_none());
}

#[test]
fn should_trim_and_deduplicate_entries_when_list_created_then_return_unique_values() {
    let headers = ExposedHeaders::list([" X-Trace ", "x-trace", "X-Span"]);

    let collected: Vec<_> = headers.iter().cloned().collect();

    assert!(matches!(&headers, ExposedHeaders::List(_)));
    assert_eq!(collected, vec!["X-Trace".to_string(), "X-Span".to_string()]);
}

#[test]
fn should_convert_single_wildcard_to_variant_when_star_provided_then_use_wildcard() {
    let headers = ExposedHeaders::list(["*"]);

    assert!(matches!(headers, ExposedHeaders::Any));
    assert_eq!(headers.header_value().as_deref(), Some("*"));
}

#[test]
fn should_return_none_header_value_when_list_empty_then_skip_header() {
    let headers = ExposedHeaders::list(std::iter::empty::<&str>());

    assert!(matches!(&headers, ExposedHeaders::List(list) if list.is_empty()));
    assert!(headers.header_value().is_none());
}

#[test]
fn should_preserve_trimmed_values_when_header_value_requested_then_join_with_commas() {
    let headers = ExposedHeaders::list(["  X-Trace  ", "X-Span", "X-Trace"]);

    assert_eq!(headers.header_value().as_deref(), Some("X-Trace,X-Span"));
}
