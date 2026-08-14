use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use domain::generate::GenerationError;
use serde::Serialize;

use crate::dto::FormParseError;

#[derive(Debug)]
pub enum ApiError {
    Generation(GenerationError),
    InvalidField { field: &'static str, value: String },
    InvalidCount { count: u8, max: u8 },
}

#[derive(Serialize)]
struct ErrorBody {
    error: String,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let message = match &self {
            ApiError::Generation(err) => err.to_string(),
            ApiError::InvalidField { field, value } => {
                format!("'{value}' is not a valid value for '{field}'")
            }
            ApiError::InvalidCount { count, max } => {
                format!("count must be between 1 and {max}, got {count}")
            }
        };
        (StatusCode::BAD_REQUEST, Json(ErrorBody { error: message })).into_response()
    }
}

impl From<GenerationError> for ApiError {
    fn from(err: GenerationError) -> Self {
        ApiError::Generation(err)
    }
}

impl From<FormParseError> for ApiError {
    fn from(err: FormParseError) -> Self {
        ApiError::InvalidField {
            field: err.field,
            value: err.value,
        }
    }
}
