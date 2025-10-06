use crate::headers::Headers;
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SimpleRejectionReason {
    OriginNotAllowed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SimpleRejection {
    pub headers: Headers,
    pub reason: SimpleRejectionReason,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PreflightRejectionReason {
    OriginNotAllowed,
    MethodNotAllowed { requested_method: String },
    HeadersNotAllowed { requested_headers: String },
    MissingAccessControlRequestMethod,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreflightRejection {
    pub headers: Headers,
    pub reason: PreflightRejectionReason,
}

#[derive(Debug, Clone)]
pub enum CorsDecision {
    PreflightAccepted { headers: Headers },
    PreflightRejected(PreflightRejection),
    SimpleAccepted { headers: Headers },
    SimpleRejected(SimpleRejection),
    NotApplicable,
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum CorsError {
    #[error(
        "custom origin callback returned OriginDecision::Any while credentials are enabled; this combination is forbidden by the CORS specification"
    )]
    InvalidOriginAnyWithCredentials,
}
