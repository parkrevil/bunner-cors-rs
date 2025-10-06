mod cors;
mod routes;

use actix_web::{App, HttpServer, web};
use cors::middleware::BunnerCors;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let app_state = cors::build_state().expect("valid CORS configuration");

    HttpServer::new(move || {
        let state = app_state.clone();
        App::new()
            .app_data(web::Data::new(state.clone()))
            .wrap(BunnerCors::new(state.cors.clone()))
            .route("/greet", web::get().to(routes::greet))
    })
    .bind(("127.0.0.1", 5002))?
    .run()
    .await
}
