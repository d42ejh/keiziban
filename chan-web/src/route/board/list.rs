use crate::routes::board::BoardInfo;
use crate::utility::extract_theme_from_session;
use crate::utility::post_graphql_with_token_ex;
use actix_identity::{CookieIdentityPolicy, Identity, IdentityService};
use actix_session::Session;
use actix_web::dev::ConnectionInfo;
use actix_web::guard::Connect;
use actix_web::{get, post, web, App, HttpRequest, HttpResponse, HttpServer, Responder};
use actix_web_flash_messages::{FlashMessage, IncomingFlashMessages};
use askama_actix::{Template, TemplateToResponse};
use chan_graphql_client::{Boards, GraphQLQuery, Response};
use reqwest::{Client, RequestBuilder};
use serde::{Deserialize, Serialize};
use std::error::Error;
use tracing::{event, Id, Level};

#[derive(Template)]
#[template(path = "board_list.html")]
struct BoardListTemplate {
    theme: String,
    flash_messages: IncomingFlashMessages,
    board_infos: Vec<BoardInfo>,
}

#[get("/board_list")]
pub async fn board_list(
    session: Session,
    reqwest_client: web::Data<Client>,
    connection_info: ConnectionInfo,
    flash_messages: IncomingFlashMessages,
    id: Identity,
) -> impl Responder {
    assert!(id.identity().is_some()); //protected route

    let graphql_url = format!(
        "{}://{}/graphql",
        connection_info.scheme(),
        connection_info.host()
    );

    let variables = chan_graphql_client::boards::Variables {
        after: None,
        before: None,
        first: Some(0),
        last: None,
    };
    let result = post_graphql_with_token_ex::<Boards, _>(
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

    let edges = data.boards.edges.as_ref().unwrap();

    event!(Level::DEBUG, "{} edges", edges.len());
    let mut board_infos = Vec::new();
    for edge in edges {
        for n in edge.iter() {
            board_infos.push(BoardInfo {
                name: n.node.name.to_owned(),
                description: n.node.description.to_owned(),
                uuid: n.node.uuid,
                created_at: n.node.created_at,
            });
        }
    }

    BoardListTemplate {
        theme: extract_theme_from_session(&session),
        flash_messages: flash_messages,
        board_infos: board_infos,
    }
    .to_response()
}
