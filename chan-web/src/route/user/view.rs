use crate::routes::thread::ThreadInfo;
use crate::routes::threadpost::ThreadPostInfo;
use crate::utility::extract_theme_from_session;
use crate::utility::post_graphql_with_token_ex;
use actix_identity::{CookieIdentityPolicy, Identity, IdentityService};
use actix_session::Session;
use actix_web::dev::ConnectionInfo;
use actix_web::guard::Connect;
use actix_web::{get, post, web, App, HttpRequest, HttpResponse, HttpServer, Responder};
use actix_web_flash_messages::{FlashMessage, IncomingFlashMessages};
use askama_actix::{Template, TemplateToResponse};
use chan_core::model::{UserStatus, UserType};
use chan_graphql_client::{GraphQLQuery, Response, ThreadPostsRange, UserById};
use reqwest::{Client, RequestBuilder};
use serde::{Deserialize, Serialize};
use std::error::Error;
use tracing::{event, Id, Level};
use uuid::Uuid;

use super::UserInfo;

#[derive(Template)]
#[template(path = "user/view.html")]
struct UserViewTemplate<'a> {
    theme: String,
    flash_messages: IncomingFlashMessages,
    user_info: UserInfo<'a>,
}

#[get("/user/{user_id}")]
pub async fn user_view(
    session: Session,
    reqwest_client: web::Data<Client>,
    connection_info: ConnectionInfo,
    flash_messages: IncomingFlashMessages,
    user_id: web::Path<String>,
    id: Identity,
) -> impl Responder {
    use std::str::FromStr;

    assert!(id.identity().is_some()); //protected route

    let graphql_url = format!(
        "{}://{}/graphql",
        connection_info.scheme(),
        connection_info.host()
    );

    let variables = chan_graphql_client::user_by_id::Variables {
        user_id: user_id.to_owned(),
    };

    let result = post_graphql_with_token_ex::<UserById, _>(
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
    assert!(data.user.is_some());
    let user = data.user.unwrap();
    let user_status = match UserStatus::from_i32(user.user_status.try_into().unwrap()) {
        Ok(s) => s,
        Err(e) => {
            return HttpResponse::InternalServerError().body(e.to_string());
        }
    };

    let user_type = match UserType::from_i32(user.user_type.try_into().unwrap()) {
        Ok(t) => t,
        Err(e) => {
            return HttpResponse::InternalServerError().body(e.to_string());
        }
    };
    UserViewTemplate {
        theme: extract_theme_from_session(&session),
        flash_messages: flash_messages,
        user_info: UserInfo {
            registered_at: &user.registered_at,
            //todo maybe make these matches to function
            user_status: match user_status {
                UserStatus::Normal => "Normal",
                UserStatus::Banned => "Banned",
                UserStatus::Suspended => "Suspended",
                UserStatus::Removed => "Removed",
            },
            user_type: match user_type {
                UserType::Admin => "Admin",
                UserType::Moderator => "Moderator",
                UserType::Normal => "Normal",
            },
            id: &user.id,
        },
    }
    .to_response()
}
