use reqwest::Url;

use crate::helpers::TestApp;
use auth_service::domain::email::Email;
use auth_service::utils::generate_auth_cookie;
use auth_service::{utils::constants::JWT_COOKIE_NAME, ErrorResponse};

#[tokio::test]
async fn should_return_400_if_jwt_cookie_missing() {
    let expected = 400;
    let app = TestApp::new().await;

    let response = app.post_logout().await;
    let actual = response.status().as_u16();

    assert_eq!(actual, expected,);
}

#[tokio::test]
async fn should_return_401_if_invalid_token() {
    let expected = 401;
    let app = TestApp::new().await;

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
}

#[tokio::test]
async fn should_return_200_if_valid_jwt_cookie() {
    let expected = 200;
    let app = TestApp::new().await;

    let auth_cookie = generate_auth_cookie(&Email::parse("foo@example.com".to_owned()).unwrap())
        .unwrap()
        .to_string();
    println!("auth_cookie: {:?}", auth_cookie);

    app.cookie_jar.add_cookie_str(
        &auth_cookie,
        &Url::parse("http://127.0.0.1").expect("Failed to parse URL"),
    );

    let response = app.post_logout().await;
    let actual = response.status().as_u16();

    assert_eq!(actual, expected,);
}

#[tokio::test]
async fn should_return_400_if_logout_called_twice_in_a_row() {
    let app = TestApp::new().await;

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
}
