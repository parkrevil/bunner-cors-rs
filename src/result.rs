use crate::headers::Headers;
use thiserror::Error;

/// Reason a simple (non-preflight) request was rejected.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SimpleRejectionReason {
    OriginNotAllowed,
}

/// Details describing why the request was blocked, including headers that still
/// need to be propagated back to the caller.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SimpleRejection {
    pub headers: Headers,
    pub reason: SimpleRejectionReason,
}

/// Fine-grained status describing why a preflight request failed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PreflightRejectionReason {
    OriginNotAllowed,
    MethodNotAllowed { requested_method: String },
    HeadersNotAllowed { requested_headers: String },
}

/// Wrapper struct that exposes the rejection reason alongside the headers that
/// must be returned to remain spec compliant.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreflightRejection {
    pub headers: Headers,
    pub reason: PreflightRejectionReason,
}

/// Outcome of evaluating a request against the configured CORS policy.
#[derive(Debug, Clone)]
pub enum CorsDecision {
    PreflightAccepted { headers: Headers },
    PreflightRejected(PreflightRejection),
    SimpleAccepted { headers: Headers },
    SimpleRejected(SimpleRejection),
    NotApplicable,
}

/// Errors raised when the CORS engine detects misbehaviour in user-provided callbacks.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum CorsError {
    #[error(
        "custom origin callback returned OriginDecision::Any while credentials are enabled; this combination is forbidden by the CORS specification"
    )]
    InvalidOriginAnyWithCredentials,
}
