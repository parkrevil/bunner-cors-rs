use axum::{
    extract::State,
    response::{Html, IntoResponse},
};

use crate::cors::AppState;

pub async fn greet(State(state): State<AppState>) -> impl IntoResponse {
    Html(format!(
        "<h1>{}</h1><p>Try calling this endpoint from your frontend to see CORS in action.</p>",
        state.greeting
    ))
}
