use crate::utility::extract_theme_from_session;
use actix_session::Session;
use actix_web::{get, post, web, App, HttpRequest, HttpResponse, HttpServer, Responder};
use actix_web_flash_messages::IncomingFlashMessages;
use askama_actix::{Template, TemplateToResponse};

#[derive(Template)]
#[template(path = "redirect.html")]
struct RedirectTemplate<'a> {
    dest_link: &'a str,
    theme: String,
    flash_messages: IncomingFlashMessages,
}

pub async fn redirect(
    session: Session,
    //  request: HttpRequest,
    path: web::Path<(String, Option<String>)>,
    flash_messages: IncomingFlashMessages,
) -> impl Responder {
    let dest_link;
    let base = &path.0;
    if path.1.is_some() {
        dest_link = format!("/{}/{}", base, path.1.as_ref().unwrap());
    } else {
        dest_link = format!("/{}", base);
    }

    RedirectTemplate {
        dest_link: &dest_link,
        theme: extract_theme_from_session(&session),
        flash_messages: flash_messages,
    }
    .to_response()
}
