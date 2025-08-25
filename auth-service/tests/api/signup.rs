use crate::helpers::{get_random_email, TestApp};
use auth_service::{routes::SignupResponse, ErrorResponse};

#[tokio::test]
async fn should_return_422_if_malformed_input() {
    let expected = 422;
    let app = TestApp::new().await;

    let test_cases = [
        // missing email
        serde_json::json!({
            "password": "password123",
            "requires2FA": true
        }),
        // missing password
        serde_json::json!({
            "email": get_random_email(),
            "requires2FA": true
        }),
        // missing requires2FA
        serde_json::json!({
            "email": get_random_email(),
            "password": "password123",
        }),
    ];

    for test_case in test_cases.iter() {
        let actual = app.post_signup(&test_case).await;
        assert_eq!(
            actual.status().as_u16(),
            expected,
            "Failed for input: {:?}",
            test_case
        );
    }
}

#[tokio::test]
async fn should_return_201_if_valid_input() {
    let expected = 201;
    let app = TestApp::new().await;
    let user = serde_json::json!({
        "email": get_random_email(),
        "password": "password123",
        "requires2FA": true
    });

    let actual = app.post_signup(&user).await;

    assert_eq!(
        actual.status().as_u16(),
        expected,
        "Failed for input: {:?}",
        user
    );

    {
        let expected = SignupResponse {
            message: "User created successfully!".to_owned(),
        };

        // Assert that we are getting the correct response body!
        assert_eq!(
            actual
                .json::<SignupResponse>()
                .await
                .expect("Could not deserialize response body to UserBody"),
            expected
        );
    }
}

#[tokio::test]
async fn should_return_400_if_invalid_input() {
    // The signup route should return a 400 HTTP status code if an invalid input is sent.
    // The input is considered invalid if:
    // - The email is empty or does not contain '@'
    // - The password is less than 8 characters

    let app = TestApp::new().await;

    let test_cases = [
        // empty email
        serde_json::json!({
            "email": "",
            "password": "password123",
            "requires2FA": true
        }),
        // missing '@' in email
        serde_json::json!({
            "email": "fooexample.com",
            "password": "password123",
            "requires2FA": true
        }),
        // short password
        serde_json::json!({
            "email": get_random_email(),
            "password": "p123",
            "requires2FA": true
        }),
    ];

    for i in test_cases.iter() {
        let response = app.post_signup(i).await;
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
async fn should_return_409_if_email_already_exists() {
    // Call the signup route twice. The second request should fail with a 409 HTTP status code

    let app = TestApp::new().await;
    let user = serde_json::json!({
        "email": get_random_email(),
        "password": "password123",
        "requires2FA": true
    });

    let _ = app.post_signup(&user).await;
    let actual = app.post_signup(&user).await;

    assert_eq!(actual.status().as_u16(), 409);

    assert_eq!(
        actual
            .json::<ErrorResponse>()
            .await
            .expect("Could not deserialize response body to ErrorResponse")
            .error,
        "User already exists".to_owned()
    );
}
