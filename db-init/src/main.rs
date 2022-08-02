use chan_core::model::{Board, Thread, ThreadPost, User, UserType};
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::PgConnection;
use dotenv::dotenv;

fn main() -> anyhow::Result<()> {
    //diesel
    dotenv().ok();
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let db_pool = Pool::builder()
        .max_size(20)
        .build(ConnectionManager::<PgConnection>::new(database_url))
        .unwrap();

    Board::create_new(&db_pool, "Official", "Official Board.").unwrap();

    let test_board = Board::create_new(&db_pool, "TestBoard", "Test board, do whatever you want.")
        .map_err(|e| anyhow::anyhow!("Failed to create board"))?;

    let password = "abcdef578439543543543";
    let user = User::create_new(&db_pool, UserType::Admin, password).unwrap();
    println!(
        "Created dev account\nid: {}\npassword: {}",
        user.id, password
    );

    Ok(())
}
