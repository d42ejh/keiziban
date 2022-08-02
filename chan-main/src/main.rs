use actix_extensible_rate_limit::RateLimiter;
use actix_files::Files;
use actix_identity::{CookieIdentityPolicy, Identity, IdentityService};
use actix_session::CookieSession;
use actix_web::cookie::Key;
use actix_web::{
    guard, middleware,
    web::{self, Data},
    App, HttpServer,
};
use actix_web_flash_messages::storage::CookieMessageStore;
use actix_web_flash_messages::FlashMessagesFramework;
use async_graphql::{EmptySubscription, Schema};
use chan_core::graphql::{MutationRoot, QueryRoot};
use chan_core::handler::{index, index_playground};
use chan_core::model::SystemInfoContext;
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::PgConnection;
use dotenv::dotenv;
use reqwest::Client;
use std::env;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{event, span, Level};
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

#[actix_web::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();

    //tracing stuffs
    tracing_subscriber::registry()
        .with(fmt::layer().with_thread_names(true))
        .with(EnvFilter::from_default_env())
        .init();

    let span = span!(Level::TRACE, "my span");

    let _ = span.enter();

    //diesel
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let db_pool = Pool::builder()
        .max_size(20)
        .build(ConnectionManager::<PgConnection>::new(database_url))
        .unwrap();

    let cd = std::env::current_dir()?.join("tantivy");
    //init tantivy
    let tantivy_index = chan_core::search_engine::init_tantivy(&cd).unwrap(); //todo from config instead of cd
    let index_writer = tantivy_index.writer(100_000_000)?;
    let index_reader = tantivy_index.reader()?;

    //system info
    let system_info_context = SystemInfoContext::new(std::time::Duration::from_secs(10));

    //graphql
    let schema = Schema::build(QueryRoot, MutationRoot, EmptySubscription)
        .data(Arc::new(RwLock::new(system_info_context)))
        .data(db_pool.clone())
        .data(tantivy_index.clone())
        .data(Arc::new(RwLock::new(index_writer)))
        .data(index_reader)
        .finish();

    //flash message related
    let key = Key::generate();
    let message_store = CookieMessageStore::builder(key).build();

    let flash_fw = FlashMessagesFramework::builder(message_store)
        .minimum_level(actix_web_flash_messages::Level::Debug)
        .build();

    //rate limiting
    use actix_extensible_rate_limit::{
        backend::memory::InMemoryBackend, backend::SimpleInputFunctionBuilder, RateLimiter,
    };
    let backend = InMemoryBackend::builder().build();

    let server = HttpServer::new(move || {
        let input = SimpleInputFunctionBuilder::new(std::time::Duration::from_secs(60), 60)
            .real_ip_key()
            .build();
        let rate_limit_middleware = RateLimiter::builder(backend.clone(), input)
            .add_headers()
            .build();

        App::new()
            .wrap(rate_limit_middleware)
            .wrap(chan_web::middleware::LoginCheck)
            .wrap(
                CookieSession::signed(&[0; 32])
                    .name("session-cookie")
                    .secure(true),
            )
            .wrap(IdentityService::new(
                CookieIdentityPolicy::new(&[0; 32]) // <- create cookie identity policy
                    .name("auth-cookie")
                    .secure(true),
            ))
            .wrap(flash_fw.clone())
            .app_data(Data::new(schema.clone()))
            .app_data(Data::new(Client::new()))
            .wrap(middleware::Compress::default())
            .wrap(middleware::Logger::default())
            .service(web::resource("/graphql").guard(guard::Post()).to(index))
            .service(
                web::resource("/graphql")
                    .guard(guard::Get())
                    .to(index_playground),
            )
            .service(chan_web::routes::root)
            .service(chan_web::routes::theme::theme_handler)
            .service(chan_web::routes::register)
            .service(chan_web::routes::registration_handler)
            .service(chan_web::routes::logout_handler)
            .service(chan_web::routes::login)
            .service(chan_web::routes::login_handler)
            .service(chan_web::routes::search)
            .service(chan_web::routes::search_handler)
            .service(chan_web::routes::board::view::board_view)
            .service(chan_web::routes::board::list::board_list)
            .service(chan_web::routes::thread::view::thread_view_range)
            .service(chan_web::routes::thread::view::thread_view_range_post)
            .service(chan_web::routes::thread::creation::thread_creation_handler)
            .service(chan_web::routes::thread::view::thread_view)
            .service(chan_web::routes::thread::removal::thread_removal_handler)
            .service(chan_web::routes::threadpost::creation::threadpost_creation_handler)
            .service(chan_web::routes::threadpost::removal::threadpost_removal_handler)
            .service(chan_web::routes::user::view::user_view)
            .service(chan_web::routes::user::change::user_type_change_handler)
            .service(chan_web::routes::rules::rules)
            .service(chan_web::routes::manage::manage)
            .service(chan_web::routes::log::log_view)
            .service(chan_web::routes::log::log_view_range)
            .service(chan_web::routes::system_info::system_info)
            .service(
                web::resource(["/redirect/{base}", "/redirect/{base}/{v}"])
                    .to(chan_web::routes::redirect::redirect),
            )
            .service(Files::new("/backgrounds", "static/backgrounds").show_files_listing())
            .service(Files::new("/css", "static/css").show_files_listing())
    });

    server.bind("127.0.0.1:8080")?.run().await?;
    Ok(())
}
