use crate::allowed_headers::AllowedHeaders;
use crate::constants::method;
use crate::origin::Origin;

#[derive(Clone)]
pub struct CorsOptions {
    pub origin: Origin,
    pub methods: Vec<String>,
    pub allowed_headers: AllowedHeaders,
    pub exposed_headers: Option<Vec<String>>,
    pub credentials: bool,
    pub max_age: Option<String>,
    pub preflight_continue: bool,
    pub options_success_status: u16,
}

impl Default for CorsOptions {
    fn default() -> Self {
        Self {
            origin: Origin::Any,
            methods: vec![
                method::GET.into(),
                method::HEAD.into(),
                method::PUT.into(),
                method::PATCH.into(),
                method::POST.into(),
                method::DELETE.into(),
            ],
            allowed_headers: AllowedHeaders::default(),
            exposed_headers: None,
            credentials: false,
            max_age: None,
            preflight_continue: false,
            options_success_status: 204,
        }
    }
}
