use bunner_cors_rs::constants::method;
use bunner_cors_rs::{
    AllowedHeaders, AllowedMethods, Cors, CorsOptions, ExposedHeaders, Origin, RequestContext,
    TimingAllowOrigin,
};

#[derive(Default)]
pub struct CorsBuilder {
    origin: Option<Origin>,
    methods: Option<AllowedMethods>,
    allowed_headers: Option<AllowedHeaders>,
    exposed_headers: Option<ExposedHeaders>,
    credentials: Option<bool>,
    max_age: Option<u64>,
    allow_null_origin: Option<bool>,
    private_network: Option<bool>,
    timing_allow_origin: Option<TimingAllowOrigin>,
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

    pub fn allowed_headers(mut self, headers: AllowedHeaders) -> Self {
        self.allowed_headers = Some(headers);
        self
    }

    pub fn exposed_headers<I, S>(mut self, headers: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.exposed_headers = Some(ExposedHeaders::list(headers));
        self
    }

    pub fn exposed_headers_config(mut self, headers: ExposedHeaders) -> Self {
        self.exposed_headers = Some(headers);
        self
    }

    pub fn credentials(mut self, enabled: bool) -> Self {
        self.credentials = Some(enabled);
        self
    }

    pub fn max_age(mut self, value: u64) -> Self {
        self.max_age = Some(value);
        self
    }

    pub fn allow_null_origin(mut self, enabled: bool) -> Self {
        self.allow_null_origin = Some(enabled);
        self
    }

    pub fn private_network(mut self, enabled: bool) -> Self {
        self.private_network = Some(enabled);
        self
    }

    pub fn timing_allow_origin(mut self, timing: TimingAllowOrigin) -> Self {
        self.timing_allow_origin = Some(timing);
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
            allow_null_origin: default_allow_null_origin,
            allow_private_network: default_private_network,
            timing_allow_origin: default_timing_allow_origin,
        } = CorsOptions::default();

        let mut origin = self.origin.unwrap_or(default_origin);
        let credentials = self.credentials.unwrap_or(default_credentials);

        if credentials {
            origin = match origin {
                Origin::Any => Origin::predicate(|_, _| true),
                other => other,
            };
        }

        Cors::new(CorsOptions {
            origin,
            methods: self.methods.unwrap_or(default_methods),
            allowed_headers: self.allowed_headers.unwrap_or(default_allowed_headers),
            exposed_headers: self.exposed_headers.unwrap_or(default_exposed_headers),
            credentials,
            max_age: self.max_age.or(default_max_age),
            allow_null_origin: self.allow_null_origin.unwrap_or(default_allow_null_origin),
            allow_private_network: self.private_network.unwrap_or(default_private_network),
            timing_allow_origin: self.timing_allow_origin.or(default_timing_allow_origin),
        })
        .expect("valid CORS configuration")
    }
}

pub struct SimpleRequestBuilder {
    method: String,
    origin: Option<String>,
    private_network: bool,
}

impl SimpleRequestBuilder {
    pub fn new() -> Self {
        Self {
            method: method::GET.into(),
            origin: None,
            private_network: false,
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

    pub fn private_network(mut self, enabled: bool) -> Self {
        self.private_network = enabled;
        self
    }

    pub fn check(self, cors: &Cors) -> bunner_cors_rs::CorsDecision {
        let SimpleRequestBuilder {
            method,
            origin,
            private_network,
        } = self;
        let ctx = RequestContext {
            method: &method,
            origin: origin.as_deref(),
            access_control_request_method: None,
            access_control_request_headers: None,
            access_control_request_private_network: private_network,
        };
        cors.check(&ctx)
            .expect("simple request evaluation should succeed")
    }
}

#[derive(Default)]
pub struct PreflightRequestBuilder {
    origin: Option<String>,
    request_method: Option<String>,
    request_headers: Option<String>,
    private_network: bool,
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

    pub fn private_network(mut self, enabled: bool) -> Self {
        self.private_network = enabled;
        self
    }

    pub fn check(self, cors: &Cors) -> bunner_cors_rs::CorsDecision {
        let PreflightRequestBuilder {
            origin,
            request_method,
            request_headers,
            private_network,
        } = self;

        let ctx = RequestContext {
            method: method::OPTIONS,
            origin: origin.as_deref(),
            access_control_request_method: request_method.as_deref(),
            access_control_request_headers: request_headers.as_deref(),
            access_control_request_private_network: private_network,
        };
        cors.check(&ctx)
            .expect("preflight request evaluation should succeed")
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
