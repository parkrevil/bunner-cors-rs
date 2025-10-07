use std::sync::Arc;

use bunner_cors_rs::{
    AllowedHeaders, AllowedMethods, Cors, CorsOptions, ExposedHeaders, Origin, ValidationError,
};

pub type SharedCors = Arc<Cors>;
pub type SharedAppState = Arc<AppState>;

#[derive(Clone)]
pub struct AppState {
    pub cors: SharedCors,
    pub greeting: &'static str,
}

pub fn build_state() -> Result<SharedAppState, ValidationError> {
    let options = CorsOptions::new()
        .origin(Origin::list(["http://api.example.com"]))
        .methods(AllowedMethods::list(["GET", "POST", "OPTIONS"]))
        .allowed_headers(AllowedHeaders::list([
            "Content-Type",
            "X-Requested-With",
            "X-Example-Trace",
        ]))
        .exposed_headers(ExposedHeaders::list(["X-Example-Trace"]))
        .credentials(true)
        .max_age(600);

    let cors = Arc::new(Cors::new(options)?);

    Ok(Arc::new(AppState {
        cors,
        greeting: "Welcome to the Hyper CORS example!",
    }))
}

pub mod middleware;
