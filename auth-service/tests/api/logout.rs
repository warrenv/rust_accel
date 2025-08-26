use reqwest::Url;
use sqlx::Connection;

use crate::helpers::get_random_email;
use crate::helpers::TestApp;
use auth_service::domain::email::Email;
use auth_service::utils::auth::generate_auth_cookie;
use auth_service::{utils::constants::JWT_COOKIE_NAME, ErrorResponse};

#[tokio::test]
async fn should_return_400_if_jwt_cookie_missing() {
    let expected = 400;
    let mut app = TestApp::new().await;

    let response = app.post_logout().await;
    let actual = response.status().as_u16();

    assert_eq!(actual, expected,);
}

#[tokio::test]
async fn should_return_401_if_invalid_token() {
    let expected = 401;
    let mut app = TestApp::new().await;

    // add invalid cookie
    app.cookie_jar.add_cookie_str(
        &format!(
            "{}=invalid; HttpOnly; SameSite=Lax; Secure; Path=/",
            JWT_COOKIE_NAME
        ),
        &Url::parse("http://127.0.0.1").expect("Failed to parse URL"),
    );

    let response = app.post_logout().await;
    let actual = response.status().as_u16();

    assert_eq!(actual, expected,);

    app.clean_up().await;
}

#[tokio::test]
async fn should_return_200_if_valid_jwt_cookie() {
    let mut app = TestApp::new().await;

    let random_email = get_random_email();

    let signup_body = serde_json::json!({
        "email": random_email,
        "password": "password123",
        "requires2FA": false
    });

    let response = app.post_signup(&signup_body).await;

    assert_eq!(response.status().as_u16(), 201);

    let login_body = serde_json::json!({
        "email": random_email,
        "password": "password123",
    });

    let response = app.post_login(&login_body).await;

    assert_eq!(response.status().as_u16(), 200);

    let auth_cookie = response
        .cookies()
        .find(|cookie| cookie.name() == JWT_COOKIE_NAME)
        .expect("No auth cookie found");

    assert!(!auth_cookie.value().is_empty());

    let token = auth_cookie.value();

    let response = app.post_logout().await;

    assert_eq!(response.status().as_u16(), 200);

    let auth_cookie = response
        .cookies()
        .find(|cookie| cookie.name() == JWT_COOKIE_NAME)
        .expect("No auth cookie found");

    assert!(auth_cookie.value().is_empty());

    {
        let banned_token_store = app.banned_token_store.write().await;
        let contains_token = banned_token_store
            .contains_token(token)
            .await
            .expect("Failed to check if token is banned");

        assert!(contains_token);
    }

    app.clean_up().await;
}

#[tokio::test]
async fn should_return_400_if_logout_called_twice_in_a_row() {
    let mut app = TestApp::new().await;

    let auth_cookie = generate_auth_cookie(&Email::parse("foo@example.com".to_owned()).unwrap())
        .unwrap()
        .to_string();
    println!("auth_cookie: {:?}", auth_cookie);

    app.cookie_jar.add_cookie_str(
        &auth_cookie,
        &Url::parse("http://127.0.0.1").expect("Failed to parse URL"),
    );

    {
        let expected = 200;
        let response = app.post_logout().await;
        let actual = response.status().as_u16();

        assert_eq!(actual, expected,);
    }

    {
        let expected = 400;
        let response = app.post_logout().await;
        let actual = response.status().as_u16();

        assert_eq!(actual, expected,);
    }

    app.clean_up().await;
}
