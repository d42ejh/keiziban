use crate::diesel::associations::HasTable;
use crate::schema::logs;
use crate::DBPool;
use async_graphql::{Error, Result, SimpleObject};
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use diesel::{Insertable, Queryable};
use tantivy::{doc, Index, IndexReader, IndexWriter, Term};
use uuid::Uuid;

#[derive(Queryable, SimpleObject, Clone)]

pub struct Log {
    #[graphql(skip)]
    primary_key: i32,
    pub timestamp: DateTime<Utc>,
    pub message: String,
    pub link: Option<String>,
    pub link_title: Option<String>,
}

impl Log {
    pub fn create_new(
        db_pool: &DBPool,
        log_message: &str,
        link: Option<&str>,
        link_title: Option<&str>,
    ) -> Result<Self> {
        //  use crate::schema::logs::dsl::*;
        let new_log = NewLog {
            timestamp: &Utc::now(),
            message: log_message,
            link: link,
            link_title: link_title,
        };
        let log = diesel::insert_into(logs::table)
            .values(new_log)
            .get_result::<Log>(&db_pool.get()?)?;
        Ok(log)
    }

    pub fn range(db_pool: &DBPool, start: i64, end: i64) -> Result<Vec<Log>> {
        use crate::schema::logs::dsl::*;
        Ok(logs
            .limit(end - start + 1)
            .order_by(primary_key)
            .offset(start)
            .load::<Log>(&db_pool.get()?)?)
    }
}

#[derive(Insertable)]
#[table_name = "logs"]
pub struct NewLog<'a> {
    pub timestamp: &'a DateTime<Utc>,
    pub message: &'a str,
    pub link: Option<&'a str>,
    pub link_title: Option<&'a str>,
}
