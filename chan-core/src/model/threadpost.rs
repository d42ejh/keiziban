use crate::schema::threadposts;
use crate::DBPool;
use async_graphql::{Error, Result, SimpleObject};
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use diesel::{Insertable, Queryable};
use tantivy::{doc, Index, IndexReader, IndexWriter, Term};
use uuid::Uuid;

#[derive(Queryable, SimpleObject, Clone)]
pub struct ThreadPost {
    #[graphql(skip)]
    primary_key: i32,
    pub uuid: Uuid,
    pub number: i32,
    pub posted_at: DateTime<Utc>,
    pub poster_user_id: String,
    pub parent_thread_id: Uuid,
    pub body_text: String,
}

impl ThreadPost {
    pub fn select_by_uuid(db_pool: &DBPool, threadpost_uuid: &Uuid) -> Result<Self> {
        use crate::schema::threadposts::dsl::*;
        //check if a thread with the uuid is exist
        let threadpost_vec = threadposts
            .filter(uuid.eq(threadpost_uuid))
            .limit(1)
            .load::<ThreadPost>(&db_pool.get()?)?;
        if threadpost_vec.len() != 1 {
            debug_assert_eq!(threadpost_vec.len(), 0);
            return Err(Error::new("Invalid threadpost uuid."));
        }
        Ok(threadpost_vec[0].clone())
    }

    /// Create a new threadpost and insert it to DB.
    /// Returns the created threadpost.
    pub fn create_new(
        db_pool: &DBPool,
        index_writer: &mut IndexWriter,
        poster_user_id: &str,
        thread_uuid: &Uuid,
        post_body: &str,
    ) -> Result<Self> {
        use crate::model::Thread;

        if post_body.len() == 0 {
            return Err(Error::new("Empty post."));
        }

        let db_connection = db_pool.get()?;
        let created_threadpost = db_connection.build_transaction().run::<_, Error, _>(|| {
            //must count threadposts in transaction
            let threadpost_count = Thread::count_threadposts(&db_pool, thread_uuid)?;

            if threadpost_count >= 1000 {
                return Err(Error::new("Thread is full."));
            }

            let new_threadpost = NewThreadPost {
                uuid: &Uuid::new_v4(),
                number: (threadpost_count + 1).try_into()?,
                posted_at: &Utc::now(),
                poster_user_id: poster_user_id,
                parent_thread_id: thread_uuid,
                body_text: post_body,
            };

            let created_threadpost = diesel::insert_into(threadposts::table)
                .values(&new_threadpost)
                .get_result::<ThreadPost>(&db_pool.get()?)?;
            let index = index_writer.index();
            let schema = index.schema();

            let body_text_field = schema.get_field("threadpost_body_text").unwrap();
            let uuid_field = schema.get_field("threadpost_uuid").unwrap();

            index_writer.add_document(doc!(
                body_text_field=>post_body,
                uuid_field=>created_threadpost.uuid.as_bytes().as_slice()
            ))?;
            index_writer.commit()?;
            Ok(created_threadpost)
        })?;

        Ok(created_threadpost)
    }

    pub fn remove_by_uuid(
        db_pool: &DBPool,
        index_writer: &mut IndexWriter,
        threadpost_uuid: &Uuid,
    ) -> Result<()> {
        use crate::schema::threadposts::dsl::*;
        let db_connection = db_pool.get()?;
        db_connection.build_transaction().run::<_, Error, _>(|| {
            diesel::delete(threadposts.filter(uuid.eq(threadpost_uuid))).execute(&db_connection)?;
            let index = index_writer.index();
            let schema = index.schema();
            let uuid_field = schema.get_field("threadpost_uuid").unwrap();
            let term = Term::from_field_bytes(uuid_field, threadpost_uuid.as_bytes());
            index_writer.delete_term(term);
            index_writer.commit()?;
            Ok(())
        })?;
        Ok(())
    }
}

/// diesel model
#[derive(Insertable)]
#[table_name = "threadposts"]
pub struct NewThreadPost<'a> {
    pub uuid: &'a Uuid,
    pub number: i32,
    pub posted_at: &'a DateTime<Utc>,
    pub poster_user_id: &'a str,
    pub parent_thread_id: &'a Uuid,
    pub body_text: &'a str,
}
