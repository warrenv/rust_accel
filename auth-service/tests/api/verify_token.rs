use crate::helpers::{get_random_email, TestApp};
use auth_service::domain::Email;
use auth_service::utils::generate_auth_cookie;
use auth_service::{utils::constants::JWT_COOKIE_NAME, ErrorResponse};

#[tokio::test]
async fn should_return_422_if_malformed_input() {
    let expected = 422;
    let app = TestApp::new().await;

    let verify_token_body = serde_json::json!({});

    let response = app.post_verify_token(&verify_token_body).await;
    let actual = response.status().as_u16();

    assert_eq!(actual, expected);
}

#[tokio::test]
async fn should_return_200_valid_token() {
    let expected = 200;
    let app = TestApp::new().await;

    let auth_cookie =
        generate_auth_cookie(&Email::parse("foo@example.com".to_owned()).unwrap()).unwrap();

    let verify_token_body = serde_json::json!({
        "token": auth_cookie.value()
    });

    let response = app.post_verify_token(&verify_token_body).await;
    let actual = response.status().as_u16();

    assert_eq!(actual, expected,);
}

#[tokio::test]
async fn should_return_401_if_invalid_token() {
    let expected = 401;
    let app = TestApp::new().await;

    let auth_cookie =
        generate_auth_cookie(&Email::parse("foo@example.com".to_owned()).unwrap()).unwrap();

    let verify_token_body = serde_json::json!({
        "token": "a_bad_token"
    });

    let response = app.post_verify_token(&verify_token_body).await;
    let actual = response.status().as_u16();

    assert_eq!(actual, expected,);
}
