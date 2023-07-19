use std::{
    error::Error,
    fmt::{self, Display, Formatter},
};

use axum::http::StatusCode;
use deadpool_diesel::InteractError;

use axum::response::{IntoResponse, Response};

#[derive(Debug)]
pub enum ErrResponse {
    S403(&'static str),
    S404(&'static str),
    S500(&'static str),
}

impl From<ErrResponse> for (StatusCode, &'static str) {
    fn from(err: ErrResponse) -> Self {
        match err {
            ErrResponse::S500(message) => (StatusCode::INTERNAL_SERVER_ERROR, message),
            ErrResponse::S404(message) => (StatusCode::NOT_FOUND, message),
            ErrResponse::S403(message) => (StatusCode::FORBIDDEN, message),
        }
    }
}

impl IntoResponse for ErrResponse {
    fn into_response(self) -> Response {
        Into::<(StatusCode, &'static str)>::into(self).into_response()
    }
}

impl Display for ErrResponse {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Error for ErrResponse {}

impl From<InteractError> for ErrResponse {
    fn from(_: InteractError) -> Self {
        ErrResponse::S500("database interaction error")
    }
}

impl From<diesel::result::Error> for ErrResponse {
    fn from(err: diesel::result::Error) -> ErrResponse {
        match err {
            diesel::result::Error::NotFound => ErrResponse::S404("data not found in database"),
            _ => ErrResponse::S500("database error"),
        }
    }
}
