use super::*;
use crate::context::RequestContext;
use std::borrow::Cow;

fn request(
    method: &'static str,
    origin: Option<&'static str>,
    acrm: Option<&'static str>,
    acrh: Option<&'static str>,
) -> RequestContext<'static> {
    RequestContext {
        method,
        origin,
        access_control_request_method: acrm,
        access_control_request_headers: acrh,
        access_control_request_private_network: false,
    }
}

mod new {
    use super::*;

    #[test]
    fn should_store_lowercase_when_components_have_uppercase_then_normalize_fields() {
        let ctx = request(
            "OPTIONS",
            Some("HTTPS://API.TEST"),
            Some("POST"),
            Some("X-CUSTOM"),
        );

        let normalized = NormalizedRequest::new(&ctx);

        assert_eq!(normalized.method, "options");
        assert_eq!(normalized.origin.as_deref(), Some("https://api.test"));
        assert_eq!(
            normalized.access_control_request_method.as_deref(),
            Some("post")
        );
        assert_eq!(
            normalized.access_control_request_headers.as_deref(),
            Some("x-custom")
        );
    }

    #[test]
    fn should_borrow_original_when_components_are_lowercase_then_avoid_allocation() {
        let ctx = request("get", Some("https://api.test"), Some("post"), Some("x-custom"));

        let normalized = NormalizedRequest::new(&ctx);

        assert!(matches!(normalized.method, Cow::Borrowed("get")));
        assert!(matches!(
            normalized.origin,
            Some(Cow::Borrowed("https://api.test"))
        ));
    }

    #[test]
    fn should_lowercase_unicode_uppercase_origin_then_normalize_non_ascii() {
        let ctx = request("GET", Some("https://DÉV.TEST"), Some("POST"), Some("X-CUSTOM"));

        let normalized = NormalizedRequest::new(&ctx);

        assert_eq!(normalized.origin.as_deref(), Some("https://dév.test"));
        assert_eq!(normalized.method, "get");
        assert_eq!(
            normalized.access_control_request_method.as_deref(),
            Some("post")
        );
        assert_eq!(
            normalized.access_control_request_headers.as_deref(),
            Some("x-custom")
        );
    }

    #[test]
    fn should_release_unicode_buffer_when_no_uppercase_then_preserve_borrowed_origin() {
        let ctx = request(
            "get",
            Some("https://mañana.test"),
            Some("post"),
            Some("x-custom"),
        );

        let normalized = NormalizedRequest::new(&ctx);

        assert!(matches!(
            normalized.origin,
            Some(Cow::Borrowed("https://mañana.test"))
        ));
        assert!(matches!(normalized.method, Cow::Borrowed("get")));
    }

    #[test]
    fn should_return_none_when_origin_header_missing_then_skip_allocation() {
        let ctx = request("get", None, Some("post"), Some("x-custom"));

        let normalized = NormalizedRequest::new(&ctx);

        assert!(normalized.origin.is_none());
    }
}

mod normalize_optional_component {
    use super::*;

    #[test]
    fn should_return_none_when_input_is_none_then_skip_normalization() {
        let normalized = NormalizedRequest::normalize_optional_component(None);

        assert!(normalized.is_none());
    }

    #[test]
    fn should_return_none_when_trimmed_value_is_empty_then_filter_out() {
        let normalized = NormalizedRequest::normalize_optional_component(Some("   \t  "));

        assert!(normalized.is_none());
    }

    #[test]
    fn should_borrow_when_value_is_already_lowercase_then_avoid_allocation() {
        let normalized = NormalizedRequest::normalize_optional_component(Some("x-custom"));

        assert!(matches!(normalized, Some(Cow::Borrowed("x-custom"))));
    }

    #[test]
    fn should_trim_and_lowercase_when_value_has_whitespace_and_uppercase_then_allocate_owned() {
        let normalized = NormalizedRequest::normalize_optional_component(Some("  X-CUSTOM  "));

        assert_eq!(normalized.as_deref(), Some("x-custom"));
        assert!(matches!(normalized, Some(Cow::Owned(_))));
    }
}

mod as_context {
    use super::*;

    #[test]
    fn should_return_normalized_view_when_context_requested_then_expose_lowercase_fields() {
        let ctx = request(
            "OPTIONS",
            Some("https://API.TEST"),
            Some("POST"),
            Some("X-CUSTOM"),
        );
        let normalized = NormalizedRequest::new(&ctx);

        let view = normalized.as_context();

        assert_eq!(view.method, "options");
        assert_eq!(view.origin, Some("https://api.test"));
        assert_eq!(view.access_control_request_method, Some("post"));
        assert_eq!(view.access_control_request_headers, Some("x-custom"));
        assert!(!view.access_control_request_private_network);
    }

    #[test]
    fn should_preserve_true_when_private_network_flag_set_then_propagate_state() {
        let ctx = RequestContext {
            method: "OPTIONS",
            origin: Some("https://api.test"),
            access_control_request_method: Some("POST"),
            access_control_request_headers: Some("X-CUSTOM"),
            access_control_request_private_network: true,
        };
        let normalized = NormalizedRequest::new(&ctx);

        let view = normalized.as_context();

        assert!(view.access_control_request_private_network);
    }
}

mod is_options {
    use super::*;

    #[test]
    fn should_return_true_when_method_is_options_then_detect_preflight() {
        let ctx = request("OPTIONS", Some("https://api.test"), None, None);
        let normalized = NormalizedRequest::new(&ctx);

        let result = normalized.is_options();

        assert!(result);
    }

    #[test]
    fn should_return_false_when_method_is_not_options_then_skip_preflight_detection() {
        let ctx = request("GET", Some("https://api.test"), None, None);
        let normalized = NormalizedRequest::new(&ctx);

        let result = normalized.is_options();

        assert!(!result);
    }
}

#[cfg(debug_assertions)]
mod pool_instrumentation {
    use super::*;

    #[test]
    fn should_release_buffers_after_drop_then_balance_stats() {
        super::normalization_pool_reset();

        {
            let ctx = request(
                "OPTIONS",
                Some("HTTPS://POOL.TEST"),
                Some("POST"),
                Some("X-CUSTOM"),
            );
            let normalized = NormalizedRequest::new(&ctx);
            assert!(matches!(normalized.method, Cow::Owned(_)));
        }

        let stats = super::normalization_pool_stats();
        assert_eq!(stats.acquired, stats.released);
        assert_eq!(stats.current_in_use, 0);
        assert!(stats.max_in_use >= 1);
    }

    #[test]
    fn should_discard_extra_buffers_when_pool_full_then_skip_reinsertion() {
        super::normalization_pool_reset();
        super::NORMALIZATION_BUFFER_POOL.with(|pool| pool.borrow_mut().clear());

        let ctx = request(
            "OPTIONS",
            Some("HTTPS://FILL.TEST"),
            Some("POST"),
            Some("X-CUSTOM"),
        );
        let mut held = Vec::with_capacity(super::NORMALIZATION_BUFFER_POOL_LIMIT);
        for _ in 0..super::NORMALIZATION_BUFFER_POOL_LIMIT {
            let normalized = NormalizedRequest::new(&ctx);
            assert!(matches!(normalized.method, Cow::Owned(_)));
            held.push(normalized);
        }

        drop(held);

        super::NORMALIZATION_BUFFER_POOL.with(|pool| {
            let pool = pool.borrow();
            assert_eq!(pool.len(), super::NORMALIZATION_BUFFER_POOL_LIMIT);
        });

        {
            let ctx = request(
                "OPTIONS",
                Some("HTTPS://OVERFLOW.TEST"),
                Some("POST"),
                Some("X-CUSTOM"),
            );
            let normalized = NormalizedRequest::new(&ctx);
            assert!(matches!(normalized.method, Cow::Owned(_)));
        }

        super::NORMALIZATION_BUFFER_POOL.with(|pool| {
            let pool = pool.borrow();
            assert_eq!(pool.len(), super::NORMALIZATION_BUFFER_POOL_LIMIT);
        });
    }
}
