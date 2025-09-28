use bunner_cors_rs::{
    AllowedHeaders, CorsDecision, CorsOptions, CorsPolicy, Header, Origin, OriginDecision,
    OriginMatcher, RequestContext,
};
use regex::Regex;
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
    header_value(headers, "Vary")
        .map(|value| {
            value
                .split(',')
                .map(|part| part.trim().to_string())
                .filter(|part| !part.is_empty())
                .collect::<BTreeSet<_>>()
        })
        .unwrap_or_default()
}

fn assert_simple<'a>(decision: CorsDecision) -> Vec<Header> {
    match decision {
        CorsDecision::Simple(result) => result.headers,
        other => panic!("expected simple decision, got {:?}", other),
    }
}

fn assert_preflight<'a>(decision: CorsDecision) -> (Vec<Header>, u16, bool) {
    match decision {
        CorsDecision::Preflight(result) => (result.headers, result.status, result.halt_response),
        other => panic!("expected preflight decision, got {:?}", other),
    }
}

#[test]
fn default_simple_request_allows_any_origin() {
    let policy = CorsPolicy::new(CorsOptions::default());
    let request = RequestContext::new("GET").with_origin(Some("https://example.com"));

    let headers = assert_simple(policy.evaluate(&request));

    assert_eq!(
        header_value(&headers, "Access-Control-Allow-Origin"),
        Some("*")
    );
    assert!(!has_header(&headers, "Vary"));
}

#[test]
fn default_simple_request_without_origin_still_allows_any() {
    let policy = CorsPolicy::new(CorsOptions::default());
    let request = RequestContext::new("GET");

    let headers = assert_simple(policy.evaluate(&request));

    assert_eq!(
        header_value(&headers, "Access-Control-Allow-Origin"),
        Some("*")
    );
}

#[test]
fn exact_origin_is_reflected_with_vary() {
    let mut options = CorsOptions::default();
    options.origin = Origin::exact("https://allowed.dev");
    let policy = CorsPolicy::new(options);

    let request = RequestContext::new("POST").with_origin(Some("https://other.dev"));
    let headers = assert_simple(policy.evaluate(&request));

    assert_eq!(
        header_value(&headers, "Access-Control-Allow-Origin"),
        Some("https://allowed.dev")
    );
    assert_eq!(
        vary_values(&headers),
        BTreeSet::from(["Origin".to_string()])
    );
}

#[test]
fn origin_list_supports_exact_and_patterns() {
    let mut options = CorsOptions::default();
    options.origin = Origin::list([
        OriginMatcher::exact("https://exact.example"),
        OriginMatcher::pattern(Regex::new(r"^https://.*\.allowed\.org$").unwrap()),
    ]);
    let policy = CorsPolicy::new(options);

    let request = RequestContext::new("GET").with_origin(Some("https://service.allowed.org"));
    let headers = assert_simple(policy.evaluate(&request));

    assert_eq!(
        header_value(&headers, "Access-Control-Allow-Origin"),
        Some("https://service.allowed.org")
    );
    assert_eq!(
        vary_values(&headers),
        BTreeSet::from(["Origin".to_string()])
    );

    let disallowed_request = RequestContext::new("GET").with_origin(Some("https://deny.dev"));
    let headers = assert_simple(policy.evaluate(&disallowed_request));

    assert!(!has_header(&headers, "Access-Control-Allow-Origin"));
    assert_eq!(
        vary_values(&headers),
        BTreeSet::from(["Origin".to_string()])
    );
}

#[test]
fn predicate_origin_allows_custom_logic() {
    let mut options = CorsOptions::default();
    options.origin = Origin::predicate(|origin, _ctx| origin.ends_with(".trusted"));
    let policy = CorsPolicy::new(options);

    let allowed = RequestContext::new("GET").with_origin(Some("https://service.trusted"));
    let allowed_headers = assert_simple(policy.evaluate(&allowed));
    assert_eq!(
        header_value(&allowed_headers, "Access-Control-Allow-Origin"),
        Some("https://service.trusted")
    );

    let denied = RequestContext::new("GET").with_origin(Some("https://service.untrusted"));
    let denied_headers = assert_simple(policy.evaluate(&denied));
    assert!(!has_header(&denied_headers, "Access-Control-Allow-Origin"));
}

#[test]
fn custom_origin_can_skip_processing() {
    let mut options = CorsOptions::default();
    options.origin = Origin::custom(|origin, _ctx| match origin {
        Some("https://allow.me") => OriginDecision::Mirror,
        _ => OriginDecision::Skip,
    });
    let policy = CorsPolicy::new(options);

    let allowed_request = RequestContext::new("GET").with_origin(Some("https://allow.me"));
    let allowed_headers = assert_simple(policy.evaluate(&allowed_request));
    assert_eq!(
        header_value(&allowed_headers, "Access-Control-Allow-Origin"),
        Some("https://allow.me")
    );

    let skipped_request = RequestContext::new("GET").with_origin(Some("https://deny.me"));
    assert!(matches!(
        policy.evaluate(&skipped_request),
        CorsDecision::NotApplicable
    ));
}

#[test]
fn default_preflight_reflects_request_headers() {
    let policy = CorsPolicy::new(CorsOptions::default());
    let request = RequestContext::new("OPTIONS")
        .with_origin(Some("https://foo.bar"))
        .with_access_control_request_headers(Some("X-Test, Content-Type"));

    let (headers, status, halt) = assert_preflight(policy.evaluate(&request));

    assert_eq!(status, 204);
    assert!(
        halt,
        "preflight should halt when preflight_continue is false"
    );
    assert_eq!(
        header_value(&headers, "Access-Control-Allow-Origin"),
        Some("*")
    );
    assert_eq!(
        header_value(&headers, "Access-Control-Allow-Headers"),
        Some("X-Test, Content-Type")
    );
    assert_eq!(
        vary_values(&headers),
        BTreeSet::from(["Access-Control-Request-Headers".into()])
    );
}

#[test]
fn preflight_with_explicit_headers_does_not_reflect_request() {
    let mut options = CorsOptions::default();
    options.allowed_headers = AllowedHeaders::list(["Content-Type", "X-Custom"]);
    let policy = CorsPolicy::new(options);

    let request = RequestContext::new("OPTIONS")
        .with_origin(Some("https://foo.bar"))
        .with_access_control_request_headers(Some("Another"));

    let (headers, _status, _halt) = assert_preflight(policy.evaluate(&request));

    assert_eq!(
        header_value(&headers, "Access-Control-Allow-Headers"),
        Some("Content-Type,X-Custom")
    );
    assert!(
        vary_values(&headers).is_empty(),
        "should not add Vary when headers list is explicit"
    );
}

#[test]
fn credentials_and_exposed_headers_are_honored() {
    let mut options = CorsOptions::default();
    options.credentials = true;
    options.exposed_headers = Some(vec!["X-Response-Time".into(), "X-Trace".into()]);
    let policy = CorsPolicy::new(options);

    let request = RequestContext::new("GET").with_origin(Some("https://foo.bar"));
    let headers = assert_simple(policy.evaluate(&request));

    assert_eq!(
        header_value(&headers, "Access-Control-Allow-Credentials"),
        Some("true")
    );
    assert_eq!(
        header_value(&headers, "Access-Control-Expose-Headers"),
        Some("X-Response-Time,X-Trace")
    );
}

#[test]
fn max_age_and_preflight_continue_affect_preflight_response() {
    let mut options = CorsOptions::default();
    options.max_age = Some("600".into());
    options.preflight_continue = true;
    let policy = CorsPolicy::new(options);

    let request = RequestContext::new("OPTIONS").with_origin(Some("https://foo.bar"));
    let (headers, status, halt) = assert_preflight(policy.evaluate(&request));

    assert_eq!(status, 204);
    assert!(
        !halt,
        "halt flag should be false when preflight_continue is true"
    );
    assert_eq!(
        header_value(&headers, "Access-Control-Max-Age"),
        Some("600")
    );
}

#[test]
fn vary_headers_are_deduplicated_and_sorted() {
    let mut options = CorsOptions::default();
    options.origin = Origin::exact("https://allowed.dev");
    options.allowed_headers = AllowedHeaders::MirrorRequest;
    let policy = CorsPolicy::new(options);

    let request = RequestContext::new("OPTIONS")
        .with_origin(Some("https://allowed.dev"))
        .with_access_control_request_headers(Some("X-Test"));

    let (headers, _status, _halt) = assert_preflight(policy.evaluate(&request));
    let vary = vary_values(&headers);

    assert_eq!(
        vary,
        BTreeSet::from([
            "Access-Control-Request-Headers".to_string(),
            "Origin".to_string()
        ])
    );
}

#[test]
fn disallowed_origin_returns_headers_without_allow_origin() {
    let mut options = CorsOptions::default();
    options.origin = Origin::list([OriginMatcher::exact("https://allow.one")]);
    let policy = CorsPolicy::new(options);

    let request = RequestContext::new("GET").with_origin(Some("https://deny.one"));
    let headers = assert_simple(policy.evaluate(&request));

    assert!(!has_header(&headers, "Access-Control-Allow-Origin"));
    assert_eq!(
        vary_values(&headers),
        BTreeSet::from(["Origin".to_string()])
    );
}
