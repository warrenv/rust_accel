use crate::helpers::{get_random_email, TestApp};
use auth_service::routes::SignupResponse;

#[tokio::test]
async fn should_return_422_if_malformed_input() {
    let app = TestApp::new().await;

    let random_email = get_random_email();

    let test_cases = [
        // missing email
        serde_json::json!({
            "password": "password123",
            "requires2FA": true
        }),
        // missing password
        serde_json::json!({
            "email": random_email,
            "requires2FA": true
        }),
        // missing requires2FA
        serde_json::json!({
            "email": random_email,
            "password": "password123",
        }),
    ];

    for test_case in test_cases.iter() {
        let actual = app.post_signup(&test_case).await;
        assert_eq!(
            actual.status().as_u16(),
            422,
            "Failed for input: {:?}",
            test_case
        );
    }
}

#[tokio::test]
async fn should_return_201_if_valid_input() {
    let app = TestApp::new().await;
    let user = serde_json::json!({
        "email": get_random_email(),
        "password": "password123",
        "requires2FA": true
    });

    let actual = app.post_signup(&user).await;

    assert_eq!(
        actual.status().as_u16(),
        201,
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
