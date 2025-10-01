use super::*;
use crate::context::RequestContext;

fn request_context(method: &'static str, origin: &'static str) -> RequestContext<'static> {
    RequestContext {
        method,
        origin,
        access_control_request_method: "GET",
        access_control_request_headers: "X-Test",
        access_control_request_private_network: false,
    }
}

mod origin_decision {
    use super::*;

    mod any {
        use super::*;

        #[test]
        fn should_return_any_variant_when_called() {
            // Arrange & Act
            let decision = OriginDecision::any();

            // Assert
            assert!(matches!(decision, OriginDecision::Any));
        }
    }

    mod exact {
        use super::*;

        #[test]
        fn should_wrap_string_given_value_provided() {
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
        fn should_return_mirror_variant_when_called() {
            // Arrange & Act
            let decision = OriginDecision::mirror();

            // Assert
            assert!(matches!(decision, OriginDecision::Mirror));
        }
    }

    mod disallow {
        use super::*;

        #[test]
        fn should_return_disallow_variant_when_called() {
            // Arrange & Act
            let decision = OriginDecision::disallow();

            // Assert
            assert!(matches!(decision, OriginDecision::Disallow));
        }
    }

    mod skip {
        use super::*;

        #[test]
        fn should_return_skip_variant_when_called() {
            // Arrange & Act
            let decision = OriginDecision::skip();

            // Assert
            assert!(matches!(decision, OriginDecision::Skip));
        }
    }

    mod from_bool {
        use super::*;

        #[test]
        fn should_convert_to_mirror_given_true() {
            // Arrange & Act
            let decision = OriginDecision::from(true);

            // Assert
            assert!(matches!(decision, OriginDecision::Mirror));
        }

        #[test]
        fn should_convert_to_skip_given_false() {
            // Arrange & Act
            let decision = OriginDecision::from(false);

            // Assert
            assert!(matches!(decision, OriginDecision::Skip));
        }
    }

    mod from_option {
        use super::*;

        #[test]
        fn should_convert_to_exact_given_option_has_value() {
            // Arrange & Act
            let decision = OriginDecision::from(Some("https://api.test"));

            // Assert
            match decision {
                OriginDecision::Exact(value) => assert_eq!(value, "https://api.test"),
                _ => panic!("expected exact variant"),
            }
        }

        #[test]
        fn should_convert_to_skip_given_option_is_none() {
            // Arrange & Act
            let decision: OriginDecision = OriginDecision::from(None::<String>);

            // Assert
            assert!(matches!(decision, OriginDecision::Skip));
        }
    }
}

mod origin_matcher {
    use super::*;
    use regex_automata::meta::Regex;

    mod exact {
        use super::*;

        #[test]
        fn should_store_string_value_when_called() {
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
        fn should_store_pattern_given_regex_provided() {
            // Arrange
            let regex = Regex::new(r"^https://.*\.test$").unwrap();

            // Act
            let matcher = OriginMatcher::pattern(regex);

            // Assert
            match matcher {
                OriginMatcher::Pattern(pattern) => {
                    assert!(pattern.is_match("https://api.test".as_bytes()))
                }
                _ => panic!("expected pattern matcher"),
            }
        }
    }

    mod pattern_str {
        use super::*;
        use std::time::Duration;

        #[test]
        fn should_return_matcher_given_pattern_valid() {
            // Arrange & Act
            let matcher = OriginMatcher::pattern_str(r"^https://.*\.test$").unwrap();

            // Assert
            assert!(matches!(matcher, OriginMatcher::Pattern(_)));
        }

        #[test]
        fn should_return_error_given_pattern_invalid() {
            // Arrange & Act
            let result = OriginMatcher::pattern_str("(");

            // Assert
            assert!(matches!(result, Err(PatternError::Build(_))));
        }

        #[test]
        fn should_fail_fast_given_pattern_exceeds_length() {
            // Arrange
            let pattern = "a".repeat(super::MAX_PATTERN_LENGTH + 1);

            // Act
            let result = OriginMatcher::pattern_str(&pattern);

            // Assert
            if let Err(PatternError::TooLong { length, max }) = result {
                assert_eq!(length, super::MAX_PATTERN_LENGTH + 1);
                assert_eq!(max, super::MAX_PATTERN_LENGTH);
            } else {
                panic!("expected too long error");
            }
        }

        #[test]
        fn should_return_timeout_error_given_budget_too_small() {
            let result = OriginMatcher::pattern_str_with_budget(".*", Duration::ZERO);

            assert!(matches!(result, Err(PatternError::Timeout { .. })));
        }
    }

    mod matches_fn {
        use super::*;

        #[test]
        fn should_compare_strings_given_exact() {
            // Arrange
            let matcher = OriginMatcher::exact("https://api.test");

            // Act
            let matches = matcher.matches("https://api.test");

            // Assert
            assert!(matches);
        }

        #[test]
        fn should_use_regex_given_pattern() {
            // Arrange
            let matcher = OriginMatcher::pattern(Regex::new(r"^https://.*$").unwrap());

            // Act
            let matches = matcher.matches("https://api.test");

            // Assert
            assert!(matches);
        }

        #[test]
        fn should_return_value_given_bool() {
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
        fn should_create_exact_matcher_given_string_provided() {
            // Arrange & Act
            let matcher = OriginMatcher::from("https://api.test".to_string());

            // Assert
            assert!(matches!(matcher, OriginMatcher::Exact(_)));
        }
    }

    mod from_str {
        use super::*;

        #[test]
        fn should_create_exact_matcher_given_str_provided() {
            // Arrange & Act
            let matcher = OriginMatcher::from("https://api.test");

            // Assert
            assert!(matches!(matcher, OriginMatcher::Exact(_)));
        }
    }

    mod from_bool {
        use super::*;

        #[test]
        fn should_create_bool_matcher_given_bool_provided() {
            // Arrange & Act
            let matcher = OriginMatcher::from(true);

            // Assert
            assert!(matches!(matcher, OriginMatcher::Bool(true)));
        }
    }
}

mod pattern_error_behavior {
    use super::*;
    use std::error::Error as _;
    use std::time::Duration;

    #[test]
    fn should_include_key_phrases_in_display_messages() {
        let build_error = match OriginMatcher::pattern_str("(") {
            Err(err) => err,
            Ok(_) => panic!("expected build error"),
        };
        assert!(build_error.to_string().contains("failed to compile"));

        let too_long = PatternError::TooLong {
            length: MAX_PATTERN_LENGTH + 10,
            max: MAX_PATTERN_LENGTH,
        };
        assert!(too_long.to_string().contains("exceeds"));

        let timeout = PatternError::Timeout {
            elapsed: Duration::from_millis(150),
            budget: Duration::from_millis(100),
        };
        assert!(
            timeout
                .to_string()
                .contains("exceeded the configured budget")
        );
    }

    #[test]
    fn should_expose_error_sources_where_available() {
        let build_error = match OriginMatcher::pattern_str("(") {
            Err(err) => err,
            Ok(_) => panic!("expected build error"),
        };
        assert!(build_error.source().is_some());

        let timeout = PatternError::Timeout {
            elapsed: Duration::from_millis(150),
            budget: Duration::from_millis(100),
        };
        assert!(timeout.source().is_none());
    }
}

mod origin_type {
    use super::*;

    mod any {
        use super::*;

        #[test]
        fn should_return_any_variant_when_called() {
            // Arrange & Act
            let origin = Origin::any();

            // Assert
            assert!(matches!(origin, Origin::Any));
        }
    }

    mod exact {
        use super::*;

        #[test]
        fn should_store_exact_string_given_value_provided() {
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
        fn should_collect_matchers_given_iterable_provided() {
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
        fn should_store_predicate_given_callable_provided() {
            // Arrange & Act
            let origin = Origin::predicate(|origin, _| origin.ends_with(".test"));

            // Assert
            assert!(matches!(origin, Origin::Predicate(_)));
        }
    }

    mod custom {
        use super::*;

        #[test]
        fn should_store_custom_logic_given_callback_provided() {
            // Arrange & Act
            let origin = Origin::custom(|_, _| OriginDecision::Mirror);

            // Assert
            assert!(matches!(origin, Origin::Custom(_)));
        }
    }

    mod disabled {
        use super::*;

        #[test]
        fn should_return_skip_decision_on_resolve_when_called() {
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
        fn should_allow_all_given_origin_any() {
            // Arrange
            let origin = Origin::any();
            let ctx = request_context("GET", "https://api.test");

            // Act
            let decision = origin.resolve(Some("https://api.test"), &ctx);

            // Assert
            assert!(matches!(decision, OriginDecision::Any));
        }

        #[test]
        fn should_return_exact_clone_given_origin_exact() {
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
        fn should_skip_given_origin_exact_has_no_request_origin() {
            let origin = Origin::exact("https://app.test");
            let ctx = request_context("GET", "https://app.test");

            let decision = origin.resolve(None, &ctx);

            assert!(matches!(decision, OriginDecision::Skip));
        }

        #[test]
        fn should_mirror_given_origin_list_matches_request() {
            // Arrange
            let origin = Origin::list(["https://api.test"]);
            let ctx = request_context("GET", "https://api.test");

            // Act
            let decision = origin.resolve(Some("https://api.test"), &ctx);

            // Assert
            assert!(matches!(decision, OriginDecision::Mirror));
        }

        #[test]
        fn should_disallow_given_origin_list_misses() {
            // Arrange
            let origin = Origin::list(["https://other.test"]);
            let ctx = request_context("GET", "https://api.test");

            // Act
            let decision = origin.resolve(Some("https://api.test"), &ctx);

            // Assert
            assert!(matches!(decision, OriginDecision::Disallow));
        }

        #[test]
        fn should_disallow_given_origin_list_has_different_scheme() {
            // Arrange
            let origin = Origin::list(["https://api.test"]);
            let ctx = request_context("GET", "http://api.test");

            // Act
            let decision = origin.resolve(Some("http://api.test"), &ctx);

            // Assert
            assert!(matches!(decision, OriginDecision::Disallow));
        }

        #[test]
        fn should_disallow_given_origin_list_contains_false_matcher() {
            // Arrange
            let origin = Origin::list([OriginMatcher::Bool(false)]);
            let ctx = request_context("GET", "https://api.test");

            // Act
            let decision = origin.resolve(Some("https://api.test"), &ctx);

            // Assert
            assert!(matches!(decision, OriginDecision::Disallow));
        }

        #[test]
        fn should_disallow_given_origin_list_has_different_port() {
            // Arrange
            let origin = Origin::list(["https://api.test:8443"]);
            let ctx = request_context("GET", "https://api.test");

            // Act
            let decision = origin.resolve(Some("https://api.test"), &ctx);

            // Assert
            assert!(matches!(decision, OriginDecision::Disallow));
        }

        #[test]
        fn should_skip_processing_given_list_without_origin_header() {
            // Arrange
            let origin = Origin::list(["https://api.test"]);
            let ctx = request_context("GET", "");

            // Act
            let decision = origin.resolve(None, &ctx);

            // Assert
            assert!(matches!(decision, OriginDecision::Skip));
        }

        #[test]
        fn should_allow_null_origin_given_origin_list_contains_null_string() {
            // Arrange
            let origin = Origin::list(["null"]);
            let ctx = request_context("GET", "null");

            // Act
            let decision = origin.resolve(Some("null"), &ctx);

            // Assert
            assert!(matches!(decision, OriginDecision::Mirror));
        }

        #[test]
        fn should_allow_all_given_origin_any_receives_null_string() {
            // Arrange
            let origin = Origin::any();
            let ctx = request_context("GET", "null");

            // Act
            let decision = origin.resolve(Some("null"), &ctx);

            // Assert
            assert!(matches!(decision, OriginDecision::Any));
        }

        #[test]
        fn should_mirror_given_predicate_true() {
            // Arrange
            let origin = Origin::predicate(|value, _| value.ends_with(".test"));
            let ctx = request_context("GET", "https://api.test");

            // Act
            let decision = origin.resolve(Some("https://api.test"), &ctx);

            // Assert
            assert!(matches!(decision, OriginDecision::Mirror));
        }

        #[test]
        fn should_disallow_given_predicate_false() {
            // Arrange
            let origin = Origin::predicate(|value, _| value == "https://allowed.test");
            let ctx = request_context("GET", "https://api.test");

            // Act
            let decision = origin.resolve(Some("https://api.test"), &ctx);

            // Assert
            assert!(matches!(decision, OriginDecision::Disallow));
        }

        #[test]
        fn should_not_mirror_origin_given_predicate_returns_false() {
            // Arrange
            let origin = Origin::predicate(|value, _| value == "https://allowed.test");
            let ctx = request_context("GET", "https://blocked.test");

            // Act
            let decision = origin.resolve(Some("https://blocked.test"), &ctx);

            // Assert
            assert!(matches!(decision, OriginDecision::Disallow));
        }

        #[test]
        fn should_skip_without_invoking_predicate_given_predicate_without_origin_header() {
            use std::sync::Arc;
            use std::sync::atomic::{AtomicBool, Ordering};

            // Arrange
            let invoked = Arc::new(AtomicBool::new(false));
            let origin = {
                let invoked = Arc::clone(&invoked);
                Origin::predicate(move |_, _| {
                    invoked.store(true, Ordering::Relaxed);
                    true
                })
            };
            let ctx = request_context("GET", "");

            // Act
            let decision = origin.resolve(None, &ctx);

            // Assert
            assert!(matches!(decision, OriginDecision::Skip));
            assert!(!invoked.load(Ordering::Relaxed));
        }

        #[test]
        fn should_forward_given_custom_callback_returns_decision() {
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

        #[test]
        fn should_allow_custom_logic_given_custom_receives_no_origin_header() {
            // Arrange
            let origin = Origin::custom(|origin, _| {
                assert!(origin.is_none());
                OriginDecision::Disallow
            });
            let ctx = request_context("GET", "");

            // Act
            let decision = origin.resolve(None, &ctx);

            // Assert
            assert!(matches!(decision, OriginDecision::Disallow));
        }
    }

    mod vary_on_disallow {
        use super::*;

        #[test]
        fn should_not_vary_given_origin_any() {
            // Arrange
            let origin = Origin::any();

            // Act
            let vary = origin.vary_on_disallow();

            // Assert
            assert!(!vary);
        }

        #[test]
        fn should_vary_given_origin_exact() {
            // Arrange
            let origin = Origin::exact("https://api.test");

            // Act
            let vary = origin.vary_on_disallow();

            // Assert
            assert!(vary);
        }
    }
}
