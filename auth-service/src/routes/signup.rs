use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use serde::{Deserialize, Serialize};

use crate::domain::{Email, Password};
use crate::{app_state::AppState, domain::User, AuthAPIError};

#[tracing::instrument(name = "Signup", skip_all, err(Debug))]
pub async fn signup(
    State(state): State<AppState>,
    Json(request): Json<SignupRequest>,
) -> Result<impl IntoResponse, AuthAPIError> {
    let email =
        Email::parse(request.email.clone()).map_err(|_| AuthAPIError::InvalidCredentials)?;
    let password =
        Password::parse(request.password.clone()).map_err(|_| AuthAPIError::InvalidCredentials)?;

    let user = User::new(email, password, request.requires_2fa);

    {
        let mut user_store = state.user_store.write().await;

        if let Ok(_) = user_store.get_user(&user.email).await {
            return Err(AuthAPIError::UserAlreadyExists);
        }

        if let Err(_) = user_store.add_user(user).await {
            return Err(AuthAPIError::UnexpectedError);
        }
    }

    let response = Json(SignupResponse {
        message: "User created successfully!".to_string(),
    });

    Ok((StatusCode::CREATED, response))
}

#[derive(Deserialize)]
pub struct SignupRequest {
    pub email: String,
    pub password: String,
    #[serde(rename = "requires2FA")]
    pub requires_2fa: bool,
}

#[derive(Deserialize, Debug, PartialEq, Serialize)]
pub struct SignupResponse {
    pub message: String,
}
