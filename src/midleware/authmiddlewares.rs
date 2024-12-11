use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpMessage, HttpResponse, ResponseError,
};
use futures::future::{ok, LocalBoxFuture, Ready};
use serde_json::json;
use std::fmt;

use crate::utils::jwt::decode_token;

pub struct Authentication;

impl<S, B> Transform<S, ServiceRequest> for Authentication
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = AuthMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(AuthMiddleware { service })
    }
}

pub struct AuthMiddleware<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for AuthMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let auth_header = req.headers().get("AUTHORIZATION").cloned();

        if let Some(auth_value) = auth_header {
            if let Ok(auth_str) = auth_value.to_str() {
                if auth_str.starts_with("Bearer ") {
                    let token = auth_str.trim_start_matches("Bearer ").trim();
                    let decode_token = decode_token(token.to_string()).expect("token invalid");
                     
                    req.extensions_mut().insert(decode_token.claims.user.id);
                        let fut = self.service.call(req);
                        return Box::pin(async move {
                            let res = fut.await?;
                            Ok(res)
                        });

                }
            }
        }

        Box::pin(async { Err(UnauthorizedError.into()) })
    }
}

#[derive(Debug)]
struct UnauthorizedError;

impl fmt::Display for UnauthorizedError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Unauthorized: Invalid or missing token")
    }
}

impl ResponseError for UnauthorizedError {
    fn error_response(&self) -> HttpResponse {
        HttpResponse::Unauthorized()
            .json(json!({"error": "Unauthorized", "message": self.to_string()}))
    }
}