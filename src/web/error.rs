use std::sync::Arc;

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;
use serde_with::serde_as;

use crate::{crypt, model, web};

pub type Result<T> = core::result::Result<T, Error>;

#[serde_as]
#[derive(Debug, Serialize, strum_macros::AsRefStr)]
#[serde(tag = "type", content = "data")]
pub enum Error {
    LoginFailUsernameNotFound,
    LoginFailedUserHasNoPassword { user_id: i64 },
    LoginFailedPasswordIncorrect { user_id: i64 },

    RpcMethodUnknown(String),
    RpcMissingParams { method: String },
    RpcFailJsonParams { method: String },

    CtxExt(web::mw_auth::CtxExtError),
    Model(model::Error),
    Crypt(crypt::Error),

    SerdeJson(String),
}

impl core::fmt::Display for Error {
    fn fmt(&self, fmt: &mut core::fmt::Formatter) -> core::result::Result<(), core::fmt::Error> {
        write!(fmt, "{self:?}")
    }
}

impl std::error::Error for Error {}

impl From<model::Error> for Error {
    fn from(val: model::Error) -> Self {
        Self::Model(val)
    }
}

impl From<web::mw_auth::CtxExtError> for Error {
    fn from(val: web::mw_auth::CtxExtError) -> Self {
        Self::CtxExt(val)
    }
}

impl From<crypt::Error> for Error {
    fn from(val: crypt::Error) -> Self {
        Self::Crypt(val)
    }
}

impl From<serde_json::Error> for Error {
    fn from(val: serde_json::Error) -> Self {
        Self::SerdeJson(val.to_string())
    }
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        let mut response = StatusCode::INTERNAL_SERVER_ERROR.into_response();

        response.extensions_mut().insert(Arc::new(self));

        response
    }
}

impl Error {
    pub fn client_status_and_error(&self) -> (StatusCode, ClientError) {
        use web::Error::*;

        match self {
            LoginFailUsernameNotFound
            | LoginFailedUserHasNoPassword { .. }
            | LoginFailedPasswordIncorrect { .. } => {
                (StatusCode::FORBIDDEN, ClientError::LOGIN_FAILED)
            }
            Model(model::Error::EntityNotFound { entity, id }) => (
                StatusCode::BAD_REQUEST,
                ClientError::ENTITY_NOT_FOUND { entity, id: *id },
            ),
            CtxExt(_) => (StatusCode::FORBIDDEN, ClientError::CTX_ERROR),
            _ => (
                StatusCode::INTERNAL_SERVER_ERROR,
                ClientError::SERVICE_ERROR,
            ),
        }
    }
}

#[derive(Debug, Serialize, strum_macros::AsRefStr)]
#[serde(tag = "message", content = "detail")]
#[allow(non_camel_case_types)]
pub enum ClientError {
    ENTITY_NOT_FOUND { entity: &'static str, id: i64 },
    LOGIN_FAILED,
    NO_AUTH,
    CTX_ERROR,
    SERVICE_ERROR,
}
