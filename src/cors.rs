use crate::context::RequestContext;
use crate::header_builder::HeaderBuilder;
use crate::headers::HeaderCollection;
use crate::normalized_request::NormalizedRequest;
use crate::options::CorsOptions;
use crate::result::{CorsDecision, PreflightResult, SimpleResult};

/// Core CORS policy engine that evaluates requests using [`CorsOptions`].
pub struct Cors {
    options: CorsOptions,
}

impl Cors {
    pub fn new(options: CorsOptions) -> Self {
        Self { options }
    }

    pub fn check(&self, request: &RequestContext<'_>) -> CorsDecision {
        let normalized_request = NormalizedRequest::new(request);
        let normalized_ctx = normalized_request.as_context();

        if normalized_request.is_options() {
            match self.process_preflight(request, &normalized_ctx) {
                Some(result) => CorsDecision::Preflight(result),
                None => CorsDecision::NotApplicable,
            }
        } else {
            match self.process_simple(request, &normalized_ctx) {
                Some(result) => CorsDecision::Simple(result),
                None => CorsDecision::NotApplicable,
            }
        }
    }

    fn process_preflight(
        &self,
        original: &RequestContext<'_>,
        normalized: &RequestContext<'_>,
    ) -> Option<PreflightResult> {
        let builder = HeaderBuilder::new(&self.options);
        let mut headers = HeaderCollection::new();
        let (origin_headers, skip) = builder.build_origin_headers(original, normalized);
        if skip {
            return None;
        }
        headers.extend(origin_headers);
        headers.extend(builder.build_credentials_header());
        headers.extend(builder.build_methods_header());
        headers.extend(builder.build_allowed_headers(original));
        headers.extend(builder.build_max_age_header());
        headers.extend(builder.build_exposed_headers());

        Some(PreflightResult {
            headers: headers.into_headers(),
            status: self.options.options_success_status,
            end_response: !self.options.preflight_continue,
        })
    }

    fn process_simple(
        &self,
        original: &RequestContext<'_>,
        normalized: &RequestContext<'_>,
    ) -> Option<SimpleResult> {
        let builder = HeaderBuilder::new(&self.options);
        let mut headers = HeaderCollection::new();
        let (origin_headers, skip) = builder.build_origin_headers(original, normalized);
        if skip {
            return None;
        }
        headers.extend(origin_headers);
        headers.extend(builder.build_credentials_header());
        headers.extend(builder.build_exposed_headers());

        Some(SimpleResult {
            headers: headers.into_headers(),
        })
    }
}

#[cfg(test)]
#[path = "cors_test.rs"]
mod cors_test;
