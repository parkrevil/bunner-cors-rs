use crate::util::normalize_lower;
use std::collections::HashSet;
use std::ops::Deref;

#[derive(Clone, PartialEq, Eq)]
pub enum AllowedHeaders {
    Any,
    List(AllowedHeaderList),
}

impl Default for AllowedHeaders {
    fn default() -> Self {
        AllowedHeaders::List(AllowedHeaderList::default())
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

        Self::List(AllowedHeaderList::new(deduped, seen))
    }

    pub fn any() -> Self {
        Self::Any
    }

    pub fn allows_headers(&self, request_headers: &str) -> bool {
        match self {
            Self::Any => true,
            Self::List(allowed) => allowed.allows_headers(request_headers),
        }
    }
}

#[derive(Clone, PartialEq, Eq, Default)]
pub struct AllowedHeaderList {
    values: Vec<String>,
    normalized: HashSet<String>,
}

impl AllowedHeaderList {
    fn new(values: Vec<String>, normalized: HashSet<String>) -> Self {
        Self { values, normalized }
    }

    pub fn values(&self) -> &[String] {
        &self.values
    }

    fn allows_headers(&self, request_headers: &str) -> bool {
        let request_headers = request_headers.trim();
        if request_headers.is_empty() {
            return true;
        }

        request_headers
            .split(',')
            .map(|value| value.trim())
            .filter(|value| !value.is_empty())
            .all(|header| {
                let normalized = normalize_lower(header);
                self.normalized.contains(normalized.as_str())
            })
    }
}

impl Deref for AllowedHeaderList {
    type Target = [String];

    fn deref(&self) -> &Self::Target {
        &self.values
    }
}

#[cfg(test)]
#[path = "allowed_headers_test.rs"]
mod allowed_headers_test;
