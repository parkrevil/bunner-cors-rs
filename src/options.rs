use crate::allowed_headers::AllowedHeaders;
use crate::allowed_methods::AllowedMethods;
use crate::origin::Origin;
use crate::timing_allow_origin::TimingAllowOrigin;

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
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.credentials && matches!(self.origin, Origin::Any) {
            return Err("credentials_require_specific_origin");
        }

        if let AllowedHeaders::List(values) = &self.allowed_headers
            && values.iter().any(|value| value == "*")
        {
            return Err("allowed_headers_wildcard_not_supported");
        }

        if let Some(values) = &self.exposed_headers
            && values.iter().any(|value| value == "*")
        {
            return Err("expose_headers_wildcard_not_supported");
        }

        Ok(())
    }
}

#[cfg(test)]
#[path = "options_test.rs"]
mod options_test;
