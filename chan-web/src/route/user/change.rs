use crate::utility::extract_theme_from_session;
use crate::utility::post_graphql_with_token_ex;
use actix_identity::{CookieIdentityPolicy, Identity, IdentityService};
use actix_session::Session;
use actix_web::dev::ConnectionInfo;
use actix_web::guard::Connect;
use actix_web::{get, post, web, App, HttpRequest, HttpResponse, HttpServer, Responder};
use actix_web_flash_messages::{FlashMessage, IncomingFlashMessages};
use askama_actix::{Template, TemplateToResponse};
use chan_graphql_client::{GraphQLQuery, ChangeUserType,Response};
use reqwest::{Client, RequestBuilder};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::str::FromStr;
use tracing::{event, Id, Level};
use uuid::Uuid;

#[derive(Serialize, Deserialize)]
pub struct UserTypeChangeParams {
    pub user_id: String,
   pub new_type: i64,
}

#[post("/user_type_change")]
pub async fn user_type_change_handler(
    params: web::Form<UserTypeChangeParams>,
    reqwest_client: web::Data<Client>,
    connection_info: ConnectionInfo,
    id: Identity,
    request: HttpRequest,
) -> impl Responder {
    assert!(id.identity().is_some());

    let variables = chan_graphql_client::change_user_type::Variables {
        user_id: params.user_id.to_owned(),
        user_type: params.new_type
    };

    let graphql_url = format!(
        "{}://{}/graphql",
        connection_info.scheme(),
        connection_info.host()
    );

    let result = post_graphql_with_token_ex::<ChangeUserType, _>(
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

    FlashMessage::success(format!("Changed user type of {}",params.user_id)).send();

    HttpResponse::Ok().finish()
}
