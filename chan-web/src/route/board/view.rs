use crate::routes::board::BoardInfo;
use crate::routes::thread::ThreadInfo;
use crate::utility::extract_theme_from_session;
use crate::utility::post_graphql_with_token_ex;
use actix_identity::{CookieIdentityPolicy, Identity, IdentityService};
use actix_session::Session;
use actix_web::dev::ConnectionInfo;
use actix_web::guard::Connect;
use actix_web::{get, post, web, App, HttpRequest, HttpResponse, HttpServer, Responder};
use actix_web_flash_messages::{FlashMessage, IncomingFlashMessages};
use askama_actix::{Template, TemplateToResponse};
use chan_graphql_client::{BoardById, ChildThreadsByBoardId, GraphQLQuery, Response};
use reqwest::{Client, RequestBuilder};
use serde::{Deserialize, Serialize};
use std::error::Error;
use tracing::{event, Id, Level};
use uuid::Uuid;

#[derive(Template)]
#[template(path = "board/view.html")]
struct BoardViewTemplate {
    theme: String,
    flash_messages: IncomingFlashMessages,
    board_info: BoardInfo,
    child_thread_infos: Vec<ThreadInfo>,
}

#[get("/board/{board_id}")]
pub async fn board_view(
    session: Session,
    reqwest_client: web::Data<Client>,
    connection_info: ConnectionInfo,
    flash_messages: IncomingFlashMessages,
    board_id: web::Path<String>,
    id: Identity,
) -> impl Responder {
    use std::str::FromStr;

    assert!(id.identity().is_some()); //protected route

    let graphql_url = format!(
        "{}://{}/graphql",
        connection_info.scheme(),
        connection_info.host()
    );

    let board_uuid = match Uuid::from_str(&board_id) {
        Ok(uuid) => uuid,
        Err(e) => {
            FlashMessage::success("Login success!").send();
            return HttpResponse::InternalServerError().body(format!("Invalid id: {}", board_id));
        }
    };

    //get board
    let variables = chan_graphql_client::board_by_id::Variables {
        board_id: board_uuid,
    };

    let result = post_graphql_with_token_ex::<BoardById, _>(
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

    assert!(data.board.is_some());
    let board = data.board.unwrap();

    //get child thread infos
    let variables = chan_graphql_client::child_threads_by_board_id::Variables {
        board_id: board_uuid,
    };

    let result = post_graphql_with_token_ex::<ChildThreadsByBoardId, _>(
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

    let mut child_thread_infos = Vec::new();
    for thread in &data.threads {
        child_thread_infos.push(ThreadInfo {
            title: thread.title.to_owned(),
            uuid: thread.uuid,
            created_at: thread.created_at,
            creator_user_id: thread.creator_user_id.to_owned(),
        });
    }
    BoardViewTemplate {
        theme: extract_theme_from_session(&session),
        flash_messages: flash_messages,
        board_info: BoardInfo {
            name: board.name,
            description: board.description,
            uuid: board.uuid,
            created_at: board.created_at,
        },
        child_thread_infos: child_thread_infos,
    }
    .to_response()
}
