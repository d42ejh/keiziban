use anyhow::Result;
use chan_core::model::{verify_token, User, UserType};
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::PgConnection;
use dotenv::dotenv;
use openssl::rand::rand_bytes;
use tracing::{event, Level};

#[test]
fn user_token_test() -> Result<()> {
    dotenv().ok();

    tracing_subscriber::fmt()
        .with_thread_names(true)
        .with_max_level(Level::DEBUG)
        .init();
    assert!(std::env::var("JWT_SECRET_KEY").is_ok());
    event!(Level::DEBUG, "{:?}", std::env::var("JWT_SECRET_KEY"));

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let db_pool = Pool::builder()
        .max_size(20)
        .build(ConnectionManager::<PgConnection>::new(database_url))?;

    let user_id = String::from("xxxxxxxxxxy");
    let user_pass = "passabc";

    let result = User::create_new(&db_pool, &user_id, UserType::Normal, &user_pass);
    assert!(result.is_ok());
    let user = result.unwrap();

    let result = User::login(&db_pool, &user_id, &user_pass);
    assert!(result.is_ok());
    let token = result.unwrap();

    let result = verify_token(&db_pool, &token);
    if result.is_err() {
        event!(Level::ERROR, "{:?}", result);
    }
    assert!(result.is_ok());
    let user_id = result.unwrap();

    assert_eq!(user_id, user.id);

    Ok(())
}
