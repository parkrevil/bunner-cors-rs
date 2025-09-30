use crate::context::RequestContext;
use crate::header_builder::{HeaderBuilder, OriginOutcome};
use crate::headers::HeaderCollection;
use crate::normalized_request::NormalizedRequest;
use crate::options::{CorsOptions, ValidationError};
use crate::result::{CorsDecision, CorsError, PreflightRejection, PreflightRejectionReason};

/// Core CORS policy engine that evaluates requests using [`CorsOptions`].
pub struct Cors {
    options: CorsOptions,
}

impl Cors {
    pub fn new(options: CorsOptions) -> Result<Self, ValidationError> {
        options.validate()?;
        Ok(Self { options })
    }

    pub fn check(&self, request: &RequestContext<'_>) -> Result<CorsDecision, CorsError> {
        let normalized_request = NormalizedRequest::new(request);
        let normalized_ctx = normalized_request.as_context();

        if normalized_request.is_options() {
            self.process_preflight(request, &normalized_ctx)
        } else {
            self.process_simple(request, &normalized_ctx)
        }
    }

    fn process_preflight(
        &self,
        original: &RequestContext<'_>,
        normalized: &RequestContext<'_>,
    ) -> Result<CorsDecision, CorsError> {
        if normalized.access_control_request_method.trim().is_empty() {
            return Ok(CorsDecision::NotApplicable);
        }
        let builder = HeaderBuilder::new(&self.options);
        let origin = Self::resolve_origin_headers(&builder, original, normalized)?;

        let Some((origin_headers, origin_allowed)) = origin else {
            return Ok(CorsDecision::NotApplicable);
        };

        let mut headers = origin_headers;

        if !origin_allowed {
            return Ok(CorsDecision::PreflightRejected(PreflightRejection {
                headers: headers.into_headers(),
                reason: PreflightRejectionReason::OriginNotAllowed,
            }));
        }

        if !self
            .options
            .methods
            .allows_method(normalized.access_control_request_method)
        {
            return Ok(CorsDecision::NotApplicable);
        }
        if !self
            .options
            .allowed_headers
            .allows_headers(normalized.access_control_request_headers)
        {
            return Ok(CorsDecision::NotApplicable);
        }
        headers.extend(builder.build_credentials_header());
        headers.extend(builder.build_methods_header());
        headers.extend(builder.build_allowed_headers());
        headers.extend(builder.build_private_network_header(original));
        headers.extend(builder.build_max_age_header());
        headers.extend(builder.build_timing_allow_origin_header());

        Ok(CorsDecision::PreflightAccepted {
            headers: headers.into_headers(),
        })
    }

    fn process_simple(
        &self,
        original: &RequestContext<'_>,
        normalized: &RequestContext<'_>,
    ) -> Result<CorsDecision, CorsError> {
        let builder = HeaderBuilder::new(&self.options);
        let origin = Self::resolve_origin_headers(&builder, original, normalized)?;

        let Some((origin_headers, origin_allowed)) = origin else {
            return Ok(CorsDecision::NotApplicable);
        };

        let mut headers = origin_headers;

        if !origin_allowed {
            return Ok(CorsDecision::SimpleAccepted {
                headers: headers.into_headers(),
            });
        }

        if !self.options.methods.allows_method(normalized.method) {
            return Ok(CorsDecision::NotApplicable);
        }
        headers.extend(builder.build_credentials_header());
        headers.extend(builder.build_private_network_header(original));
        headers.extend(builder.build_exposed_headers());
        headers.extend(builder.build_timing_allow_origin_header());

        Ok(CorsDecision::SimpleAccepted {
            headers: headers.into_headers(),
        })
    }

    fn resolve_origin_headers(
        builder: &HeaderBuilder<'_>,
        original: &RequestContext<'_>,
        normalized: &RequestContext<'_>,
    ) -> Result<Option<(HeaderCollection, bool)>, CorsError> {
        match builder.build_origin_headers(original, normalized)? {
            OriginOutcome::Skip => Ok(None),
            OriginOutcome::Disallow(headers) => Ok(Some((headers, false))),
            OriginOutcome::Allow(headers) => Ok(Some((headers, true))),
        }
    }
}

#[cfg(test)]
#[path = "cors_test.rs"]
mod cors_test;
