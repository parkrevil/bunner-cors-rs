use bunner_cors_rs::{
    AllowedHeaders, CorsDecision, CorsOptions, CorsPolicy, Header, Origin, OriginDecision,
    OriginMatcher, RequestContext,
};
use std::collections::BTreeSet;

fn header_value<'a>(headers: &'a [Header], name: &str) -> Option<&'a str> {
    headers
        .iter()
        .find(|header| header.name.eq_ignore_ascii_case(name))
        .map(|header| header.value.as_str())
}

fn has_header(headers: &[Header], name: &str) -> bool {
    header_value(headers, name).is_some()
}

fn vary_values(headers: &[Header]) -> BTreeSet<String> {
    header_value(headers, constants::HEADER_VARY)
        .map(|value| {
            value
                .split(',')
                .map(|part| part.trim().to_string())
                .filter(|part| !part.is_empty())
                .collect::<BTreeSet<_>>()
        })
        .unwrap_or_default()
}

fn assert_simple(decision: CorsDecision) -> Vec<Header> {
    match decision {
        CorsDecision::Simple(result) => result.headers,
        other => panic!("expected simple decision, got {:?}", other),
    }
}

fn assert_preflight(decision: CorsDecision) -> (Vec<Header>, u16, bool) {
    match decision {
        CorsDecision::Preflight(result) => (result.headers, result.status, result.halt_response),
        other => panic!("expected preflight decision, got {:?}", other),
    }
}

mod constants {
    pub const HEADER_ALLOW_ORIGIN: &str = "Access-Control-Allow-Origin";
    pub const HEADER_ALLOW_HEADERS: &str = "Access-Control-Allow-Headers";
    pub const HEADER_ALLOW_CREDENTIALS: &str = "Access-Control-Allow-Credentials";
    pub const HEADER_EXPOSE_HEADERS: &str = "Access-Control-Expose-Headers";
    pub const HEADER_MAX_AGE: &str = "Access-Control-Max-Age";
    pub const HEADER_VARY: &str = "Vary";
    pub const HEADER_ORIGIN: &str = "Origin";
    pub const HEADER_REQUEST_HEADERS: &str = "Access-Control-Request-Headers";

    pub const METHOD_GET: &str = "GET";
    pub const METHOD_POST: &str = "POST";
    pub const METHOD_PUT: &str = "PUT";
    pub const METHOD_DELETE: &str = "DELETE";
    pub const METHOD_OPTIONS: &str = "OPTIONS";
}

mod helpers {
    use super::constants;
    use super::*;

    #[derive(Default)]
    pub struct PolicyBuilder {
        origin: Option<Origin>,
        methods: Option<Vec<String>>,
        allowed_headers: Option<AllowedHeaders>,
        exposed_headers: Option<Vec<String>>,
        credentials: Option<bool>,
        max_age: Option<String>,
        preflight_continue: Option<bool>,
    }

    impl PolicyBuilder {
        pub fn new() -> Self {
            Self::default()
        }

        pub fn origin(mut self, origin: Origin) -> Self {
            self.origin = Some(origin);
            self
        }

        pub fn methods<I, S>(mut self, methods: I) -> Self
        where
            I: IntoIterator<Item = S>,
            S: Into<String>,
        {
            self.methods = Some(methods.into_iter().map(Into::into).collect());
            self
        }

        pub fn allowed_headers(mut self, headers: AllowedHeaders) -> Self {
            self.allowed_headers = Some(headers);
            self
        }

        pub fn exposed_headers<I, S>(mut self, headers: I) -> Self
        where
            I: IntoIterator<Item = S>,
            S: Into<String>,
        {
            self.exposed_headers = Some(headers.into_iter().map(Into::into).collect());
            self
        }

        pub fn credentials(mut self, enabled: bool) -> Self {
            self.credentials = Some(enabled);
            self
        }

        pub fn max_age(mut self, value: impl Into<String>) -> Self {
            self.max_age = Some(value.into());
            self
        }

        pub fn preflight_continue(mut self, enabled: bool) -> Self {
            self.preflight_continue = Some(enabled);
            self
        }

        pub fn build(self) -> CorsPolicy {
            let CorsOptions {
                origin: default_origin,
                methods: default_methods,
                allowed_headers: default_allowed_headers,
                exposed_headers: default_exposed_headers,
                credentials: default_credentials,
                max_age: default_max_age,
                preflight_continue: default_preflight_continue,
                options_success_status: default_success_status,
            } = CorsOptions::default();

            CorsPolicy::new(CorsOptions {
                origin: self.origin.unwrap_or(default_origin),
                methods: self.methods.unwrap_or(default_methods),
                allowed_headers: self.allowed_headers.unwrap_or(default_allowed_headers),
                exposed_headers: self.exposed_headers.or(default_exposed_headers),
                credentials: self.credentials.unwrap_or(default_credentials),
                max_age: self.max_age.or(default_max_age),
                preflight_continue: self
                    .preflight_continue
                    .unwrap_or(default_preflight_continue),
                options_success_status: default_success_status,
            })
        }
    }

    pub struct SimpleRequestBuilder {
        method: String,
        origin: Option<String>,
    }

    impl SimpleRequestBuilder {
        pub fn new() -> Self {
            Self {
                method: constants::METHOD_GET.into(),
                origin: None,
            }
        }

        pub fn method(mut self, method: impl Into<String>) -> Self {
            self.method = method.into();
            self
        }

        pub fn origin(mut self, origin: impl Into<String>) -> Self {
            self.origin = Some(origin.into());
            self
        }

        pub fn evaluate(self, policy: &CorsPolicy) -> CorsDecision {
            let SimpleRequestBuilder { method, origin } = self;
            let mut ctx = RequestContext::new(&method);
            ctx = ctx.with_origin(origin.as_deref());
            policy.evaluate(&ctx)
        }
    }

    #[derive(Default)]
    pub struct PreflightRequestBuilder {
        origin: Option<String>,
        request_method: Option<String>,
        request_headers: Option<String>,
    }

    impl PreflightRequestBuilder {
        pub fn new() -> Self {
            Self::default()
        }

        pub fn origin(mut self, origin: impl Into<String>) -> Self {
            self.origin = Some(origin.into());
            self
        }

        pub fn request_method(mut self, method: impl Into<String>) -> Self {
            self.request_method = Some(method.into());
            self
        }

        pub fn request_headers(mut self, headers: impl Into<String>) -> Self {
            self.request_headers = Some(headers.into());
            self
        }

        pub fn evaluate(self, policy: &CorsPolicy) -> CorsDecision {
            let PreflightRequestBuilder {
                origin,
                request_method,
                request_headers,
            } = self;

            let mut ctx = RequestContext::new(constants::METHOD_OPTIONS);
            ctx = ctx.with_origin(origin.as_deref());
            ctx = ctx.with_access_control_request_method(request_method.as_deref());
            ctx = ctx.with_access_control_request_headers(request_headers.as_deref());
            policy.evaluate(&ctx)
        }
    }

    pub fn policy() -> PolicyBuilder {
        PolicyBuilder::new()
    }

    pub fn simple_request() -> SimpleRequestBuilder {
        SimpleRequestBuilder::new()
    }

    pub fn preflight_request() -> PreflightRequestBuilder {
        PreflightRequestBuilder::new()
    }
}

mod simple_requests {
    use super::helpers::{policy, simple_request};
    use super::*;

    #[test]
    fn default_simple_request_allows_any_origin() {
        let policy = policy().build();
        let headers = assert_simple(
            simple_request()
                .origin("https://example.com")
                .evaluate(&policy),
        );

        assert_eq!(
            header_value(&headers, constants::HEADER_ALLOW_ORIGIN),
            Some("*")
        );
        assert!(!has_header(&headers, constants::HEADER_VARY));
    }

    #[test]
    fn default_simple_request_without_origin_still_allows_any() {
        let policy = policy().build();
        let headers = assert_simple(simple_request().evaluate(&policy));

        assert_eq!(
            header_value(&headers, constants::HEADER_ALLOW_ORIGIN),
            Some("*")
        );
    }
}

mod origin_configuration {
    use super::helpers::{policy, simple_request};
    use super::*;
    use regex::Regex;

    #[test]
    fn exact_origin_is_reflected_with_vary() {
        let policy = policy()
            .origin(Origin::exact("https://allowed.dev"))
            .build();

        let headers = assert_simple(
            simple_request()
                .method(constants::METHOD_POST)
                .origin("https://other.dev")
                .evaluate(&policy),
        );

        assert_eq!(
            header_value(&headers, constants::HEADER_ALLOW_ORIGIN),
            Some("https://allowed.dev")
        );
        assert_eq!(
            vary_values(&headers),
            BTreeSet::from([constants::HEADER_ORIGIN.to_string()])
        );
    }

    #[test]
    fn origin_list_supports_exact_and_patterns() {
        let policy = policy()
            .origin(Origin::list([
                OriginMatcher::exact("https://exact.example"),
                OriginMatcher::pattern(Regex::new(r"^https://.*\.allowed\.org$").unwrap()),
            ]))
            .build();

        let headers = assert_simple(
            simple_request()
                .origin("https://service.allowed.org")
                .evaluate(&policy),
        );

        assert_eq!(
            header_value(&headers, constants::HEADER_ALLOW_ORIGIN),
            Some("https://service.allowed.org")
        );
        assert_eq!(
            vary_values(&headers),
            BTreeSet::from([constants::HEADER_ORIGIN.to_string()])
        );

        let headers = assert_simple(
            simple_request()
                .origin("https://deny.dev")
                .evaluate(&policy),
        );

        assert!(!has_header(&headers, constants::HEADER_ALLOW_ORIGIN));
        assert_eq!(
            vary_values(&headers),
            BTreeSet::from([constants::HEADER_ORIGIN.to_string()])
        );
    }

    #[test]
    fn predicate_origin_allows_custom_logic() {
        let policy = policy()
            .origin(Origin::predicate(|origin, _ctx| {
                origin.ends_with(".trusted")
            }))
            .build();

        let allowed_headers = assert_simple(
            simple_request()
                .origin("https://service.trusted")
                .evaluate(&policy),
        );
        assert_eq!(
            header_value(&allowed_headers, constants::HEADER_ALLOW_ORIGIN),
            Some("https://service.trusted")
        );

        let denied_headers = assert_simple(
            simple_request()
                .origin("https://service.untrusted")
                .evaluate(&policy),
        );
        assert!(!has_header(&denied_headers, constants::HEADER_ALLOW_ORIGIN));
    }

    #[test]
    fn custom_origin_can_skip_processing() {
        let policy = policy()
            .origin(Origin::custom(|origin, _ctx| match origin {
                Some("https://allow.me") => OriginDecision::Mirror,
                _ => OriginDecision::Skip,
            }))
            .build();

        let allowed_headers = assert_simple(
            simple_request()
                .origin("https://allow.me")
                .evaluate(&policy),
        );
        assert_eq!(
            header_value(&allowed_headers, constants::HEADER_ALLOW_ORIGIN),
            Some("https://allow.me")
        );

        assert!(matches!(
            simple_request().origin("https://deny.me").evaluate(&policy),
            CorsDecision::NotApplicable
        ));
    }

    #[test]
    fn disallowed_origin_returns_headers_without_allow_origin() {
        let policy = policy()
            .origin(Origin::list([OriginMatcher::exact("https://allow.one")]))
            .build();

        let headers = assert_simple(
            simple_request()
                .origin("https://deny.one")
                .evaluate(&policy),
        );

        assert!(!has_header(&headers, constants::HEADER_ALLOW_ORIGIN));
        assert_eq!(
            vary_values(&headers),
            BTreeSet::from([constants::HEADER_ORIGIN.to_string()])
        );
    }
}

mod preflight_requests {
    use super::helpers::{policy, preflight_request};
    use super::*;

    #[test]
    fn default_preflight_reflects_request_headers() {
        let policy = policy().build();
        let (headers, status, halt) = assert_preflight(
            preflight_request()
                .origin("https://foo.bar")
                .request_method(constants::METHOD_GET)
                .request_headers("X-Test, Content-Type")
                .evaluate(&policy),
        );

        assert_eq!(status, 204);
        assert!(
            halt,
            "preflight should halt when preflight_continue is false"
        );
        assert_eq!(
            header_value(&headers, constants::HEADER_ALLOW_ORIGIN),
            Some("*")
        );
        assert_eq!(
            header_value(&headers, constants::HEADER_ALLOW_HEADERS),
            Some("X-Test, Content-Type")
        );
        assert_eq!(
            vary_values(&headers),
            BTreeSet::from([constants::HEADER_REQUEST_HEADERS.into()])
        );
    }

    #[test]
    fn preflight_with_disallowed_method_is_not_applicable() {
        assert!(matches!(
            preflight_request()
                .origin("https://foo.bar")
                .request_method(constants::METHOD_DELETE)
                .evaluate(
                    &policy()
                        .methods([constants::METHOD_GET, constants::METHOD_POST])
                        .build()
                ),
            CorsDecision::NotApplicable
        ));
    }

    #[test]
    fn preflight_with_disallowed_header_is_not_applicable() {
        assert!(matches!(
            preflight_request()
                .origin("https://foo.bar")
                .request_method(constants::METHOD_GET)
                .request_headers("X-Disallowed")
                .evaluate(
                    &policy()
                        .allowed_headers(AllowedHeaders::list(["X-Allowed"]))
                        .build()
                ),
            CorsDecision::NotApplicable
        ));
    }
}

mod header_configuration {
    use super::helpers::{policy, preflight_request, simple_request};
    use super::*;

    #[test]
    fn preflight_with_explicit_headers_does_not_reflect_request() {
        let policy = policy()
            .allowed_headers(AllowedHeaders::list(["Content-Type", "X-Custom"]))
            .build();

        let (headers, _status, _halt) = assert_preflight(
            preflight_request()
                .origin("https://foo.bar")
                .request_method(constants::METHOD_POST)
                .request_headers("X-Custom")
                .evaluate(&policy),
        );

        assert_eq!(
            header_value(&headers, constants::HEADER_ALLOW_HEADERS),
            Some("Content-Type,X-Custom")
        );
        assert!(
            vary_values(&headers).is_empty(),
            "should not add Vary when headers list is explicit"
        );
    }

    #[test]
    fn credentials_and_exposed_headers_are_honored() {
        let policy = policy()
            .credentials(true)
            .exposed_headers(["X-Response-Time", "X-Trace"])
            .build();

        let headers = assert_simple(simple_request().origin("https://foo.bar").evaluate(&policy));

        assert_eq!(
            header_value(&headers, constants::HEADER_ALLOW_CREDENTIALS),
            Some("true")
        );
        assert_eq!(
            header_value(&headers, constants::HEADER_EXPOSE_HEADERS),
            Some("X-Response-Time,X-Trace")
        );
    }

    #[test]
    fn vary_headers_are_deduplicated_and_sorted() {
        let policy = policy()
            .origin(Origin::exact("https://allowed.dev"))
            .allowed_headers(AllowedHeaders::MirrorRequest)
            .build();

        let (headers, _status, _halt) = assert_preflight(
            preflight_request()
                .origin("https://allowed.dev")
                .request_method(constants::METHOD_PUT)
                .request_headers("X-Test")
                .evaluate(&policy),
        );
        let vary = vary_values(&headers);

        assert_eq!(
            vary,
            BTreeSet::from([
                constants::HEADER_REQUEST_HEADERS.to_string(),
                constants::HEADER_ORIGIN.to_string()
            ])
        );
    }
}

mod misc_configuration {
    use super::helpers::{policy, preflight_request};
    use super::*;

    #[test]
    fn max_age_and_preflight_continue_affect_preflight_response() {
        let policy = policy().max_age("600").preflight_continue(true).build();

        let (headers, status, halt) = assert_preflight(
            preflight_request()
                .origin("https://foo.bar")
                .request_method(constants::METHOD_GET)
                .evaluate(&policy),
        );

        assert_eq!(status, 204);
        assert!(
            !halt,
            "halt flag should be false when preflight_continue is true"
        );
        assert_eq!(
            header_value(&headers, constants::HEADER_MAX_AGE),
            Some("600")
        );
    }
}

mod property_based {
    use super::constants;
    use super::helpers::{policy, preflight_request, simple_request};
    use super::*;
    use proptest::prelude::*;

    fn staggered_case(input: &str) -> String {
        input
            .chars()
            .enumerate()
            .map(|(idx, ch)| {
                if idx % 2 == 0 {
                    ch.to_ascii_lowercase()
                } else {
                    ch.to_ascii_uppercase()
                }
            })
            .collect()
    }

    fn subdomain_strategy() -> impl Strategy<Value = String> {
        proptest::string::string_regex("[a-z0-9]{1,16}").unwrap()
    }

    fn header_name_strategy() -> impl Strategy<Value = String> {
        proptest::string::string_regex("[A-Za-z]{1,16}").unwrap()
    }

    proptest! {
        #[test]
        fn exact_origin_reflects_arbitrary_https_subdomain(subdomain in subdomain_strategy()) {
            let origin = format!("https://{}.example.com", subdomain);

            let headers = assert_simple(
                simple_request()
                    .origin(origin.as_str())
                    .evaluate(
                        &policy()
                            .origin(Origin::exact(origin.clone()))
                            .build()
                    ),
            );

            prop_assert_eq!(
                header_value(&headers, constants::HEADER_ALLOW_ORIGIN),
                Some(origin.as_str())
            );
        }

        #[test]
        fn allowed_headers_matching_is_case_insensitive(header in header_name_strategy()) {
            let allowed = header.to_uppercase();
            let request_variant = staggered_case(&header);

            let decision = preflight_request()
                .origin("https://prop.test")
                .request_method(constants::METHOD_GET)
                .request_headers(request_variant)
                .evaluate(
                    &policy()
                        .allowed_headers(AllowedHeaders::list([allowed.clone()]))
                        .build()
                );

            prop_assert!(matches!(decision, CorsDecision::Preflight(_)));
        }
    }
}

mod snapshot_validations {
    use super::constants;
    use super::helpers::{policy, preflight_request};
    use super::*;
    use insta::assert_yaml_snapshot;
    use serde::Serialize;

    #[derive(Serialize)]
    struct HeaderSnapshot {
        name: String,
        value: String,
    }

    #[derive(Serialize)]
    struct PreflightSnapshot {
        status: u16,
        halt: bool,
        headers: Vec<HeaderSnapshot>,
    }

    fn capture_preflight(
        policy: &CorsPolicy,
        request: super::helpers::PreflightRequestBuilder,
    ) -> PreflightSnapshot {
        let (headers, status, halt) = assert_preflight(request.evaluate(policy));
        PreflightSnapshot {
            status,
            halt,
            headers: headers
                .into_iter()
                .map(|header| HeaderSnapshot {
                    name: header.name,
                    value: header.value,
                })
                .collect(),
        }
    }

    #[test]
    fn default_preflight_snapshot() {
        let snapshot = capture_preflight(
            &policy().build(),
            preflight_request()
                .origin("https://snapshot.dev")
                .request_method(constants::METHOD_GET)
                .request_headers("X-Debug, Content-Type"),
        );

        assert_yaml_snapshot!("default_preflight_snapshot", snapshot);
    }

    #[test]
    fn mirror_origin_preflight_snapshot() {
        let policy = policy()
            .origin(Origin::list([OriginMatcher::exact("https://mirror.dev")]))
            .credentials(true)
            .allowed_headers(AllowedHeaders::MirrorRequest)
            .max_age("3600")
            .build();

        let snapshot = capture_preflight(
            &policy,
            preflight_request()
                .origin("https://mirror.dev")
                .request_method(constants::METHOD_POST)
                .request_headers("X-Trace-Id"),
        );

        assert_yaml_snapshot!("mirror_origin_preflight_snapshot", snapshot);
    }
}
