use std::future::{ready, Ready};
use std::pin::Pin;
use std::task::{Context, Poll};

use actix_web::body::{EitherBody, MessageBody};
use actix_web::dev::{Service, ServiceRequest, ServiceResponse, Transform};
use actix_web::http::{
    header::{HeaderMap, HeaderName, HeaderValue},
    StatusCode,
};
use actix_web::{Error, HttpRequest, HttpResponse, HttpResponseBuilder};
use bunner_cors_rs::{
    constants::header, CorsDecision, CorsError, Headers, PreflightRejectionReason, RequestContext,
};

use super::SharedCors;

type LocalBoxFuture<'a, T> = Pin<Box<dyn std::future::Future<Output = T> + 'a>>;

pub struct BunnerCors {
    cors: SharedCors,
}

impl BunnerCors {
    pub fn new(cors: SharedCors) -> Self {
        Self { cors }
    }
}

impl<S, B> Transform<S, ServiceRequest> for BunnerCors
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: MessageBody + 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type Transform = BunnerCorsMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(BunnerCorsMiddleware {
            service,
            cors: self.cors.clone(),
        }))
    }
}

pub struct BunnerCorsMiddleware<S> {
    service: S,
    cors: SharedCors,
}

impl<S, B> Service<ServiceRequest> for BunnerCorsMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: MessageBody + 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let cors = self.cors.clone();
        let owned_ctx = OwnedRequestContext::from_request(req.request());
        let context = owned_ctx.as_request_context();

        match cors.check(&context) {
            Ok(CorsDecision::PreflightAccepted { headers }) => {
                Box::pin(
                    async move { Ok(preflight_response(req, StatusCode::NO_CONTENT, headers)) },
                )
            }
            Ok(CorsDecision::PreflightRejected(rejection)) => {
                let reason = rejection_message(&rejection.reason);
                Box::pin(async move { Ok(preflight_rejection(req, rejection.headers, &reason)) })
            }
            Ok(CorsDecision::SimpleAccepted { headers }) => {
                let fut = self.service.call(req);
                Box::pin(async move {
                    let mut res = fut.await?.map_into_left_body();
                    apply_headers(res.headers_mut(), &headers);
                    Ok(res)
                })
            }
            Ok(CorsDecision::NotApplicable) => {
                let fut = self.service.call(req);
                Box::pin(async move { Ok(fut.await?.map_into_left_body()) })
            }
            Err(err) => Box::pin(async move { Ok(internal_error(req, err)) }),
        }
    }
}

fn preflight_response<B>(
    req: ServiceRequest,
    status: StatusCode,
    headers: Headers,
) -> ServiceResponse<EitherBody<B>> {
    let mut builder = HttpResponse::build(status);
    insert_headers(&mut builder, &headers);
    let response = builder.finish().map_into_right_body();
    req.into_response(response)
}

fn preflight_rejection<B>(
    req: ServiceRequest,
    headers: Headers,
    message: &str,
) -> ServiceResponse<EitherBody<B>> {
    let mut builder = HttpResponse::Forbidden();
    insert_headers(&mut builder, &headers);
    let response = builder.body(message.to_string()).map_into_right_body();
    req.into_response(response)
}

fn internal_error<B>(req: ServiceRequest, err: CorsError) -> ServiceResponse<EitherBody<B>> {
    let response = HttpResponse::InternalServerError()
        .body(format!("CORS configuration error: {err}"))
        .map_into_right_body();
    req.into_response(response)
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

fn insert_headers(builder: &mut HttpResponseBuilder, headers: &Headers) {
    for (name, value) in headers.iter() {
        if let (Ok(header_name), Ok(header_value)) = (
            HeaderName::try_from(name.as_str()),
            HeaderValue::from_str(value),
        ) {
            builder.insert_header((header_name, header_value));
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
    fn from_request(request: &HttpRequest) -> Self {
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
