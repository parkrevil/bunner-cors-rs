use axum::{
    body::Body,
    extract::{Request, State},
    http::{HeaderMap, HeaderName, HeaderValue, StatusCode},
    middleware::Next,
    response::Response,
};
use bunner_cors_rs::{
    CorsDecision, CorsError, Headers, PreflightRejectionReason, RequestContext, SimpleRejection,
    SimpleRejectionReason, constants::header,
};

use super::{AppState, SharedCors};

pub async fn cors_middleware(
    State(state): State<AppState>,
    request: Request,
    next: Next,
) -> Response {
    let cors: SharedCors = state.cors.clone();

    let owned_ctx = OwnedRequestContext::from_request(&request);
    let context = owned_ctx.as_request_context();

    match cors.check(&context) {
        Ok(CorsDecision::PreflightAccepted { headers }) => {
            preflight_response(StatusCode::NO_CONTENT, headers)
        }
        Ok(CorsDecision::PreflightRejected(rejection)) => {
            let mut response = preflight_response(StatusCode::FORBIDDEN, rejection.headers);
            let message = rejection_message(&rejection.reason);
            *response.body_mut() = Body::from(message);
            response
        }
        Ok(CorsDecision::SimpleAccepted { headers }) => {
            let mut response = next.run(request).await;
            apply_headers(response.headers_mut(), &headers);
            response
        }
        Ok(CorsDecision::SimpleRejected(rejection)) => simple_rejection_response(rejection),
        Ok(CorsDecision::NotApplicable) => next.run(request).await,
        Err(err) => middleware_error_response(err),
    }
}

fn middleware_error_response(err: CorsError) -> Response {
    let mut response = Response::builder()
        .status(StatusCode::INTERNAL_SERVER_ERROR)
        .body(Body::empty())
        .unwrap();

    *response.body_mut() = Body::from(format!("CORS configuration error: {err}"));
    response
}

fn preflight_response(status: StatusCode, headers: Headers) -> Response {
    let mut response = Response::builder()
        .status(status)
        .body(Body::empty())
        .unwrap();

    apply_headers(response.headers_mut(), &headers);
    response
}

fn simple_rejection_response(rejection: SimpleRejection) -> Response {
    let mut response = Response::builder()
        .status(StatusCode::FORBIDDEN)
        .body(Body::empty())
        .unwrap();

    apply_headers(response.headers_mut(), &rejection.headers);
    *response.body_mut() = Body::from(simple_rejection_message(&rejection.reason));
    response
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

fn simple_rejection_message(reason: &SimpleRejectionReason) -> &'static str {
    match reason {
        SimpleRejectionReason::OriginNotAllowed => "Simple request rejected: origin not allowed",
    }
}

struct OwnedRequestContext {
    method: String,
    origin: Option<String>,
    access_control_request_method: Option<String>,
    access_control_request_headers: Option<String>,
    access_control_request_private_network: bool,
}

impl OwnedRequestContext {
    fn from_request(request: &Request) -> Self {
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
            origin: self.origin.as_deref(),
            access_control_request_method: self.access_control_request_method.as_deref(),
            access_control_request_headers: self.access_control_request_headers.as_deref(),
            access_control_request_private_network: self.access_control_request_private_network,
        }
    }
}

fn header_value(headers: &HeaderMap, name: &str) -> Option<String> {
    headers
        .get(name)
        .and_then(|value| value.to_str().ok())
        .map(|value| value.to_string())
}
