use crate::constant::THEME_SESSION_KEY;
use actix_session::Session;
use actix_web::HttpResponse;
use chan_graphql_client::GraphQLQuery;
use graphql_client::Response;

pub fn extract_theme_from_session(session: &Session) -> String {
    let result = session.get::<String>(THEME_SESSION_KEY);
    if result.is_err() {
        return "default".to_string();
    }
    let opt = result.unwrap();
    if opt.is_none() {
        return "default".to_string();
    }
    opt.unwrap()
}

pub async fn post_graphql_with_token_ex<Q: GraphQLQuery, U: reqwest::IntoUrl>(
    client: &reqwest::Client,
    url: U,
    variables: Q::Variables,
    token_string: &str,
) -> anyhow::Result<Q::ResponseData> {
    use anyhow::anyhow;
    let response = match post_graphql_with_token::<Q, _>(client, url, variables, token_string).await
    {
        Ok(response) => response,
        Err(e) => {
            return Err(anyhow!(format!("GraphQL returned an error: {:?}", e)));
        }
    };

    if let Some(errors) = response.errors {
        let mut error_str = String::from("GraphQL response errors\n");
        for error in &errors {
            error_str.push_str(&format!("{}\n", error));
        }
        return Err(anyhow!(error_str));
    }

    assert!(response.data.is_some());
    Ok(response.data.unwrap())
}

async fn post_graphql_with_token<Q: GraphQLQuery, U: reqwest::IntoUrl>(
    client: &reqwest::Client,
    url: U,
    variables: Q::Variables,
    token_string: &str,
) -> Result<Response<Q::ResponseData>, reqwest::Error> {
    let body = Q::build_query(variables);
    let reqwest_response = client
        .post(url)
        .header("Token", token_string)
        .json(&body)
        .send()
        .await?;

    Ok(reqwest_response.json().await?)
}
