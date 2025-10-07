use crate::constants::method;
use crate::util::{equals_ignore_case, normalize_lower};
use std::collections::HashSet;
use std::ops::{Deref, DerefMut};

/// Declarative allow-list for HTTP methods accepted on cross-origin requests.
///
/// Instances of this type are typically created through [`AllowedMethods::list`] and
/// consumed by [`CorsOptions`]. The collection preserves insertion order and performs
/// case-insensitive comparisons when evaluating incoming requests.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct AllowedMethods(Vec<String>);

impl AllowedMethods {
    /// Builds a normalized allow-list from the provided iterator.
    ///
    /// Whitespace is trimmed and duplicate values (ignoring ASCII case) are removed
    /// to ensure the generated header values are stable and spec compliant.
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

        Self(deduped)
    }

    /// Serializes the configured methods into a canonical header string.
    ///
    /// Returns `None` when the list is empty so callers can skip emitting
    /// `Access-Control-Allow-Methods` for default scenarios.
    pub fn header_value(&self) -> Option<String> {
        if self.0.is_empty() {
            None
        } else {
            Some(self.0.join(","))
        }
    }

    /// Returns `true` when the provided method matches an entry in the allow-list.
    ///
    /// The comparison is case-insensitive and treats empty or whitespace-only
    /// methods as invalid.
    pub fn allows_method(&self, method: &str) -> bool {
        let method = method.trim();
        if method.is_empty() {
            return false;
        }

        self.0
            .iter()
            .any(|allowed| equals_ignore_case(allowed, method))
    }

    /// Provides an iterator over the stored methods, preserving insertion order.
    pub fn iter(&self) -> impl Iterator<Item = &String> {
        self.0.iter()
    }

    /// Consumes the structure and returns the owned list of methods.
    pub fn into_inner(self) -> Vec<String> {
        self.0
    }

    /// Returns an immutable slice of the stored methods for ergonomic borrowing.
    pub fn as_slice(&self) -> &[String] {
        &self.0
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

impl From<Vec<String>> for AllowedMethods {
    fn from(values: Vec<String>) -> Self {
        AllowedMethods(values)
    }
}

impl From<AllowedMethods> for Vec<String> {
    fn from(methods: AllowedMethods) -> Self {
        methods.0
    }
}

impl Deref for AllowedMethods {
    type Target = [String];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for AllowedMethods {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<'a> IntoIterator for &'a AllowedMethods {
    type Item = &'a String;
    type IntoIter = std::slice::Iter<'a, String>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl IntoIterator for AllowedMethods {
    type Item = String;
    type IntoIter = std::vec::IntoIter<String>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

#[cfg(test)]
#[path = "allowed_methods_test.rs"]
mod allowed_methods_test;
