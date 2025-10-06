use std::future::Future;
use std::pin::Pin;

use bunner_cors_rs::constants::header;
use bunner_cors_rs::{CorsDecision, CorsError, Headers, PreflightRejectionReason, RequestContext};
use http_body_util::Full;
use hyper::body::{Bytes, Incoming};
use hyper::http::StatusCode;
use hyper::http::header::{HeaderMap, HeaderName, HeaderValue};
use hyper::service::Service;
use hyper::{Request, Response};

use super::SharedCors;

type BoxFuture<T> = Pin<Box<dyn Future<Output = T> + Send>>;

pub type CorsBody = Full<Bytes>;

/// Hyper middleware that mirrors the pattern described in the
/// official "Getting Started with a Server Middleware" guide:
/// https://hyper.rs/guides/1/server/middleware/
#[derive(Clone)]
pub struct BunnerCors<S> {
    inner: S,
    cors: SharedCors,
}

impl<S> BunnerCors<S> {
    pub fn new(cors: SharedCors, inner: S) -> Self {
        Self { inner, cors }
    }
}

impl<S> Service<Request<Incoming>> for BunnerCors<S>
where
    S: Service<Request<Incoming>, Response = Response<CorsBody>> + Clone + Send + 'static,
    S::Future: Send + 'static,
    S::Error: Send + 'static,
{
    type Response = Response<CorsBody>;
    type Error = S::Error;
    type Future = BoxFuture<Result<Self::Response, Self::Error>>;

    fn call(&self, req: Request<Incoming>) -> Self::Future {
        let cors = self.cors.clone();
        let owned_ctx = OwnedRequestContext::from_request(&req);
        let decision = cors.check(&owned_ctx.as_request_context());

        match decision {
            Ok(CorsDecision::PreflightAccepted { headers }) => {
                Box::pin(async move { Ok(preflight_response(StatusCode::NO_CONTENT, headers)) })
            }
            Ok(CorsDecision::PreflightRejected(rejection)) => {
                let message = rejection_message(&rejection.reason);
                Box::pin(
                    async move { Ok(preflight_rejection(rejection.headers, message.as_str())) },
                )
            }
            Ok(CorsDecision::SimpleAccepted { headers }) => {
                let inner = self.inner.clone();
                Box::pin(async move {
                    let mut response = inner.call(req).await?;
                    apply_headers(response.headers_mut(), &headers);
                    Ok(response)
                })
            }
            Ok(CorsDecision::NotApplicable) => {
                let inner = self.inner.clone();
                Box::pin(async move { inner.call(req).await })
            }
            Err(err) => Box::pin(async move { Ok(internal_error(err)) }),
        }
    }
}

fn preflight_response(status: StatusCode, headers: Headers) -> Response<CorsBody> {
    let mut builder = Response::builder().status(status);
    if let Some(map) = builder.headers_mut() {
        insert_headers(map, &headers);
    }
    builder
        .body(Full::new(Bytes::new()))
        .expect("failed to build preflight response")
}

fn preflight_rejection(headers: Headers, message: &str) -> Response<CorsBody> {
    let mut builder = Response::builder().status(StatusCode::FORBIDDEN);
    if let Some(map) = builder.headers_mut() {
        insert_headers(map, &headers);
    }
    builder
        .body(Full::new(Bytes::from(message.to_string())))
        .expect("failed to build rejection response")
}

fn internal_error(err: CorsError) -> Response<CorsBody> {
    Response::builder()
        .status(StatusCode::INTERNAL_SERVER_ERROR)
        .body(Full::new(Bytes::from(format!(
            "CORS configuration error: {err}"
        ))))
        .expect("failed to build internal error response")
}

fn apply_headers(map: &mut HeaderMap, headers: &Headers) {
    for (name, value) in headers.iter() {
        if let (Ok(header_name), Ok(header_value)) = (
            HeaderName::try_from(name.as_str()),
            HeaderValue::from_str(value),
        ) {
            map.insert(header_name, header_value);
        }
    }
}

fn insert_headers(map: &mut HeaderMap, headers: &Headers) {
    for (name, value) in headers.iter() {
        if let (Ok(header_name), Ok(header_value)) = (
            HeaderName::try_from(name.as_str()),
            HeaderValue::from_str(value),
        ) {
            map.insert(header_name, header_value);
        }
    }
}

fn rejection_message(reason: &PreflightRejectionReason) -> String {
    match reason {
        PreflightRejectionReason::OriginNotAllowed => {
            "Preflight rejected: origin not allowed".into()
        }
        PreflightRejectionReason::MethodNotAllowed { requested_method } => {
            format!("Preflight rejected: method '{requested_method}' not allowed")
        }
        PreflightRejectionReason::HeadersNotAllowed { requested_headers } => {
            format!("Preflight rejected: headers '{requested_headers}' not allowed")
        }
        PreflightRejectionReason::MissingAccessControlRequestMethod => {
            "Preflight rejected: Access-Control-Request-Method header missing".into()
        }
    }
}

struct OwnedRequestContext {
    method: String,
    origin: String,
    access_control_request_method: String,
    access_control_request_headers: String,
    access_control_request_private_network: bool,
}

impl OwnedRequestContext {
    fn from_request(request: &Request<Incoming>) -> Self {
        let headers = request.headers();

        Self {
            method: request.method().as_str().to_string(),
            origin: header_value(headers, header::ORIGIN),
            access_control_request_method: header_value(
                headers,
                header::ACCESS_CONTROL_REQUEST_METHOD,
            ),
            access_control_request_headers: header_value(
                headers,
                header::ACCESS_CONTROL_REQUEST_HEADERS,
            ),
            access_control_request_private_network: headers
                .get(header::ACCESS_CONTROL_REQUEST_PRIVATE_NETWORK)
                .and_then(|value| value.to_str().ok())
                .map(|value| value.eq_ignore_ascii_case("true"))
                .unwrap_or(false),
        }
    }

    fn as_request_context(&self) -> RequestContext<'_> {
        RequestContext {
            method: &self.method,
            origin: &self.origin,
            access_control_request_method: &self.access_control_request_method,
            access_control_request_headers: &self.access_control_request_headers,
            access_control_request_private_network: self.access_control_request_private_network,
        }
    }
}

fn header_value(headers: &HeaderMap, name: &str) -> String {
    headers
        .get(name)
        .and_then(|value| value.to_str().ok())
        .map(|value| value.to_string())
        .unwrap_or_default()
}
