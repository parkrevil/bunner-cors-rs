/// Minimal request metadata required to evaluate CORS rules.
///
/// The struct intentionally mirrors the fields used by the specification so the
/// library can remain framework agnostic. Callers typically populate it from the
/// incoming HTTP request before passing it to [`Cors::check`](crate::Cors::check).
#[derive(Debug, Clone)]
pub struct RequestContext<'a> {
    /// HTTP method of the incoming request.
    pub method: &'a str,
    /// Value of the `Origin` header, if supplied by the client.
    pub origin: Option<&'a str>,
    /// Value of the `Access-Control-Request-Method` header used by CORS preflight.
    pub access_control_request_method: Option<&'a str>,
    /// Value of the `Access-Control-Request-Headers` header used by CORS preflight.
    pub access_control_request_headers: Option<&'a str>,
    /// Indicates that the request is asking for private network access.
    pub access_control_request_private_network: bool,
}
