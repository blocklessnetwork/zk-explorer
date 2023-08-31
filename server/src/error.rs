use axum::{http::StatusCode, response::IntoResponse};

pub type AxumResult<T> = core::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    NotFound,
    FailedToCreate,
    FailedToUpdate,
    FailedToVerify,
    LoginFail,
}

impl IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        match self {
            Error::NotFound => (StatusCode::NOT_FOUND, "Not Found").into_response(),
            Error::FailedToCreate => (StatusCode::BAD_REQUEST, "Failed to Create").into_response(),
            _ => (StatusCode::INTERNAL_SERVER_ERROR, "Unhandled Client Error").into_response(),
        }
    }
}
