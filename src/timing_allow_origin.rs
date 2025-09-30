use std::collections::HashSet;

/// Configuration for the `Timing-Allow-Origin` response header.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TimingAllowOrigin {
    /// Emit the wildcard `*`, allowing timing information to be shared with any origin.
    Any,
    /// Emit a space-separated list of origins that may receive timing information.
    List(Vec<String>),
}

impl TimingAllowOrigin {
    /// Construct a variant representing `Timing-Allow-Origin: *`.
    pub fn any() -> Self {
        Self::Any
    }

    /// Construct a variant representing an explicit list of origins.
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

    /// Return the header value representation.
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
