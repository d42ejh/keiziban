use crate::utility::extract_theme_from_session;
use actix_session::Session;
use actix_web::{get,   Responder};
use actix_web_flash_messages::{ IncomingFlashMessages};
use askama_actix::{Template, TemplateToResponse};
//use tracing::{event, Id, Level};

#[derive(Template)]
#[template(path = "manage.html")]
struct ManageTemplate {
    theme: String,
    flash_messages: IncomingFlashMessages,
}

#[get("/manage")]
pub async fn manage(session: Session, flash_messages: IncomingFlashMessages) -> impl Responder {
    ManageTemplate {
        theme: extract_theme_from_session(&session),
        flash_messages: flash_messages,
    }
    .to_response()
}
