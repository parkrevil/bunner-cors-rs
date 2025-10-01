pub mod header {
    pub const ACCESS_CONTROL_ALLOW_ORIGIN: &str = "Access-Control-Allow-Origin";
    pub const ACCESS_CONTROL_ALLOW_METHODS: &str = "Access-Control-Allow-Methods";
    pub const ACCESS_CONTROL_ALLOW_HEADERS: &str = "Access-Control-Allow-Headers";
    pub const ACCESS_CONTROL_ALLOW_CREDENTIALS: &str = "Access-Control-Allow-Credentials";
    pub const ACCESS_CONTROL_ALLOW_PRIVATE_NETWORK: &str = "Access-Control-Allow-Private-Network";
    pub const ACCESS_CONTROL_EXPOSE_HEADERS: &str = "Access-Control-Expose-Headers";
    pub const ACCESS_CONTROL_MAX_AGE: &str = "Access-Control-Max-Age";
    pub const ACCESS_CONTROL_REQUEST_HEADERS: &str = "Access-Control-Request-Headers";
    pub const ACCESS_CONTROL_REQUEST_METHOD: &str = "Access-Control-Request-Method";
    pub const ACCESS_CONTROL_REQUEST_PRIVATE_NETWORK: &str =
        "Access-Control-Request-Private-Network";
    pub const TIMING_ALLOW_ORIGIN: &str = "Timing-Allow-Origin";
    pub const ORIGIN: &str = "Origin";
    pub const VARY: &str = "Vary";
}

pub mod method {
    pub const DELETE: &str = "DELETE";
    pub const GET: &str = "GET";
    pub const HEAD: &str = "HEAD";
    pub const OPTIONS: &str = "OPTIONS";
    pub const PATCH: &str = "PATCH";
    pub const POST: &str = "POST";
    pub const PUT: &str = "PUT";
}
