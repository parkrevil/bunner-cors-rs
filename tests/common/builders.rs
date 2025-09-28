use bunner_cors_rs::constants::method;
use bunner_cors_rs::{AllowedHeaders, AllowedMethods, Cors, CorsOptions, Origin, RequestContext};

#[derive(Default)]
pub struct CorsBuilder {
    origin: Option<Origin>,
    methods: Option<AllowedMethods>,
    allowed_headers: Option<AllowedHeaders>,
    exposed_headers: Option<Vec<String>>,
    credentials: Option<bool>,
    max_age: Option<String>,
    preflight_continue: Option<bool>,
}

impl CorsBuilder {
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
        self.methods = Some(AllowedMethods::list(methods));
        self
    }

    pub fn methods_any(mut self) -> Self {
        self.methods = Some(AllowedMethods::any());
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

    pub fn build(self) -> Cors {
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

        Cors::new(CorsOptions {
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
            method: method::GET.into(),
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

    pub fn check(self, cors: &Cors) -> bunner_cors_rs::CorsDecision {
        let SimpleRequestBuilder { method, origin } = self;
        let ctx = RequestContext {
            method: &method,
            origin: origin.as_deref().unwrap_or(""),
            access_control_request_method: "",
            access_control_request_headers: "",
        };
        cors.check(&ctx)
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

    pub fn check(self, cors: &Cors) -> bunner_cors_rs::CorsDecision {
        let PreflightRequestBuilder {
            origin,
            request_method,
            request_headers,
        } = self;

        let ctx = RequestContext {
            method: method::OPTIONS,
            origin: origin.as_deref().unwrap_or(""),
            access_control_request_method: request_method.as_deref().unwrap_or(""),
            access_control_request_headers: request_headers.as_deref().unwrap_or(""),
        };
        cors.check(&ctx)
    }
}

pub fn cors() -> CorsBuilder {
    CorsBuilder::new()
}

pub fn simple_request() -> SimpleRequestBuilder {
    SimpleRequestBuilder::new()
}

pub fn preflight_request() -> PreflightRequestBuilder {
    PreflightRequestBuilder::new()
}
