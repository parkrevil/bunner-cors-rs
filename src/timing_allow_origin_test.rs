use super::TimingAllowOrigin;

#[test]
fn when_any_should_return_wildcard() {
    let timing = TimingAllowOrigin::any();
    assert_eq!(timing.header_value(), Some("*".to_string()));
}

#[test]
fn when_list_has_values_should_join_with_spaces() {
    let timing = TimingAllowOrigin::list(["https://a.test", "https://b.test"]);
    assert_eq!(
        timing.header_value(),
        Some("https://a.test https://b.test".to_string())
    );
}

#[test]
fn when_list_is_empty_should_return_none() {
    let timing = TimingAllowOrigin::list(Vec::<String>::new());
    assert_eq!(timing.header_value(), None);
}

#[test]
fn when_list_has_duplicates_should_deduplicate_case_insensitively() {
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
fn when_list_contains_whitespace_entries_should_retain_single_empty_value() {
    let timing = TimingAllowOrigin::list(["   ", "https://metrics.test"]);

    assert_eq!(
        timing,
        TimingAllowOrigin::List(vec![String::new(), "https://metrics.test".to_string()])
    );
}
