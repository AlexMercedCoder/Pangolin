//! Per-request correlation IDs, access logging, and metric recording.
//!
//! Nothing propagated a request or trace identifier before, so two log lines
//! from the same request could not be tied together (A-17/A-18). This
//! middleware assigns an ID (honouring an inbound `X-Request-Id` from a trusted
//! gateway), puts it on a tracing span that covers the whole handler, echoes it
//! back on the response, and feeds latency and status into [`crate::metrics`].

use std::time::Instant;

use axum::extract::{MatchedPath, Request};
use axum::http::HeaderValue;
use axum::middleware::Next;
use axum::response::Response;
use tracing::Instrument;
use uuid::Uuid;

/// Header used to carry the correlation ID in both directions.
pub const REQUEST_ID_HEADER: &str = "x-request-id";

/// A correlation ID attached to the request extensions.
#[derive(Debug, Clone)]
pub struct RequestId(pub String);

/// Longest inbound request ID we will echo, to bound log-line size.
const MAX_INBOUND_REQUEST_ID: usize = 128;

fn inbound_request_id(req: &Request) -> Option<String> {
    let value = req.headers().get(REQUEST_ID_HEADER)?.to_str().ok()?;
    // Only accept a conservative character set: this value ends up in logs.
    if value.is_empty()
        || value.len() > MAX_INBOUND_REQUEST_ID
        || !value
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.'))
    {
        return None;
    }
    Some(value.to_string())
}

/// Assign a request ID, time the request, log it, and record metrics.
pub async fn track_request(mut req: Request, next: Next) -> Response {
    let request_id = inbound_request_id(&req).unwrap_or_else(|| Uuid::new_v4().to_string());
    req.extensions_mut().insert(RequestId(request_id.clone()));

    let method = req.method().clone();
    // Prefer the matched route pattern over the raw path, so metric labels stay
    // bounded: `/v1/:prefix/namespaces/:namespace` rather than one series per
    // namespace a user happens to create.
    let route = req
        .extensions()
        .get::<MatchedPath>()
        .map(|p| p.as_str().to_string())
        .unwrap_or_else(|| req.uri().path().to_string());

    let span = tracing::info_span!(
        "http_request",
        request_id = %request_id,
        method = %method,
        route = %route,
    );

    let started = Instant::now();
    // `Instrument` rather than `span.enter()`: a span guard held across an
    // await point attaches the span to whatever task the executor runs next.
    let mut response = next.run(req).instrument(span.clone()).await;
    let elapsed = started.elapsed();
    let _enter = span.enter();
    let status = response.status();

    if let Ok(value) = HeaderValue::from_str(&request_id) {
        response.headers_mut().insert(REQUEST_ID_HEADER, value);
    }

    crate::metrics::record_request(
        method.as_str(),
        &route,
        status.as_u16(),
        elapsed.as_secs_f64(),
    );

    if status.is_server_error() {
        tracing::error!(
            status = status.as_u16(),
            latency_ms = elapsed.as_millis() as u64,
            "request failed"
        );
    } else if status == axum::http::StatusCode::UNAUTHORIZED
        || status == axum::http::StatusCode::FORBIDDEN
    {
        crate::metrics::inc(&crate::metrics::AUTH_FAILURE);
        tracing::info!(
            status = status.as_u16(),
            latency_ms = elapsed.as_millis() as u64,
            "request rejected"
        );
    } else {
        crate::metrics::inc(&crate::metrics::AUTH_SUCCESS);
        tracing::debug!(
            status = status.as_u16(),
            latency_ms = elapsed.as_millis() as u64,
            "request completed"
        );
    }

    response
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::Request as HttpRequest;

    fn request_with_id(value: &str) -> Request {
        HttpRequest::builder()
            .uri("/health")
            .header(REQUEST_ID_HEADER, value)
            .body(Body::empty())
            .unwrap()
    }

    #[test]
    fn well_formed_inbound_ids_are_accepted() {
        assert_eq!(
            inbound_request_id(&request_with_id("abc-123_XYZ.4")),
            Some("abc-123_XYZ.4".to_string())
        );
    }

    #[test]
    fn hostile_inbound_ids_are_ignored() {
        // Newlines would forge log lines; overlong values bloat them.
        assert_eq!(inbound_request_id(&request_with_id("a b")), None);
        assert_eq!(inbound_request_id(&request_with_id("../etc/passwd")), None);
        assert_eq!(
            inbound_request_id(&request_with_id(&"x".repeat(MAX_INBOUND_REQUEST_ID + 1))),
            None
        );
        assert_eq!(inbound_request_id(&request_with_id("")), None);
    }
}
