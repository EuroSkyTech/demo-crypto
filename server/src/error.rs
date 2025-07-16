use std::sync::PoisonError;

pub(crate) use anyhow::Context;
use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use tracing::error;

#[derive(Debug)]
pub struct Error(anyhow::Error);

impl Error {
    pub(crate) fn new(msg: &'static str) -> Self {
        Error(anyhow::anyhow!(msg))
    }

    pub(crate) fn from_poison<T>(_msg: PoisonError<T>) -> Self {
        Self::new("failed to acquire lock")
    }
}

impl<E> From<E> for Error
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        Error(err.into())
    }
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        error!("{}", self.0);

        let res = (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": "Internal server error"})),
        );

        res.into_response()
    }
}

pub(crate) type Result<T> = std::result::Result<T, Error>;
