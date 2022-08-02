use crate::utility::extract_theme_from_session;
use actix_session::Session;
use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use actix_web_flash_messages::IncomingFlashMessages;
use askama_actix::{Template, TemplateToResponse};

#[derive(Template)]
#[template(path = "root.html")]
struct RootTemplate {
    theme: String,
    flash_messages: IncomingFlashMessages,
}

#[get("/")]
pub async fn root(session: Session, flash_messages: IncomingFlashMessages) -> impl Responder {
    RootTemplate {
        theme: extract_theme_from_session(&session),
        flash_messages: flash_messages,
    }
    .to_response()
}
