use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use serde::{Deserialize, Serialize};

use crate::{
    app_state::AppState,
    domain::{AuthAPIError, Email, LoginAttemptId, TwoFACode},
};

#[axum::debug_handler]
pub async fn verify_2fa(
    State(state): State<AppState>,
    Json(request): Json<Verify2FARequest>,
) -> Result<impl IntoResponse, AuthAPIError> {
    let email =
        Email::parse(request.email.clone()).map_err(|_| AuthAPIError::InvalidCredentials)?;

    let login_attempt_id = LoginAttemptId::parse(request.login_attempt_id.clone())
        .map_err(|_| AuthAPIError::InvalidCredentials)?;

    let two_fa_code = TwoFACode::parse(request.two_fa_code.clone())
        .map_err(|_| AuthAPIError::InvalidCredentials)?;

    let two_fa_code_store = state.two_fa_code_store.write().await;

    // Call `two_fa_code_store.get_code`. If the call fails
    // return a `AuthAPIError::IncorrectCredentials`.
    let code_tuple = match two_fa_code_store.get_code(&email).await {
        Ok(x) => (x.0, x.1),
        Err(_) => return Err(AuthAPIError::InvalidCredentials),
    };

    // TODO: Validate that the `login_attempt_id` and `two_fa_code`
    // in the request body matches values in the `code_tuple`.
    // If not, return a `AuthAPIError::IncorrectCredentials`.
    if code_tuple.0 != login_attempt_id || code_tuple.1 != two_fa_code {
        return Err(AuthAPIError::IncorrectCredentials);
    }

    Ok(StatusCode::OK.into_response())
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Verify2FARequest {
    email: String,
    #[serde(rename = "loginAttemptId")]
    login_attempt_id: String,
    #[serde(rename = "2FACode")]
    two_fa_code: String,
}
