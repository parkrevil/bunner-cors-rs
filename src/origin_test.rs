use super::*;
use crate::context::RequestContext;
use regex::Regex;

fn request_context(method: &'static str, origin: &'static str) -> RequestContext<'static> {
    RequestContext {
        method,
        origin,
        access_control_request_method: "GET",
        access_control_request_headers: "X-Test",
    }
}

mod origin_decision {
    use super::*;

    mod any {
        use super::*;

        #[test]
        fn when_called_should_return_any_variant() {
            // Arrange & Act
            let decision = OriginDecision::any();

            // Assert
            assert!(matches!(decision, OriginDecision::Any));
        }
    }

    mod exact {
        use super::*;

        #[test]
        fn when_value_provided_should_wrap_string() {
            // Arrange & Act
            let decision = OriginDecision::exact("https://api.test");

            // Assert
            match decision {
                OriginDecision::Exact(value) => assert_eq!(value, "https://api.test"),
                _ => panic!("expected exact variant"),
            }
        }
    }

    mod mirror {
        use super::*;

        #[test]
        fn when_called_should_return_mirror_variant() {
            // Arrange & Act
            let decision = OriginDecision::mirror();

            // Assert
            assert!(matches!(decision, OriginDecision::Mirror));
        }
    }

    mod disallow {
        use super::*;

        #[test]
        fn when_called_should_return_disallow_variant() {
            // Arrange & Act
            let decision = OriginDecision::disallow();

            // Assert
            assert!(matches!(decision, OriginDecision::Disallow));
        }
    }

    mod skip {
        use super::*;

        #[test]
        fn when_called_should_return_skip_variant() {
            // Arrange & Act
            let decision = OriginDecision::skip();

            // Assert
            assert!(matches!(decision, OriginDecision::Skip));
        }
    }

    mod from_bool {
        use super::*;

        #[test]
        fn when_true_should_convert_to_mirror() {
            // Arrange & Act
            let decision = OriginDecision::from(true);

            // Assert
            assert!(matches!(decision, OriginDecision::Mirror));
        }

        #[test]
        fn when_false_should_convert_to_skip() {
            // Arrange & Act
            let decision = OriginDecision::from(false);

            // Assert
            assert!(matches!(decision, OriginDecision::Skip));
        }
    }

    mod from_option {
        use super::*;

        #[test]
        fn when_option_has_value_should_convert_to_exact() {
            // Arrange & Act
            let decision = OriginDecision::from(Some("https://api.test"));

            // Assert
            match decision {
                OriginDecision::Exact(value) => assert_eq!(value, "https://api.test"),
                _ => panic!("expected exact variant"),
            }
        }

        #[test]
        fn when_option_is_none_should_convert_to_skip() {
            // Arrange & Act
            let decision: OriginDecision = OriginDecision::from(None::<String>);

            // Assert
            assert!(matches!(decision, OriginDecision::Skip));
        }
    }
}

mod origin_matcher {
    use super::*;

    mod exact {
        use super::*;

        #[test]
        fn when_called_should_store_string_value() {
            // Arrange & Act
            let matcher = OriginMatcher::exact("https://api.test");

            // Assert
            match matcher {
                OriginMatcher::Exact(value) => assert_eq!(value, "https://api.test"),
                _ => panic!("expected exact matcher"),
            }
        }
    }

    mod pattern {
        use super::*;

        #[test]
        fn when_regex_provided_should_store_pattern() {
            // Arrange
            let regex = Regex::new(r"^https://.*\.test$").unwrap();

            // Act
            let matcher = OriginMatcher::pattern(regex);

            // Assert
            match matcher {
                OriginMatcher::Pattern(pattern) => assert!(pattern.is_match("https://api.test")),
                _ => panic!("expected pattern matcher"),
            }
        }
    }

    mod pattern_str {
        use super::*;

        #[test]
        fn when_pattern_valid_should_return_matcher() {
            // Arrange & Act
            let matcher = OriginMatcher::pattern_str(r"^https://.*\.test$").unwrap();

            // Assert
            assert!(matches!(matcher, OriginMatcher::Pattern(_)));
        }

        #[test]
        fn when_pattern_invalid_should_return_error() {
            // Arrange & Act
            let result = OriginMatcher::pattern_str("(");

            // Assert
            assert!(result.is_err());
        }
    }

    mod matches_fn {
        use super::*;

        #[test]
        fn when_exact_should_compare_strings() {
            // Arrange
            let matcher = OriginMatcher::exact("https://api.test");

            // Act
            let matches = matcher.matches("https://api.test");

            // Assert
            assert!(matches);
        }

        #[test]
        fn when_pattern_should_use_regex() {
            // Arrange
            let matcher = OriginMatcher::pattern(Regex::new(r"^https://.*$").unwrap());

            // Act
            let matches = matcher.matches("https://api.test");

            // Assert
            assert!(matches);
        }

        #[test]
        fn when_bool_should_return_value() {
            // Arrange
            let matcher = OriginMatcher::Bool(false);

            // Act
            let matches = matcher.matches("https://api.test");

            // Assert
            assert!(!matches);
        }
    }

    mod from_string {
        use super::*;

        #[test]
        fn when_string_provided_should_create_exact_matcher() {
            // Arrange & Act
            let matcher = OriginMatcher::from("https://api.test".to_string());

            // Assert
            assert!(matches!(matcher, OriginMatcher::Exact(_)));
        }
    }

    mod from_str {
        use super::*;

        #[test]
        fn when_str_provided_should_create_exact_matcher() {
            // Arrange & Act
            let matcher = OriginMatcher::from("https://api.test");

            // Assert
            assert!(matches!(matcher, OriginMatcher::Exact(_)));
        }
    }

    mod from_bool {
        use super::*;

        #[test]
        fn when_bool_provided_should_create_bool_matcher() {
            // Arrange & Act
            let matcher = OriginMatcher::from(true);

            // Assert
            assert!(matches!(matcher, OriginMatcher::Bool(true)));
        }
    }
}

mod origin_type {
    use super::*;

    mod any {
        use super::*;

        #[test]
        fn when_called_should_return_any_variant() {
            // Arrange & Act
            let origin = Origin::any();

            // Assert
            assert!(matches!(origin, Origin::Any));
        }
    }

    mod exact {
        use super::*;

        #[test]
        fn when_value_provided_should_store_exact_string() {
            // Arrange & Act
            let origin = Origin::exact("https://api.test");

            // Assert
            match origin {
                Origin::Exact(value) => assert_eq!(value, "https://api.test"),
                _ => panic!("expected exact origin"),
            }
        }
    }

    mod list {
        use super::*;

        #[test]
        fn when_iterable_provided_should_collect_matchers() {
            // Arrange & Act
            let origin = Origin::list(["https://api.test", "https://other.test"]);

            // Assert
            match origin {
                Origin::List(values) => {
                    assert_eq!(values.len(), 2);
                }
                _ => panic!("expected list origin"),
            }
        }
    }

    mod predicate {
        use super::*;

        #[test]
        fn when_callable_provided_should_store_predicate() {
            // Arrange & Act
            let origin = Origin::predicate(|origin, _| origin.ends_with(".test"));

            // Assert
            assert!(matches!(origin, Origin::Predicate(_)));
        }
    }

    mod custom {
        use super::*;

        #[test]
        fn when_callback_provided_should_store_custom_logic() {
            // Arrange & Act
            let origin = Origin::custom(|_, _| OriginDecision::Mirror);

            // Assert
            assert!(matches!(origin, Origin::Custom(_)));
        }
    }

    mod disabled {
        use super::*;

        #[test]
        fn when_called_should_return_skip_decision_on_resolve() {
            // Arrange
            let origin = Origin::disabled();
            let ctx = request_context("GET", "https://api.test");

            // Act
            let decision = origin.resolve(Some("https://api.test"), &ctx);

            // Assert
            assert!(matches!(decision, OriginDecision::Skip));
        }
    }

    mod resolve {
        use super::*;

        #[test]
        fn when_origin_any_should_allow_all() {
            // Arrange
            let origin = Origin::any();
            let ctx = request_context("GET", "https://api.test");

            // Act
            let decision = origin.resolve(Some("https://api.test"), &ctx);

            // Assert
            assert!(matches!(decision, OriginDecision::Any));
        }

        #[test]
        fn when_origin_exact_should_return_exact_clone() {
            // Arrange
            let origin = Origin::exact("https://api.test");
            let ctx = request_context("GET", "https://api.test");

            // Act
            let decision = origin.resolve(Some("https://api.test"), &ctx);

            // Assert
            match decision {
                OriginDecision::Exact(value) => assert_eq!(value, "https://api.test"),
                _ => panic!("expected exact decision"),
            }
        }

        #[test]
        fn when_origin_list_matches_request_should_mirror() {
            // Arrange
            let origin = Origin::list(["https://api.test"]);
            let ctx = request_context("GET", "https://api.test");

            // Act
            let decision = origin.resolve(Some("https://api.test"), &ctx);

            // Assert
            assert!(matches!(decision, OriginDecision::Mirror));
        }

        #[test]
        fn when_origin_list_misses_should_disallow() {
            // Arrange
            let origin = Origin::list(["https://other.test"]);
            let ctx = request_context("GET", "https://api.test");

            // Act
            let decision = origin.resolve(Some("https://api.test"), &ctx);

            // Assert
            assert!(matches!(decision, OriginDecision::Disallow));
        }

        #[test]
        fn when_list_without_origin_header_should_disallow() {
            // Arrange
            let origin = Origin::list(["https://api.test"]);
            let ctx = request_context("GET", "");

            // Act
            let decision = origin.resolve(None, &ctx);

            // Assert
            assert!(matches!(decision, OriginDecision::Disallow));
        }

        #[test]
        fn when_predicate_true_should_mirror() {
            // Arrange
            let origin = Origin::predicate(|value, _| value.ends_with(".test"));
            let ctx = request_context("GET", "https://api.test");

            // Act
            let decision = origin.resolve(Some("https://api.test"), &ctx);

            // Assert
            assert!(matches!(decision, OriginDecision::Mirror));
        }

        #[test]
        fn when_predicate_false_should_disallow() {
            // Arrange
            let origin = Origin::predicate(|value, _| value == "https://allowed.test");
            let ctx = request_context("GET", "https://api.test");

            // Act
            let decision = origin.resolve(Some("https://api.test"), &ctx);

            // Assert
            assert!(matches!(decision, OriginDecision::Disallow));
        }

        #[test]
        fn when_custom_callback_returns_decision_should_forward() {
            // Arrange
            let origin = Origin::custom(|_, _| OriginDecision::Exact("https://custom.test".into()));
            let ctx = request_context("GET", "https://api.test");

            // Act
            let decision = origin.resolve(Some("https://api.test"), &ctx);

            // Assert
            match decision {
                OriginDecision::Exact(value) => assert_eq!(value, "https://custom.test"),
                _ => panic!("expected custom decision"),
            }
        }
    }

    mod vary_on_disallow {
        use super::*;

        #[test]
        fn when_origin_any_should_not_vary() {
            // Arrange
            let origin = Origin::any();

            // Act
            let vary = origin.vary_on_disallow();

            // Assert
            assert!(!vary);
        }

        #[test]
        fn when_origin_exact_should_vary() {
            // Arrange
            let origin = Origin::exact("https://api.test");

            // Act
            let vary = origin.vary_on_disallow();

            // Assert
            assert!(vary);
        }
    }
}
