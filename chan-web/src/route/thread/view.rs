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
use chan_graphql_client::{GraphQLQuery, Response, ThreadById, ThreadPostsRange};
use reqwest::{Client, RequestBuilder};
use serde::{Deserialize, Serialize};
use std::error::Error;
use tracing::{event, Id, Level};
use uuid::Uuid;

#[derive(Template)]
#[template(path = "thread/view.html")]
struct ThreadViewTemplate<'a> {
    theme: String,
    flash_messages: IncomingFlashMessages,
    thread_info: ThreadInfo,
    threadposts: Vec<ThreadPostInfo<'a>>,
}

#[get("/thread/{thread_id}")]
pub async fn thread_view(
    session: Session,
    reqwest_client: web::Data<Client>,
    connection_info: ConnectionInfo,
    flash_messages: IncomingFlashMessages,
    thread_uuid: web::Path<String>,
    id: Identity,
) -> impl Responder {
    assert!(id.identity().is_some()); //protected route
                                      //redirect
    let redirect_url = format!("/thread/{}/1/1000", thread_uuid);
    HttpResponse::Found()
        .append_header((actix_web::http::header::LOCATION, redirect_url))
        .finish()
}

#[derive(Serialize, Deserialize)]
pub struct ThreadRangeViewFormParams {
    pub thread_uuid: Uuid,
    pub l: u16,
    pub r: u16,
}

#[post("/thread_view_range")]
pub async fn thread_view_range_post(
    params: web::Form<ThreadRangeViewFormParams>,
    id: Identity,
) -> impl Responder {
    assert!(id.identity().is_some()); //protected route
                                      //redirect
    let redirect_url = format!("/thread/{}/{}/{}", params.thread_uuid, params.l, params.r);
    HttpResponse::Found()
        .append_header((actix_web::http::header::LOCATION, redirect_url))
        .finish()
}

#[get("/thread/{thread_id}/{l}/{r}")]
pub async fn thread_view_range(
    session: Session,
    reqwest_client: web::Data<Client>,
    connection_info: ConnectionInfo,
    flash_messages: IncomingFlashMessages,
    id: Identity,
    path: web::Path<(String, u16, u16)>,
) -> impl Responder {
    use std::str::FromStr;

    assert!(id.identity().is_some()); //protected route

    let graphql_url = format!(
        "{}://{}/graphql",
        connection_info.scheme(),
        connection_info.host()
    );

    let thread_uuid = match Uuid::from_str(&path.0) {
        Ok(uuid) => uuid,
        Err(e) => {
            FlashMessage::success("Login success!").send();
            return HttpResponse::InternalServerError().body(format!("Invalid id: {}", &path.0));
        }
    };

    let variables = chan_graphql_client::thread_by_id::Variables {
        thread_id: thread_uuid,
    };

    let result = post_graphql_with_token_ex::<ThreadById, _>(
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

    assert!(data.thread.is_some());
    let thread = data.thread.unwrap();
    let start = std::cmp::max(1, path.1) - 1;
    let end = std::cmp::max(1, std::cmp::min(1000, path.2)) - 1;

    //get thread posts
    let variables = chan_graphql_client::thread_posts_range::Variables {
        parent_thread_id: thread_uuid,
        start: Some(start.into()),
        end: Some(end.into()),
    };

    let result = post_graphql_with_token_ex::<ThreadPostsRange, _>(
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
    event!(Level::DEBUG, "posts {:?}", data.threadposts_by_thread_id);
    let mut threadpost_infos = Vec::new();
    for threadpost in &data.threadposts_by_thread_id {
        threadpost_infos.push(ThreadPostInfo {
            number: threadpost.number.try_into().unwrap(),
            body_text: &threadpost.body_text,
            posted_at: &threadpost.posted_at,
            poster_user_id: &threadpost.poster_user_id,
            uuid: &threadpost.uuid,
        });
    }
    ThreadViewTemplate {
        theme: extract_theme_from_session(&session),
        flash_messages: flash_messages,
        thread_info: ThreadInfo {
            title: thread.title,
            uuid: thread.uuid,
            created_at: thread.created_at,
            creator_user_id: thread.creator_user_id,
        },
        threadposts: threadpost_infos,
    }
    .to_response()
}
