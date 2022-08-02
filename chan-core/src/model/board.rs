use std::str::FromStr;

use crate::model::Thread;
use crate::schema::boards::{self};
use crate::DBPool;
use async_graphql::{Error, Result, SimpleObject};
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use diesel::{Insertable, Queryable};
use tantivy::{
    collector::{Count, TopDocs},
    query::{FuzzyTermQuery, QueryParser},
    Index, IndexWriter, ReloadPolicy, Term,
};
use tracing::{event, Level};
use uuid::Uuid;

use super::ThreadPost;

#[derive(Queryable, SimpleObject, Clone)]
pub struct Board {
    #[graphql(skip)]
    primary_key: i32,
    pub uuid: Uuid,
    pub created_at: DateTime<Utc>,
    pub name: String,
    pub description: String,
}

impl Board {
    /// Create a new board and insert it to DB.
    /// Returns the created board.
    pub fn create_new(db_pool: &DBPool, board_name: &str, board_description: &str) -> Result<Self> {
        let new_board = NewBoard {
            uuid: &Uuid::new_v4(),
            created_at: &Utc::now(),
            name: board_name,
            description: board_description,
        };
        let created_board = diesel::insert_into(boards::table)
            .values(&new_board)
            .get_result::<Board>(&db_pool.get()?)?;
        Ok(created_board)
    }

    pub fn select_by_uuid(db_pool: &DBPool, board_uuid: &Uuid) -> Result<Option<Self>> {
        use crate::schema::boards::dsl::*;
        //check if a thread with the uuid is exists
        let board_vec = boards
            .filter(uuid.eq(board_uuid))
            .limit(1)
            .load::<Board>(&db_pool.get()?)?;
        if board_vec.len() != 1 {
            debug_assert_eq!(board_vec.len(), 0);
            return Ok(None);
        }
        Ok(Some(board_vec[0].clone()))
    }

    /*
    pub fn remove_by_uuid(
        db_pool: &DBPool,
        index_writer: &mut IndexWriter,
        board_uuid: &Uuid,
    ) -> Result<()> {
        use crate::schema::boards::dsl::*;
        let db_connection = db_pool.get()?;
        db_connection.build_transaction().run::<_, Error, _>(|| {
            diesel::delete(boards.filter(uuid.eq(board_uuid))).execute(&db_connection)?;
            let index = index_writer.index();
            let schema = index.schema();
            let uuid_field = schema.get_field("board_uuid").unwrap();
            let term = Term::from_field_bytes(uuid_field, board_uuid.as_bytes());
            index_writer.delete_term(term);
            index_writer.commit()?;
            Ok(())
        })?;
        Ok(())
    }
    */

    pub fn search_by_keyword(
        db_pool: &DBPool,
        index: &Index,
        search_keyword: &str,
    ) -> Result<Vec<Board>> {
        event!(Level::DEBUG, "search by keyword");

        let schema = index.schema();
        let board_name = schema.get_field("board_name").unwrap();
        let board_description = schema.get_field("board_description").unwrap();
        let board_uuid = schema.get_field("board_uuid").unwrap();

        let mut ret_boards = Vec::new();

        let reader = index
            .reader_builder()
            .reload_policy(ReloadPolicy::OnCommit) //TODO examine
            .try_into()?; //todo use with context

        {
            let searcher = reader.searcher();
            let query_parser = QueryParser::for_index(&index, vec![board_name, board_description]);
            {
                let term = Term::from_field_text(board_name, search_keyword);
                let query = FuzzyTermQuery::new(term, 2, true);
                let (top_docs, count) =
                    searcher.search(&query, &(TopDocs::with_limit(20), Count))?;
                event!(Level::DEBUG, "Search OK");
                for (score, doc_address) in top_docs {
                    let doc = searcher.doc(doc_address)?;
                    let opt = doc.get_first(board_uuid);
                    assert!(opt.is_some());
                    let board_uuid_value = opt.unwrap();

                    debug_assert!(board_uuid_value.as_text().is_some());

                    let board_uuid = Uuid::from_str(board_uuid_value.as_text().unwrap())?;
                    if let Some(board) = Board::select_by_uuid(db_pool, &board_uuid)? {
                        ret_boards.push(board);
                    }
                    /*
                     for v in doc.get_all(board_name) {
                        event!(Level::DEBUG, "Search result {:?} score: {}", v,score);
                    }
                    */
                }
            }

            //todo https://github.com/quickwit-oss/tantivy/blob/main/examples/basic_search.rs
            //todo!();
        }
        Ok(ret_boards)
    }

    pub fn pagination_query(db_pool: &DBPool, limit: i64, offset: i64) -> Result<Vec<Board>> {
        use crate::schema::boards::dsl::boards;
        Ok(boards
            .limit(limit)
            .offset(offset)
            .load::<Board>(&db_pool.get()?)?)
    }

    pub fn count(db_pool: &DBPool) -> Result<usize> {
        use crate::schema::boards::dsl::boards;
        let count: i64 = boards.count().get_result(&db_pool.get()?)?;
        Ok(count as usize)
    }

    pub fn child_threads(db_pool: &DBPool, parent_board_uuid: &Uuid) -> Result<Vec<Thread>> {
        use crate::schema::threads::dsl::*;
        Ok(threads
            .filter(parent_board_id.eq(parent_board_uuid))
            .load::<Thread>(&db_pool.get()?)?)
    }
}

/// diesel model
#[derive(Insertable)]
#[table_name = "boards"]
struct NewBoard<'a> {
    pub uuid: &'a Uuid,
    pub created_at: &'a DateTime<Utc>,
    pub name: &'a str,
    pub description: &'a str,
}
