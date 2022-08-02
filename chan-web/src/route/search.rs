use crate::utility::extract_theme_from_session;
use crate::utility::post_graphql_with_token_ex;
use actix_identity::{CookieIdentityPolicy, Identity, IdentityService};
use actix_session::Session;
use actix_web::dev::ConnectionInfo;
use actix_web::guard::Connect;
use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use actix_web_flash_messages::{FlashMessage, IncomingFlashMessages};
use askama_actix::{Template, TemplateToResponse};
use chan_graphql_client::{GraphQLQuery, Response, SearchTopK};
use reqwest::{Client, RequestBuilder};
use serde::{Deserialize, Serialize};
use std::error::Error;
use tracing::{event, Id, Level};

#[derive(Debug)]
struct SearchResult<'a> {
    score: String,
    object_type: &'a str,
    uuid: &'a uuid::Uuid,
}

#[derive(Template)]
#[template(path = "search_result.html")]
struct SearchResultTemplate<'a> {
    theme: String,
    flash_messages: IncomingFlashMessages,
    search_results: Vec<SearchResult<'a>>,
}

#[derive(Template)]
#[template(path = "search.html")]
struct SearchTemplate {
    theme: String,
    flash_messages: IncomingFlashMessages,
}

#[get("/search")]
pub async fn search(session: Session, flash_messages: IncomingFlashMessages) -> impl Responder {
    SearchTemplate {
        theme: extract_theme_from_session(&session),
        flash_messages: flash_messages,
    }
    .to_response()
}

#[derive(Serialize, Deserialize)]
pub struct SearchFormParams {
    pub keyword: String,
}

#[post("/search")]
pub async fn search_handler(
    params: web::Form<SearchFormParams>,
    reqwest_client: web::Data<Client>,
    connection_info: ConnectionInfo,
    session: Session,
    id: Identity,
    flash_messages: IncomingFlashMessages,
) -> impl Responder {
    assert!(id.identity().is_some()); //protected route

    let variables = chan_graphql_client::search_top_k::Variables {
        search_keyword: params.keyword.to_owned(),
        search_thread: true,
        search_threadpost: true,
        k: 25,
    };

    let graphql_url = format!(
        "{}://{}/graphql",
        connection_info.scheme(),
        connection_info.host()
    );

    let result = post_graphql_with_token_ex::<SearchTopK, _>(
        &reqwest_client,
        &graphql_url,
        variables,
        &id.identity().unwrap(),
    )
    .await;
    if result.is_err() {
        let error = result.as_ref().err().unwrap();
        return HttpResponse::InternalServerError().body(error.to_string());
    }

    let data = result.unwrap();
    let mut search_results = Vec::new();
    for result in &data.search_top_k {
        event!(Level::DEBUG, "{:?}", result);
        search_results.push(SearchResult {
            score: match result.score {
                Some(s) => s.to_string(),
                None => String::new(),
            },
            object_type: &result.object_type,
            uuid: &result.uuid,
        })
    }

    FlashMessage::success("TODO!").send();

    SearchResultTemplate {
        theme: extract_theme_from_session(&session),
        flash_messages: flash_messages,
        search_results: search_results,
    }
    .to_response()
    //todo show search result
    /*
    HttpResponse::Found()
        .append_header((actix_web::http::header::LOCATION, "/search"))
        .finish()
        */
}
