#[derive(Debug, Clone)]
pub struct RequestContext<'a> {
    pub method: &'a str,
    pub origin: &'a str,
    pub access_control_request_method: Option<&'a str>,
    pub access_control_request_headers: Option<&'a str>,
    pub access_control_request_private_network: bool,
}
