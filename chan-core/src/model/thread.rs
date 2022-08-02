use crate::model::{threadpost::NewThreadPost, ThreadPost};
use crate::schema::threads;
use crate::DBPool;
use async_graphql::{Error, Result, SimpleObject};
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use diesel::{Insertable, Queryable};
use tantivy::{doc, Index, IndexReader, IndexWriter, Term};
use uuid::Uuid;

#[derive(Queryable, SimpleObject, Clone)]
pub struct Thread {
    #[graphql(skip)]
    primary_key: i32,
    pub uuid: Uuid,
    pub created_at: DateTime<Utc>,
    pub parent_board_id: Uuid,
    pub title: String,
    pub creator_user_id: String,
}

impl Thread {
    /// Create a new thread and insert it to DB.
    /// Returns the created thread.
    pub fn create_new(
        db_pool: &DBPool,
        index_writer: &mut IndexWriter,
        creator_user_id: &str,
        thread_title: &str,
        parent_board_uuid: &Uuid,
        first_post_text: &str,
    ) -> Result<Self> {
        let new_thread = NewThread {
            uuid: &Uuid::new_v4(),
            created_at: &Utc::now(),
            parent_board_id: parent_board_uuid,
            title: thread_title,
            creator_user_id: creator_user_id,
        };

        let db_connection = db_pool.get()?;
        let created_thread = db_connection.build_transaction().run::<_, Error, _>(|| {
            let created_thread = diesel::insert_into(threads::table)
                .values(&new_thread)
                .get_result::<Thread>(&db_pool.get()?)?;
            let index = index_writer.index();
            let schema = index.schema();

            let title_field = schema.get_field("thread_title").unwrap();
            let uuid_field = schema.get_field("thread_uuid").unwrap();

            index_writer.add_document(doc!(
                title_field=>thread_title,
                uuid_field=>created_thread.uuid.as_bytes().as_slice()
            ))?;

            //first post
            let new_threadpost = NewThreadPost {
                uuid: &Uuid::new_v4(),
                number: 1, //first
                posted_at: &Utc::now(),
                poster_user_id: creator_user_id,
                parent_thread_id: &created_thread.uuid,
                body_text: first_post_text,
            };

            use crate::schema::threadposts;
            let created_threadpost = diesel::insert_into(threadposts::table)
                .values(&new_threadpost)
                .get_result::<ThreadPost>(&db_pool.get()?)?;
            let index = index_writer.index();
            let schema = index.schema();

            let body_text_field = schema.get_field("threadpost_body_text").unwrap();
            let uuid_field = schema.get_field("threadpost_uuid").unwrap();

            index_writer.add_document(doc!(
                body_text_field=>first_post_text,
                uuid_field=>created_threadpost.uuid.as_bytes().as_slice()
            ))?;

            index_writer.commit()?;

            Ok(created_thread)
        })?;

        Ok(created_thread)
    }

    pub fn select_by_uuid(db_pool: &DBPool, thread_uuid: &Uuid) -> Result<Option<Self>> {
        use crate::schema::threads::dsl::*;
        //check if a thread with the uuid is exist
        let thread_vec = threads
            .filter(uuid.eq(thread_uuid))
            .limit(1)
            .load::<Thread>(&db_pool.get()?)?;
        if thread_vec.len() != 1 {
            debug_assert_eq!(thread_vec.len(), 0);
            return Ok(None); //not found
        }
        Ok(Some(thread_vec[0].clone()))
    }

    pub fn remove_by_uuid(
        db_pool: &DBPool,
        index_writer: &mut IndexWriter,
        thread_uuid: &Uuid,
    ) -> Result<()> {
        use crate::schema::threads::dsl::*;
        let db_connection = db_pool.get()?;
        db_connection.build_transaction().run::<_, Error, _>(|| {
            diesel::delete(threads.filter(uuid.eq(thread_uuid))).execute(&db_connection)?;
            let index = index_writer.index();
            let schema = index.schema();
            let uuid_field = schema.get_field("thread_uuid").unwrap();
            let term = Term::from_field_bytes(uuid_field, thread_uuid.as_bytes());
            index_writer.delete_term(term);
            index_writer.commit()?;
            Ok(())
        })?;
        Ok(())
    }

    pub fn count_threadposts(db_pool: &DBPool, thread_uuid: &Uuid) -> Result<u64> {
        use crate::schema::threadposts::dsl::*;

        let db_connection = db_pool.get()?;
        let count: i64 = threadposts
            .filter(parent_thread_id.eq(thread_uuid))
            .count()
            .get_result(&db_connection)?;
        Ok(count as u64)
    }

    /// ThreadPost range query
    pub fn thread_post_range(
        db_pool: &DBPool,
        thread_uuid: &Uuid,
        start: i64,
        end: i64,
    ) -> Result<Vec<ThreadPost>> {
        use crate::schema::threadposts::dsl::*;
        Ok(threadposts
            .filter(parent_thread_id.eq(thread_uuid))
            .limit(end - start + 1)
            .order_by(primary_key)
            .offset(start)
            .load::<ThreadPost>(&db_pool.get()?)?)
    }
}

/// diesel model
#[derive(Insertable)]
#[table_name = "threads"]
struct NewThread<'a> {
    pub uuid: &'a Uuid,
    pub created_at: &'a DateTime<Utc>,
    pub parent_board_id: &'a Uuid,
    pub title: &'a str,
    pub creator_user_id: &'a str,
}
