use sqlx::PgPool;
use std::sync::Arc;
use tokio::sync::RwLock;

use auth_service::{
    app_state::{AppState, BannedTokenStoreType, TwoFACodeStoreType},
    get_postgres_pool,
    get_redis_client,
    //services::data_stores::hashmap_two_fa_code_store::HashmapTwoFACodeStore,
    //services::data_stores::hashmap_user_store::HashmapUserStore,
    //services::data_stores::hashset_banned_token_store::HashsetBannedTokenStore,
    services::data_stores::postgres_user_store::PostgresUserStore,
    services::data_stores::redis_banned_token_store::RedisBannedTokenStore,
    services::data_stores::redis_two_fa_code_store::RedisTwoFACodeStore,
    services::mock_email_client::MockEmailClient,
    utils::{
        constants::{prod, DATABASE_URL, REDIS_HOST_NAME},
        init_tracing,
    },
    Application,
};

#[tokio::main]
async fn main() {
    init_tracing();

    let pg_pool = configure_postgresql().await;
    //let user_store = Arc::new(RwLock::new(HashmapUserStore::default()));
    let user_store = Arc::new(RwLock::new(PostgresUserStore::new(pg_pool)));

    //let banned_token_store = Arc::new(RwLock::new(HashsetBannedTokenStore::default()));
    let redis_connection = Arc::new(RwLock::new(configure_redis()));
    let banned_token_store: BannedTokenStoreType = Arc::new(RwLock::new(
        RedisBannedTokenStore::new(redis_connection.clone()),
    ));

    //let two_fa_code_store = Arc::new(RwLock::new(HashmapTwoFACodeStore::default()));
    let redis_connection2 = Arc::new(RwLock::new(configure_redis()));
    let two_fa_code_store: TwoFACodeStoreType = Arc::new(RwLock::new(RedisTwoFACodeStore::new(
        redis_connection2.clone(),
    )));

    let email_client = Arc::new(RwLock::new(MockEmailClient));

    let app_state = AppState {
        user_store,
        banned_token_store,
        two_fa_code_store,
        email_client,
    };

    let app = Application::build(app_state, prod::APP_ADDRESS)
        .await
        .expect("Failed to build app");

    app.run().await.expect("Failed to run app");
}

async fn configure_postgresql() -> PgPool {
    // Create a new database connection pool
    let pg_pool = get_postgres_pool(&DATABASE_URL)
        .await
        .expect("Failed to create Postgres connection pool!");

    // Run database migrations against our test database!
    sqlx::migrate!()
        .run(&pg_pool)
        .await
        .expect("Failed to run migrations");

    pg_pool
}

fn configure_redis() -> redis::Connection {
    get_redis_client(REDIS_HOST_NAME.to_owned())
        .expect("Failed to get Redis client")
        .get_connection()
        .expect("Failed to get Redis connection")
}
