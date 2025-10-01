use super::TimingAllowOrigin;

#[test]
fn should_return_wildcard_given_any() {
    let timing = TimingAllowOrigin::any();
    assert_eq!(timing.header_value(), Some("*".to_string()));
}

#[test]
fn should_join_with_spaces_given_list_has_values() {
    let timing = TimingAllowOrigin::list(["https://a.test", "https://b.test"]);
    assert_eq!(
        timing.header_value(),
        Some("https://a.test https://b.test".to_string())
    );
}

#[test]
fn should_return_none_given_list_is_empty() {
    let timing = TimingAllowOrigin::list(Vec::<String>::new());
    assert_eq!(timing.header_value(), None);
}

#[test]
fn should_deduplicate_case_insensitively_given_list_has_duplicates() {
    let timing = TimingAllowOrigin::list([
        " https://metrics.test ",
        "HTTPS://METRICS.TEST",
        "https://dash.test",
    ]);

    assert_eq!(
        timing,
        TimingAllowOrigin::List(vec![
            "https://metrics.test".to_string(),
            "https://dash.test".to_string(),
        ])
    );
}

#[test]
fn should_retain_single_empty_value_given_list_contains_whitespace_entries() {
    let timing = TimingAllowOrigin::list(["   ", "https://metrics.test"]);

    assert_eq!(
        timing,
        TimingAllowOrigin::List(vec![String::new(), "https://metrics.test".to_string()])
    );
}
