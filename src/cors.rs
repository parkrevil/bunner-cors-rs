use crate::context::RequestContext;
use crate::header_builder::HeaderBuilder;
use crate::normalized_request::NormalizedRequest;
use crate::options::{CorsOptions, ValidationError};
use crate::origin::OriginDecision;
use crate::result::{CorsDecision, CorsError, PreflightRejection, PreflightRejectionReason};

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
        let builder = HeaderBuilder::new(&self.options);
        let (mut headers, decision) = builder.build_origin_headers(original, normalized)?;

        match decision {
            OriginDecision::Skip => return Ok(CorsDecision::NotApplicable),
            OriginDecision::Disallow => {
                return Ok(CorsDecision::SimpleAccepted {
                    headers: headers.into_headers(),
                });
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
