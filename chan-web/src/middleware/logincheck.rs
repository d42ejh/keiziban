use actix_identity::{ RequestIdentity};
use actix_web::body::EitherBody;
use actix_web::dev::{self, ServiceRequest, ServiceResponse};
use actix_web::dev::{Service, Transform};
use actix_web::{http, Error, HttpResponse};
use futures::future::LocalBoxFuture;
use std::future::{ready, Ready};
use std::rc::Rc;
use tracing::{event, Level};
/*
use actix_web::{
    body::{BoxBody, EitherBody, MessageBody},
    dev::{Service, ServiceRequest, ServiceResponse, Transform},
    http, Error, FromRequest, HttpMessage, HttpResponse, Result,
};
use futures_util::future::{FutureExt as _, LocalBoxFuture};
//https://github.com/actix/actix-web/blob/master/actix-web/MIGRATION-4.0.md#response-body-types
use futures::future::{ok, ready, Either, Ready};
use futures::Future;
use futures::FutureExt;
use std::pin::Pin;
use std::task::{Context, Poll};
use tracing::{event, Level};
*/
//https://github.com/actix/examples/tree/master/basics //middle ware examples
//https://docs.rs/actix-web/4.0.0-rc.3/actix_web/web/struct.HttpResponse.html
//https://docs.rs/actix-web/4.0.0-rc.3/actix_web/body/struct.BoxBody.html
//https://github.com/actix/actix-web/issues/1517
//https://github.com/actix/actix-web/issues/1499
//https://github.com/actix/actix-extras/blob/master/actix-identity/src/middleware.rs
pub struct LoginCheck;

impl<S: 'static, B> Transform<S, ServiceRequest> for LoginCheck
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type InitError = ();
    type Transform = LoginCheckMiddleWare<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(LoginCheckMiddleWare {
            service: Rc::new(service),
        }))
    }
}

pub struct LoginCheckMiddleWare<S> {
    // This is special: We need this to avoid lifetime issues.
    service: Rc<S>,
}

impl<S, B> Service<ServiceRequest> for LoginCheckMiddleWare<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    dev::forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let svc = self.service.clone();

        Box::pin(async move {
            let id = req.get_identity();

            let is_logged_in = if id.is_some() { true } else { false };
            if is_logged_in
                || req.path() == "/login"
                || req.path() == "/register"
                || req.path() == "/"
                || req.path() == "/graphql"
            {
                event!(
                    Level::DEBUG,
                    "Allow request to {} logged in: {}",
                    req.path(),
                    is_logged_in
                );

                //already logged in or allowed paths, allow request.
                let res = svc.call(req).await?;
                Ok(res.map_into_left_body())
            } else {
                // hasn't logged in, redirect to login page
                let (request, _) = req.into_parts();
                let response = HttpResponse::Found()
                    .insert_header((http::header::LOCATION, "/login"))
                    .finish()
                    .map_into_right_body();

                Ok(ServiceResponse::new(request, response))
            }
            // println!("request body: {:?}", body);

            // println!("response: {:?}", res.headers());
        })
    }
}
