use axum::extract::State;
use axum::{routing::post, Json, Router};
use serde::Deserialize;
use serde_json::{json, Value};
use tower_cookies::Cookies;

use crate::crypt::{pwd, EncryptContent};
use crate::ctx::Ctx;
use crate::model::user::{UserBmc, UserForLogin};
use crate::model::ModelManager;
use crate::web::{self, remove_token_cookie, Error, Result};

pub fn routes(mm: ModelManager) -> Router {
    Router::new()
        .route("/api/login", post(api_login_handler))
        .route("/api/logout", post(api_logout_handler))
        .with_state(mm)
}

#[derive(Debug, Deserialize)]
pub struct LoginPayload {
    username: String,
    pwd: String,
}

async fn api_login_handler(
    State(mm): State<ModelManager>,
    cookies: Cookies,
    Json(payload): Json<LoginPayload>,
) -> Result<Json<Value>> {
    let LoginPayload {
        username,
        pwd: pwd_clear,
    } = payload;

    let root_ctx = Ctx::root_ctx();

    let user: UserForLogin = UserBmc::first_by_username(&root_ctx, &mm, &username)
        .await?
        .ok_or(Error::LoginFailUsernameNotFound)?;

    let user_id = user.id;

    let Some(pwd) = user.pwd else {
        return Err(Error::LoginFailedUserHasNoPassword { user_id });
    };

    pwd::validate_pwd(
        &EncryptContent {
            content: pwd_clear.clone(),
            salt: user.pwd_salt.to_string(),
        },
        &pwd,
    )
    .map_err(|_| Error::LoginFailedPasswordIncorrect { user_id })?;

    web::set_token_cookie(&cookies, &user.username, &user.token_salt.to_string())?;

    let body = Json(json!({
        "result": {
            "success": true,
        }
    }));

    Ok(body)
}

#[derive(Debug, Deserialize)]
struct LogoutPayload {
    logout: bool,
}

async fn api_logout_handler(
    cookies: Cookies,
    Json(payload): Json<LogoutPayload>,
) -> Result<Json<Value>> {
    let LogoutPayload { logout } = payload;

    if logout {
        remove_token_cookie(&cookies)?;
    }

    let body = Json(json!({
        "result": {
            "logout": logout,
        }
    }));

    Ok(body)
}
