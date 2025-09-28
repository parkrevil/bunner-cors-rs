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

pub type OriginPredicateFn = dyn for<'a> Fn(&str, &RequestContext<'a>) -> bool + Send + Sync;

/// Origin 허용 전략.
#[derive(Clone)]
pub enum Origin {
    Wildcard,
    Exact(String),
    List(Vec<String>),
    Predicate(Arc<OriginPredicateFn>),
}

impl Origin {
    pub(crate) fn allows(&self, origin: &str, ctx: &RequestContext<'_>) -> bool {
        match self {
            Origin::Wildcard => true,
            Origin::Exact(expected) => origin == expected,
            Origin::List(values) => values.iter().any(|value| value == origin),
            Origin::Predicate(checker) => checker(origin, ctx),
        }
    }
}

/// CORS 설정값.
#[derive(Clone)]
pub struct CorsOptions {
    pub origin: Origin,
    pub methods: Vec<String>,
    pub allowed_headers: Option<Vec<String>>,
    pub exposed_headers: Option<Vec<String>>,
    pub credentials: bool,
    pub max_age: Option<u64>,
    pub preflight_continue: bool,
    pub options_success_status: u16,
}

impl Default for CorsOptions {
    fn default() -> Self {
        Self {
            origin: Origin::Wildcard,
            methods: vec![
                "GET".into(),
                "HEAD".into(),
                "PUT".into(),
                "PATCH".into(),
                "POST".into(),
                "DELETE".into(),
            ],
            allowed_headers: None,
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
    pub fn new(options: CorsOptions) -> Self {
        Self { options }
    }

    pub fn evaluate(&self, request: &RequestContext<'_>) -> CorsDecision {
        if request.method.eq_ignore_ascii_case("OPTIONS") {
            CorsDecision::Preflight(self.evaluate_preflight(request))
        } else {
            CorsDecision::Simple(self.evaluate_simple(request))
        }
    }

    fn evaluate_preflight(&self, request: &RequestContext<'_>) -> PreflightResult {
        let mut headers = Vec::new();
        headers.extend(self.build_origin_headers(request));
        headers.extend(self.build_credentials_header());
        headers.extend(self.build_methods_header());
        headers.extend(self.build_allowed_headers(request));
        headers.extend(self.build_max_age_header());
        headers.extend(self.build_exposed_headers());

        PreflightResult {
            headers,
            status: self.options.options_success_status,
            halt_response: !self.options.preflight_continue,
        }
    }

    fn evaluate_simple(&self, request: &RequestContext<'_>) -> SimpleResult {
        let mut headers = Vec::new();
        headers.extend(self.build_origin_headers(request));
        headers.extend(self.build_credentials_header());
        headers.extend(self.build_exposed_headers());

        SimpleResult { headers }
    }

    fn build_origin_headers(&self, request: &RequestContext<'_>) -> Vec<Header> {
        let mut headers = Vec::new();
        if let Some(origin) = request.origin {
            match &self.options.origin {
                Origin::Wildcard => {
                    headers.push(Header::new("Access-Control-Allow-Origin", "*"));
                }
                Origin::Exact(expected) => {
                    if origin == expected {
                        headers.push(Header::new("Access-Control-Allow-Origin", expected.clone()));
                    }
                }
                other => {
                    if other.allows(origin, request) {
                        headers.push(Header::new("Access-Control-Allow-Origin", origin));
                        headers.push(Header::new("Vary", "Origin"));
                    }
                }
            }
        }
        headers
    }

    fn build_methods_header(&self) -> Vec<Header> {
        if self.options.methods.is_empty() {
            return Vec::new();
        }
        let methods = self.options.methods.join(",");
        vec![Header::new("Access-Control-Allow-Methods", methods)]
    }

    fn build_credentials_header(&self) -> Vec<Header> {
        if self.options.credentials {
            vec![Header::new("Access-Control-Allow-Credentials", "true")]
        } else {
            Vec::new()
        }
    }

    fn build_allowed_headers(&self, request: &RequestContext<'_>) -> Vec<Header> {
        match (
            &self.options.allowed_headers,
            request.access_control_request_headers,
        ) {
            (Some(headers), _) if !headers.is_empty() => {
                vec![Header::new(
                    "Access-Control-Allow-Headers",
                    headers.join(","),
                )]
            }
            (None, Some(request_headers)) if !request_headers.is_empty() => {
                vec![
                    Header::new("Vary", "Access-Control-Request-Headers"),
                    Header::new("Access-Control-Allow-Headers", request_headers),
                ]
            }
            _ => Vec::new(),
        }
    }

    fn build_exposed_headers(&self) -> Vec<Header> {
        match &self.options.exposed_headers {
            Some(headers) if !headers.is_empty() => vec![Header::new(
                "Access-Control-Expose-Headers",
                headers.join(","),
            )],
            _ => Vec::new(),
        }
    }

    fn build_max_age_header(&self) -> Vec<Header> {
        match self.options.max_age {
            Some(value) => vec![Header::new("Access-Control-Max-Age", value.to_string())],
            None => Vec::new(),
        }
    }
}
