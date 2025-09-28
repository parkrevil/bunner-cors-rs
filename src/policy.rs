use crate::constants::{header, method};
use regex::Regex;
use std::collections::BTreeSet;
use std::sync::Arc;

/// CORS 처리 시 필요한 헤더를 표현하는 단순 구조체.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Header {
    pub name: String,
    pub value: String,
}

impl Header {
    pub fn new<N: Into<String>, V: Into<String>>(name: N, value: V) -> Self {
        Self {
            name: name.into(),
            value: value.into(),
        }
    }
}

#[derive(Default)]
struct HeaderCollection {
    headers: Vec<Header>,
    vary_values: BTreeSet<String>,
}

impl HeaderCollection {
    fn new() -> Self {
        Self::default()
    }

    fn push(&mut self, header: Header) {
        if header.name.eq_ignore_ascii_case(header::VARY) {
            self.add_vary(header.value);
        } else {
            self.headers.push(header);
        }
    }

    fn add_vary<S: Into<String>>(&mut self, value: S) {
        for part in value.into().split(',') {
            let trimmed = part.trim();
            if !trimmed.is_empty() {
                self.vary_values.insert(trimmed.to_string());
            }
        }
    }

    fn extend(&mut self, mut other: HeaderCollection) {
        for header in other.headers.drain(..) {
            self.push(header);
        }
        for value in other.vary_values {
            self.vary_values.insert(value);
        }
    }

    fn into_headers(mut self) -> Vec<Header> {
        if !self.vary_values.is_empty() {
            let value = self.vary_values.into_iter().collect::<Vec<_>>().join(", ");
            self.headers.push(Header::new(header::VARY, value));
        }
        self.headers
    }
}

pub type OriginPredicateFn = dyn for<'a> Fn(&str, &RequestContext<'a>) -> bool + Send + Sync;
pub type OriginCallbackFn =
    dyn for<'a> Fn(Option<&'a str>, &RequestContext<'a>) -> OriginDecision + Send + Sync;

/// Origin 허용 전략.
#[derive(Clone, Default)]
pub enum Origin {
    #[default]
    Any,
    Exact(String),
    List(Vec<OriginMatcher>),
    Predicate(Arc<OriginPredicateFn>),
    Custom(Arc<OriginCallbackFn>),
}

/// Origin 결정 결과.
#[derive(Debug, Clone)]
pub enum OriginDecision {
    Any,
    Exact(String),
    Mirror,
    Disallow,
    Skip,
}

impl OriginDecision {
    pub fn any() -> Self {
        Self::Any
    }

    pub fn exact<S: Into<String>>(value: S) -> Self {
        Self::Exact(value.into())
    }

    pub fn mirror() -> Self {
        Self::Mirror
    }

    pub fn disallow() -> Self {
        Self::Disallow
    }

    pub fn skip() -> Self {
        Self::Skip
    }
}

impl From<bool> for OriginDecision {
    fn from(value: bool) -> Self {
        if value {
            OriginDecision::Mirror
        } else {
            OriginDecision::Skip
        }
    }
}

impl<T> From<Option<T>> for OriginDecision
where
    T: Into<String>,
{
    fn from(value: Option<T>) -> Self {
        match value {
            Some(inner) => OriginDecision::Exact(inner.into()),
            None => OriginDecision::Skip,
        }
    }
}

/// Origin 매칭을 표현.
#[derive(Clone)]
pub enum OriginMatcher {
    Exact(String),
    Pattern(Regex),
    Bool(bool),
}

impl OriginMatcher {
    pub fn exact<S: Into<String>>(value: S) -> Self {
        Self::Exact(value.into())
    }

    pub fn pattern(regex: Regex) -> Self {
        Self::Pattern(regex)
    }

    fn matches(&self, candidate: &str) -> bool {
        match self {
            OriginMatcher::Exact(value) => value == candidate,
            OriginMatcher::Pattern(regex) => regex.is_match(candidate),
            OriginMatcher::Bool(value) => *value,
        }
    }
}

impl From<String> for OriginMatcher {
    fn from(value: String) -> Self {
        OriginMatcher::Exact(value)
    }
}

impl From<&str> for OriginMatcher {
    fn from(value: &str) -> Self {
        OriginMatcher::Exact(value.to_owned())
    }
}

impl From<bool> for OriginMatcher {
    fn from(value: bool) -> Self {
        OriginMatcher::Bool(value)
    }
}

/// 허용되는 요청 헤더 설정.
#[derive(Clone, Default, PartialEq, Eq)]
pub enum AllowedHeaders {
    #[default]
    MirrorRequest,
    List(Vec<String>),
}

impl AllowedHeaders {
    pub fn list<I, S>(values: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        Self::List(values.into_iter().map(Into::into).collect())
    }
}

impl Origin {
    pub fn any() -> Self {
        Self::Any
    }

    pub fn exact<S: Into<String>>(value: S) -> Self {
        Self::Exact(value.into())
    }

    pub fn list<I, T>(values: I) -> Self
    where
        I: IntoIterator<Item = T>,
        T: Into<OriginMatcher>,
    {
        Self::List(values.into_iter().map(Into::into).collect())
    }

    pub fn predicate<F>(predicate: F) -> Self
    where
        F: for<'a> Fn(&str, &RequestContext<'a>) -> bool + Send + Sync + 'static,
    {
        Self::Predicate(Arc::new(predicate))
    }

    pub fn custom<F>(callback: F) -> Self
    where
        F: for<'a> Fn(Option<&'a str>, &RequestContext<'a>) -> OriginDecision
            + Send
            + Sync
            + 'static,
    {
        Self::Custom(Arc::new(callback))
    }

    pub fn disabled() -> Self {
        Self::custom(|_, _| OriginDecision::Skip)
    }

    fn resolve(&self, request_origin: Option<&str>, ctx: &RequestContext<'_>) -> OriginDecision {
        match self {
            Origin::Any => OriginDecision::Any,
            Origin::Exact(value) => OriginDecision::Exact(value.clone()),
            Origin::List(matchers) => {
                if let Some(origin) = request_origin {
                    if matchers.iter().any(|matcher| matcher.matches(origin)) {
                        OriginDecision::Mirror
                    } else {
                        OriginDecision::Disallow
                    }
                } else {
                    OriginDecision::Disallow
                }
            }
            Origin::Predicate(predicate) => {
                if let Some(origin) = request_origin {
                    if predicate(origin, ctx) {
                        OriginDecision::Mirror
                    } else {
                        OriginDecision::Disallow
                    }
                } else {
                    OriginDecision::Disallow
                }
            }
            Origin::Custom(callback) => callback(request_origin, ctx),
        }
    }

    fn vary_on_disallow(&self) -> bool {
        !matches!(self, Origin::Any)
    }
}
/// CORS 설정값.
#[derive(Clone)]
pub struct CorsOptions {
    pub origin: Origin,
    pub methods: Vec<String>,
    pub allowed_headers: AllowedHeaders,
    /// Optional alias matching the legacy Node configuration field name `headers`.
    pub headers: Option<AllowedHeaders>,
    pub exposed_headers: Option<Vec<String>>,
    pub credentials: bool,
    pub max_age: Option<String>,
    pub preflight_continue: bool,
    pub options_success_status: u16,
}

impl Default for CorsOptions {
    fn default() -> Self {
        Self {
            origin: Origin::Any,
            methods: vec![
                method::GET.into(),
                method::HEAD.into(),
                method::PUT.into(),
                method::PATCH.into(),
                method::POST.into(),
                method::DELETE.into(),
            ],
            allowed_headers: AllowedHeaders::default(),
            headers: None,
            exposed_headers: None,
            credentials: false,
            max_age: None,
            preflight_continue: false,
            options_success_status: 204,
        }
    }
}

/// 요청과 관련된 최소 컨텍스트.
#[derive(Debug, Clone)]
pub struct RequestContext<'a> {
    pub method: &'a str,
    pub origin: Option<&'a str>,
    pub access_control_request_method: Option<&'a str>,
    pub access_control_request_headers: Option<&'a str>,
}

impl<'a> RequestContext<'a> {
    pub fn new(method: &'a str) -> Self {
        Self {
            method,
            origin: None,
            access_control_request_method: None,
            access_control_request_headers: None,
        }
    }

    pub fn with_origin(mut self, origin: Option<&'a str>) -> Self {
        self.origin = origin;
        self
    }

    pub fn with_access_control_request_method(mut self, value: Option<&'a str>) -> Self {
        self.access_control_request_method = value;
        self
    }

    pub fn with_access_control_request_headers(mut self, value: Option<&'a str>) -> Self {
        self.access_control_request_headers = value;
        self
    }
}

/// Preflight 결과.
#[derive(Debug, Clone)]
pub struct PreflightResult {
    pub headers: Vec<Header>,
    pub status: u16,
    pub halt_response: bool,
}

/// 일반 요청 결과.
#[derive(Debug, Clone)]
pub struct SimpleResult {
    pub headers: Vec<Header>,
}

/// 평가 결과.
#[derive(Debug, Clone)]
pub enum CorsDecision {
    Preflight(PreflightResult),
    Simple(SimpleResult),
    NotApplicable,
}

/// 코어 CORS 정책 엔진.
pub struct CorsPolicy {
    options: CorsOptions,
}

impl CorsPolicy {
    pub fn new(mut options: CorsOptions) -> Self {
        if let Some(headers_alias) = options.headers.take() {
            if matches!(options.allowed_headers, AllowedHeaders::MirrorRequest) {
                options.allowed_headers = headers_alias;
            } else {
                options.headers = Some(headers_alias);
            }
        }

        Self { options }
    }

    pub fn evaluate(&self, request: &RequestContext<'_>) -> CorsDecision {
        if request.method.eq_ignore_ascii_case(method::OPTIONS) {
            match self.evaluate_preflight(request) {
                Some(result) => CorsDecision::Preflight(result),
                None => CorsDecision::NotApplicable,
            }
        } else {
            match self.evaluate_simple(request) {
                Some(result) => CorsDecision::Simple(result),
                None => CorsDecision::NotApplicable,
            }
        }
    }

    fn evaluate_preflight(&self, request: &RequestContext<'_>) -> Option<PreflightResult> {
        let mut headers = HeaderCollection::new();
        let (origin_headers, skip) = self.build_origin_headers(request);
        if skip {
            return None;
        }
        headers.extend(origin_headers);
        headers.extend(self.build_credentials_header());
        headers.extend(self.build_methods_header());
        headers.extend(self.build_allowed_headers(request));
        headers.extend(self.build_max_age_header());
        headers.extend(self.build_exposed_headers());

        Some(PreflightResult {
            headers: headers.into_headers(),
            status: self.options.options_success_status,
            halt_response: !self.options.preflight_continue,
        })
    }

    fn evaluate_simple(&self, request: &RequestContext<'_>) -> Option<SimpleResult> {
        let mut headers = HeaderCollection::new();
        let (origin_headers, skip) = self.build_origin_headers(request);
        if skip {
            return None;
        }
        headers.extend(origin_headers);
        headers.extend(self.build_credentials_header());
        headers.extend(self.build_exposed_headers());

        Some(SimpleResult {
            headers: headers.into_headers(),
        })
    }

    fn build_origin_headers(&self, request: &RequestContext<'_>) -> (HeaderCollection, bool) {
        let mut headers = HeaderCollection::new();
        let decision = self.options.origin.resolve(request.origin, request);

        match decision {
            OriginDecision::Any => {
                headers.push(Header::new(header::ACCESS_CONTROL_ALLOW_ORIGIN, "*"));
            }
            OriginDecision::Exact(value) => {
                headers.add_vary(header::ORIGIN);
                headers.push(Header::new(header::ACCESS_CONTROL_ALLOW_ORIGIN, value));
            }
            OriginDecision::Mirror => {
                headers.add_vary(header::ORIGIN);
                if let Some(origin) = request.origin {
                    headers.push(Header::new(header::ACCESS_CONTROL_ALLOW_ORIGIN, origin));
                }
            }
            OriginDecision::Disallow => {
                if self.options.origin.vary_on_disallow() {
                    headers.add_vary(header::ORIGIN);
                }
            }
            OriginDecision::Skip => {
                return (HeaderCollection::new(), true);
            }
        }

        (headers, false)
    }

    fn build_methods_header(&self) -> HeaderCollection {
        let mut headers = HeaderCollection::new();
        if !self.options.methods.is_empty() {
            let methods = self.options.methods.join(",");
            headers.push(Header::new(header::ACCESS_CONTROL_ALLOW_METHODS, methods));
        }
        headers
    }

    fn build_credentials_header(&self) -> HeaderCollection {
        let mut headers = HeaderCollection::new();
        if self.options.credentials {
            headers.push(Header::new(
                header::ACCESS_CONTROL_ALLOW_CREDENTIALS,
                "true",
            ));
        }
        headers
    }

    fn build_allowed_headers(&self, request: &RequestContext<'_>) -> HeaderCollection {
        let mut headers = HeaderCollection::new();
        match &self.options.allowed_headers {
            AllowedHeaders::List(values) => {
                if !values.is_empty() {
                    headers.push(Header::new(
                        header::ACCESS_CONTROL_ALLOW_HEADERS,
                        values.join(","),
                    ));
                }
            }
            AllowedHeaders::MirrorRequest => {
                headers.add_vary(header::ACCESS_CONTROL_REQUEST_HEADERS);
                if let Some(request_headers) = request.access_control_request_headers
                    && !request_headers.is_empty()
                {
                    headers.push(Header::new(
                        header::ACCESS_CONTROL_ALLOW_HEADERS,
                        request_headers,
                    ));
                }
            }
        }
        headers
    }

    fn build_exposed_headers(&self) -> HeaderCollection {
        let mut headers = HeaderCollection::new();
        if let Some(values) = &self.options.exposed_headers
            && !values.is_empty()
        {
            headers.push(Header::new(
                header::ACCESS_CONTROL_EXPOSE_HEADERS,
                values.join(","),
            ));
        }
        headers
    }

    fn build_max_age_header(&self) -> HeaderCollection {
        let mut headers = HeaderCollection::new();
        if let Some(value) = &self.options.max_age
            && !value.is_empty()
        {
            headers.push(Header::new(header::ACCESS_CONTROL_MAX_AGE, value.clone()));
        }
        headers
    }
}
