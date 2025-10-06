use std::sync::Arc;

use bunner_cors_rs::{
    AllowedHeaders, AllowedMethods, Cors, CorsOptions, ExposedHeaders, Origin, ValidationError,
};

pub type SharedCors = Arc<Cors>;

#[derive(Clone)]
pub struct AppState {
    pub cors: SharedCors,
    pub greeting: &'static str,
}

pub fn build_state() -> Result<AppState, ValidationError> {
    let options = CorsOptions {
        origin: Origin::list(["http://api.example.com"]),
        methods: AllowedMethods::list(["GET", "POST", "OPTIONS"]),
        allowed_headers: AllowedHeaders::list([
            "Content-Type",
            "X-Requested-With",
            "X-Example-Trace",
        ]),
        exposed_headers: ExposedHeaders::list(["X-Example-Trace"]),
        credentials: true,
        max_age: Some(600),
        allow_null_origin: false,
        allow_private_network: false,
        timing_allow_origin: None,
    };

    let cors = Arc::new(Cors::new(options)?);

    Ok(AppState {
        cors,
        greeting: "Welcome to the Axum CORS example!",
    })
}

pub mod middleware;
