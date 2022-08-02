use crate::utility::extract_theme_from_session;
use crate::utility::post_graphql_with_token_ex;
use actix_identity::Identity;
use actix_session::Session;
use actix_web::dev::ConnectionInfo;
use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use actix_web_flash_messages::IncomingFlashMessages;
use askama_actix::{Template, TemplateToResponse};
use chan_core::model::SystemInfo;
use chan_graphql_client::{get_system_info, GetSystemInfo, GraphQLQuery, Response};
use reqwest::{Client, RequestBuilder};

#[derive(Template)]
#[template(path = "system_info.html")]
struct SystemInfoTemplate {
    theme: String,
    flash_messages: IncomingFlashMessages,
    system_info: SystemInfo,
}

#[get("/system_info")]
pub async fn system_info(
    session: Session,
    reqwest_client: web::Data<Client>,
    flash_messages: IncomingFlashMessages,
    connection_info: ConnectionInfo,
    id: Identity,
) -> impl Responder {
    assert!(id.identity().is_some());

    let variables = get_system_info::Variables {};

    let graphql_url = format!(
        "{}://{}/graphql",
        connection_info.scheme(),
        connection_info.host()
    );

    let result = post_graphql_with_token_ex::<GetSystemInfo, _>(
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
    let system_info = data.system_info;

    SystemInfoTemplate {
        theme: extract_theme_from_session(&session),
        flash_messages: flash_messages,
        system_info: SystemInfo {
            free_mem: (system_info.free_mem / (1024 * 1024)) as usize, //mb
            total_mem_available: (system_info.total_mem_available / (1024 * 1024)) as usize,
        },
    }
    .to_response()
}
