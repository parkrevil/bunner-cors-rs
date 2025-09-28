use crate::constants::method;

/// Configuration for the `Access-Control-Allow-Methods` response header.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum AllowedMethods {
    /// Emit the wildcard `*` to allow any method, mirroring express-cors behaviour.
    Any,
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
        Self::List(values.into_iter().map(Into::into).collect())
    }

    /// Construct the wildcard variant (`*`).
    pub fn any() -> Self {
        Self::Any
    }

    /// Return the header value representation, if any.
    pub fn header_value(&self) -> Option<String> {
        match self {
            AllowedMethods::Any => Some("*".to_string()),
            AllowedMethods::List(values) if values.is_empty() => None,
            AllowedMethods::List(values) => Some(values.join(",")),
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
