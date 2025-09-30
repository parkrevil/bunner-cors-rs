use std::collections::HashSet;

/// Configuration for the `Access-Control-Allow-Headers` response value.
#[derive(Clone, PartialEq, Eq)]
pub enum AllowedHeaders {
    List(Vec<String>),
    /// Wildcard: always allowed and emits "*" on preflight
    Any,
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
            let key = trimmed.to_ascii_lowercase();
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
                            .any(|allowed_header| allowed_header.eq_ignore_ascii_case(header))
                    })
            }
        }
    }
}

#[cfg(test)]
#[path = "allowed_headers_test.rs"]
mod allowed_headers_test;
