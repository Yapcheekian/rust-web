use crate::crypt::token::{validate_web_token, Token};
use crate::ctx::Ctx;
use crate::model::user::{UserBmc, UserForAuth};
use crate::model::ModelManager;
use crate::web::{set_token_cookie, AUTH_TOKEN};
use crate::web::{Error, Result};
use async_trait::async_trait;
use axum::extract::FromRequestParts;
use axum::extract::State;
use axum::http::request::Parts;
use axum::http::Request;
use axum::middleware::Next;
use axum::response::Response;
use serde::Serialize;
use tower_cookies::{Cookie, Cookies};
use tracing::info;

pub async fn mw_ctx_require<B>(
    ctx: Result<CtxW>,
    req: Request<B>,
    next: Next<B>,
) -> Result<Response> {
    info!("->> {:<12} - mw_ctx_require - {ctx:?}", "MIDDLEWARE");

    ctx?;

    Ok(next.run(req).await)
}

pub async fn mw_ctx_resolve<B>(
    mm: State<ModelManager>,
    cookies: Cookies,
    mut req: Request<B>,
    next: Next<B>,
) -> Result<Response> {
    info!("->> {:<12} - mw_ctx_resolve", "MIDDLEWARE");

    let ctx_ext_result = _ctx_resolve(mm, &cookies).await;

    if ctx_ext_result.is_err() && !matches!(ctx_ext_result, Err(CtxExtError::TokenNotInCookie)) {
        cookies.remove(Cookie::named(AUTH_TOKEN));
    }

    req.extensions_mut().insert(ctx_ext_result);

    Ok(next.run(req).await)
}

async fn _ctx_resolve(mm: State<ModelManager>, cookies: &Cookies) -> CtxExtResult {
    let token = cookies
        .get(AUTH_TOKEN)
        .map(|c| c.value().to_string())
        .ok_or(CtxExtError::TokenNotInCookie)?;

    let token: Token = token.parse().map_err(|_| CtxExtError::TokenWrongFormat)?;

    let user: UserForAuth = UserBmc::first_by_username(&Ctx::root_ctx(), &mm, &token.ident)
        .await
        .map_err(|ex| CtxExtError::ModelAccessError(ex.to_string()))?
        .ok_or(CtxExtError::UserNotFound)?;

    validate_web_token(&token, &user.token_salt.to_string())
        .map_err(|_| CtxExtError::FailValidate)?;

    set_token_cookie(&cookies, &user.username, &user.token_salt.to_string())
        .map_err(|_| CtxExtError::CannotSetTokenCookie)?;

    Ctx::new(user.id)
        .map(CtxW)
        .map_err(|ex| CtxExtError::CtxCreateFail(ex.to_string()))
}

#[derive(Debug, Clone)]
pub struct CtxW(pub Ctx);

#[async_trait]
impl<S: Send + Sync> FromRequestParts<S> for CtxW {
    type Rejection = Error;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self> {
        info!("->> {:<12} - Ctx", "EXTRACTOR");

        parts
            .extensions
            .get::<CtxExtResult>()
            .ok_or(Error::CtxExt(CtxExtError::CtxNotInRequestExt))?
            .clone()
            .map_err(Error::CtxExt)
    }
}

type CtxExtResult = core::result::Result<CtxW, CtxExtError>;

#[derive(Clone, Serialize, Debug)]
pub enum CtxExtError {
    TokenNotInCookie,
    TokenWrongFormat,

    UserNotFound,
    ModelAccessError(String),
    FailValidate,
    CannotSetTokenCookie,

    CtxNotInRequestExt,
    CtxCreateFail(String),
}
