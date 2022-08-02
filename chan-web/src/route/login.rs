use crate::utility::extract_theme_from_session;
use crate::utility::post_graphql_with_token_ex;
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

#[derive(Template)]
#[template(path = "login.html")]
struct LoginTemplate {
    theme: String,
    flash_messages: IncomingFlashMessages,
}

#[get("/login")]
pub async fn login(session: Session, flash_messages: IncomingFlashMessages) -> impl Responder {
    LoginTemplate {
        theme: extract_theme_from_session(&session),
        flash_messages: flash_messages,
    }
    .to_response()
}

#[derive(Serialize, Deserialize)]
pub struct LoginFormParams {
    pub id: String,
    pub password: String,
}

#[post("/login")]
pub async fn login_handler(
    params: web::Form<LoginFormParams>,
    reqwest_client: web::Data<Client>,
    connection_info: ConnectionInfo,
    id: Identity,
) -> impl Responder {
    if id.identity().is_some() {
        return HttpResponse::InternalServerError().body("Already logged in.");
    }

    let variables = chan_graphql_client::login::Variables {
        user_id: params.id.to_owned(),
        password: params.password.to_owned(),
    };

    let graphql_url = format!(
        "{}://{}/graphql",
        connection_info.scheme(),
        connection_info.host()
    );

    let result = post_graphql_with_token_ex::<Login, _>(
        &reqwest_client,
        graphql_url,
        variables,
        "dummytoken", /*No need to use token */
    )
    .await;

    if result.is_err() {
        debug_assert!(result.as_ref().err().is_some());
        let error = result.as_ref().err().unwrap();
        return HttpResponse::InternalServerError().body(error.to_string());
    }

    let data = result.unwrap();

    id.remember(data.login); //save token to cookie

    FlashMessage::success("Login success!").send();

    //redirect
    HttpResponse::Found()
        .append_header((actix_web::http::header::LOCATION, "/"))
        .finish()
}
