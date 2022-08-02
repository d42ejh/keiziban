use crate::model::{
    verify_token, Board, Log, SystemInfo, SystemInfoContext, Thread, ThreadPost, User, UserType,
};
use crate::DBPool;
//use async_graphql::*;
use async_graphql::{
    connection::{query, Connection, Edge, EmptyFields},
    Context, EmptySubscription, Object, Result, Schema, SimpleObject,
};
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, PooledConnection};
use diesel::PgConnection;
use std::sync::Arc;
use tantivy::{
    collector::TopDocs, query::FuzzyTermQuery, query::QueryParser, DocAddress, DocId, Document,
    Index, IndexReader, IndexWriter, Score,
};
use tokio::sync::RwLock;
use tracing::{event, Level};
use uuid::Uuid;

/*DB types */
pub type DBConnection = PooledConnection<ConnectionManager<PgConnection>>;
pub type ChanSchema = Schema<QueryRoot, MutationRoot, EmptySubscription>;

pub struct TokenString(pub String);

#[derive(SimpleObject)]
struct SearchResult {
    object_type: String,
    uuid: Uuid,
    score: Option<f32>,
}

pub struct QueryRoot;

#[Object]
impl QueryRoot {
    /// SystemInfo
    async fn system_info(&self, context: &Context<'_>) -> Result<SystemInfo> {
        let db_pool = context.data::<DBPool>()?;
        let token = context.data::<TokenString>()?;
        //verify token
        verify_token(&db_pool, &token.0)?;

        let sys_info_ctx = context.data::<Arc<RwLock<SystemInfoContext>>>()?;
        let info;
        {
            let mut sys = sys_info_ctx.write().await;
            info = sys.get_info()?;
        }
        Ok(info)
    }

    /// Find user by ID.
    async fn user(&self, context: &Context<'_>, user_id: String) -> Result<Option<User>> {
        let db_pool = context.data::<DBPool>()?;

        let token = context.data::<TokenString>()?;
        //verify token
        verify_token(&db_pool, &token.0)?;

        User::select_by_user_id(&db_pool, &user_id)
    }

    /// Find board by ID.
    async fn board(&self, context: &Context<'_>, board_id: Uuid) -> Result<Option<Board>> {
        let db_pool = context.data::<DBPool>()?;

        let token = context.data::<TokenString>()?;
        //verify token
        verify_token(&db_pool, &token.0)?;
        Board::select_by_uuid(&db_pool, &board_id)
    }

    async fn boards(
        &self,
        context: &Context<'_>,
        after: Option<String>,
        before: Option<String>,
        first: Option<i32>,
        last: Option<i32>,
    ) -> Result<Connection<usize, Board, EmptyFields, EmptyFields>> {
        let db_pool = context.data::<DBPool>()?;
        let token = context.data::<TokenString>()?;

        verify_token(&db_pool, &token.0)?;

        query(
            after,
            before,
            first,
            last,
            |after, before, first, last| async move {
                let start = after.map(|after| after + 1).unwrap_or(0);
                let board_count = Board::count(&db_pool).unwrap_or(100);
                let mut end = Board::count(&db_pool).unwrap_or(100) + 1;

                if let Some(before) = before {
                    if before == 0 {
                        return Ok(Connection::new(false, false));
                    }
                    end = before;
                }

                event!(Level::DEBUG, "Fetch board range {} ~ {}", start, end);

                //fetch boards
                let boards = Board::pagination_query(
                    &db_pool,
                    //std::cmp::max(0,end-start-1).try_into().unwrap(),
                    (end - start - 1).try_into().unwrap(),
                    start.try_into().unwrap(),
                )?;
                let mut connection = Connection::new(start > 0, end < board_count);
                connection.append(
                    boards
                        .into_iter()
                        .enumerate()
                        .map(|(index, board)| Edge::new(start + index, board)),
                );
                Ok::<_, async_graphql::Error>(connection)
            },
        )
        .await
    }

    // Search boards by keyword.
    // TODO: better function name
    async fn boards_by_keyword(
        &self,
        context: &Context<'_>,
        search_keyword: String,
    ) -> Result<Vec<Board>> {
        let db_pool = context.data::<DBPool>()?;
        let token = context.data::<TokenString>()?;

        //verify token
        verify_token(&db_pool, &token.0)?;

        //do search
        let index = context.data::<Index>()?;
        let boards = Board::search_by_keyword(&db_pool, &index, &search_keyword)?;

        Ok(boards)
    }

    async fn threads(&self, context: &Context<'_>, board_id: Uuid) -> Result<Vec<Thread>> {
        let db_pool = context.data::<DBPool>()?;
        let token = context.data::<TokenString>()?;

        //verify token
        verify_token(&db_pool, &token.0)?;
        let threads = Board::child_threads(&db_pool, &board_id);
        threads
    }

    /// Find thread by ID.
    async fn thread(&self, context: &Context<'_>, thread_id: Uuid) -> Result<Option<Thread>> {
        let db_pool = context.data::<DBPool>()?;

        let token = context.data::<TokenString>()?;
        //verify token
        verify_token(&db_pool, &token.0)?;
        Thread::select_by_uuid(&db_pool, &thread_id)
    }

    async fn threadposts_by_thread_id(
        &self,
        context: &Context<'_>,
        parent_thread_id: Uuid,
        start: Option<i32>,
        end: Option<i32>,
    ) -> Result<Vec<ThreadPost>> {
        let db_pool = context.data::<DBPool>()?;

        let token = context.data::<TokenString>()?;
        //verify token
        verify_token(&db_pool, &token.0)?;

        let start = start.unwrap_or(0);
        let end = end.unwrap_or(1000);
        Thread::thread_post_range(&db_pool, &parent_thread_id, start.into(), end.into())
    }

    async fn logs(
        &self,
        context: &Context<'_>,
        start: Option<i32>,
        end: Option<i32>,
    ) -> Result<Vec<Log>> {
        let db_pool = context.data::<DBPool>()?;

        let token = context.data::<TokenString>()?;
        //verify token
        verify_token(&db_pool, &token.0)?;
        let start = start.unwrap_or(0);
        let end = end.unwrap_or(1000);
        Log::range(&db_pool, start.into(), end.into())
    }

    async fn search_top_k(
        &self,
        context: &Context<'_>,
        keyword: String,
        k: i32,
        search_thread: bool,
        search_threadpost: bool,
    ) -> Result<Vec<SearchResult>> {
        let db_pool = context.data::<DBPool>()?;
        let token = context.data::<TokenString>()?;
        //verify token
        verify_token(&db_pool, &token.0)?;

        if !search_thread && !search_threadpost {
            return Err(async_graphql::Error::new(
                "Invalid search target combination.",
            ));
        }

        let index_reader = context.data::<IndexReader>()?;
        let searcher = index_reader.searcher();
        let index = searcher.index();
        let schema = index.schema();

        let mut target_fields = Vec::new();
        if search_thread {
            target_fields.push(schema.get_field("thread_title").unwrap());
        }
        if search_threadpost {
            target_fields.push(schema.get_field("threadpost_body_text").unwrap());
        }
        debug_assert!(target_fields.len() != 0);
        let query_parser = QueryParser::for_index(&index, target_fields);
        let query = query_parser.parse_query(&keyword)?;

        let top_docs: Vec<(Score, DocAddress)> =
            searcher.search(&query, &TopDocs::with_limit(k as usize))?;
        let mut results: Vec<SearchResult> = Vec::new();
        for (score, doc_address) in top_docs {
            let retrived_doc = searcher.doc(doc_address)?;
            let doc = schema.to_named_doc(&retrived_doc);

            let mut push_result = |key: &str| -> Result<()> {
                if let Some(vals) = doc.0.get(key) {
                    if vals.len() != 1 {
                        unreachable!();
                    }
                    let id = match vals[0].as_bytes() {
                        Some(id) => id,
                        None => {
                            unreachable!();
                        }
                    };
                    results.push(SearchResult {
                        object_type: key.to_owned(),
                        uuid: Uuid::from_slice(id)?,
                        score: Some(score),
                    });

                    event!(Level::DEBUG, "{} {:?} score {}", key, id, score);
                }
                Ok(())
            };
            push_result("thread_uuid")?;
            push_result("threadpost_uuid")?;
        }
        event!(Level::DEBUG, "Got {} search results", results.len());

        Ok(results)
    }
}

pub struct MutationRoot;

#[Object]
impl MutationRoot {
    async fn sign_up(&self, context: &Context<'_>, password: String) -> Result<User> {
        let db_pool = context.data::<DBPool>()?;

        //sign up
        let created_user = User::create_new(&db_pool, UserType::Normal, &password)?;

        //log
        Log::create_new(
            &db_pool,
            &format!("{} joined the network...", created_user.id),
            None,
            None,
        )?;

        Ok(created_user)
    }

    async fn login(
        &self,
        context: &Context<'_>,
        user_id: String,
        password: String,
    ) -> Result<String> {
        let db_pool = context.data::<DBPool>()?;

        //try login
        let token = User::login(&db_pool, &user_id, &password)?;
        Ok(token)
    }

    /// Create a new board.
    /// Returns new board's id.
    /// TODO: maybe return error when indexing is failed(also delete the created board)
    /*
    async fn create_board(
        &self,
        context: &Context<'_>,
        board_name: String,
        board_description: String,
    ) -> Result<Uuid> {
        use tantivy::doc;

        let db_pool = context.data::<DBPool>()?;
        let token = context.data::<TokenString>()?;

        //verify token
        let user_id = verify_token(&db_pool, &token.0)?;

        let new_board = Board::create_new(&db_pool, &user_id, &board_name, &board_description)?;

        //index the created board
        let index = context.data::<Index>()?;

        let schema = index.schema();
        let board_name = schema.get_field("board_name").unwrap();
        let board_uuid = schema.get_field("board_uuid").unwrap();
        let board_description = schema.get_field("board_description").unwrap();

        let mut index_writer = index.writer(10_000_000)?;
        index_writer.add_document(doc!(
            board_name=>new_board.name,
            board_uuid=>new_board.uuid.to_string(),
            board_description=>new_board.description
        ))?;

        //commit the change
        index_writer.commit()?;

        Ok(new_board.uuid)
    }*/

    async fn create_thread(
        &self,
        context: &Context<'_>,
        thread_title: String,
        parent_board_uuid: Uuid,
        first_post_text: String,
    ) -> Result<Uuid> {
        let db_pool = context.data::<DBPool>()?;
        let token = context.data::<TokenString>()?;
        //verify token
        let user_id = verify_token(&db_pool, &token.0)?;

        let index_writer = context.data::<Arc<RwLock<IndexWriter>>>()?;

        let new_thread;
        {
            let mut index_writer = index_writer.write().await;
            //create new thread
            new_thread = Thread::create_new(
                &db_pool,
                &mut index_writer,
                &user_id,
                &thread_title,
                &parent_board_uuid,
                &first_post_text,
            )?;
        }

        let thread_link = format!("/thread/{}", new_thread.uuid);
        //log
        Log::create_new(
            &db_pool,
            &format!("{} created a new thread.", user_id),
            Some(&thread_link),
            Some(&new_thread.title),
        )?;

        Ok(new_thread.uuid)
    }

    // i32 is dummy
    /// Only for admin and moderator
    async fn remove_thread(&self, context: &Context<'_>, thread_uuid: Uuid) -> Result<i32> {
        let db_pool = context.data::<DBPool>()?;
        let token = context.data::<TokenString>()?;
        //verify token

        let user_id = verify_token(&db_pool, &token.0)?;
        //check user type
        let user = User::select_by_user_id(&db_pool, &user_id)?;
        if user.is_none() {
            return Err(async_graphql::Error::new("User does not exist"));
        }
        let user = user.unwrap();
        let user_type = UserType::from_i32(user.user_type)?;
        if user_type != UserType::Admin && user_type != UserType::Moderator {
            return Err(async_graphql::Error::new("Not allowed."));
        }

        assert!(user_type == UserType::Admin || user_type == UserType::Moderator);
        let index_writer = context.data::<Arc<RwLock<IndexWriter>>>()?;
        {
            let mut index_writer = index_writer.write().await;
            Thread::remove_by_uuid(&db_pool, &mut index_writer, &thread_uuid)?;
        }
        Ok(0x69) //return dummy
    }

    async fn post_threadpost(
        &self,
        context: &Context<'_>,
        thread_uuid: Uuid,
        post_body: String,
    ) -> Result<Uuid> {
        let db_pool = context.data::<DBPool>()?;
        let token = context.data::<TokenString>()?;

        //verify token
        let poster_user_id = verify_token(&db_pool, &token.0)?;

        let index_writer = context.data::<Arc<RwLock<IndexWriter>>>()?;
        //TODO: check whether the parent thread is already full or not.

        //create new thread post
        let new_threadpost;
        {
            let mut index_writer = index_writer.write().await;
            new_threadpost = ThreadPost::create_new(
                &db_pool,
                &mut index_writer,
                &poster_user_id,
                &thread_uuid,
                &post_body,
            )?;
        }
        //insert?

        Ok(new_threadpost.uuid)
    }

    // i32 is dummy
    /// Only for admin and moderator
    async fn remove_threadpost(&self, context: &Context<'_>, threadpost_uuid: Uuid) -> Result<i32> {
        let db_pool = context.data::<DBPool>()?;
        let token = context.data::<TokenString>()?;
        //verify token

        let user_id = verify_token(&db_pool, &token.0)?;
        //check user type
        let user = User::select_by_user_id(&db_pool, &user_id)?;
        if user.is_none() {
            return Err(async_graphql::Error::new("User does not exist"));
        }
        let user = user.unwrap();
        let user_type = UserType::from_i32(user.user_type)?;
        if user_type != UserType::Admin && user_type != UserType::Moderator {
            return Err(async_graphql::Error::new("Not allowed."));
        }

        assert!(user_type == UserType::Admin || user_type == UserType::Moderator);
        let index_writer = context.data::<Arc<RwLock<IndexWriter>>>()?;
        {
            let mut index_writer = index_writer.write().await;
            ThreadPost::remove_by_uuid(&db_pool, &mut index_writer, &threadpost_uuid)?;
        }
        Ok(0x69) //return dummy
    }

    /// Only for admin
    /// Return value is dummy
    async fn change_user_type(
        &self,
        context: &Context<'_>,
        user_id: String,
        new_type: i32,
    ) -> Result<i32> {
        let db_pool = context.data::<DBPool>()?;
        let token = context.data::<TokenString>()?;
        //verify token

        let issuer_user_id = verify_token(&db_pool, &token.0)?;

        //check issuer user type
        let issuer_user = User::select_by_user_id(&db_pool, &issuer_user_id)?;
        if issuer_user.is_none() {
            return Err(async_graphql::Error::new("User does not exist"));
        }
        let issuer_user = issuer_user.unwrap();
        let user_type = UserType::from_i32(issuer_user.user_type)?;
        if user_type != UserType::Admin {
            return Err(async_graphql::Error::new("Not allowed."));
        }
        assert_eq!(user_type, UserType::Admin);
        let new_type = UserType::from_i32(new_type)?;
        if new_type == UserType::Admin {
            return Err(async_graphql::Error::new("Not allowed!"));
        }

        User::change_type(&db_pool, &user_id, new_type)?;

        Ok(0x69)
    }
}
