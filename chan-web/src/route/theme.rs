use crate::constant::THEME_SESSION_KEY;
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

#[derive(Serialize, Deserialize)]
pub struct ThemeFormParams {
    theme: String,
}

#[post("/theme")]
pub async fn theme_handler(
    id: Identity,
    session: Session,
    params: web::Form<ThemeFormParams>,
    request: HttpRequest,
) -> impl Responder {
    assert!(id.identity().is_some()); //protected route

    if let Err(e) = session.insert(THEME_SESSION_KEY, &params.theme) {
        FlashMessage::error(format!("Failed to change the theme: {}",e)).send();
    } else {
        FlashMessage::success(format!("Changed the theme({}).", &params.theme)).send();
    }

    HttpResponse::Found()
        .append_header((actix_web::http::header::LOCATION, "/"))
        .finish()
}
