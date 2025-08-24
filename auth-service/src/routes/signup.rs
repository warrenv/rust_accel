use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use serde::{Deserialize, Serialize};

use crate::{app_state::AppState, domain::User, AuthAPIError};

pub async fn signup(
    State(state): State<AppState>,
    Json(request): Json<SignupRequest>,
) -> Result<impl IntoResponse, AuthAPIError> {
    let email = request.email.clone();
    let password = request.password.clone();

    // TODO: early return AuthAPIError::InvalidCredentials if:
    // - email is empty or does not contain '@'
    if email == "" || !email.contains("@") {
        return Err(AuthAPIError::InvalidCredentials);
    };

    // - password is less than 8 characters
    if password.len() < 8 {
        return Err(AuthAPIError::InvalidCredentials);
    };

    let user = User::new(request.email, request.password, request.requires_2fa);

    {
        let mut user_store = state.user_store.write().await;

        // TODO: early return AuthAPIError::UserAlreadyExists if email exists in user_store.
        if let Ok(_) = user_store.get_user(&user.email).await {
            return Err(AuthAPIError::UserAlreadyExists);
        }

        // TODO: instead of using unwrap, early return AuthAPIError::UnexpectedError if add_user() fails.
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
