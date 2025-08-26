use auth_service::{
    domain::{Email, LoginAttemptId, TwoFACode, TwoFACodeStore},
    routes::TwoFactorAuthResponse,
    utils::constants::JWT_COOKIE_NAME,
    ErrorResponse,
};

use crate::helpers::{get_random_email, TestApp};

#[tokio::test]
async fn should_return_422_if_malformed_input() {
    let mut app = TestApp::new().await;

    let random_email = get_random_email();
    let login_attempt_id = LoginAttemptId::default().as_ref().to_owned();

    let test_cases = [
        serde_json::json!({
            "2FACode": "123456",
        }),
        serde_json::json!({
            "email": random_email,
        }),
        serde_json::json!({
            "loginAttemptId": login_attempt_id,
        }),
        serde_json::json!({
            "2FACode": "123456",
            "email": random_email,
        }),
        serde_json::json!({
            "2FACode": "123456",
            "loginAttemptId": login_attempt_id,
        }),
        serde_json::json!({
            "email": random_email,
            "loginAttemptId": login_attempt_id,
        }),
        serde_json::json!({}),
    ];

    for test_case in test_cases {
        let response = app.post_verify_2fa(&test_case).await;

        assert_eq!(
            response.status().as_u16(),
            422,
            "Failed for input: {:?}",
            test_case
        );
    }

    app.clean_up().await;
}

#[tokio::test]
async fn should_return_400_if_invalid_input() {
    let mut app = TestApp::new().await;

    let random_email = get_random_email();
    let login_attempt_id = LoginAttemptId::default().as_ref().to_owned();

    let test_cases = [
        serde_json::json!({
            "email": "fooexample.com",
            "2FACode": "123456",
            "loginAttemptId": login_attempt_id,
        }),
        serde_json::json!({
            "email": "foo@example.com",
            "2FACode": "A23456",
            "loginAttemptId": login_attempt_id,
        }),
        serde_json::json!({
            "email": "foo@example.com",
            "2FACode": "123456",
            "loginAttemptId": "login_attempt_id",
        }),
    ];

    for test_case in test_cases {
        let response = app.post_verify_2fa(&test_case).await;

        assert_eq!(
            response.status().as_u16(),
            400,
            "Failed for input: {:?}",
            test_case
        );
    }

    app.clean_up().await;
}

#[tokio::test]
async fn should_return_401_if_incorrect_credentials() {
    //TODO:    todo!()

    //app.clean_up().await;
}

#[tokio::test]
async fn should_return_401_if_old_code() {
    // Call login twice. Then, attempt to call verify-fa with the 2FA code from the first login requet. This should fail.
    //TODO:    todo!()

    //app.clean_up().await;
}

#[tokio::test]
async fn should_return_200_if_correct_code() {
    // Make sure to assert the auth cookie gets set
    //TODO: todo!()

    //app.clean_up().await;
}

#[tokio::test]
async fn should_return_401_if_same_code_twice() {
    //TODO: todo!()

    //app.clean_up().await;
}
