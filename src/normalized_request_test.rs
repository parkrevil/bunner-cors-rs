use super::*;
use crate::context::RequestContext;
use std::borrow::Cow;

fn request(
    method: &'static str,
    origin: &'static str,
    acrm: &'static str,
    acrh: &'static str,
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
        let ctx = request("OPTIONS", "HTTPS://API.TEST", "POST", "X-CUSTOM");

        let normalized = NormalizedRequest::new(&ctx);

        assert_eq!(normalized.method, "options");
        assert_eq!(normalized.origin, "https://api.test");
        assert_eq!(normalized.access_control_request_method, "post");
        assert_eq!(normalized.access_control_request_headers, "x-custom");
    }

    #[test]
    fn should_borrow_original_when_components_are_lowercase_then_avoid_allocation() {
        let ctx = request("get", "https://api.test", "post", "x-custom");

        let normalized = NormalizedRequest::new(&ctx);

        assert!(matches!(normalized.method, Cow::Borrowed("get")));
        assert!(matches!(
            normalized.origin,
            Cow::Borrowed("https://api.test")
        ));
    }

    #[test]
    fn should_lowercase_unicode_uppercase_origin_then_normalize_non_ascii() {
        let ctx = request("GET", "https://DÉV.TEST", "POST", "X-CUSTOM");

        let normalized = NormalizedRequest::new(&ctx);

        assert_eq!(normalized.origin, "https://dév.test");
        assert_eq!(normalized.method, "get");
        assert_eq!(normalized.access_control_request_method, "post");
        assert_eq!(normalized.access_control_request_headers, "x-custom");
    }

    #[test]
    fn should_remain_empty_without_allocation_when_origin_is_empty_then_preserve_borrowed_slice() {
        let ctx = request("get", "", "post", "x-custom");

        let normalized = NormalizedRequest::new(&ctx);

        assert!(normalized.origin.is_empty());
        assert!(matches!(normalized.origin, Cow::Borrowed("")));
    }
}

mod as_context {
    use super::*;

    #[test]
    fn should_return_normalized_view_when_context_requested_then_expose_lowercase_fields() {
        let ctx = request("OPTIONS", "https://API.TEST", "POST", "X-CUSTOM");
        let normalized = NormalizedRequest::new(&ctx);

        let view = normalized.as_context();

        assert_eq!(view.method, "options");
        assert_eq!(view.origin, "https://api.test");
        assert_eq!(view.access_control_request_method, "post");
        assert_eq!(view.access_control_request_headers, "x-custom");
        assert!(!view.access_control_request_private_network);
    }

    #[test]
    fn should_preserve_true_when_private_network_flag_set_then_propagate_state() {
        let ctx = RequestContext {
            method: "OPTIONS",
            origin: "https://api.test",
            access_control_request_method: "POST",
            access_control_request_headers: "X-CUSTOM",
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
        let ctx = request("OPTIONS", "https://api.test", "", "");
        let normalized = NormalizedRequest::new(&ctx);

        let result = normalized.is_options();

        assert!(result);
    }

    #[test]
    fn should_return_false_when_method_is_not_options_then_skip_preflight_detection() {
        let ctx = request("GET", "https://api.test", "", "");
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
            let ctx = request("OPTIONS", "HTTPS://POOL.TEST", "POST", "X-CUSTOM");
            let normalized = NormalizedRequest::new(&ctx);
            assert!(matches!(normalized.method, Cow::Owned(_))); // ensure allocation
        }

        let stats = super::normalization_pool_stats();
        assert_eq!(stats.acquired, stats.released);
        assert_eq!(stats.current_in_use, 0);
        assert!(stats.max_in_use >= 1);
    }
}
