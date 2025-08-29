use crate::helpers::{get_random_email, TestApp};
use auth_service::domain::Email;
use auth_service::routes::TwoFactorAuthResponse;
use auth_service::{routes::SignupResponse, utils::constants::JWT_COOKIE_NAME, ErrorResponse};
use secrecy::{ExposeSecret, Secret};
use wiremock::matchers::method;
use wiremock::matchers::path;
use wiremock::Mock;
use wiremock::ResponseTemplate;

#[tokio::test]
async fn should_return_422_if_malformed_credentials() {
    let mut app = TestApp::new().await;

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

    app.clean_up().await;
}

#[tokio::test]
async fn should_return_400_if_invalid_input() {
    // Call the log-in route with invalid credentials and assert that a
    // 400 HTTP status code is returned along with the appropriate error message.
    //todo!()
    let mut app = TestApp::new().await;

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

    app.clean_up().await;
}

#[tokio::test]
async fn should_return_401_if_incorrect_credentials() {
    // Call the log-in route with incorrect credentials and assert
    // that a 401 HTTP status code is returned along with the appropriate error message.

    let mut app = TestApp::new().await;

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

    //Mock::given(path("/email"))
    //    .and(method("POST"))
    //    .respond_with(ResponseTemplate::new(200))
    //    .expect(1)
    //    .mount(&app.email_server)
    //    .await;

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

    app.clean_up().await;
}

#[tokio::test]
async fn should_return_200_if_valid_credentials_and_2fa_disabled() {
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

    app.clean_up().await;
}

#[tokio::test]
async fn should_return_206_if_valid_credentials_and_2fa_enabled() {
    let mut app = TestApp::new().await;

    let random_email = get_random_email();

    let signup_body = serde_json::json!({
        "email": random_email,
        "password": "password123",
        "requires2FA": true
    });

    let response = app.post_signup(&signup_body).await;

    assert_eq!(response.status().as_u16(), 201);

    // Define an expectation for the mock server
    Mock::given(path("/email")) // Expect an HTTP request to the "/email" path
        .and(method("POST")) // Expect the HTTP method to be POST
        .respond_with(ResponseTemplate::new(200)) // Respond with an HTTP 200 OK status
        .expect(1) // Expect this request to be made exactly once
        .mount(&app.email_server) // Mount this expectation on the mock email server
        .await; // Await the asynchronous operation to ensure the mock server is set up before proceeding

    let login_body = serde_json::json!({
        "email": random_email,
        "password": "password123",
    });

    let response = app.post_login(&login_body).await;

    assert_eq!(response.status().as_u16(), 206);

    assert_eq!(
        response
            .json::<TwoFactorAuthResponse>()
            .await
            .expect("Could not deserialize response body to TwoFactorAuthResponse")
            .message,
        "2FA required".to_owned()
    );

    {
        // TODO: assert that `json_body.login_attempt_id` is stored inside `app.two_fa_code_store`
        let two_fa_code_store = app.two_fa_code_store.read().await;
        let actual = two_fa_code_store
            .get_code(&Email::parse(Secret::new(random_email)).unwrap())
            .await;

        assert_eq!(actual.is_ok(), true);
    }

    app.clean_up().await;
}
