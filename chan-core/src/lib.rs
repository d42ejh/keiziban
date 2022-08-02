#[macro_use]
extern crate diesel;
#[macro_use]
extern crate num_derive;
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::PgConnection;

pub type DBPool = Pool<ConnectionManager<PgConnection>>;

pub mod graphql;
pub mod handler;
pub mod model;
mod schema;
pub mod search_engine;

#[cfg(test)]
mod tests {}
