use crate::util::normalize_lower;
use std::collections::HashSet;
use std::ops::Deref;

/// Configuration mirror of the `Access-Control-Expose-Headers` response header.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ExposedHeaders {
    List(ExposedHeaderList),
    Any,
}

impl Default for ExposedHeaders {
    fn default() -> Self {
        Self::List(ExposedHeaderList::default())
    }
}

impl ExposedHeaders {
    /// Builds an allow-list from the provided iterator, automatically trimming
    /// whitespace and removing duplicates.
    pub fn list<I, S>(values: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let mut seen = HashSet::new();
        let mut deduped: Vec<String> = Vec::new();

        for value in values.into_iter() {
            let trimmed = value.into().trim().to_string();
            let key = if trimmed.is_empty() {
                "".to_string()
            } else {
                normalize_lower(&trimmed)
            };

            if seen.insert(key) {
                deduped.push(trimmed);
            }
        }

        if deduped.len() == 1 && deduped[0] == "*" {
            return Self::Any;
        }

        Self::List(ExposedHeaderList::new(deduped))
    }

    /// Serializes the configuration into a header-ready value.
    pub fn header_value(&self) -> Option<String> {
        match self {
            Self::List(values) if values.is_empty() => None,
            Self::List(values) => Some(values.join(",")),
            Self::Any => Some("*".to_string()),
        }
    }

    /// Returns an iterator over the explicitly configured header names.
    ///
    /// When configured as [`Self::Any`], the iterator is empty because "*" is
    /// represented via the header value rather than as an explicit element.
    pub fn iter(&self) -> ExposedHeadersIter<'_> {
        match self {
            Self::List(values) => ExposedHeadersIter::List(values.values.iter()),
            Self::Any => ExposedHeadersIter::Empty,
        }
    }
}

/// Iterator type returned by [`ExposedHeaders::iter`].
pub enum ExposedHeadersIter<'a> {
    Empty,
    List(std::slice::Iter<'a, String>),
}

impl<'a> Iterator for ExposedHeadersIter<'a> {
    type Item = &'a String;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            ExposedHeadersIter::Empty => None,
            ExposedHeadersIter::List(iter) => iter.next(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct ExposedHeaderList {
    values: Vec<String>,
}

impl ExposedHeaderList {
    fn new(values: Vec<String>) -> Self {
        Self { values }
    }

    pub fn values(&self) -> &[String] {
        &self.values
    }

    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }
}

impl Deref for ExposedHeaderList {
    type Target = [String];

    fn deref(&self) -> &Self::Target {
        &self.values
    }
}

#[cfg(test)]
#[path = "exposed_headers_test.rs"]
mod exposed_headers_test;
