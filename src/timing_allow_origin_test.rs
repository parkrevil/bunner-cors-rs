use super::TimingAllowOrigin;

mod list {
    use super::*;

    #[test]
    fn should_join_with_spaces_when_list_has_values_then_emit_space_delimited_header() {
        let timing = TimingAllowOrigin::list(["https://a.test", "https://b.test"]);

        let header = timing.header_value();

        assert_eq!(header, Some("https://a.test https://b.test".to_string()));
    }

    #[test]
    fn should_return_none_when_list_is_empty_then_skip_header_value() {
        let timing = TimingAllowOrigin::list(Vec::<String>::new());

        let header = timing.header_value();

        assert_eq!(header, None);
    }

    #[test]
    fn should_deduplicate_case_insensitively_when_list_has_duplicates_then_keep_first_entries() {
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
    fn should_retain_single_empty_value_when_list_contains_whitespace_entries_then_preserve_positions()
     {
        let timing = TimingAllowOrigin::list(["   ", "https://metrics.test"]);

        assert_eq!(
            timing,
            TimingAllowOrigin::List(vec![String::new(), "https://metrics.test".to_string()])
        );
    }
}

#[test]
fn should_return_wildcard_when_any_variant_then_emit_star_header() {
    let timing = TimingAllowOrigin::Any;

    let header = timing.header_value();

    assert_eq!(header, Some("*".to_string()));
}
