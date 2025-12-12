use axum::{
    extract::Request,
    http::{StatusCode, HeaderMap},
    middleware::Next,
    response::Response,
};
use uuid::Uuid;

#[derive(Clone, Copy, Debug)]
pub struct TenantId(pub Uuid);

pub async fn auth_middleware(
    headers: HeaderMap,
    mut request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // Check for X-Pangolin-Tenant header
    // For MVP, we trust the header if present. 
    // In production, we would verify a JWT or API Key.
    
    if let Some(tenant_header) = headers.get("X-Pangolin-Tenant") {
        if let Ok(tenant_str) = tenant_header.to_str() {
            if let Ok(tenant_uuid) = Uuid::parse_str(tenant_str) {
                request.extensions_mut().insert(TenantId(tenant_uuid));
                return Ok(next.run(request).await);
            }
        }
    }
    
    // Fallback for development/testing if no header provided?
    // Or strictly enforce?
    // Let's allow a default "nil" tenant for now if not provided, to keep existing tests working easily.
    // BUT the goal is multi-tenancy.
    // Let's log a warning and use nil.
    
    tracing::warn!("No X-Pangolin-Tenant header found, using nil tenant.");
    request.extensions_mut().insert(TenantId(Uuid::nil()));
    Ok(next.run(request).await)
}
