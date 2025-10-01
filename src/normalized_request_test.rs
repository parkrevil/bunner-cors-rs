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
    fn should_store_lowercase_given_components_have_uppercase() {
        let ctx = request("OPTIONS", "HTTPS://API.TEST", "POST", "X-CUSTOM");

        let normalized = NormalizedRequest::new(&ctx);

        assert_eq!(normalized.method, "options");
        assert_eq!(normalized.origin, "https://api.test");
        assert_eq!(normalized.access_control_request_method, "post");
        assert_eq!(normalized.access_control_request_headers, "x-custom");
    }

    #[test]
    fn should_borrow_original_given_components_are_lowercase() {
        let ctx = request("get", "https://api.test", "post", "x-custom");

        let normalized = NormalizedRequest::new(&ctx);

        assert!(matches!(normalized.method, Cow::Borrowed("get")));
        assert!(matches!(
            normalized.origin,
            Cow::Borrowed("https://api.test")
        ));
    }

    #[test]
    fn should_remain_empty_without_allocation_given_origin_is_empty() {
        let ctx = request("get", "", "post", "x-custom");

        let normalized = NormalizedRequest::new(&ctx);

        assert!(normalized.origin.is_empty());
        assert!(matches!(normalized.origin, Cow::Borrowed("")));
    }
}

mod as_context {
    use super::*;

    #[test]
    fn should_return_normalized_view_when_called() {
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
    fn should_preserve_true_given_private_network_flag_set() {
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
    fn should_return_true_given_method_is_options() {
        let ctx = request("OPTIONS", "https://api.test", "", "");
        let normalized = NormalizedRequest::new(&ctx);

        let result = normalized.is_options();

        assert!(result);
    }

    #[test]
    fn should_return_false_given_method_is_not_options() {
        let ctx = request("GET", "https://api.test", "", "");
        let normalized = NormalizedRequest::new(&ctx);

        let result = normalized.is_options();

        assert!(!result);
    }
}
