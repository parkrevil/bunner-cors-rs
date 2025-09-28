use bunner_cors_rs::{CorsDecision, Headers};

pub fn assert_simple(decision: CorsDecision) -> Headers {
    match decision {
        CorsDecision::Simple(result) => result.headers,
        other => panic!("expected simple decision, got {:?}", other),
    }
}

pub fn assert_preflight(decision: CorsDecision) -> (Headers, u16, bool) {
    match decision {
        CorsDecision::Preflight(result) => (result.headers, result.status, result.end_response),
        other => panic!("expected preflight decision, got {:?}", other),
    }
}
