#![allow(dead_code)]

use std::collections::HashSet;

use bunner_cors_rs::{CorsDecision, Headers};

use super::headers::{header_value, vary_values};

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

pub fn assert_header_eq(headers: &Headers, name: &str, expected: &str) {
    match header_value(headers, name) {
        Some(value) => assert_eq!(
            value, expected,
            "expected {name} to be {expected}, got {value}"
        ),
        None => panic!("expected header {name} to equal {expected}, but header was missing"),
    }
}

pub fn assert_vary_eq<I, S>(headers: &Headers, expected: I)
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    let actual = vary_values(headers);
    let expected_set: HashSet<String> = expected
        .into_iter()
        .map(|value| value.as_ref().to_string())
        .collect();

    assert_eq!(
        actual, expected_set,
        "expected Vary {:?}, got {:?}",
        expected_set, actual
    );
}

pub fn assert_vary_contains(headers: &Headers, name: &str) {
    let values = vary_values(headers);
    assert!(
        values.contains(name),
        "expected Vary to contain {name}, actual values were {:?}",
        values
    );
}

pub fn assert_vary_not_contains(headers: &Headers, name: &str) {
    let values = vary_values(headers);
    assert!(
        !values.contains(name),
        "expected Vary to not contain {name}, actual values were {:?}",
        values
    );
}

pub fn assert_vary_is_empty(headers: &Headers) {
    let values = vary_values(headers);
    assert!(
        values.is_empty(),
        "expected Vary to be empty, actual values were {:?}",
        values
    );
}
