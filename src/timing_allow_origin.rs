use std::collections::HashSet;

/// Represents the `Timing-Allow-Origin` response configuration that enables
/// browsers to expose detailed Resource Timing data.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TimingAllowOrigin {
    Any,
    List(Vec<String>),
}

impl TimingAllowOrigin {
    /// Creates a deduplicated allow-list. Whitespace is trimmed and duplicate
    /// entries (case-insensitive) are removed to produce a stable header value.
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

    /// Serializes the configuration into a value suitable for
    /// `Timing-Allow-Origin`.
    pub fn header_value(&self) -> Option<String> {
        match self {
            Self::Any => Some("*".to_string()),
            Self::List(values) if values.is_empty() => None,
            Self::List(values) => Some(values.join(" ")),
        }
    }
}

#[cfg(test)]
#[path = "timing_allow_origin_test.rs"]
mod timing_allow_origin_test;
