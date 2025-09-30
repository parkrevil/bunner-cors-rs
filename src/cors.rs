use crate::context::RequestContext;
use crate::header_builder::{HeaderBuilder, OriginOutcome};
use crate::headers::HeaderCollection;
use crate::normalized_request::NormalizedRequest;
use crate::options::{CorsOptions, ValidationError};
use crate::result::{CorsDecision, PreflightResult, SimpleResult};

/// Core CORS policy engine that evaluates requests using [`CorsOptions`].
pub struct Cors {
    options: CorsOptions,
}

impl Cors {
    pub fn new(options: CorsOptions) -> Result<Self, ValidationError> {
        options.validate()?;
        Ok(Self { options })
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
        if normalized.access_control_request_method.trim().is_empty() {
            return None;
        }
        let builder = HeaderBuilder::new(&self.options);
        let mut headers = HeaderCollection::new();
        let (origin_headers, origin_allowed) =
            Self::resolve_origin_headers(&builder, original, normalized)?;

        headers.extend(origin_headers);

        if !origin_allowed {
            return Some(PreflightResult {
                headers: headers.into_headers(),
                status: self.options.options_success_status,
                end_response: true,
            });
        }

        if !self
            .options
            .methods
            .allows_method(normalized.access_control_request_method)
        {
            return None;
        }
        if !self
            .options
            .allowed_headers
            .allows_headers(normalized.access_control_request_headers)
        {
            return None;
        }
        headers.extend(builder.build_credentials_header());
        headers.extend(builder.build_methods_header());
        headers.extend(builder.build_allowed_headers());
        headers.extend(builder.build_private_network_header(original));
        headers.extend(builder.build_max_age_header());
        headers.extend(builder.build_timing_allow_origin_header());

        Some(PreflightResult {
            headers: headers.into_headers(),
            status: self.options.options_success_status,
            end_response: true,
        })
    }

    fn process_simple(
        &self,
        original: &RequestContext<'_>,
        normalized: &RequestContext<'_>,
    ) -> Option<SimpleResult> {
        let builder = HeaderBuilder::new(&self.options);
        let mut headers = HeaderCollection::new();
        let (origin_headers, origin_allowed) =
            Self::resolve_origin_headers(&builder, original, normalized)?;

        headers.extend(origin_headers);

        if !origin_allowed {
            return Some(SimpleResult {
                headers: headers.into_headers(),
            });
        }

        if !self.options.methods.allows_method(normalized.method) {
            return None;
        }
        headers.extend(builder.build_credentials_header());
        headers.extend(builder.build_private_network_header(original));
        headers.extend(builder.build_exposed_headers());
        headers.extend(builder.build_timing_allow_origin_header());

        Some(SimpleResult {
            headers: headers.into_headers(),
        })
    }

    fn resolve_origin_headers(
        builder: &HeaderBuilder<'_>,
        original: &RequestContext<'_>,
        normalized: &RequestContext<'_>,
    ) -> Option<(HeaderCollection, bool)> {
        match builder.build_origin_headers(original, normalized) {
            OriginOutcome::Skip => None,
            OriginOutcome::Disallow(headers) => Some((headers, false)),
            OriginOutcome::Allow(headers) => Some((headers, true)),
        }
    }
}

#[cfg(test)]
#[path = "cors_test.rs"]
mod cors_test;
