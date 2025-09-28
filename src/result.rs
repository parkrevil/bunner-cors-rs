use crate::headers::Headers;

/// Result for a preflight evaluation.
#[derive(Debug, Clone)]
pub struct PreflightResult {
    pub headers: Headers,
    pub status: u16,
    pub halt_response: bool,
}

/// Result for a simple request evaluation.
#[derive(Debug, Clone)]
pub struct SimpleResult {
    pub headers: Headers,
}

/// Overall decision returned by the policy engine.
#[derive(Debug, Clone)]
pub enum CorsDecision {
    Preflight(PreflightResult),
    Simple(SimpleResult),
    NotApplicable,
}
