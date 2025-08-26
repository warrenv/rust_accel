use sqlx::PgPool;
use std::sync::Arc;
use tokio::sync::RwLock;

use auth_service::{
    app_state::AppState,
    get_postgres_pool,
    services::hashmap_two_fa_code_store::HashmapTwoFACodeStore,
    services::hashmap_user_store::HashmapUserStore,
    services::hashset_banned_token_store::HashsetBannedTokenStore,
    services::mock_email_client::MockEmailClient,
    utils::constants::{prod, DATABASE_URL},
    Application,
};

#[tokio::main]
async fn main() {
    let user_store = Arc::new(RwLock::new(HashmapUserStore::default()));
    let banned_token_store = Arc::new(RwLock::new(HashsetBannedTokenStore::default()));
    let two_fa_code_store = Arc::new(RwLock::new(HashmapTwoFACodeStore::default()));
    let email_client = Arc::new(RwLock::new(MockEmailClient));
    let pg_pool = configure_postgresql().await;

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
