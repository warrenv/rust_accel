use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use serde::{Deserialize, Serialize};

use crate::{
    app_state::AppState,
    domain::{Email, Password, User},
    AuthAPIError,
};

pub async fn login(
    State(state): State<AppState>,
    Json(request): Json<LoginRequest>,
) -> Result<impl IntoResponse, AuthAPIError> {
    let email =
        Email::parse(request.email.clone()).map_err(|_| AuthAPIError::InvalidCredentials)?;
    let password =
        Password::parse(request.password.clone()).map_err(|_| AuthAPIError::InvalidCredentials)?;

    let user_store = &state.user_store.read().await;

    // TODO: call `user_store.validate_user` and return
    // `AuthAPIError::IncorrectCredentials` if valudation fails.
    println!("calling validate_user");
    user_store
        .validate_user(&email, &password)
        .await
        .map_err(|_| AuthAPIError::IncorrectCredentials)?;

    // TODO: call `user_store.get_user`. Return AuthAPIError::IncorrectCredentials if the operation fails.
    println!("calling get_user");
    let user = user_store
        .get_user(&email)
        .await
        .map_err(|_| AuthAPIError::IncorrectCredentials)?;

    Ok(StatusCode::OK.into_response())
}

#[derive(Deserialize, Debug, PartialEq, Serialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}
