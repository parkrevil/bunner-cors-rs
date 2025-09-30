use crate::headers::Headers;
use thiserror::Error;

/// Reasons why a preflight request was rejected.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PreflightRejectionReason {
    OriginNotAllowed,
    MethodNotAllowed { requested_method: String },
    HeadersNotAllowed { requested_headers: String },
    MissingAccessControlRequestMethod,
}

/// Detailed outcome when a preflight request is rejected.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreflightRejection {
    pub headers: Headers,
    pub reason: PreflightRejectionReason,
}

/// Overall decision returned by the policy engine.
#[derive(Debug, Clone)]
pub enum CorsDecision {
    PreflightAccepted { headers: Headers },
    PreflightRejected(PreflightRejection),
    SimpleAccepted { headers: Headers },
    NotApplicable,
}

/// Errors that can be produced during CORS evaluation.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum CorsError {
    #[error(
        "custom origin callback returned OriginDecision::Any while credentials are enabled; this combination is forbidden by the CORS specification"
    )]
    InvalidOriginAnyWithCredentials,
}
