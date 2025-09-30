use crate::allowed_headers::AllowedHeaders;
use crate::allowed_methods::AllowedMethods;
use crate::constants::header;
use crate::origin::Origin;
use crate::timing_allow_origin::TimingAllowOrigin;
use std::error::Error;
use std::fmt::{self, Display};

fn is_http_token(value: &str) -> bool {
    !value.is_empty()
        && value.bytes().all(|byte| {
            matches!(
                byte,
                b'0'..=b'9'
                    | b'A'..=b'Z'
                    | b'a'..=b'z'
                    | b'!'
                    | b'#'
                    | b'$'
                    | b'%'
                    | b'&'
                    | b'\''
                    | b'*'
                    | b'+'
                    | b'-'
                    | b'.'
                    | b'^'
                    | b'_'
                    | b'`'
                    | b'|'
                    | b'~'
            )
        })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidationError {
    CredentialsRequireSpecificOrigin,
    AllowedHeadersListCannotContainWildcard,
    AllowedHeadersListContainsInvalidToken,
    ExposeHeadersWildcardRequiresCredentialsDisabled,
    ExposeHeadersWildcardCannotBeCombined,
    ExposeHeadersListContainsInvalidToken,
    InvalidSuccessStatus(u16),
    InvalidMaxAge(String),
    PrivateNetworkRequiresCredentials,
    PrivateNetworkRequiresSpecificOrigin,
    PrivateNetworkRequestHeaderNotAllowed,
    AllowedMethodsCannotContainEmptyToken,
    AllowedMethodsCannotContainWildcard,
    AllowedMethodsListContainsInvalidToken,
    AllowedHeadersCannotContainEmptyToken,
    ExposeHeadersCannotContainEmptyValue,
    TimingAllowOriginWildcardNotAllowedWithCredentials,
    TimingAllowOriginCannotContainEmptyValue,
}

impl Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ValidationError::CredentialsRequireSpecificOrigin => f.write_str(
                "When credentials are enabled, you must configure a specific allowed origin instead of \"*\".",
            ),
            ValidationError::AllowedHeadersListCannotContainWildcard => f.write_str(
                "Allowed headers lists cannot include \"*\". Use AllowedHeaders::any() to allow all headers.",
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
            ValidationError::InvalidSuccessStatus(status) => write!(
                f,
                "The options success status of {status} is outside the 200-299 range required by the CORS spec."
            ),
            ValidationError::InvalidMaxAge(value) => write!(
                f,
                "The max-age value '{value}' must be a non-negative integer representing seconds."
            ),
            ValidationError::PrivateNetworkRequiresCredentials => f.write_str(
                "Allowing private network access requires enabling credentials so Access-Control-Allow-Credentials can be set to true.",
            ),
            ValidationError::PrivateNetworkRequiresSpecificOrigin => f.write_str(
                "Allowing private network access requires configuring a specific allowed origin instead of \"*\".",
            ),
            ValidationError::PrivateNetworkRequestHeaderNotAllowed => f.write_str(
                "Allowing private network access requires permitting the Access-Control-Request-Private-Network header.",
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

#[derive(Clone)]
pub struct CorsOptions {
    pub origin: Origin,
    pub methods: AllowedMethods,
    pub allowed_headers: AllowedHeaders,
    pub exposed_headers: Option<Vec<String>>,
    pub credentials: bool,
    pub max_age: Option<String>,
    pub preflight_continue: bool,
    pub options_success_status: u16,
    pub allow_private_network: bool,
    pub timing_allow_origin: Option<TimingAllowOrigin>,
}

impl Default for CorsOptions {
    fn default() -> Self {
        Self {
            origin: Origin::Any,
            methods: AllowedMethods::default(),
            allowed_headers: AllowedHeaders::default(),
            exposed_headers: None,
            credentials: false,
            max_age: None,
            preflight_continue: false,
            options_success_status: 204,
            allow_private_network: false,
            timing_allow_origin: None,
        }
    }
}

impl CorsOptions {
    pub fn validate(&self) -> Result<(), ValidationError> {
        if self.credentials && matches!(self.origin, Origin::Any) {
            return Err(ValidationError::CredentialsRequireSpecificOrigin);
        }

        if let AllowedHeaders::List(values) = &self.allowed_headers
            && values.iter().any(|value| value == "*")
        {
            return Err(ValidationError::AllowedHeadersListCannotContainWildcard);
        }

        match &self.methods {
            AllowedMethods::List(values) => {
                if values.iter().any(|value| value.trim().is_empty()) {
                    return Err(ValidationError::AllowedMethodsCannotContainEmptyToken);
                }
                if values.iter().any(|value| value.trim() == "*") {
                    return Err(ValidationError::AllowedMethodsCannotContainWildcard);
                }
                if values
                    .iter()
                    .map(|value| value.trim())
                    .any(|value| !is_http_token(value))
                {
                    return Err(ValidationError::AllowedMethodsListContainsInvalidToken);
                }
            }
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

        if let Some(values) = &self.exposed_headers {
            if values.iter().any(|value| value.trim().is_empty()) {
                return Err(ValidationError::ExposeHeadersCannotContainEmptyValue);
            }

            if values.iter().any(|value| value.trim() == "*") {
                if self.credentials {
                    return Err(ValidationError::ExposeHeadersWildcardRequiresCredentialsDisabled);
                }

                if values.len() > 1 {
                    return Err(ValidationError::ExposeHeadersWildcardCannotBeCombined);
                }
            }

            if values
                .iter()
                .map(|value| value.trim())
                .any(|value| !is_http_token(value))
            {
                return Err(ValidationError::ExposeHeadersListContainsInvalidToken);
            }
        }

        if !(200..=299).contains(&self.options_success_status) {
            return Err(ValidationError::InvalidSuccessStatus(
                self.options_success_status,
            ));
        }

        if let Some(value) = &self.max_age {
            let trimmed = value.trim();
            if trimmed.is_empty() || trimmed.parse::<u64>().is_err() {
                return Err(ValidationError::InvalidMaxAge(value.clone()));
            }
        }

        if self.allow_private_network {
            if !self.credentials {
                return Err(ValidationError::PrivateNetworkRequiresCredentials);
            }
            if matches!(self.origin, Origin::Any) {
                return Err(ValidationError::PrivateNetworkRequiresSpecificOrigin);
            }
            if let AllowedHeaders::List(values) = &self.allowed_headers {
                let header_allowed = values.iter().any(|value| {
                    value.eq_ignore_ascii_case(header::ACCESS_CONTROL_REQUEST_PRIVATE_NETWORK)
                });
                if !header_allowed {
                    return Err(ValidationError::PrivateNetworkRequestHeaderNotAllowed);
                }
            }
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
