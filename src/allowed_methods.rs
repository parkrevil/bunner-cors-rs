use crate::constants::method;

/// Configuration for the `Access-Control-Allow-Methods` response header.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum AllowedMethods {
    /// Emit a comma-separated list of methods. Case-sensitive to preserve caller intent.
    List(Vec<String>),
}

impl AllowedMethods {
    /// Construct an explicit list of allowed methods.
    pub fn list<I, S>(values: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let mut deduped: Vec<String> = Vec::new();
        for value in values.into_iter() {
            let trimmed = value.into().trim().to_string();
            if deduped
                .iter()
                .any(|existing| existing.eq_ignore_ascii_case(trimmed.as_str()))
            {
                continue;
            }
            deduped.push(trimmed);
        }

        Self::List(deduped)
    }

    /// Return the header value representation, if any.
    pub fn header_value(&self) -> Option<String> {
        match self {
            AllowedMethods::List(values) if values.is_empty() => None,
            AllowedMethods::List(values) => Some(values.join(",")),
        }
    }

    /// Determine whether the provided method is allowed for a preflight request.
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
