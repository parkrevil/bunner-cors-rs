use bunner_cors_rs::{CorsDecision, Header};

pub fn assert_simple(decision: CorsDecision) -> Vec<Header> {
    match decision {
        CorsDecision::Simple(result) => result.headers,
        other => panic!("expected simple decision, got {:?}", other),
    }
}

pub fn assert_preflight(decision: CorsDecision) -> (Vec<Header>, u16, bool) {
    match decision {
        CorsDecision::Preflight(result) => (result.headers, result.status, result.halt_response),
        other => panic!("expected preflight decision, got {:?}", other),
    }
}
