use crate::utility::extract_theme_from_session;
use crate::utility::post_graphql_with_token_ex;
use actix_identity::{CookieIdentityPolicy, Identity, IdentityService};
use actix_session::Session;
use actix_web::dev::ConnectionInfo;
use actix_web::guard::Connect;
use actix_web::{get, post, web, App, HttpRequest, HttpResponse, HttpServer, Responder};
use actix_web_flash_messages::{FlashMessage, IncomingFlashMessages};
use askama_actix::{Template, TemplateToResponse};
use chan_graphql_client::{GraphQLQuery, Login, RemoveThread, Response};
use reqwest::{Client, RequestBuilder};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::str::FromStr;
use tracing::{event, Id, Level};
use uuid::Uuid;

#[derive(Serialize, Deserialize)]
pub struct ThreadRemovalParams {
    pub thread_uuid: Uuid,
}

#[post("/thread_removal")]
pub async fn thread_removal_handler(
    params: web::Form<ThreadRemovalParams>,
    reqwest_client: web::Data<Client>,
    connection_info: ConnectionInfo,
    id: Identity,
    request: HttpRequest,
) -> impl Responder {
    assert!(id.identity().is_some());

    let variables = chan_graphql_client::remove_thread::Variables {
        thread_uuid: params.thread_uuid,
    };

    let graphql_url = format!(
        "{}://{}/graphql",
        connection_info.scheme(),
        connection_info.host()
    );

    let result = post_graphql_with_token_ex::<RemoveThread, _>(
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

    event!(Level::DEBUG, "Removed thread {}", params.thread_uuid);
    FlashMessage::success(format!("Removed thread {}", params.thread_uuid)).send();

    HttpResponse::Ok().finish()
}
