use axum::{http::StatusCode, response::IntoResponse, Json};
//use axum_extra::extract::cookie::CookieJar;
use serde::{Deserialize, Serialize};

use crate::{domain::AuthAPIError, utils::auth::validate_token};

//pub async fn verify_token(jar: CookieJar) -> Result<impl IntoResponse, AuthAPIError> {
pub async fn verify_token(
    //    State(state): State<AppState>,
    //    jar: CookieJar,
    Json(request): Json<VerifytokenRequest>,
) -> Result<impl IntoResponse, AuthAPIError> {
    //    let cookie = match jar.get(JWT_COOKIE_NAME) {
    //        Some(x) => x,
    //        None => return Err(AuthAPIError::MissingToken),
    //    };

    let _claims = match validate_token(&request.token).await {
        Ok(x) => x,
        Err(_) => return Err(AuthAPIError::InvalidToken),
    };

    Ok(StatusCode::OK.into_response())
}

#[derive(Deserialize, Debug, PartialEq, Serialize)]
pub struct VerifytokenRequest {
    pub token: String,
}
