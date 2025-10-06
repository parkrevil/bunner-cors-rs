mod cors;
mod routes;

use std::net::SocketAddr;

use axum::{Router, routing::get};
use cors::middleware::cors_middleware;

#[tokio::main]
async fn main() {
    let app_state = cors::build_state().expect("valid CORS configuration");

    let app = Router::new()
        .route("/greet", get(routes::greet))
        .route_layer(axum::middleware::from_fn_with_state(
            app_state.clone(),
            cors_middleware,
        ))
        .with_state(app_state);

    let addr: SocketAddr = "127.0.0.1:5001".parse().unwrap();
    println!("Axum example running on http://{addr}");

    axum::serve(tokio::net::TcpListener::bind(addr).await.unwrap(), app)
        .await
        .unwrap();
}
