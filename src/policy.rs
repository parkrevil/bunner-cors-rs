use crate::allowed_headers::AllowedHeaders;
use crate::constants::{header, method};
use crate::context::RequestContext;
use crate::headers::{Header, HeaderCollection};
use crate::options::CorsOptions;
use crate::origin::OriginDecision;
use crate::result::{CorsDecision, PreflightResult, SimpleResult};

/// Core CORS policy engine that evaluates requests using [`CorsOptions`].
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
        let decision = self.options.origin.resolve(
            if request.origin.is_empty() {
                None
            } else {
                Some(request.origin)
            },
            request,
        );

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
                if !request.origin.is_empty() {
                    headers.push(Header::new(
                        header::ACCESS_CONTROL_ALLOW_ORIGIN,
                        request.origin,
                    ));
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
                if !request.access_control_request_headers.is_empty() {
                    headers.push(Header::new(
                        header::ACCESS_CONTROL_ALLOW_HEADERS,
                        request.access_control_request_headers,
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
