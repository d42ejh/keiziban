use crate::utility::extract_theme_from_session;
use crate::utility::post_graphql_with_token_ex;
use actix_session::Session;
use actix_web::dev::ConnectionInfo;
use actix_web::guard::Connect;
use actix_web::{get, post, web, App, HttpRequest, HttpResponse, HttpServer, Responder};
use actix_web_flash_messages::{FlashMessage, IncomingFlashMessages};
use askama_actix::{Template, TemplateToResponse};
use chan_graphql_client::{register_account, GraphQLQuery, RegisterAccount, Response};
use reqwest::{Client, RequestBuilder};
use serde::{Deserialize, Serialize};
use std::error::Error;
use tracing::{event, Level};

#[derive(Template)]
#[template(path = "register.html")]
struct RegisterTemplate {
    theme: String,
    flash_messages: IncomingFlashMessages,
}

#[get("/register")]
pub async fn register(session: Session, flash_messages: IncomingFlashMessages) -> impl Responder {
    RegisterTemplate {
        theme: extract_theme_from_session(&session),
        flash_messages: flash_messages,
    }
    .to_response()
}

#[derive(Serialize, Deserialize)]
pub struct RegisterFormParams {
    pub password: String,
}

#[post("/register")]
pub async fn registration_handler(
    params: web::Form<RegisterFormParams>,
    reqwest_client: web::Data<Client>,
    connection_info: ConnectionInfo,
) -> impl Responder {
    let variables = register_account::Variables {
        password: params.password.to_owned(),
    };

    let graphql_url = format!(
        "{}://{}/graphql",
        connection_info.scheme(),
        connection_info.host()
    );

    let result = post_graphql_with_token_ex::<RegisterAccount, _>(
        &reqwest_client,
        &graphql_url,
        variables,
        "dummytoken", /* no need to use token */
    )
    .await;

    if result.is_err() {
        debug_assert!(result.as_ref().err().is_some());
        let error = result.as_ref().err().unwrap();
        return HttpResponse::InternalServerError().body(error.to_string());
    }

    let data = result.unwrap();
    let user = data.sign_up;

    FlashMessage::success("Registration success! ").send();
    FlashMessage::info(format!("Your id is {} (Don't forget this ID!) ", user.id)).send();

    //redirect to login page
    HttpResponse::Found()
        .append_header((actix_web::http::header::LOCATION, "/login"))
        .finish()
}
