//! This module holds the errors and the error conversion for handlers
//! that are returned from handlers

use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::response::Response;
use axum::Json;
use swaggapi::as_responses::simple_responses;
use swaggapi::as_responses::AsResponses;
use swaggapi::as_responses::SimpleResponse;
use swaggapi::internals::SchemaGenerator;
use swaggapi::re_exports::openapiv3;
use swaggapi::re_exports::openapiv3::MediaType;
use swaggapi::re_exports::openapiv3::Responses;
use thiserror::Error;

use crate::http::common::schemas::ApiStatusCode;

/// A type alias that includes the ApiError
pub type ApiResult<T> = Result<T, ApiError>;

/// The common error that is returned from the handlers
#[derive(Debug, Error)]
#[allow(missing_docs)]
pub enum ApiError {
    #[error("Unauthenticated")]
    Unauthenticated,

    #[error("An internal server error occurred")]
    InternalServerError,
    #[error("Error occurred while accessing the session: {0}")]
    SessionError(#[from] tower_sessions::session::Error),
    #[error("Database error occurred: {0}")]
    Database(#[from] rorm::Error),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        const UNAUTHENTICATED: &str = "Unauthenticated";
        const INTERNAL: &str = "Internal server error occurred";

        let (status_code, message) = match self {
            ApiError::Unauthenticated => {
                (ApiStatusCode::Unauthenticated, UNAUTHENTICATED.to_string())
            }
            ApiError::InternalServerError | ApiError::SessionError(_) | ApiError::Database(_) => {
                (ApiStatusCode::InternalServerError, INTERNAL.to_string())
            }
        };

        let res = (
            if status_code as u16 >= 2000 {
                StatusCode::BAD_REQUEST
            } else {
                StatusCode::INTERNAL_SERVER_ERROR
            },
            Json(crate::http::common::schemas::ApiErrorResponse {
                status_code,
                message,
            }),
        );

        res.into_response()
    }
}

impl AsResponses for ApiError {
    fn responses(gen: &mut SchemaGenerator) -> Responses {
        let media_type = Some(MediaType {
            schema: Some(gen.generate::<crate::http::common::schemas::ApiErrorResponse>()),
            ..Default::default()
        });

        simple_responses([
            SimpleResponse {
                status_code: openapiv3::StatusCode::Code(400),
                mime_type: "application/json".parse().unwrap(),
                description: "Client side error".to_string(),
                media_type: media_type.clone(),
            },
            SimpleResponse {
                status_code: openapiv3::StatusCode::Code(500),
                mime_type: "application/json".parse().unwrap(),
                description: "Server side error".to_string(),
                media_type,
            },
        ])
    }
}