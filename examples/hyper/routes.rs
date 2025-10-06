use std::convert::Infallible;
use std::future::Future;
use std::pin::Pin;

use http_body_util::Full;
use hyper::body::{Bytes, Incoming};
use hyper::http::header::CONTENT_TYPE;
use hyper::http::{Method, StatusCode};
use hyper::service::Service;
use hyper::{Request, Response};

use crate::cors::middleware::CorsBody;
use crate::cors::SharedAppState;

type BoxFuture<T> = Pin<Box<dyn Future<Output = T> + Send>>;

#[derive(Clone)]
pub struct Router {
    state: SharedAppState,
}

pub fn router(state: SharedAppState) -> Router {
    Router { state }
}

impl Service<Request<Incoming>> for Router {
    type Response = Response<CorsBody>;
    type Error = Infallible;
    type Future = BoxFuture<Result<Self::Response, Self::Error>>;

    fn call(&self, req: Request<Incoming>) -> Self::Future {
        let state = self.state.clone();

        Box::pin(async move {
            let response = match (req.method(), req.uri().path()) {
                (&Method::GET, "/greet") => greet(state),
                _ => not_found(),
            };

            Ok(response)
        })
    }
}

fn greet(state: SharedAppState) -> Response<CorsBody> {
    let body = format!(
        "<h1>{}</h1><p>Try calling this endpoint from your frontend to see CORS in action.</p>",
        state.greeting
    );

    Response::builder()
        .status(StatusCode::OK)
        .header(CONTENT_TYPE, "text/html; charset=utf-8")
        .body(Full::new(Bytes::from(body)))
        .expect("valid response")
}

fn not_found() -> Response<CorsBody> {
    Response::builder()
        .status(StatusCode::NOT_FOUND)
        .body(Full::new(Bytes::from("Not Found")))
        .expect("valid response")
}
