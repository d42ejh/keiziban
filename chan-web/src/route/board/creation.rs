use crate::utility::extract_theme_from_session;
use crate::utility::post_graphql_with_token_ex;
use actix_identity::{CookieIdentityPolicy, Identity, IdentityService};
use actix_session::Session;
use actix_web::dev::ConnectionInfo;
use actix_web::guard::Connect;
use actix_web::{get, post, web, App, HttpRequest, HttpResponse, HttpServer, Responder};
use actix_web_flash_messages::{FlashMessage, IncomingFlashMessages};
use askama_actix::{Template, TemplateToResponse};
use chan_graphql_client::{CreateBoard, GraphQLQuery, Login, Response};
use reqwest::{Client, RequestBuilder};
use serde::{Deserialize, Serialize};
use std::error::Error;
use tracing::{event, Id, Level};

#[derive(Template)]
#[template(path = "board_creation.html")]
struct BoardCreationTemplate {
    theme: String,
    flash_messages: IncomingFlashMessages,
}

#[get("/board_creation")]
pub async fn board_creation(
    session: Session,
    flash_messages: IncomingFlashMessages,
) -> impl Responder {
    BoardCreationTemplate {
        theme: extract_theme_from_session(&session),
        flash_messages: flash_messages,
    }
    .to_response()
}

#[derive(Serialize, Deserialize)]
pub struct BoardCreationParams {
    pub name: String,
    pub description: String,
}

#[post("/board_creation")]
pub async fn board_creation_handler(
    params: web::Form<BoardCreationParams>,
    reqwest_client: web::Data<Client>,
    connection_info: ConnectionInfo,
    id: Identity,
) -> impl Responder {
    assert!(id.identity().is_some());

    let variables = chan_graphql_client::create_board::Variables {
        board_name: params.name.to_owned(),
        board_description: params.description.to_owned(),
    };

    let graphql_url = format!(
        "{}://{}/graphql",
        connection_info.scheme(),
        connection_info.host()
    );

    let result = post_graphql_with_token_ex::<CreateBoard, _>(
        &reqwest_client,
        &graphql_url,
        variables,
        &id.identity().unwrap(),
    )
    .await;

    if result.is_err() {
        debug_assert!(result.as_ref().err().is_some());
        let error = result.as_ref().err().unwrap();
        return HttpResponse::InternalServerError().body(error.to_string());
    }
    let data = result.unwrap();

    event!(Level::DEBUG, "Created board {}", data.create_board);

    FlashMessage::success(format!(
        "Created a new board successfully! {}",
        data.create_board
    ))
    .send();

    //todo redirect to created board
    //redirect
    HttpResponse::Found()
        .append_header((actix_web::http::header::LOCATION, "/"))
        .finish()
}
