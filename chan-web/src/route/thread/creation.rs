use crate::utility::extract_theme_from_session;
use crate::utility::post_graphql_with_token_ex;
use actix_identity::{CookieIdentityPolicy, Identity, IdentityService};
use actix_session::Session;
use actix_web::dev::ConnectionInfo;
use actix_web::guard::Connect;
use actix_web::{get, post, web, App, HttpRequest, HttpResponse, HttpServer, Responder};
use actix_web_flash_messages::{FlashMessage, IncomingFlashMessages};
use askama_actix::{Template, TemplateToResponse};
use chan_graphql_client::{CreateThread, GraphQLQuery, Login, Response};
use reqwest::{Client, RequestBuilder};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::str::FromStr;
use tracing::{event, Id, Level};
use uuid::Uuid;

#[derive(Serialize, Deserialize)]
pub struct ThreadCreationParams {
    pub title: String,
    pub post: String,
    pub parent_board_uuid: String,
}

#[post("/thread_creation")]
pub async fn thread_creation_handler(
    params: web::Form<ThreadCreationParams>,
    reqwest_client: web::Data<Client>,
    connection_info: ConnectionInfo,
    id: Identity,
) -> impl Responder {
    assert!(id.identity().is_some());

    let parent_board_uuid = match Uuid::from_str(&params.parent_board_uuid) {
        Ok(id) => id,
        Err(_) => return HttpResponse::InternalServerError().body("Invalid uuid."),
    };

    let variables = chan_graphql_client::create_thread::Variables {
        thread_title: params.title.to_owned(),
        parent_board_uuid: parent_board_uuid,
        first_post_text: params.post.to_owned(),
    };

    let graphql_url = format!(
        "{}://{}/graphql",
        connection_info.scheme(),
        connection_info.host()
    );

    let result = post_graphql_with_token_ex::<CreateThread, _>(
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

    event!(Level::DEBUG, "Created thread {}", data.create_thread);

    FlashMessage::success(format!(
        "Created a new thread successfully! {}",
        data.create_thread
    ))
    .send();

    //todo redirect to created thread
    //redirect
    let redirect_url = format!("/thread/{}", data.create_thread);
    HttpResponse::Found()
        .append_header((actix_web::http::header::LOCATION, redirect_url))
        .finish()
}
