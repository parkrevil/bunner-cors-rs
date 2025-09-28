/// Minimal request context passed to the policy engine.
#[derive(Debug, Clone)]
pub struct RequestContext<'a> {
    pub method: &'a str,
    pub origin: &'a str,
    pub access_control_request_method: &'a str,
    pub access_control_request_headers: &'a str,
}
