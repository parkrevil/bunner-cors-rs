use crate::context::RequestContext;
use crate::header_builder::HeaderBuilder;
use crate::normalized_request::NormalizedRequest;
use crate::options::{CorsOptions, ValidationError};
use crate::origin::OriginDecision;
use crate::result::{
    CorsDecision, CorsError, PreflightRejection, PreflightRejectionReason, SimpleRejection,
    SimpleRejectionReason,
};

/// High-level entry point that evaluates incoming requests against a [`CorsOptions`]
/// configuration and produces a [`CorsDecision`].
///
/// The struct is intentionally lightweight; cloning it is cheap because the heavy
/// lifting happens per-request.
pub struct Cors {
    options: CorsOptions,
}

impl Cors {
    /// Creates a new CORS evaluator, validating the provided options before use.
    ///
    /// The validation step mirrors the logic executed during request processing,
    /// so failing fast here prevents inconsistent behaviour later in the pipeline.
    pub fn new(options: CorsOptions) -> Result<Self, ValidationError> {
        options.validate()?;
        Ok(Self { options })
    }

    /// Evaluates an incoming request and determines the appropriate CORS response.
    ///
    /// The method normalizes the raw request metadata, automatically dispatching
    /// to the preflight or simple request handling paths as defined by the CORS
    /// specification. The resulting [`CorsDecision`] encapsulates both header
    /// mutations and rejection reasons so callers can surface precise feedback to
    /// upstream layers.
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
        // Steps through the CORS preflight algorithm. We follow the WHATWG
        // reference flow: verify request metadata, emit allow headers, and
        // short-circuit with an explicit [`PreflightRejection`] when the request
        // violates policy. This keeps the observable behaviour identical to
        // browser expectations while allowing servers to reason about rejections
        // programmatically.
        let Some(requested_method) = normalized
            .access_control_request_method
            .filter(|method| !method.trim().is_empty())
        else {
            return Ok(CorsDecision::NotApplicable);
        };
        let builder = HeaderBuilder::new(&self.options);
        let (mut headers, decision) = builder.build_origin_headers(original, normalized)?;

        match decision {
            OriginDecision::Skip => return Ok(CorsDecision::NotApplicable),
            OriginDecision::Disallow => {
                return Ok(CorsDecision::PreflightRejected(PreflightRejection {
                    headers: headers.into_headers(),
                    reason: PreflightRejectionReason::OriginNotAllowed,
                }));
            }
            OriginDecision::Any | OriginDecision::Mirror | OriginDecision::Exact(_) => {}
        }

        if !self.options.methods.allows_method(requested_method) {
            return Ok(CorsDecision::PreflightRejected(PreflightRejection {
                headers: headers.into_headers(),
                reason: PreflightRejectionReason::MethodNotAllowed {
                    requested_method: requested_method.to_string(),
                },
            }));
        }
        if let Some(requested_headers) = normalized.access_control_request_headers
            && !self
                .options
                .allowed_headers
                .allows_headers(requested_headers)
        {
            return Ok(CorsDecision::PreflightRejected(PreflightRejection {
                headers: headers.into_headers(),
                reason: PreflightRejectionReason::HeadersNotAllowed {
                    requested_headers: requested_headers.to_string(),
                },
            }));
        }
        headers.extend(builder.build_credentials_header());
        headers.extend(builder.build_methods_header());
        headers.extend(builder.build_allowed_headers());
        headers.extend(builder.build_private_network_header(original));
        headers.extend(builder.build_max_age_header());

        Ok(CorsDecision::PreflightAccepted {
            headers: headers.into_headers(),
        })
    }

    fn process_simple(
        &self,
        original: &RequestContext<'_>,
        normalized: &RequestContext<'_>,
    ) -> Result<CorsDecision, CorsError> {
        // Handles non-preflight requests. This path intentionally mirrors the
        // same origin resolution logic as `process_preflight`, but limits the
        // emitted headers to those allowed on "simple" requests. Returning
        // [`CorsDecision::NotApplicable`] allows upstream orchestration layers
        // to fall back to default behaviour for requests that never needed CORS.
        let builder = HeaderBuilder::new(&self.options);
        let (mut headers, decision) = builder.build_origin_headers(original, normalized)?;

        match decision {
            OriginDecision::Skip => return Ok(CorsDecision::NotApplicable),
            OriginDecision::Disallow => {
                return Ok(CorsDecision::SimpleRejected(SimpleRejection {
                    headers: headers.into_headers(),
                    reason: SimpleRejectionReason::OriginNotAllowed,
                }));
            }
            OriginDecision::Any | OriginDecision::Mirror | OriginDecision::Exact(_) => {}
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
}

#[cfg(test)]
#[path = "cors_test.rs"]
mod cors_test;
