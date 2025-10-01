use crate::constants::header;
use std::collections::HashMap;

pub type Headers = std::collections::HashMap<String, String>;

#[derive(Debug, Default, Clone)]
pub(crate) struct HeaderCollection {
    headers: Headers,
}

impl HeaderCollection {
    pub(crate) fn new() -> Self {
        Self::with_estimate(16)
    }

    pub(crate) fn with_estimate(estimate: usize) -> Self {
        let capacity = estimate.max(16);
        Self {
            headers: HashMap::with_capacity(capacity),
        }
    }

    pub(crate) fn push(&mut self, name: String, value: String) {
        if name.eq_ignore_ascii_case(header::VARY) {
            self.add_vary(value);
        } else {
            self.headers.insert(name, value);
        }
    }

    pub(crate) fn add_vary<S: Into<String>>(&mut self, value: S) {
        let mut entries: Vec<String> = self
            .headers
            .get(header::VARY)
            .map(|existing| {
                existing
                    .split(',')
                    .map(|part| part.trim().to_string())
                    .filter(|part| !part.is_empty())
                    .collect()
            })
            .unwrap_or_default();

        let incoming = value.into().trim().to_string();
        if !incoming.is_empty() {
            entries.push(incoming);
        }

        if entries.is_empty() {
            self.headers.remove(header::VARY);
            return;
        }

        let mut deduped: Vec<String> = Vec::with_capacity(entries.len());
        for entry in entries {
            if deduped
                .iter()
                .any(|existing| existing.eq_ignore_ascii_case(&entry))
            {
                continue;
            }
            deduped.push(entry);
        }

        let value = deduped.join(", ");
        self.headers.insert(header::VARY.to_string(), value);
    }

    pub(crate) fn extend(&mut self, other: HeaderCollection) {
        for (name, value) in other.headers {
            if name.eq_ignore_ascii_case(header::VARY) {
                self.add_vary(value);
            } else {
                self.headers.insert(name, value);
            }
        }
    }

    pub(crate) fn into_headers(self) -> Headers {
        self.headers
    }
}

#[cfg(test)]
#[path = "headers_test.rs"]
mod headers_test;
