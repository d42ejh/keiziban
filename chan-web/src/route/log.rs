use crate::utility::extract_theme_from_session;
use crate::utility::post_graphql_with_token_ex;
use actix_identity::{CookieIdentityPolicy, Identity, IdentityService};
use actix_session::Session;
use actix_web::dev::ConnectionInfo;
use actix_web::guard::Connect;
use actix_web::{get, post, web, App, HttpRequest, HttpResponse, HttpServer, Responder};
use actix_web_flash_messages::{FlashMessage, IncomingFlashMessages};
use askama_actix::{Template, TemplateToResponse};
use chan_graphql_client::{GraphQLQuery, LogsRange, Response};
use reqwest::{Client, RequestBuilder};
use serde::{Deserialize, Serialize};
use std::error::Error;
use tracing::{event, Id, Level};

struct LogInfo<'a> {
    pub message: &'a str,
    pub timestamp: &'a chrono::DateTime<chrono::Utc>,
    pub link: &'a str,
    pub link_title: &'a str,
}

#[derive(Template)]
#[template(path = "log.html")]
struct LogTemplate<'a> {
    theme: String,
    flash_messages: IncomingFlashMessages,
    logs: Vec<LogInfo<'a>>,
}

#[get("/log")]
pub async fn log_view(
    session: Session,
    reqwest_client: web::Data<Client>,
    connection_info: ConnectionInfo,
    flash_messages: IncomingFlashMessages,
    id: Identity,
) -> impl Responder {
    assert!(id.identity().is_some()); //protected route
                                      //redirect
    HttpResponse::Found()
        .append_header((actix_web::http::header::LOCATION, "/log/1/10"))
        .finish()
}

#[get("/log/{l}/{r}")]
pub async fn log_view_range(
    session: Session,
    reqwest_client: web::Data<Client>,
    connection_info: ConnectionInfo,
    flash_messages: IncomingFlashMessages,
    id: Identity,
    path: web::Path<(u32, u32)>,
) -> impl Responder {
    assert!(id.identity().is_some()); //protected route

    let graphql_url = format!(
        "{}://{}/graphql",
        connection_info.scheme(),
        connection_info.host()
    );

    let variables = chan_graphql_client::logs_range::Variables {
        start: Some((path.0 - 1).into()),
        end: Some((path.1 - 1).into()),
    };
    let result = post_graphql_with_token_ex::<LogsRange, _>(
        &reqwest_client,
        graphql_url,
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

    event!(Level::DEBUG, "{} logs", data.logs.len());
    let mut log_infos = Vec::new();
    for log in &data.logs {
        log_infos.push(LogInfo {
            timestamp: &log.timestamp,
            message: &log.message,
            link: match &log.link {
                Some(l) => l,
                None => "",
            },
            link_title: match &log.link_title {
                Some(l) => l,
                None => "",
            },
        });
    }

    LogTemplate {
        theme: extract_theme_from_session(&session),
        flash_messages: flash_messages,
        logs: log_infos,
    }
    .to_response()
}
