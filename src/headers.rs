use crate::constants::header;
use std::collections::BTreeSet;

/// Simple response header representation used by the CORS engine.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Header {
    pub name: String,
    pub value: String,
}

impl Header {
    pub fn new<N: Into<String>, V: Into<String>>(name: N, value: V) -> Self {
        Self {
            name: name.into(),
            value: value.into(),
        }
    }
}

#[derive(Default)]
pub(crate) struct HeaderCollection {
    headers: Vec<Header>,
    vary_values: BTreeSet<String>,
}

impl HeaderCollection {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    pub(crate) fn push(&mut self, header: Header) {
        if header.name.eq_ignore_ascii_case(header::VARY) {
            self.add_vary(header.value);
        } else {
            self.headers.push(header);
        }
    }

    pub(crate) fn add_vary<S: Into<String>>(&mut self, value: S) {
        for part in value.into().split(',') {
            let trimmed = part.trim();
            if !trimmed.is_empty() {
                self.vary_values.insert(trimmed.to_string());
            }
        }
    }

    pub(crate) fn extend(&mut self, mut other: HeaderCollection) {
        for header in other.headers.drain(..) {
            self.push(header);
        }
        for value in other.vary_values {
            self.vary_values.insert(value);
        }
    }

    pub(crate) fn into_headers(mut self) -> Vec<Header> {
        if !self.vary_values.is_empty() {
            let value = self.vary_values.into_iter().collect::<Vec<_>>().join(", ");
            self.headers.push(Header::new(header::VARY, value));
        }
        self.headers
    }
}
