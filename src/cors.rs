use crate::allowed_headers::AllowedHeaders;
use crate::constants::header;
use crate::context::RequestContext;
use crate::headers::{Header, HeaderCollection};
use crate::options::CorsOptions;
use crate::origin::OriginDecision;
use crate::result::{CorsDecision, PreflightResult, SimpleResult};
use std::borrow::Cow;

/// Core CORS policy engine that evaluates requests using [`CorsOptions`].
pub struct Cors {
    options: CorsOptions,
}

impl Cors {
    pub fn new(options: CorsOptions) -> Self {
        Self { options }
    }

    pub fn evaluate(&self, request: &RequestContext<'_>) -> CorsDecision {
        let normalized_request = NormalizedRequest::new(request);
        let normalized_ctx = normalized_request.as_context();

        if normalized_request.is_options() {
            match self.evaluate_preflight(request, &normalized_ctx) {
                Some(result) => CorsDecision::Preflight(result),
                None => CorsDecision::NotApplicable,
            }
        } else {
            match self.evaluate_simple(request, &normalized_ctx) {
                Some(result) => CorsDecision::Simple(result),
                None => CorsDecision::NotApplicable,
            }
        }
    }

    fn evaluate_preflight(
        &self,
        original: &RequestContext<'_>,
        normalized: &RequestContext<'_>,
    ) -> Option<PreflightResult> {
        let mut headers = HeaderCollection::new();
        let (origin_headers, skip) = self.build_origin_headers(original, normalized);
        if skip {
            return None;
        }
        headers.extend(origin_headers);
        headers.extend(self.build_credentials_header());
        headers.extend(self.build_methods_header());
        headers.extend(self.build_allowed_headers(original));
        headers.extend(self.build_max_age_header());
        headers.extend(self.build_exposed_headers());

        Some(PreflightResult {
            headers: headers.into_headers(),
            status: self.options.options_success_status,
            halt_response: !self.options.preflight_continue,
        })
    }

    fn evaluate_simple(
        &self,
        original: &RequestContext<'_>,
        normalized: &RequestContext<'_>,
    ) -> Option<SimpleResult> {
        let mut headers = HeaderCollection::new();
        let (origin_headers, skip) = self.build_origin_headers(original, normalized);
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

    fn build_origin_headers(
        &self,
        original: &RequestContext<'_>,
        normalized: &RequestContext<'_>,
    ) -> (HeaderCollection, bool) {
        let mut headers = HeaderCollection::new();
        let decision = self.options.origin.resolve(
            if normalized.origin.is_empty() {
                None
            } else {
                Some(normalized.origin)
            },
            normalized,
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
                if !original.origin.is_empty() {
                    headers.push(Header::new(
                        header::ACCESS_CONTROL_ALLOW_ORIGIN,
                        original.origin,
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

struct NormalizedRequest<'a> {
    method: Cow<'a, str>,
    origin: Cow<'a, str>,
    access_control_request_method: Cow<'a, str>,
    access_control_request_headers: Cow<'a, str>,
}

impl<'a> NormalizedRequest<'a> {
    fn new(request: &'a RequestContext<'a>) -> Self {
        Self {
            method: Self::normalize_component(request.method),
            origin: Self::normalize_component(request.origin),
            access_control_request_method: Self::normalize_component(
                request.access_control_request_method,
            ),
            access_control_request_headers: Self::normalize_component(
                request.access_control_request_headers,
            ),
        }
    }

    fn normalize_component(value: &'a str) -> Cow<'a, str> {
        if value.bytes().any(|byte| byte.is_ascii_uppercase()) {
            Cow::Owned(value.to_ascii_lowercase())
        } else {
            Cow::Borrowed(value)
        }
    }

    fn as_context(&self) -> RequestContext<'_> {
        RequestContext {
            method: self.method.as_ref(),
            origin: self.origin.as_ref(),
            access_control_request_method: self.access_control_request_method.as_ref(),
            access_control_request_headers: self.access_control_request_headers.as_ref(),
        }
    }

    fn is_options(&self) -> bool {
        self.method.as_ref() == "options"
    }
}
