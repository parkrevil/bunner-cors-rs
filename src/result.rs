use crate::headers::Headers;
use thiserror::Error;

/// Headers and response metadata emitted for either a preflight or simple request.
#[derive(Debug, Clone)]
pub struct CorsResult {
    pub headers: Headers,
    pub status: Option<u16>,
    pub end_response: bool,
}

/// Overall decision returned by the policy engine.
#[derive(Debug, Clone)]
pub enum CorsDecision {
    Preflight(CorsResult),
    Simple(CorsResult),
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
