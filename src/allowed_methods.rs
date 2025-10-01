use crate::constants::method;
use std::collections::HashSet;

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum AllowedMethods {
    List(Vec<String>),
}

impl AllowedMethods {
    pub fn list<I, S>(values: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let mut seen = HashSet::new();
        let mut deduped: Vec<String> = Vec::new();
        for value in values.into_iter() {
            let trimmed = value.into().trim().to_string();
            let key = trimmed.to_ascii_lowercase();
            if seen.insert(key) {
                deduped.push(trimmed);
            }
        }

        Self::List(deduped)
    }

    pub fn header_value(&self) -> Option<String> {
        match self {
            AllowedMethods::List(values) if values.is_empty() => None,
            AllowedMethods::List(values) => Some(values.join(",")),
        }
    }

    pub fn allows_method(&self, method: &str) -> bool {
        let method = method.trim();
        if method.is_empty() {
            return false;
        }

        match self {
            AllowedMethods::List(values) => values
                .iter()
                .any(|allowed| allowed.eq_ignore_ascii_case(method)),
        }
    }
}

impl Default for AllowedMethods {
    fn default() -> Self {
        Self::list([
            method::GET,
            method::HEAD,
            method::PUT,
            method::PATCH,
            method::POST,
            method::DELETE,
        ])
    }
}

#[cfg(test)]
#[path = "allowed_methods_test.rs"]
mod allowed_methods_test;
