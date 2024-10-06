use actix_web::{dev::ServiceRequest, Error};
use futures_util::future::{ok, Ready};
use futures_util::FutureExt;
use std::task::{Context, Poll};
use std::env;

pub struct ApiKeyMiddleware;

impl<S, B> actix_service::Transform<S, ServiceRequest> for ApiKeyMiddleware
where
    S: actix_service::Service<ServiceRequest, Response = actix_web::dev::ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
{
    type Response = actix_web::dev::ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = ApiKeyMiddlewareService<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(ApiKeyMiddlewareService { service })
    }
}

pub struct ApiKeyMiddlewareService<S> {
    service: S,
}

impl<S, B> actix_service::Service<ServiceRequest> for ApiKeyMiddlewareService<S>
where
    S: actix_service::Service<ServiceRequest, Response = actix_web::dev::ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
{
    type Response = actix_web::dev::ServiceResponse<B>;
    type Error = Error;
    type Future = futures_util::future::LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let api_key = req.headers().get("x-api-key").cloned();
        let fut = self.service.call(req);

        async move {
            if let Some(api_key) = api_key {
                let expected_api_key = env::var("ADMIN_API_KEY").expect("ADMIN_API_KEY must be set");
                if api_key.to_str().unwrap_or("") == expected_api_key {
                    return fut.await;
                }
            }
            Err(actix_web::error::ErrorUnauthorized("Invalid API key")).into()
        }
        .boxed_local()
    }
}