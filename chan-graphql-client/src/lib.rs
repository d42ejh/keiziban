pub use graphql_client::{GraphQLQuery, Response};
use uuid::Uuid;

type UUID = Uuid;
type DateTime = chrono::DateTime<chrono::Utc>;

// The paths are relative to the directory where your `Cargo.toml` is located.
// Both json and the GraphQL schema language are supported as sources for the schema
#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "schema.graphql",
    query_path = "query/registration.graphql",
    response_derives = "Debug"
)]
pub struct RegisterAccount;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "schema.graphql",
    query_path = "query/login.graphql",
    response_derives = "Debug"
)]
pub struct Login;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "schema.graphql",
    query_path = "query/search_top_k.graphql",
    response_derives = "Debug"
)]
pub struct SearchTopK;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "schema.graphql",
    query_path = "query/boards.graphql",
    response_derives = "Debug"
)]
pub struct Boards;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "schema.graphql",
    query_path = "query/board_by_id.graphql",
    response_derives = "Debug"
)]
pub struct BoardById;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "schema.graphql",
    query_path = "query/thread_by_id.graphql",
    response_derives = "Debug"
)]
pub struct ThreadById;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "schema.graphql",
    query_path = "query/child_threads_by_board_id.graphql",
    response_derives = "Debug"
)]
pub struct ChildThreadsByBoardId;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "schema.graphql",
    query_path = "query/create_thread.graphql",
    response_derives = "Debug"
)]
pub struct CreateThread;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "schema.graphql",
    query_path = "query/threadposts_range.graphql",
    response_derives = "Debug"
)]
pub struct ThreadPostsRange;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "schema.graphql",
    query_path = "query/post_threadpost.graphql",
    response_derives = "Debug"
)]
pub struct PostThreadPost;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "schema.graphql",
    query_path = "query/user_by_id.graphql",
    response_derives = "Debug"
)]
pub struct UserById;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "schema.graphql",
    query_path = "query/remove_thread.graphql",
    response_derives = "Debug"
)]
pub struct RemoveThread;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "schema.graphql",
    query_path = "query/remove_threadpost.graphql",
    response_derives = "Debug"
)]
pub struct RemoveThreadPost;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "schema.graphql",
    query_path = "query/logs_range.graphql",
    response_derives = "Debug"
)]
pub struct LogsRange;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "schema.graphql",
    query_path = "query/get_system_info.graphql",
    response_derives = "Debug"
)]
pub struct GetSystemInfo;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "schema.graphql",
    query_path = "query/change_user_type.graphql",
    response_derives = "Debug"
)]
pub struct ChangeUserType;
#[cfg(test)]
mod tests {}
