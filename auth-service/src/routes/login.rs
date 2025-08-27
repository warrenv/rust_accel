use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use axum_extra::extract::CookieJar;
use serde::{Deserialize, Serialize};

use crate::{
    app_state::AppState,
    domain::{AuthAPIError, Email, LoginAttemptId, Password, TwoFACode},
    utils::auth::generate_auth_cookie,
};

#[tracing::instrument(name = "login", skip_all)]
pub async fn login(
    State(state): State<AppState>,
    jar: CookieJar,
    Json(request): Json<LoginRequest>,
) -> (CookieJar, Result<impl IntoResponse, AuthAPIError>) {
    //) -> Result<impl IntoResponse, AuthAPIError> {
    //    let email =
    //        Email::parse(request.email.clone()).map_err(|_| AuthAPIError::InvalidCredentials)?;
    //    let password =
    //        Password::parse(request.password.clone()).map_err(|_| AuthAPIError::InvalidCredentials)?;

    let password = match Password::parse(request.password) {
        Ok(password) => password,
        Err(_) => return (jar, Err(AuthAPIError::InvalidCredentials)),
    };

    let email = match Email::parse(request.email) {
        Ok(email) => email,
        Err(_) => return (jar, Err(AuthAPIError::InvalidCredentials)),
    };

    let user_store = &state.user_store.read().await;

    //user_store
    //    .validate_user(&email, &password)
    //    .await
    //    .map_err(|_| AuthAPIError::IncorrectCredentials)?;
    if user_store.validate_user(&email, &password).await.is_err() {
        return (jar, Err(AuthAPIError::IncorrectCredentials));
    }

    let user = match user_store.get_user(&email).await {
        Ok(user) => user,
        Err(_) => return (jar, Err(AuthAPIError::IncorrectCredentials)),
    };

    // Call the generate_auth_cookie function defined in the auth module.
    // If the function call fails return AuthAPIError::UnexpectedError.
    let auth_cookie = match generate_auth_cookie(&email) {
        Ok(x) => x,
        //Err(_) => return (jar, Err(AuthAPIError::UnexpectedError)),
        Err(e) => return (jar, Err(AuthAPIError::UnexpectedError(e))), // Updated!
    };

    let jar = jar.add(auth_cookie);

    // Handle request based on user's 2FA configuration
    println!("REQUIRES 2fa: {:?}", user.requires_2fa);
    match user.requires_2fa {
        true => handle_2fa(&user.email, &state, jar).await,
        false => handle_no_2fa(&user.email, jar).await,
    }
}

#[derive(Deserialize, Debug, PartialEq, Serialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[tracing::instrument(name = "handle_2fa", skip_all)]
async fn handle_2fa(
    email: &Email,
    state: &AppState,
    jar: CookieJar,
) -> (
    CookieJar,
    Result<(StatusCode, Json<LoginResponse>), AuthAPIError>,
) {
    //    let x = LoginResponse::TwoFactorAuth(TwoFactorAuthResponse {
    //        message: "2FA required".to_owned(),
    //        login_attempt_id: "123456".to_owned(),
    //    });

    // First, we must generate a new random login attempt ID and 2FA code
    let login_attempt_id = LoginAttemptId::default();
    let two_fa_code = TwoFACode::default();

    {
        let two_fa_code_store = &mut state.two_fa_code_store.write().await;

        if let Err(e) = two_fa_code_store
            .add_code(email.clone(), login_attempt_id.clone(), two_fa_code.clone())
            .await
        {
            //            return (jar, Err(AuthAPIError::UnexpectedError));
            return (jar, Err(AuthAPIError::UnexpectedError(e.into())));
        }
    }

    {
        let email_client = &mut state.email_client.read().await;

        //if let Err(e) = email_client
        //    .send_email(
        //        email,
        //        "Here is your 2FA code",
        //        &two_fa_code.clone().as_ref(),
        //    )
        //    .await
        //{
        //    //return (jar, Err(AuthAPIError::UnexpectedError));
        //    return (jar, Err(AuthAPIError::UnexpectedError(e.into())));
        //}
        if let Err(e) = email_client
            .send_email(email, "2FA Code", two_fa_code.as_ref())
            .await
        {
            return (jar, Err(AuthAPIError::UnexpectedError(e)));
        }
    }

    let response = Json(LoginResponse::TwoFactorAuth(TwoFactorAuthResponse {
        message: "2FA required".to_owned(),
        login_attempt_id: login_attempt_id.as_ref().to_string(),
    }));

    //    (jar, Ok((StatusCode::OK, response)))

    (jar, Ok((StatusCode::PARTIAL_CONTENT, response)))
}

#[tracing::instrument(name = "handle_no_2fa", skip_all)]
async fn handle_no_2fa(
    email: &Email,
    jar: CookieJar,
) -> (
    CookieJar,
    Result<(StatusCode, Json<LoginResponse>), AuthAPIError>,
) {
    let auth_cookie = match generate_auth_cookie(email) {
        Ok(cookie) => cookie,
        Err(e) => return (jar, Err(AuthAPIError::UnexpectedError(e))), // Updated!
    };

    let updated_jar = jar.add(auth_cookie);

    (
        updated_jar,
        Ok((StatusCode::OK, Json(LoginResponse::RegularAuth))),
    )
}
//async fn handle_no_2fa(
//    _email: &Email,
//    jar: CookieJar,
//) -> (
//    CookieJar,
//    Result<(StatusCode, Json<LoginResponse>), AuthAPIError>,
//) {
//    (jar, Ok((StatusCode::OK, Json(LoginResponse::RegularAuth))))
//}

// The login route can return 2 possible success responses.
// This enum models each response!
#[derive(Debug, Serialize)]
#[serde(untagged)]
pub enum LoginResponse {
    RegularAuth,
    TwoFactorAuth(TwoFactorAuthResponse),
}

// If a user requires 2FA, this JSON body should be returned!
#[derive(Debug, Serialize, Deserialize)]
pub struct TwoFactorAuthResponse {
    pub message: String,
    #[serde(rename = "loginAttemptId")]
    pub login_attempt_id: String,
}
