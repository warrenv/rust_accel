use axum::{extract::State, http::StatusCode, Json};
use serde::{Deserialize, Serialize};

use crate::{domain::AuthAPIError, utils::auth::validate_token, AppState};

#[tracing::instrument(name = "verify_token", skip_all)]
pub async fn verify_token(
    State(state): State<AppState>,
    Json(request): Json<VerifyTokenRequest>,
) -> Result<StatusCode, AuthAPIError> {
    match validate_token(&request.token, state.banned_token_store.clone()).await {
        Ok(_) => Ok(StatusCode::OK),
        Err(_) => Err(AuthAPIError::InvalidToken),
    }
}

#[derive(Deserialize, Debug, PartialEq, Serialize)]
pub struct VerifyTokenRequest {
    pub token: String,
}
