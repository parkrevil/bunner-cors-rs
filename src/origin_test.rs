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
            let decision = OriginDecision::any();

            assert!(matches!(decision, OriginDecision::Any));
        }
    }

    mod exact {
        use super::*;

        #[test]
        fn should_wrap_string_given_value_provided() {
            let decision = OriginDecision::exact("https://api.test");

            match decision {
                OriginDecision::Exact(value) => assert_eq!(value, "https://api.test"),
                _ => panic!("expected exact variant"),
            }
        }

        mod compile_pattern {
            use super::*;
            use std::time::Duration;

            #[test]
            fn should_compile_case_insensitively() {
                let regex = OriginMatcher::compile_pattern("^https://svc$", Duration::from_secs(1))
                    .expect("pattern should compile");

                assert!(regex.is_match("https://SVC".as_bytes()));
                assert!(regex.is_match("https://svc".as_bytes()));
            }

            #[test]
            fn should_return_timeout_error_given_zero_budget() {
                let result = OriginMatcher::compile_pattern(".*", Duration::ZERO);

                assert!(matches!(result, Err(PatternError::Timeout { .. })));
            }

            #[test]
            fn should_return_too_long_error_given_pattern_exceeds_limit() {
                let pattern = "a".repeat(super::MAX_PATTERN_LENGTH + 1);

                let result = OriginMatcher::compile_pattern(&pattern, Duration::from_secs(1));

                if let Err(PatternError::TooLong { length, max }) = result {
                    assert_eq!(length, super::MAX_PATTERN_LENGTH + 1);
                    assert_eq!(max, super::MAX_PATTERN_LENGTH);
                } else {
                    panic!("expected too long error");
                }
            }
        }
    }

    mod mirror {
        use super::*;

        #[test]
        fn should_return_mirror_variant_when_called() {
            let decision = OriginDecision::mirror();

            assert!(matches!(decision, OriginDecision::Mirror));
        }
    }

    mod disallow {
        use super::*;

        #[test]
        fn should_return_disallow_variant_when_called() {
            let decision = OriginDecision::disallow();

            assert!(matches!(decision, OriginDecision::Disallow));
        }
    }

    mod skip {
        use super::*;

        #[test]
        fn should_return_skip_variant_when_called() {
            let decision = OriginDecision::skip();

            assert!(matches!(decision, OriginDecision::Skip));
        }
    }

    mod from_bool {
        use super::*;

        #[test]
        fn should_convert_to_mirror_given_true() {
            let decision = OriginDecision::from(true);

            assert!(matches!(decision, OriginDecision::Mirror));
        }

        #[test]
        fn should_convert_to_skip_given_false() {
            let decision = OriginDecision::from(false);

            assert!(matches!(decision, OriginDecision::Skip));
        }
    }

    mod from_option {
        use super::*;

        #[test]
        fn should_convert_to_exact_given_option_has_value() {
            let decision = OriginDecision::from(Some("https://api.test"));

            match decision {
                OriginDecision::Exact(value) => assert_eq!(value, "https://api.test"),
                _ => panic!("expected exact variant"),
            }
        }

        #[test]
        fn should_convert_to_skip_given_option_is_none() {
            let decision: OriginDecision = OriginDecision::from(None::<String>);

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
            let matcher = OriginMatcher::exact("https://api.test");

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
            let regex = Regex::new(r"^https://.*\.test$").unwrap();

            let matcher = OriginMatcher::pattern(regex);

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
            let matcher = OriginMatcher::pattern_str(r"^https://.*\.test$").unwrap();

            assert!(matches!(matcher, OriginMatcher::Pattern(_)));
        }

        #[test]
        fn should_return_error_given_pattern_invalid() {
            let result = OriginMatcher::pattern_str("(");

            assert!(matches!(result, Err(PatternError::Build(_))));
        }

        #[test]
        fn should_fail_fast_given_pattern_exceeds_length() {
            let pattern = "a".repeat(super::MAX_PATTERN_LENGTH + 1);

            let result = OriginMatcher::pattern_str(&pattern);

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
            let matcher = OriginMatcher::exact("https://api.test");

            let matches = matcher.matches("https://api.test");

            assert!(matches);
        }

        #[test]
        fn should_use_regex_given_pattern() {
            let matcher = OriginMatcher::pattern(Regex::new(r"^https://.*$").unwrap());

            let matches = matcher.matches("https://api.test");

            assert!(matches);
        }

        #[test]
        fn should_return_value_given_bool() {
            let matcher = OriginMatcher::Bool(false);

            let matches = matcher.matches("https://api.test");

            assert!(!matches);
        }
    }

    mod from_string {
        use super::*;

        #[test]
        fn should_create_exact_matcher_given_string_provided() {
            let matcher = OriginMatcher::from("https://api.test".to_string());

            assert!(matches!(matcher, OriginMatcher::Exact(_)));
        }
    }

    mod from_str {
        use super::*;

        #[test]
        fn should_create_exact_matcher_given_str_provided() {
            let matcher = OriginMatcher::from("https://api.test");

            assert!(matches!(matcher, OriginMatcher::Exact(_)));
        }
    }

    mod from_bool {
        use super::*;

        #[test]
        fn should_create_bool_matcher_given_bool_provided() {
            let matcher = OriginMatcher::from(true);

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
            let origin = Origin::any();

            assert!(matches!(origin, Origin::Any));
        }
    }

    mod exact {
        use super::*;

        #[test]
        fn should_store_exact_string_given_value_provided() {
            let origin = Origin::exact("https://api.test");

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
            let origin = Origin::list(["https://api.test", "https://other.test"]);

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
            let origin = Origin::predicate(|origin, _| origin.ends_with(".test"));

            assert!(matches!(origin, Origin::Predicate(_)));
        }
    }

    mod custom {
        use super::*;

        #[test]
        fn should_store_custom_logic_given_callback_provided() {
            let origin = Origin::custom(|_, _| OriginDecision::Mirror);

            assert!(matches!(origin, Origin::Custom(_)));
        }
    }

    mod disabled {
        use super::*;

        #[test]
        fn should_return_skip_decision_on_resolve_when_called() {
            let origin = Origin::disabled();
            let ctx = request_context("GET", "https://api.test");

            let decision = origin.resolve(Some("https://api.test"), &ctx);

            assert!(matches!(decision, OriginDecision::Skip));
        }
    }

    mod resolve {
        use super::*;

        #[test]
        fn should_allow_all_given_origin_any() {
            let origin = Origin::any();
            let ctx = request_context("GET", "https://api.test");

            let decision = origin.resolve(Some("https://api.test"), &ctx);

            assert!(matches!(decision, OriginDecision::Any));
        }

        #[test]
        fn should_return_exact_clone_given_origin_exact() {
            let origin = Origin::exact("https://api.test");
            let ctx = request_context("GET", "https://api.test");

            let decision = origin.resolve(Some("https://api.test"), &ctx);

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
            let origin = Origin::list(["https://api.test"]);
            let ctx = request_context("GET", "https://api.test");

            let decision = origin.resolve(Some("https://api.test"), &ctx);

            assert!(matches!(decision, OriginDecision::Mirror));
        }

        #[test]
        fn should_disallow_given_origin_list_misses() {
            let origin = Origin::list(["https://other.test"]);
            let ctx = request_context("GET", "https://api.test");

            let decision = origin.resolve(Some("https://api.test"), &ctx);

            assert!(matches!(decision, OriginDecision::Disallow));
        }

        #[test]
        fn should_disallow_given_origin_list_has_different_scheme() {
            let origin = Origin::list(["https://api.test"]);
            let ctx = request_context("GET", "http://api.test");

            let decision = origin.resolve(Some("http://api.test"), &ctx);

            assert!(matches!(decision, OriginDecision::Disallow));
        }

        #[test]
        fn should_disallow_given_origin_list_contains_false_matcher() {
            let origin = Origin::list([OriginMatcher::Bool(false)]);
            let ctx = request_context("GET", "https://api.test");

            let decision = origin.resolve(Some("https://api.test"), &ctx);

            assert!(matches!(decision, OriginDecision::Disallow));
        }

        #[test]
        fn should_disallow_given_origin_list_has_different_port() {
            let origin = Origin::list(["https://api.test:8443"]);
            let ctx = request_context("GET", "https://api.test");

            let decision = origin.resolve(Some("https://api.test"), &ctx);

            assert!(matches!(decision, OriginDecision::Disallow));
        }

        #[test]
        fn should_skip_processing_given_list_without_origin_header() {
            let origin = Origin::list(["https://api.test"]);
            let ctx = request_context("GET", "");

            let decision = origin.resolve(None, &ctx);

            assert!(matches!(decision, OriginDecision::Skip));
        }

        #[test]
        fn should_allow_null_origin_given_origin_list_contains_null_string() {
            let origin = Origin::list(["null"]);
            let ctx = request_context("GET", "null");

            let decision = origin.resolve(Some("null"), &ctx);

            assert!(matches!(decision, OriginDecision::Mirror));
        }

        #[test]
        fn should_allow_all_given_origin_any_receives_null_string() {
            let origin = Origin::any();
            let ctx = request_context("GET", "null");

            let decision = origin.resolve(Some("null"), &ctx);

            assert!(matches!(decision, OriginDecision::Any));
        }

        #[test]
        fn should_mirror_given_predicate_true() {
            let origin = Origin::predicate(|value, _| value.ends_with(".test"));
            let ctx = request_context("GET", "https://api.test");

            let decision = origin.resolve(Some("https://api.test"), &ctx);

            assert!(matches!(decision, OriginDecision::Mirror));
        }

        #[test]
        fn should_disallow_given_predicate_false() {
            let origin = Origin::predicate(|value, _| value == "https://allowed.test");
            let ctx = request_context("GET", "https://api.test");

            let decision = origin.resolve(Some("https://api.test"), &ctx);

            assert!(matches!(decision, OriginDecision::Disallow));
        }

        #[test]
        fn should_not_mirror_origin_given_predicate_returns_false() {
            let origin = Origin::predicate(|value, _| value == "https://allowed.test");
            let ctx = request_context("GET", "https://blocked.test");

            let decision = origin.resolve(Some("https://blocked.test"), &ctx);

            assert!(matches!(decision, OriginDecision::Disallow));
        }

        #[test]
        fn should_skip_without_invoking_predicate_given_predicate_without_origin_header() {
            use std::sync::Arc;
            use std::sync::atomic::{AtomicBool, Ordering};

            let invoked = Arc::new(AtomicBool::new(false));
            let origin = {
                let invoked = Arc::clone(&invoked);
                Origin::predicate(move |_, _| {
                    invoked.store(true, Ordering::Relaxed);
                    true
                })
            };
            let ctx = request_context("GET", "");

            let decision = origin.resolve(None, &ctx);

            assert!(matches!(decision, OriginDecision::Skip));
            assert!(!invoked.load(Ordering::Relaxed));
        }

        #[test]
        fn should_forward_given_custom_callback_returns_decision() {
            let origin = Origin::custom(|_, _| OriginDecision::Exact("https://custom.test".into()));
            let ctx = request_context("GET", "https://api.test");

            let decision = origin.resolve(Some("https://api.test"), &ctx);

            match decision {
                OriginDecision::Exact(value) => assert_eq!(value, "https://custom.test"),
                _ => panic!("expected custom decision"),
            }
        }

        #[test]
        fn should_allow_custom_logic_given_custom_receives_no_origin_header() {
            let origin = Origin::custom(|origin, _| {
                assert!(origin.is_none());
                OriginDecision::Disallow
            });
            let ctx = request_context("GET", "");

            let decision = origin.resolve(None, &ctx);

            assert!(matches!(decision, OriginDecision::Disallow));
        }
    }

    mod vary_on_disallow {
        use super::*;

        #[test]
        fn should_not_vary_given_origin_any() {
            let origin = Origin::any();

            let vary = origin.vary_on_disallow();

            assert!(!vary);
        }

        #[test]
        fn should_vary_given_origin_exact() {
            let origin = Origin::exact("https://api.test");

            let vary = origin.vary_on_disallow();

            assert!(vary);
        }
    }
}
