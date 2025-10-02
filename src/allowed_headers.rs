use crate::case::{equals_ignore_case, normalize_lower};
use std::collections::HashSet;

#[derive(Clone, PartialEq, Eq)]
pub enum AllowedHeaders {
    Any,
    List(Vec<String>),
}

impl Default for AllowedHeaders {
    fn default() -> Self {
        AllowedHeaders::List(Vec::new())
    }
}

impl AllowedHeaders {
    pub fn list<I, S>(values: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let mut seen = HashSet::new();
        let mut deduped: Vec<String> = Vec::new();
        for value in values.into_iter() {
            let trimmed = value.into().trim().to_string();
            let key = normalize_lower(&trimmed);
            if seen.insert(key) {
                deduped.push(trimmed);
            }
        }

        Self::List(deduped)
    }

    pub fn any() -> Self {
        Self::Any
    }

    pub fn allows_headers(&self, request_headers: &str) -> bool {
        match self {
            Self::Any => true,
            Self::List(allowed) => {
                let request_headers = request_headers.trim();
                if request_headers.is_empty() {
                    return true;
                }

                request_headers
                    .split(',')
                    .map(|value| value.trim())
                    .filter(|value| !value.is_empty())
                    .all(|header| {
                        allowed
                            .iter()
                            .any(|allowed_header| equals_ignore_case(allowed_header, header))
                    })
            }
        }
    }
}

#[cfg(test)]
#[path = "allowed_headers_test.rs"]
mod allowed_headers_test;
