use crate::utility::extract_theme_from_session;
use actix_identity::{CookieIdentityPolicy, Identity, IdentityService};
use actix_session::Session;
use actix_web::dev::ConnectionInfo;
use actix_web::guard::Connect;
use actix_web::{get, post, web, App, HttpRequest, HttpResponse, HttpServer, Responder};
use actix_web_flash_messages::{FlashMessage, IncomingFlashMessages};
use askama_actix::{Template, TemplateToResponse};
use chan_graphql_client::{GraphQLQuery, Login, Response};
use reqwest::{Client, RequestBuilder};
use serde::{Deserialize, Serialize};
use std::error::Error;
use tracing::{event, Id, Level};

#[get("/logout")]
pub async fn logout_handler(id: Identity) -> impl Responder {
    assert!(id.identity().is_some());
    id.forget();

    FlashMessage::success("Logged out.").send();

    //redirect
    HttpResponse::Found()
        .append_header((actix_web::http::header::LOCATION, "/"))
        .finish()
}
