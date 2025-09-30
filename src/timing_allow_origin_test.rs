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
