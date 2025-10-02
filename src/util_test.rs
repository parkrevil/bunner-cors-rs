use super::*;

mod normalize_lower {
    use super::*;

    #[test]
    fn should_return_ascii_lowercase_when_input_ascii_then_use_fast_path() {
        let result = normalize_lower("HeAdEr");

        assert_eq!(result, "header");
    }

    #[test]
    fn should_return_unicode_lowercase_when_input_unicode_then_preserve_characters() {
        let result = normalize_lower("TÉST");

        assert_eq!(result, "tést");
    }

    #[test]
    fn should_return_ascii_lowercase_when_input_empty_then_return_empty_string() {
        let result = normalize_lower("");

        assert_eq!(result, "");
    }
}

mod equals_ignore_case {
    use super::*;

    #[test]
    fn should_return_true_when_ascii_values_match_case_insensitively_then_detect_equality() {
        let result = equals_ignore_case("FoO", "fOo");

        assert!(result);
    }

    #[test]
    fn should_return_false_when_ascii_values_differ_then_detect_inequality() {
        let result = equals_ignore_case("Foo", "Bar");

        assert!(!result);
    }

    #[test]
    fn should_return_true_when_unicode_values_match_case_insensitively_then_detect_equality() {
        let result = equals_ignore_case("TÉST", "tést");

        assert!(result);
    }

    #[test]
    fn should_return_false_when_unicode_values_differ_then_detect_inequality() {
        let result = equals_ignore_case("Ápp", "Ápd");

        assert!(!result);
    }
}

mod is_http_token {
    use super::*;

    #[test]
    fn should_return_true_when_value_contains_valid_token_characters_then_accept_value() {
        assert!(is_http_token("X-Custom"));
        assert!(is_http_token("token123"));
    }

    #[test]
    fn should_return_false_when_value_contains_invalid_character_then_reject_value() {
        assert!(!is_http_token("Header:Value"));
        assert!(!is_http_token(" space"));
    }

    #[test]
    fn should_return_false_when_value_empty_then_reject_value() {
        assert!(!is_http_token(""));
    }
}
