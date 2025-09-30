use crate::allowed_headers::AllowedHeaders;
use crate::allowed_methods::AllowedMethods;
use crate::origin::Origin;
use crate::timing_allow_origin::TimingAllowOrigin;
use std::error::Error;
use std::fmt::{self, Display};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidationError {
    CredentialsRequireSpecificOrigin,
    AllowedHeadersListCannotContainWildcard,
    ExposeHeadersListCannotContainWildcard,
    InvalidSuccessStatus(u16),
    InvalidMaxAge(String),
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
            ValidationError::ExposeHeadersListCannotContainWildcard => f.write_str(
                "Exposed headers lists cannot include \"*\". Use None to omit the header or list specific names.",
            ),
            ValidationError::InvalidSuccessStatus(status) => write!(
                f,
                "The options success status of {status} is outside the 200-299 range required by the CORS spec."
            ),
            ValidationError::InvalidMaxAge(value) => write!(
                f,
                "The max-age value '{value}' must be a non-negative integer representing seconds."
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

        if let Some(values) = &self.exposed_headers
            && values.iter().any(|value| value == "*")
        {
            return Err(ValidationError::ExposeHeadersListCannotContainWildcard);
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

        Ok(())
    }
}

#[cfg(test)]
#[path = "options_test.rs"]
mod options_test;
