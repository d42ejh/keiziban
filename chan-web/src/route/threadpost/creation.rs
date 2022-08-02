use crate::utility::extract_theme_from_session;
use crate::utility::post_graphql_with_token_ex;
use actix_identity::{CookieIdentityPolicy, Identity, IdentityService};
use actix_session::Session;
use actix_web::dev::ConnectionInfo;
use actix_web::guard::Connect;
use actix_web::{get, post, web, App, HttpRequest, HttpResponse, HttpServer, Responder};
use actix_web_flash_messages::{FlashMessage, IncomingFlashMessages};
use askama_actix::{Template, TemplateToResponse};
use chan_graphql_client::{GraphQLQuery, Login, PostThreadPost, Response};
use reqwest::{Client, RequestBuilder};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::str::FromStr;
use tracing::{event, Id, Level};
use uuid::Uuid;

#[derive(Serialize, Deserialize)]
pub struct ThreadPostCreationParams {
    pub post: String,
    pub parent_thread_uuid: String,
}

#[post("/threadpost_creation")]
pub async fn threadpost_creation_handler(
    params: web::Form<ThreadPostCreationParams>,
    reqwest_client: web::Data<Client>,
    connection_info: ConnectionInfo,
    id: Identity,
    request: HttpRequest,
) -> impl Responder {
    assert!(id.identity().is_some());

    let parent_thread_uuid = match Uuid::from_str(&params.parent_thread_uuid) {
        Ok(id) => id,
        Err(_) => return HttpResponse::InternalServerError().body("Invalid uuid."),
    };

    let variables = chan_graphql_client::post_thread_post::Variables {
        thread_uuid: parent_thread_uuid,
        post_body: params.post.to_owned(),
    };

    let graphql_url = format!(
        "{}://{}/graphql",
        connection_info.scheme(),
        connection_info.host()
    );

    let result = post_graphql_with_token_ex::<PostThreadPost, _>(
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

    event!(Level::DEBUG, "Created thread {}", data.post_threadpost);

    FlashMessage::success(format!(
        "Posted the new threadpost successfully! {}",
        data.post_threadpost
    ))
    .send();

    let redirect_dest = format!("/redirect/thread/{}", parent_thread_uuid);
    event!(Level::DEBUG, "Redirect dest {}", &redirect_dest);
    //redirect
    HttpResponse::Found()
        .append_header((actix_web::http::header::LOCATION, redirect_dest))
        .finish()
}
