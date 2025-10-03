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
        fn should_return_any_variant_when_called_then_provide_wildcard_decision() {
            let decision = OriginDecision::any();

            assert!(matches!(decision, OriginDecision::Any));
        }
    }

    mod exact {
        use super::*;

        #[test]
        fn should_wrap_string_when_value_provided_then_return_exact_variant() {
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
            fn should_compile_case_insensitively_when_pattern_valid_then_match_inputs() {
                let regex = OriginMatcher::compile_pattern("^https://svc$", Duration::from_secs(1))
                    .expect("pattern should compile");

                assert!(regex.is_match("https://SVC".as_bytes()));
                assert!(regex.is_match("https://svc".as_bytes()));
            }

            #[test]
            fn should_return_timeout_error_when_budget_zero_then_abort_compilation() {
                let result = OriginMatcher::compile_pattern(".*", Duration::ZERO);

                assert!(matches!(result, Err(PatternError::Timeout { .. })));
            }

            #[test]
            fn should_return_too_long_error_when_pattern_exceeds_limit_then_reject_compilation() {
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
        fn should_return_mirror_variant_when_called_then_create_reflection_decision() {
            let decision = OriginDecision::mirror();

            assert!(matches!(decision, OriginDecision::Mirror));
        }
    }

    mod disallow {
        use super::*;

        #[test]
        fn should_return_disallow_variant_when_called_then_block_origin() {
            let decision = OriginDecision::disallow();

            assert!(matches!(decision, OriginDecision::Disallow));
        }
    }

    mod skip {
        use super::*;

        #[test]
        fn should_return_skip_variant_when_called_then_skip_processing() {
            let decision = OriginDecision::skip();

            assert!(matches!(decision, OriginDecision::Skip));
        }
    }

    mod from_bool {
        use super::*;

        #[test]
        fn should_convert_to_mirror_when_input_true_then_reflect_origin() {
            let decision = OriginDecision::from(true);

            assert!(matches!(decision, OriginDecision::Mirror));
        }

        #[test]
        fn should_convert_to_skip_when_input_false_then_skip_origin() {
            let decision = OriginDecision::from(false);

            assert!(matches!(decision, OriginDecision::Skip));
        }
    }

    mod from_option {
        use super::*;

        #[test]
        fn should_convert_to_exact_when_option_has_value_then_capture_origin() {
            let decision = OriginDecision::from(Some("https://api.test"));

            match decision {
                OriginDecision::Exact(value) => assert_eq!(value, "https://api.test"),
                _ => panic!("expected exact variant"),
            }
        }

        #[test]
        fn should_convert_to_skip_when_option_none_then_skip_processing() {
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
        fn should_store_string_value_when_exact_used_then_capture_origin() {
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
        fn should_store_pattern_when_regex_provided_then_enable_matching() {
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
        fn should_return_pattern_matcher_when_pattern_valid_then_compile_successfully() {
            let matcher = OriginMatcher::pattern_str(r"^https://.*\.test$").unwrap();

            assert!(matches!(matcher, OriginMatcher::Pattern(_)));
        }

        #[test]
        fn should_return_error_when_pattern_invalid_then_fail_compilation() {
            let result = OriginMatcher::pattern_str("(");

            assert!(matches!(result, Err(PatternError::Build(_))));
        }

        #[test]
        fn should_fail_fast_when_pattern_exceeds_length_then_reject_compilation() {
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
        fn should_cache_pattern_then_bypass_budget_on_subsequent_calls() {
            super::clear_regex_cache();
            let pattern = r"^https://cached\.allowed$";

            let first = OriginMatcher::pattern_str(pattern).expect("initial compile");
            assert!(matches!(first, OriginMatcher::Pattern(_)));
            assert!(super::regex_cache_contains(pattern));
            let entries_after_first = super::regex_cache_size();
            assert!(entries_after_first >= 1);

            let second =
                OriginMatcher::pattern_str_with_budget(pattern, Duration::ZERO).expect("cached");
            assert!(matches!(second, OriginMatcher::Pattern(_)));
            assert!(super::regex_cache_contains(pattern));
            assert_eq!(super::regex_cache_size(), entries_after_first);
        }

        #[test]
        fn should_return_timeout_error_when_budget_too_small_then_abort_compilation() {
            super::clear_regex_cache();
            let result = OriginMatcher::pattern_str_with_budget(".*", Duration::ZERO);

            assert!(matches!(result, Err(PatternError::Timeout { .. })));
        }

        #[test]
        fn should_compile_with_budget_then_cache_pattern() {
            super::clear_regex_cache();
            let pattern = r"^https://budget\.test$";

            let matcher =
                OriginMatcher::pattern_str_with_budget(pattern, Duration::from_millis(25))
                    .expect("pattern should compile within budget");

            assert!(matches!(matcher, OriginMatcher::Pattern(_)));
            assert!(super::regex_cache_contains(pattern));

            super::clear_regex_cache();
        }

        #[test]
        fn should_recover_from_poisoned_cache_then_continue_operations() {
            use std::panic::{catch_unwind, AssertUnwindSafe};

            super::clear_regex_cache();

            let poison_lock = || {
                let _ = catch_unwind(AssertUnwindSafe(|| {
                    let _guard = super::super::REGEX_CACHE.write().unwrap();
                    panic!("poison cache");
                }));
            };

            let pattern = r"^https://poisoned\.test$";

            poison_lock();
            assert!(super::super::OriginMatcher::cached_pattern(pattern).is_none());

            poison_lock();
            let regex = Regex::new(pattern).unwrap();
            super::super::OriginMatcher::cache_pattern(pattern, &regex);

            assert!(super::super::OriginMatcher::cached_pattern(pattern).is_some());

            poison_lock();
            assert!(super::regex_cache_contains(pattern));
            assert_eq!(super::regex_cache_size(), 1);

            poison_lock();
            super::clear_regex_cache();
            assert_eq!(super::regex_cache_size(), 0);
        }
    }

    mod matches_fn {
        use super::*;

        #[test]
        fn should_compare_strings_when_exact_matcher_used_then_match_literal() {
            let matcher = OriginMatcher::exact("https://api.test");

            let matches = matcher.matches("https://api.test");

            assert!(matches);
        }

        #[test]
        fn should_use_regex_when_pattern_matcher_used_then_validate_origin() {
            let matcher = OriginMatcher::pattern(Regex::new(r"^https://.*$").unwrap());

            let matches = matcher.matches("https://api.test");

            assert!(matches);
        }

        #[test]
        fn should_return_bool_value_when_bool_matcher_used_then_reflect_flag() {
            let matcher = OriginMatcher::Bool(false);

            let matches = matcher.matches("https://api.test");

            assert!(!matches);
        }
    }

    mod from_string {
        use super::*;

        #[test]
        fn should_create_exact_matcher_when_string_provided_then_capture_owned_value() {
            let matcher = OriginMatcher::from("https://api.test".to_string());

            assert!(matches!(matcher, OriginMatcher::Exact(_)));
        }
    }

    mod from_str {
        use super::*;

        #[test]
        fn should_create_exact_matcher_when_str_provided_then_capture_borrowed_value() {
            let matcher = OriginMatcher::from("https://api.test");

            assert!(matches!(matcher, OriginMatcher::Exact(_)));
        }
    }

    mod from_bool {
        use super::*;

        #[test]
        fn should_create_bool_matcher_when_bool_provided_then_store_flag() {
            let matcher = OriginMatcher::from(true);

            assert!(matches!(matcher, OriginMatcher::Bool(true)));
        }
    }
}

mod origin_list_behavior {
    use super::*;
    use regex_automata::meta::Regex;

    fn list_from<I, T>(values: I) -> OriginList
    where
        I: IntoIterator<Item = T>,
        T: Into<OriginMatcher>,
    {
        match Origin::list(values) {
            Origin::List(list) => list,
            _ => unreachable!(),
        }
    }

    #[test]
    fn should_report_empty_when_no_matchers_then_return_true() {
        let list = list_from(Vec::<OriginMatcher>::new());

        assert!(list.is_empty());
        assert_eq!(list.len(), 0);
    }

    #[test]
    fn should_iterate_insertion_order_when_iter_called_then_return_matchers() {
        let list = list_from([
            OriginMatcher::exact("https://one.test"),
            OriginMatcher::exact("https://two.test"),
        ]);

        let collected: Vec<_> = list
            .iter()
            .map(|matcher| match matcher {
                OriginMatcher::Exact(value) => value.as_str(),
                _ => "unexpected",
            })
            .collect();

        assert_eq!(collected, vec!["https://one.test", "https://two.test"]);
    }

    #[test]
    fn should_use_linear_scan_when_list_small_then_match_via_original_matchers() {
        let list = list_from([
            OriginMatcher::pattern(Regex::new(r"^https://allowed\.service$").unwrap()),
            OriginMatcher::exact("https://fallback.test"),
        ]);

        assert!(list.matches("https://allowed.service"));
        assert!(list.matches("https://FALLBACK.TEST"));
        assert!(!list.matches("https://denied.service"));
    }

    #[test]
    fn should_use_ascii_hash_lookup_when_many_matchers_then_match_case_insensitively() {
        let list = list_from([
            OriginMatcher::exact("https://alpha.test"),
            OriginMatcher::exact("https://beta.test"),
            OriginMatcher::exact("https://gamma.test"),
            OriginMatcher::exact("https://delta.test"),
            OriginMatcher::exact("https://allowed.test"),
        ]);

        assert!(list.matches("https://ALLOWED.TEST"));
        assert!(!list.matches("https://blocked.test"));
    }

    #[test]
    fn should_match_unicode_exact_when_candidate_requires_case_folding_then_normalize() {
        let list = list_from([
            OriginMatcher::exact("Straße"),
            OriginMatcher::exact("München"),
            OriginMatcher::exact("東京"),
            OriginMatcher::exact("Δelta"),
            OriginMatcher::exact("пример"),
        ]);

        assert!(list.matches("Straße"));
        assert!(list.matches("straße"));
    }

    #[test]
    fn should_match_unicode_exact_when_linear_scan_disabled_then_use_compiled_set() {
        let matchers = vec![
            OriginMatcher::exact("Straße".to_string()),
            OriginMatcher::exact("Ålesund".to_string()),
            OriginMatcher::exact("東京".to_string()),
            OriginMatcher::exact("Δelta".to_string()),
            OriginMatcher::exact("пример".to_string()),
        ];
        let compiled = super::CompiledOriginList::compile(&matchers);

        assert!(!compiled.prefer_linear_scan);
        assert!(compiled.matches("Straße", &matchers));
        assert!(compiled.matches("straße", &matchers));
    }

    #[test]
    fn should_match_using_regex_when_no_exact_match_then_use_compiled_pattern() {
        let list = list_from([
            OriginMatcher::exact("https://alpha.test"),
            OriginMatcher::exact("https://beta.test"),
            OriginMatcher::exact("https://gamma.test"),
            OriginMatcher::exact("https://delta.test"),
            OriginMatcher::pattern(Regex::new(r"^https://allowed\..+$").unwrap()),
        ]);

        assert!(list.matches("https://allowed.service"));
        assert!(!list.matches("https://denied.service"));
    }
}

mod ascii_case_helpers {
    #[test]
    fn should_compare_ascii_exact_structs_case_insensitively() {
        let left = super::AsciiExact::new("HTTPS://API.TEST".to_string());
        let right = super::AsciiExact::new("https://api.test".to_string());

        assert!(super::AsciiExact::eq(&left, &right));
        assert!(super::AsciiExact::eq(&right, &left));
    }

    #[test]
    fn should_compare_ascii_exact_with_case_insensitive_wrapper_then_ignore_case() {
        let exact = super::AsciiExact::new("HTTPS://API.TEST".to_string());
        let wrapper = super::AsciiCaseInsensitive::new("https://api.test");

        assert!(<super::AsciiExact as PartialEq<super::AsciiCaseInsensitive>>::eq(
            &exact,
            wrapper,
        ));
    }

    #[test]
    fn should_compare_case_insensitive_wrapper_with_ascii_exact_then_ignore_case() {
        let exact = super::AsciiExact::new("https://api.test".to_string());
        let wrapper = super::AsciiCaseInsensitive::new("HTTPS://API.TEST");

        assert!(<super::AsciiCaseInsensitive as PartialEq<super::AsciiExact>>::eq(
            wrapper,
            &exact,
        ));
    }
}

mod pattern_error_behavior {
    use super::*;
    use std::error::Error as _;
    use std::time::Duration;

    #[test]
    fn should_include_key_phrases_when_errors_display_then_improve_diagnostics() {
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
    fn should_expose_error_sources_when_available_then_surface_root_cause() {
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
        fn should_return_any_variant_when_called_then_configure_wildcard_origin() {
            let origin = Origin::any();

            assert!(matches!(origin, Origin::Any));
        }
    }

    mod exact {
        use super::*;

        #[test]
        fn should_store_exact_string_when_value_provided_then_capture_origin() {
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
        fn should_collect_matchers_when_iterable_provided_then_build_origin_list() {
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
        fn should_store_predicate_when_callable_provided_then_capture_logic() {
            let origin = Origin::predicate(|origin, _| origin.ends_with(".test"));

            assert!(matches!(origin, Origin::Predicate(_)));
        }
    }

    mod custom {
        use super::*;

        #[test]
        fn should_store_custom_logic_when_callback_provided_then_capture_behavior() {
            let origin = Origin::custom(|_, _| OriginDecision::Mirror);

            assert!(matches!(origin, Origin::Custom(_)));
        }
    }

    mod disabled {
        use super::*;

        #[test]
        fn should_return_skip_decision_when_origin_disabled_then_skip_processing() {
            let origin = Origin::disabled();
            let ctx = request_context("GET", "https://api.test");

            let decision = origin.resolve(Some("https://api.test"), &ctx);

            assert!(matches!(decision, OriginDecision::Skip));
        }
    }

    mod resolve {
        use super::*;

        #[test]
        fn should_return_any_decision_when_origin_any_then_allow_all_origins() {
            let origin = Origin::any();
            let ctx = request_context("GET", "https://api.test");

            let decision = origin.resolve(Some("https://api.test"), &ctx);

            assert!(matches!(decision, OriginDecision::Any));
        }

        #[test]
        fn should_return_exact_decision_when_origin_exact_then_clone_value() {
            let origin = Origin::exact("https://api.test");
            let ctx = request_context("GET", "https://api.test");

            let decision = origin.resolve(Some("https://api.test"), &ctx);

            match decision {
                OriginDecision::Exact(value) => assert_eq!(value, "https://api.test"),
                _ => panic!("expected exact decision"),
            }
        }

        #[test]
        fn should_return_skip_decision_when_origin_exact_missing_request_origin_then_skip_processing()
         {
            let origin = Origin::exact("https://app.test");
            let ctx = request_context("GET", "https://app.test");

            let decision = origin.resolve(None, &ctx);

            assert!(matches!(decision, OriginDecision::Skip));
        }

        #[test]
        fn should_return_mirror_decision_when_origin_list_matches_request_then_reflect_origin() {
            let origin = Origin::list(["https://api.test"]);
            let ctx = request_context("GET", "https://api.test");

            let decision = origin.resolve(Some("https://api.test"), &ctx);

            assert!(matches!(decision, OriginDecision::Mirror));
        }

        #[test]
        fn should_return_mirror_decision_when_origin_list_matches_case_insensitively_then_reflect_origin()
         {
            let origin = Origin::list(["https://api.test"]);
            let ctx = request_context("GET", "https://api.test");

            let decision = origin.resolve(Some("HTTPS://API.TEST"), &ctx);

            assert!(matches!(decision, OriginDecision::Mirror));
        }

        #[test]
        fn should_return_disallow_decision_when_origin_list_misses_then_block_origin() {
            let origin = Origin::list(["https://other.test"]);
            let ctx = request_context("GET", "https://api.test");

            let decision = origin.resolve(Some("https://api.test"), &ctx);

            assert!(matches!(decision, OriginDecision::Disallow));
        }

        #[test]
        fn should_return_disallow_decision_when_origin_list_has_different_scheme_then_block_origin()
        {
            let origin = Origin::list(["https://api.test"]);
            let ctx = request_context("GET", "http://api.test");

            let decision = origin.resolve(Some("http://api.test"), &ctx);

            assert!(matches!(decision, OriginDecision::Disallow));
        }

        #[test]
        fn should_return_disallow_decision_when_origin_list_contains_false_matcher_then_block_origin()
         {
            let origin = Origin::list([OriginMatcher::Bool(false)]);
            let ctx = request_context("GET", "https://api.test");

            let decision = origin.resolve(Some("https://api.test"), &ctx);

            assert!(matches!(decision, OriginDecision::Disallow));
        }

        #[test]
        fn should_return_mirror_decision_when_origin_list_contains_true_matcher_then_allow_all_origins()
         {
            let origin = Origin::list([OriginMatcher::Bool(true)]);
            let ctx = request_context("GET", "https://api.test");

            let decision = origin.resolve(Some("https://edge.allowed"), &ctx);

            assert!(matches!(decision, OriginDecision::Mirror));
        }

        #[test]
        fn should_return_disallow_decision_when_origin_list_has_different_port_then_block_origin() {
            let origin = Origin::list(["https://api.test:8443"]);
            let ctx = request_context("GET", "https://api.test");

            let decision = origin.resolve(Some("https://api.test"), &ctx);

            assert!(matches!(decision, OriginDecision::Disallow));
        }

        #[test]
        fn should_return_disallow_decision_when_origin_length_exceeds_limit_then_block_request() {
            let origin = Origin::any();
            let ctx = request_context("GET", "https://edge.test");
            let long_origin = format!("https://{}", "a".repeat(super::MAX_ORIGIN_LENGTH + 10));

            let decision = origin.resolve(Some(&long_origin), &ctx);

            assert!(matches!(decision, OriginDecision::Disallow));
        }

        #[test]
        fn should_match_unicode_origins_case_insensitively_then_allow_exact_origin() {
            let origin = Origin::exact("https://TÉST.dev");
            let ctx = request_context("GET", "https://tést.dev");

            let decision = origin.resolve(Some("https://tést.dev"), &ctx);

            match decision {
                OriginDecision::Exact(value) => assert_eq!(value, "https://TÉST.dev"),
                _ => panic!("expected exact decision"),
            }
        }

        #[test]
        fn should_return_mirror_decision_when_origin_list_contains_unicode_exact_then_reflect_origin()
         {
            let origin = Origin::list(["https://TÉST.dev"]);
            let ctx = request_context("GET", "https://tést.dev");

            let decision = origin.resolve(Some("https://tést.dev"), &ctx);

            assert!(matches!(decision, OriginDecision::Mirror));
        }

        #[test]
        fn should_return_skip_decision_when_origin_list_missing_request_origin_then_skip_processing()
         {
            let origin = Origin::list(["https://api.test"]);
            let ctx = request_context("GET", "");

            let decision = origin.resolve(None, &ctx);

            assert!(matches!(decision, OriginDecision::Skip));
        }

        #[test]
        fn should_return_mirror_decision_when_origin_list_contains_null_string_then_allow_null_origin()
         {
            let origin = Origin::list(["null"]);
            let ctx = request_context("GET", "null");

            let decision = origin.resolve(Some("null"), &ctx);

            assert!(matches!(decision, OriginDecision::Mirror));
        }

        #[test]
        fn should_return_any_decision_when_origin_any_receives_null_string_then_allow_null_origin()
        {
            let origin = Origin::any();
            let ctx = request_context("GET", "null");

            let decision = origin.resolve(Some("null"), &ctx);

            assert!(matches!(decision, OriginDecision::Any));
        }

        #[test]
        fn should_return_mirror_decision_when_predicate_matches_then_reflect_origin() {
            let origin = Origin::predicate(|value, _| value.ends_with(".test"));
            let ctx = request_context("GET", "https://api.test");

            let decision = origin.resolve(Some("https://api.test"), &ctx);

            assert!(matches!(decision, OriginDecision::Mirror));
        }

        #[test]
        fn should_return_disallow_decision_when_predicate_rejects_origin_then_block_request() {
            let origin = Origin::predicate(|value, _| value == "https://allowed.test");
            let ctx = request_context("GET", "https://api.test");

            let decision = origin.resolve(Some("https://api.test"), &ctx);

            assert!(matches!(decision, OriginDecision::Disallow));
        }

        #[test]
        fn should_return_disallow_decision_when_predicate_returns_false_then_block_request() {
            let origin = Origin::predicate(|value, _| value == "https://allowed.test");
            let ctx = request_context("GET", "https://blocked.test");

            let decision = origin.resolve(Some("https://blocked.test"), &ctx);

            assert!(matches!(decision, OriginDecision::Disallow));
        }

        #[test]
        fn should_return_skip_decision_when_origin_header_missing_then_avoid_invoking_predicate() {
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
        fn should_forward_decision_when_custom_callback_returns_value_then_propagate_result() {
            let origin = Origin::custom(|_, _| OriginDecision::Exact("https://custom.test".into()));
            let ctx = request_context("GET", "https://api.test");

            let decision = origin.resolve(Some("https://api.test"), &ctx);

            match decision {
                OriginDecision::Exact(value) => assert_eq!(value, "https://custom.test"),
                _ => panic!("expected custom decision"),
            }
        }

        #[test]
        fn should_return_disallow_decision_when_custom_callback_receives_no_origin_then_handle_missing_header()
         {
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
        fn should_return_false_when_origin_any_then_skip_vary_header() {
            let origin = Origin::any();

            let vary = origin.vary_on_disallow();

            assert!(!vary);
        }

        #[test]
        fn should_return_true_when_origin_exact_then_emit_vary_header() {
            let origin = Origin::exact("https://api.test");

            let vary = origin.vary_on_disallow();

            assert!(vary);
        }
    }
}
