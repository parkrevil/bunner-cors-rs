/// Minimal request context passed to the policy engine.
#[derive(Debug, Clone)]
pub struct RequestContext<'a> {
    pub method: &'a str,
    pub origin: Option<&'a str>,
    pub access_control_request_method: Option<&'a str>,
    pub access_control_request_headers: Option<&'a str>,
}

impl<'a> RequestContext<'a> {
    pub fn new(method: &'a str) -> Self {
        Self {
            method,
            origin: None,
            access_control_request_method: None,
            access_control_request_headers: None,
        }
    }

    pub fn with_origin(mut self, origin: Option<&'a str>) -> Self {
        self.origin = origin;
        self
    }

    pub fn with_access_control_request_method(mut self, value: Option<&'a str>) -> Self {
        self.access_control_request_method = value;
        self
    }

    pub fn with_access_control_request_headers(mut self, value: Option<&'a str>) -> Self {
        self.access_control_request_headers = value;
        self
    }
}
