use crate::graphql::ChanSchema;
use crate::graphql::TokenString;
use actix_web::http::header::HeaderMap;
use actix_web::{web, HttpRequest, HttpResponse, Result};
use async_graphql::http::{playground_source, GraphQLPlaygroundConfig};
use async_graphql_actix_web::{GraphQLRequest, GraphQLResponse};
use tracing::{event, Level};

fn get_token_from_headers(headers: &HeaderMap) -> Option<TokenString> {
    headers
        .get("Token")
        .and_then(|value| value.to_str().map(|s| TokenString(s.to_string())).ok())
}

pub async fn index(
    schema: web::Data<ChanSchema>,
    http_request: HttpRequest,
    graphql_req: GraphQLRequest,
) -> GraphQLResponse {
    let mut request = graphql_req.into_inner();
    if let Some(token) = get_token_from_headers(http_request.headers()) {
        event!(Level::DEBUG, "request with the token ",);
        request = request.data(token);
    }
    schema.execute(request).await.into()
}

pub async fn index_playground() -> Result<HttpResponse> {
    let source = playground_source(
        GraphQLPlaygroundConfig::new("/graphql").subscription_endpoint("/graphql"),
    );
    Ok(HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(source))
}
