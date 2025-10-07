use crate::allowed_headers::AllowedHeaders;
use crate::allowed_methods::AllowedMethods;
use crate::exposed_headers::ExposedHeaders;
use crate::origin::Origin;
use crate::timing_allow_origin::TimingAllowOrigin;
use crate::util::is_http_token;
use std::error::Error;
use std::fmt::{self, Display};

/// Enumerates misconfigurations that prevent a [`CorsOptions`] instance from being
/// used safely.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidationError {
    /// Credentials can only be enabled when a specific origin is configured.
    CredentialsRequireSpecificOrigin,
    /// Wildcard request headers are forbidden when credentials are enabled.
    AllowedHeadersAnyNotAllowedWithCredentials,
    /// `*` is not allowed inside explicit header lists.
    AllowedHeadersListCannotContainWildcard,
    /// Header allow-lists may only include valid HTTP tokens.
    AllowedHeadersListContainsInvalidToken,
    /// Exposing all headers requires credentials to be disabled.
    ExposeHeadersWildcardRequiresCredentialsDisabled,
    /// `*` cannot be combined with other exposed header values.
    ExposeHeadersWildcardCannotBeCombined,
    /// Exposed header values must be valid HTTP tokens.
    ExposeHeadersListContainsInvalidToken,
    /// Private network access requires credentials to be enabled.
    PrivateNetworkRequiresCredentials,
    /// Private network access requires a specific origin, not `*`.
    PrivateNetworkRequiresSpecificOrigin,
    /// Allowed methods lists cannot contain empty values.
    AllowedMethodsCannotContainEmptyToken,
    /// `*` is not allowed inside explicit method lists.
    AllowedMethodsCannotContainWildcard,
    /// Allowed methods must be valid HTTP tokens.
    AllowedMethodsListContainsInvalidToken,
    /// Allowed headers lists cannot contain empty values.
    AllowedHeadersCannotContainEmptyToken,
    /// Exposed headers lists cannot contain empty values.
    ExposeHeadersCannotContainEmptyValue,
    /// Timing-Allow-Origin cannot be wildcarded when credentials are enabled.
    TimingAllowOriginWildcardNotAllowedWithCredentials,
    /// Timing-Allow-Origin lists cannot contain empty values.
    TimingAllowOriginCannotContainEmptyValue,
}

impl Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ValidationError::CredentialsRequireSpecificOrigin => f.write_str(
                "When credentials are enabled, you must configure a specific allowed origin instead of \"*\".",
            ),
            ValidationError::AllowedHeadersAnyNotAllowedWithCredentials => f.write_str(
                "AllowedHeaders::Any cannot be used when credentials are enabled. Configure an explicit header allow list instead.",
            ),
            ValidationError::AllowedHeadersListCannotContainWildcard => f.write_str(
                "Allowed headers lists cannot include \"*\". Use AllowedHeaders::Any to allow all headers.",
            ),
            ValidationError::AllowedHeadersListContainsInvalidToken => f.write_str(
                "Allowed headers lists may only contain valid HTTP header field names.",
            ),
            ValidationError::ExposeHeadersWildcardRequiresCredentialsDisabled => f
                .write_str(
                    "Exposed headers wildcard (\"*\") can only be used when credentials are disabled.",
                ),
            ValidationError::ExposeHeadersWildcardCannotBeCombined => f.write_str(
                "The exposed headers wildcard (\"*\") cannot be combined with additional header names.",
            ),
            ValidationError::ExposeHeadersListContainsInvalidToken => f.write_str(
                "Exposed headers lists may only contain valid HTTP header field names.",
            ),
            ValidationError::PrivateNetworkRequiresCredentials => f.write_str(
                "Allowing private network access requires enabling credentials so Access-Control-Allow-Credentials can be set to true.",
            ),
            ValidationError::PrivateNetworkRequiresSpecificOrigin => f.write_str(
                "Allowing private network access requires configuring a specific allowed origin instead of \"*\".",
            ),

            ValidationError::AllowedMethodsCannotContainEmptyToken => f.write_str(
                "Allowed methods lists cannot contain empty or whitespace-only entries.",
            ),
            ValidationError::AllowedMethodsCannotContainWildcard => f.write_str(
                "Allowed methods lists cannot include the wildcard (\"*\").",
            ),
            ValidationError::AllowedMethodsListContainsInvalidToken => f.write_str(
                "Allowed methods lists may only contain valid HTTP method tokens.",
            ),
            ValidationError::AllowedHeadersCannotContainEmptyToken => f.write_str(
                "Allowed headers lists cannot contain empty or whitespace-only entries.",
            ),
            ValidationError::ExposeHeadersCannotContainEmptyValue => f.write_str(
                "Exposed headers cannot contain empty or whitespace-only entries.",
            ),
            ValidationError::TimingAllowOriginWildcardNotAllowedWithCredentials => f
                .write_str(
                    "Timing-Allow-Origin cannot be a wildcard when credentials are enabled.",
                ),
            ValidationError::TimingAllowOriginCannotContainEmptyValue => f.write_str(
                "Timing-Allow-Origin lists cannot contain empty or whitespace-only entries.",
            ),
        }
    }
}

impl Error for ValidationError {}

/// Configuration entry point for the CORS engine.
///
/// The struct is intentionally builder-friendly: individual setters consume and
/// return `Self` to enable fluent configuration chains. Use [`CorsOptions::validate`]
/// or [`Cors::new`](crate::Cors::new) to ensure the configuration is internally
/// consistent before responding to requests.
#[derive(Clone)]
pub struct CorsOptions {
    /// Defines which origins may access the resource.
    pub origin: Origin,
    /// Declares which HTTP methods are allowed for cross-origin requests.
    pub methods: AllowedMethods,
    /// Controls which request headers are allowed during preflight.
    pub allowed_headers: AllowedHeaders,
    /// Specifies which response headers should be exposed to the browser.
    pub exposed_headers: ExposedHeaders,
    /// Enables `Access-Control-Allow-Credentials` when set.
    pub credentials: bool,
    /// When present, sets the `Access-Control-Max-Age` header in seconds.
    pub max_age: Option<u64>,
    /// Allows treating the literal `Origin: null` as an allowed origin.
    pub allow_null_origin: bool,
    /// Enables `Access-Control-Allow-Private-Network` during preflight.
    pub allow_private_network: bool,
    /// Configures the `Timing-Allow-Origin` header.
    pub timing_allow_origin: Option<TimingAllowOrigin>,
}

impl Default for CorsOptions {
    fn default() -> Self {
        Self {
            origin: Origin::Any,
            methods: AllowedMethods::default(),
            allowed_headers: AllowedHeaders::default(),
            exposed_headers: ExposedHeaders::default(),
            credentials: false,
            max_age: None,
            allow_null_origin: false,
            allow_private_network: false,
            timing_allow_origin: None,
        }
    }
}

impl CorsOptions {
    /// Returns the default configuration, equivalent to [`Default::default`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the allowed origin policy.
    pub fn origin(mut self, origin: Origin) -> Self {
        self.origin = origin;
        self
    }

    /// Replaces the allowed methods list.
    pub fn methods(mut self, methods: AllowedMethods) -> Self {
        self.methods = methods;
        self
    }

    /// Replaces the allowed headers configuration.
    pub fn allowed_headers(mut self, allowed_headers: AllowedHeaders) -> Self {
        self.allowed_headers = allowed_headers;
        self
    }

    /// Replaces the exposed headers configuration.
    pub fn exposed_headers(mut self, exposed_headers: ExposedHeaders) -> Self {
        self.exposed_headers = exposed_headers;
        self
    }

    /// Enables or disables credential support.
    pub fn credentials(mut self, enabled: bool) -> Self {
        self.credentials = enabled;
        self
    }

    /// Sets the `Access-Control-Max-Age` header to the provided number of seconds.
    pub fn max_age(mut self, value: u64) -> Self {
        self.max_age = Some(value);
        self
    }

    /// Grants or revokes support for `Origin: null` requests.
    pub fn allow_null_origin(mut self, enabled: bool) -> Self {
        self.allow_null_origin = enabled;
        self
    }

    /// Enables or disables private network preflight support.
    pub fn allow_private_network(mut self, enabled: bool) -> Self {
        self.allow_private_network = enabled;
        self
    }

    /// Replaces the `Timing-Allow-Origin` configuration.
    pub fn timing_allow_origin(mut self, value: TimingAllowOrigin) -> Self {
        self.timing_allow_origin = Some(value);
        self
    }

    /// Ensures the configuration adheres to the CORS specification.
    ///
    /// The validation focuses on combinations that would otherwise produce
    /// undefined or non-spec-compliant behaviour, enabling library users to catch
    /// mistakes during initialization rather than at runtime.
    pub fn validate(&self) -> Result<(), ValidationError> {
        if self.credentials && matches!(self.origin, Origin::Any) {
            if self.allow_private_network {
                return Err(ValidationError::PrivateNetworkRequiresSpecificOrigin);
            }
            return Err(ValidationError::CredentialsRequireSpecificOrigin);
        }

        if self.credentials && matches!(self.allowed_headers, AllowedHeaders::Any) {
            return Err(ValidationError::AllowedHeadersAnyNotAllowedWithCredentials);
        }

        if let AllowedHeaders::List(values) = &self.allowed_headers
            && values.iter().any(|value| value == "*")
        {
            return Err(ValidationError::AllowedHeadersListCannotContainWildcard);
        }

        if self.methods.iter().any(|value| value.trim().is_empty()) {
            return Err(ValidationError::AllowedMethodsCannotContainEmptyToken);
        }
        if self.methods.iter().any(|value| value.trim() == "*") {
            return Err(ValidationError::AllowedMethodsCannotContainWildcard);
        }
        if self
            .methods
            .iter()
            .map(|value| value.trim())
            .any(|value| !is_http_token(value))
        {
            return Err(ValidationError::AllowedMethodsListContainsInvalidToken);
        }

        if let AllowedHeaders::List(values) = &self.allowed_headers
            && values.iter().any(|value| value.trim().is_empty())
        {
            return Err(ValidationError::AllowedHeadersCannotContainEmptyToken);
        }

        if let AllowedHeaders::List(values) = &self.allowed_headers
            && values
                .iter()
                .map(|value| value.trim())
                .any(|value| !is_http_token(value))
        {
            return Err(ValidationError::AllowedHeadersListContainsInvalidToken);
        }

        match &self.exposed_headers {
            ExposedHeaders::Any => {
                if self.credentials {
                    return Err(ValidationError::ExposeHeadersWildcardRequiresCredentialsDisabled);
                }
            }
            ExposedHeaders::List(values) => {
                if values.values().iter().any(|value| value.trim().is_empty()) {
                    return Err(ValidationError::ExposeHeadersCannotContainEmptyValue);
                }

                if values
                    .values()
                    .iter()
                    .map(|value| value.trim())
                    .any(|value| !is_http_token(value))
                {
                    return Err(ValidationError::ExposeHeadersListContainsInvalidToken);
                }

                if values.values().iter().any(|value| value.trim() == "*") {
                    return Err(ValidationError::ExposeHeadersWildcardCannotBeCombined);
                }
            }
        }

        if self.allow_private_network && !self.credentials {
            return Err(ValidationError::PrivateNetworkRequiresCredentials);
        }

        if self.credentials && matches!(self.timing_allow_origin, Some(TimingAllowOrigin::Any)) {
            return Err(ValidationError::TimingAllowOriginWildcardNotAllowedWithCredentials);
        }

        if let Some(TimingAllowOrigin::List(values)) = &self.timing_allow_origin
            && values.iter().any(|value| value.trim().is_empty())
        {
            return Err(ValidationError::TimingAllowOriginCannotContainEmptyValue);
        }

        Ok(())
    }
}

#[cfg(test)]
#[path = "options_test.rs"]
mod options_test;
