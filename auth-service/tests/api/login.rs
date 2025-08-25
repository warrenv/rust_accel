use crate::helpers::{get_random_email, TestApp};
use auth_service::{routes::SignupResponse, utils::constants::JWT_COOKIE_NAME, ErrorResponse};

#[tokio::test]
async fn should_return_422_if_malformed_credentials() {
    let app = TestApp::new().await;

    let test_cases = [
        // missing email
        serde_json::json!({
            "password": "password12345"
        }),
        // missing password
        serde_json::json!({
            "password": "password12345"
        }),
        // empty payload
        serde_json::json!({}),
    ];

    for test_case in test_cases.iter() {
        let expected = 422;
        let actual = app.post_login(&test_case).await;
        assert_eq!(
            actual.status().as_u16(),
            expected,
            "Failed for input: {:?}",
            test_case
        );
    }
}

#[tokio::test]
async fn should_return_400_if_invalid_input() {
    // Call the log-in route with invalid credentials and assert that a
    // 400 HTTP status code is returned along with the appropriate error message.
    //todo!()
    let app = TestApp::new().await;

    let test_cases = [
        // empty email
        serde_json::json!({
            "email": "",
            "password": "password123",
        }),
        // missing '@' in email
        serde_json::json!({
            "email": "fooexample.com",
            "password": "password123",
        }),
        // short password
        serde_json::json!({
            "email": get_random_email(),
            "password": "p123",
        }),
    ];

    for i in test_cases.iter() {
        let response = app.post_login(i).await;
        assert_eq!(response.status().as_u16(), 400, "Failed for input: {:?}", i);

        assert_eq!(
            response
                .json::<ErrorResponse>()
                .await
                .expect("Could not deserialize response body to ErrorResponse")
                .error,
            "Invalid credentials".to_owned()
        );
    }
}

#[tokio::test]
async fn should_return_401_if_incorrect_credentials() {
    // Call the log-in route with incorrect credentials and assert
    // that a 401 HTTP status code is returned along with the appropriate error message.

    let app = TestApp::new().await;

    {
        let user = serde_json::json!({
            "email": "foo@example.com",
            "password": "password123",
            "requires2FA": true
        });

        let expected = 201;
        let actual = app.post_signup(&user).await;

        assert_eq!(
            actual.status().as_u16(),
            expected,
            "Failed for input: {:?}",
            user
        );
    }

    let test_cases = [
        // incorrect password
        serde_json::json!({
            "email": "foo@example.com",
            "password": "PASSWORD123",
        }),
        // incorrect email
        serde_json::json!({
            "email": "FOO@example.com",
            "password": "PASSWORD123",
        }),
    ];

    for i in test_cases.iter() {
        let response = app.post_login(i).await;
        assert_eq!(response.status().as_u16(), 401, "Failed for input: {:?}", i);

        assert_eq!(
            response
                .json::<ErrorResponse>()
                .await
                .expect("Could not deserialize response body to ErrorResponse")
                .error,
            "Incorrect credentials".to_owned()
        );
    }
}

#[tokio::test]
async fn should_return_200_if_valid_credentials_and_2fa_disabled() {
    let app = TestApp::new().await;

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
}
