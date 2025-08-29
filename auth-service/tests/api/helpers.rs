use auth_service::get_redis_client;
use reqwest::cookie::Jar;
use reqwest::Client;
use secrecy::Secret;
use sqlx::postgres::{PgConnectOptions, PgConnection, PgPoolOptions};
use sqlx::Connection;
use sqlx::Executor;
use sqlx::PgPool;
use std::str::FromStr;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;
use wiremock::MockServer;

use auth_service::{
    app_state::{AppState, BannedTokenStoreType, EmailClientType, TwoFACodeStoreType},
    domain::Email,
    get_postgres_pool,
    //services::data_stores::hashmap_two_fa_code_store::HashmapTwoFACodeStore,
    //services::data_stores::hashmap_user_store::HashmapUserStore,
    services::data_stores::hashset_banned_token_store::HashsetBannedTokenStore,
    services::data_stores::PostgresUserStore,
    services::data_stores::RedisBannedTokenStore,
    services::data_stores::RedisTwoFACodeStore,
    services::mock_email_client::MockEmailClient,
    services::postmark_email_client::PostmarkEmailClient,
    utils::constants::{test, DATABASE_URL, REDIS_HOST_NAME},
    Application,
};

pub struct TestApp {
    pub address: String,
    pub cookie_jar: Arc<Jar>,
    pub http_client: reqwest::Client,
    pub email_server: MockServer,
    pub banned_token_store: BannedTokenStoreType,
    pub two_fa_code_store: TwoFACodeStoreType,
    pub email_client: EmailClientType,
    pub clean_up_called: bool,
    pub db_name: String,
}

impl TestApp {
    pub async fn new() -> Self {
        // We are creating a new database for each test case, and we need to ensure each database has a unique name!
        let db_name = Uuid::new_v4().to_string();
        let pg_pool = configure_postgresql(&db_name).await;

        //let user_store = Arc::new(RwLock::new(HashmapUserStore::default()));
        let user_store = Arc::new(RwLock::new(PostgresUserStore::new(pg_pool)));

        //    let banned_token_store = Arc::new(RwLock::new(HashsetBannedTokenStore::default()));
        let redis_connection = Arc::new(RwLock::new(configure_redis()));
        let banned_token_store: BannedTokenStoreType = Arc::new(RwLock::new(
            RedisBannedTokenStore::new(redis_connection.clone()),
        ));

        //let two_fa_code_store = Arc::new(RwLock::new(HashmapTwoFACodeStore::default()));
        let redis_connection2 = Arc::new(RwLock::new(configure_redis()));
        let two_fa_code_store: TwoFACodeStoreType = Arc::new(RwLock::new(
            RedisTwoFACodeStore::new(redis_connection2.clone()),
        ));

        let email_server = MockServer::start().await;
        let base_url = email_server.uri();
        let email_client = Arc::new(RwLock::new(configure_postmark_email_client(base_url)));
        //        let email_client = Arc::new(RwLock::new(MockEmailClient));

        let app_state = AppState {
            user_store,
            banned_token_store: banned_token_store.clone(),
            two_fa_code_store: two_fa_code_store.clone(),
            email_client: email_client.clone(),
        };

        let app = Application::build(app_state, test::APP_ADDRESS)
            .await
            .expect("Failed to build app");

        let address = format!("http://{}", app.address.clone());

        // Run the auth service in a separate async task
        // to avoid blocking the main test thread.
        #[allow(clippy::let_underscore_future)]
        let _ = tokio::spawn(app.run());

        let cookie_jar = Arc::new(Jar::default());
        let http_client = reqwest::Client::builder()
            .cookie_provider(cookie_jar.clone())
            .build()
            .unwrap();

        Self {
            address,
            cookie_jar,
            http_client,
            email_server,
            banned_token_store,
            two_fa_code_store,
            email_client,
            clean_up_called: false,
            db_name,
        }
    }

    pub async fn clean_up(&mut self) {
        delete_database(&self.db_name).await;
        self.clean_up_called = true
    }

    pub async fn get_root(&self) -> reqwest::Response {
        self.http_client
            .get(&format!("{}/", &self.address))
            .send()
            .await
            .expect("Failed to execute request.")
    }

    //pub async fn post_signup(&self) -> reqwest::Response {
    //    self.http_client
    //        .post(&format!("{}/signup", &self.address))
    //        .send()
    //        .await
    //        .expect("Failed to execute request.")
    //}
    pub async fn post_signup<Body>(&self, body: &Body) -> reqwest::Response
    where
        Body: serde::Serialize,
    {
        self.http_client
            .post(&format!("{}/signup", &self.address))
            .json(body)
            .send()
            .await
            .expect("Failed to execute request.")
    }

    //X    pub async fn post_login(&self) -> reqwest::Response {
    pub async fn post_login<Body>(&self, body: &Body) -> reqwest::Response
    where
        Body: serde::Serialize,
    {
        self.http_client
            .post(&format!("{}/login", &self.address))
            .json(body)
            .send()
            .await
            .expect("Failed to execute request.")
    }

    pub async fn post_logout(&self) -> reqwest::Response {
        self.http_client
            .post(&format!("{}/logout", &self.address))
            .send()
            .await
            .expect("Failed to execute request.")
    }

    pub async fn post_verify_2fa<Body>(&self, body: &Body) -> reqwest::Response
    where
        Body: serde::Serialize,
    {
        self.http_client
            .post(format!("{}/verify-2fa", &self.address))
            .json(body)
            .send()
            .await
            .expect("Failed to execute request.")
    }

    pub async fn post_verify_token<Body>(&self, body: &Body) -> reqwest::Response
    where
        Body: serde::Serialize,
    {
        self.http_client
            .post(&format!("{}/verify-token", &self.address))
            .json(body)
            .send()
            .await
            .expect("Failed to execute request.")
    }
}

pub fn get_random_email() -> String {
    format!("{}@example.com", Uuid::new_v4())
}

async fn configure_postgresql(db_name: &str) -> PgPool {
    let postgresql_conn_url = DATABASE_URL.to_owned();

    configure_database(&postgresql_conn_url, &db_name).await;

    let postgresql_conn_url_with_db = format!("{}/{}", postgresql_conn_url, db_name);

    // Create a new connection pool and return it
    get_postgres_pool(&postgresql_conn_url_with_db)
        .await
        .expect("Failed to create Postgres connection pool!")
}

async fn configure_database(db_conn_string: &str, db_name: &str) {
    // Create database connection
    let connection = PgPoolOptions::new()
        .connect(db_conn_string)
        .await
        .expect("Failed to create Postgres connection pool.");

    // Create a new database
    connection
        .execute(format!(r#"CREATE DATABASE "{}";"#, db_name).as_str())
        .await
        .expect("Failed to create database.");

    // Connect to new database
    let db_conn_string = format!("{}/{}", db_conn_string, db_name);

    let connection = PgPoolOptions::new()
        .connect(&db_conn_string)
        .await
        .expect("Failed to create Postgres connection pool.");

    // Run migrations against new database
    sqlx::migrate!()
        .run(&connection)
        .await
        .expect("Failed to migrate the database");
}

async fn delete_database(db_name: &str) {
    let postgresql_conn_url: String = DATABASE_URL.to_owned();

    let connection_options = PgConnectOptions::from_str(&postgresql_conn_url)
        .expect("Failed to parse PostgreSQL connection string");

    let mut connection = PgConnection::connect_with(&connection_options)
        .await
        .expect("Failed to connect to Postgres");

    // Kill any active connections to the database
    connection
        .execute(
            format!(
                r#"
                SELECT pg_terminate_backend(pg_stat_activity.pid)
                FROM pg_stat_activity
                WHERE pg_stat_activity.datname = '{}'
                  AND pid <> pg_backend_pid();
        "#,
                db_name
            )
            .as_str(),
        )
        .await
        .expect("Failed to drop the database.");

    // Drop the database
    connection
        .execute(format!(r#"DROP DATABASE "{}";"#, db_name).as_str())
        .await
        .expect("Failed to drop the database.");
}

fn configure_redis() -> redis::Connection {
    get_redis_client(REDIS_HOST_NAME.to_owned())
        .expect("Failed to get Redis client")
        .get_connection()
        .expect("Failed to get Redis connection")
}

fn configure_postmark_email_client(base_url: String) -> PostmarkEmailClient {
    let postmark_auth_token = Secret::new("auth_token".to_owned());

    let sender = Email::parse(Secret::new(test::email_client::SENDER.to_owned())).unwrap();

    let http_client = Client::builder()
        .timeout(test::email_client::TIMEOUT)
        .build()
        .expect("Failed to build HTTP client");

    PostmarkEmailClient::new(base_url, sender, postmark_auth_token, http_client)
}
