use actix_web::{HttpResponse, Responder, http::header::CONTENT_TYPE, web};

use crate::cors::AppState;

pub async fn greet(state: web::Data<AppState>) -> impl Responder {
    HttpResponse::Ok()
        .insert_header((CONTENT_TYPE, "text/html; charset=utf-8"))
        .body(format!(
            "<h1>{}</h1><p>Actix Web is now serving with bunner-cors-rs.</p>",
            state.greeting
        ))
}
