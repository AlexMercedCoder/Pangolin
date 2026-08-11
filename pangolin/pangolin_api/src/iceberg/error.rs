//! The Iceberg REST error envelope.
//!
//! The spec requires errors on `/v1/*` to be shaped as
//!
//! ```json
//! { "error": { "message": "...", "type": "...", "code": 409 } }
//! ```
//!
//! Pangolin emitted a flat `{"error": "<string>"}`, and most Iceberg handlers
//! bypassed the error type entirely and returned bare `(StatusCode, &str)`
//! tuples with a plain-text body (A-6, and still open as B16j at the August
//! audit despite this module reading as though it were resolved). Every return
//! in `iceberg/` now routes through the helpers below; there are zero bare
//! tuples left in that module. Engines parse the envelope to tell a
//! `NoSuchTableException` from a `CommitFailedException`, which is what drives
//! their retry logic, so a non-conforming body breaks retries rather than
//! merely looking untidy.

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::{Deserialize, Serialize};

/// The `error` object of an Iceberg REST error response.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct IcebergErrorDetail {
    pub message: String,
    #[serde(rename = "type")]
    pub error_type: String,
    pub code: u16,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub stack: Vec<String>,
}

/// A spec-conforming Iceberg REST error body.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct IcebergErrorResponse {
    pub error: IcebergErrorDetail,
}

impl IcebergErrorResponse {
    pub fn new(status: StatusCode, error_type: &str, message: impl Into<String>) -> Self {
        Self {
            error: IcebergErrorDetail {
                message: message.into(),
                error_type: error_type.to_string(),
                code: status.as_u16(),
                stack: Vec::new(),
            },
        }
    }
}

/// Build a spec-conforming error response.
pub fn iceberg_error(status: StatusCode, error_type: &str, message: &str) -> Response {
    (
        status,
        Json(IcebergErrorResponse::new(status, error_type, message)),
    )
        .into_response()
}

/// `404` for a table that does not exist.
pub fn no_such_table(identifier: &str) -> Response {
    iceberg_error(
        StatusCode::NOT_FOUND,
        "NoSuchTableException",
        &format!("Table does not exist: {identifier}"),
    )
}

/// `404` for a namespace that does not exist.
pub fn no_such_namespace(namespace: &str) -> Response {
    iceberg_error(
        StatusCode::NOT_FOUND,
        "NoSuchNamespaceException",
        &format!("Namespace does not exist: {namespace}"),
    )
}

/// `403` for an authenticated caller lacking permission.
pub fn forbidden(detail: &str) -> Response {
    iceberg_error(StatusCode::FORBIDDEN, "ForbiddenException", detail)
}

/// `404` for a view that does not exist.
pub fn no_such_view(identifier: &str) -> Response {
    iceberg_error(
        StatusCode::NOT_FOUND,
        "NoSuchViewException",
        &format!("View does not exist: {identifier}"),
    )
}

/// `400` for a malformed or unusable request.
pub fn bad_request(detail: &str) -> Response {
    iceberg_error(StatusCode::BAD_REQUEST, "BadRequestException", detail)
}

/// `409` for a table that already exists.
pub fn table_already_exists(identifier: &str) -> Response {
    iceberg_error(
        StatusCode::CONFLICT,
        "AlreadyExistsException",
        &format!("Table already exists: {identifier}"),
    )
}

/// `409` for a namespace that already exists.
pub fn namespace_already_exists(namespace: &str) -> Response {
    iceberg_error(
        StatusCode::CONFLICT,
        "AlreadyExistsException",
        &format!("Namespace already exists: {namespace}"),
    )
}

/// `409` for a namespace that still has children.
pub fn namespace_not_empty(namespace: &str) -> Response {
    iceberg_error(
        StatusCode::CONFLICT,
        "NamespaceNotEmptyException",
        &format!("Namespace is not empty: {namespace}"),
    )
}

/// `500`, with the underlying cause logged rather than returned.
pub fn internal(context: &str) -> Response {
    iceberg_error(
        StatusCode::INTERNAL_SERVER_ERROR,
        "InternalServerError",
        context,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn envelope_matches_the_spec_shape() {
        let body = IcebergErrorResponse::new(
            StatusCode::CONFLICT,
            "CommitFailedException",
            "ref main points at 222, expected 111",
        );
        let json = serde_json::to_value(&body).unwrap();
        assert_eq!(json["error"]["code"], 409);
        assert_eq!(json["error"]["type"], "CommitFailedException");
        assert_eq!(
            json["error"]["message"],
            "ref main points at 222, expected 111"
        );
        // `stack` is optional and must be omitted when empty.
        assert!(json["error"].get("stack").is_none());
    }

    #[test]
    fn envelope_round_trips() {
        let body = IcebergErrorResponse::new(StatusCode::NOT_FOUND, "NoSuchTableException", "gone");
        let text = serde_json::to_string(&body).unwrap();
        let back: IcebergErrorResponse = serde_json::from_str(&text).unwrap();
        assert_eq!(back, body);
    }
}
