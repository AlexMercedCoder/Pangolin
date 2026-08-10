use axum::{
    extract::{Request, State},
    http::{header, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};
use base64::{engine::general_purpose::STANDARD, Engine as _};
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use pangolin_core::user::{UserRole, UserSession};
use pangolin_store::CatalogStore;
use std::sync::Arc;
use uuid::Uuid;

use crate::auth::Claims;

// Claims struct and implementations moved to auth.rs

/// Generate JWT token for user session
pub fn generate_token(session: UserSession, secret: &str) -> Result<String, String> {
    let claims = Claims::from(session);

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|e| format!("Failed to generate token: {}", e))
}

/// Verify and decode JWT token
pub fn verify_token(token: &str, secret: &str) -> Result<Claims, String> {
    decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )
    .map(|data| data.claims)
    .map_err(|e| format!("Invalid token: {}", e))
}

/// Create user session with expiration
pub fn create_session(
    user_id: Uuid,
    username: String,
    tenant_id: Option<Uuid>,
    role: UserRole,
    expiration_secs: u64,
) -> UserSession {
    let now = Utc::now();
    let expires_at = now + Duration::seconds(expiration_secs as i64);

    UserSession {
        user_id,
        username,
        tenant_id,
        role,
        issued_at: now,
        expires_at,
        // Sessions built here are not derived from a presented JWT: they are
        // either about to have a token minted *from* them (login), or backed by
        // an API key / root basic auth, which have nothing to revoke. Sessions
        // that do come from a bearer token get their `jti` in
        // `Claims::to_session`.
        token_id: None,
    }
}

/// Helper function to allow Root users to override tenant context via X-Pangolin-Tenant header
fn apply_root_tenant_override(req: &mut Request, default_tenant: Uuid) {
    tracing::debug!("Root user detected, checking for X-Pangolin-Tenant header");
    if let Some(tenant_header) = req.headers().get("X-Pangolin-Tenant") {
        if let Ok(tenant_str) = tenant_header.to_str() {
            tracing::debug!("X-Pangolin-Tenant header value: {}", tenant_str);
            if let Ok(override_uuid) = Uuid::parse_str(tenant_str) {
                tracing::info!(
                    "Root user overriding tenant context: {} -> {}",
                    default_tenant,
                    override_uuid
                );
                req.extensions_mut()
                    .insert(crate::auth::TenantId(override_uuid));
            } else {
                tracing::warn!(
                    "Failed to parse X-Pangolin-Tenant header as UUID: {}",
                    tenant_str
                );
            }
        }
    }
}

/// Record an authentication-related audit event.
///
/// Audit writes used to be discarded with `let _ = ...` throughout the
/// codebase. Auth events in particular were never recorded at all (C-19), so an
/// incident responder had nothing to look at. The write is awaited and a
/// failure is logged at error level rather than being dropped silently.
async fn audit_auth_event(
    store: &Arc<dyn CatalogStore + Send + Sync>,
    tenant_id: Uuid,
    user_id: Option<Uuid>,
    username: impl Into<String>,
    action: pangolin_core::audit::AuditAction,
    detail: serde_json::Value,
) {
    let entry = pangolin_core::audit::AuditLogEntry::success(
        tenant_id,
        user_id,
        username.into(),
        action,
        pangolin_core::audit::ResourceType::Token,
        None,
        "auth".to_string(),
    )
    .with_metadata(detail);

    if let Err(e) = store.log_audit_event(tenant_id, entry).await {
        tracing::error!(error = %e, "failed to write authentication audit event");
    }
}

/// Look up a service user by the key ID embedded in a presented API key.
///
/// This is the fix for A-12. The old code ran `bcrypt::verify` against *every*
/// service user in *every* tenant until one matched — roughly 100-250 ms of CPU
/// per candidate, before any rate limiting, reachable with a bogus key. Here the
/// candidate set is narrowed by comparing public key IDs (a plain string
/// compare), so exactly one bcrypt verification runs, or none at all.
async fn authenticate_api_key(
    store: &Arc<dyn CatalogStore + Send + Sync>,
    presented: &str,
) -> Option<pangolin_core::user::ServiceUser> {
    let presented_key_id = crate::api_key::key_id_of(presented);

    if presented_key_id.is_none() && !crate::config::allow_legacy_api_keys() {
        tracing::warn!(
            "rejected an API key in the pre-key-id format; rotate the key, or set \
             PANGOLIN_ALLOW_LEGACY_API_KEYS=true to accept legacy keys during migration"
        );
        return None;
    }

    let tenants = store.list_tenants(None).await.ok()?;
    for tenant in tenants {
        let Ok(service_users) = store.list_service_users(tenant.id, None).await else {
            continue;
        };
        for service_user in service_users {
            let stored_key_id = crate::api_key::stored_key_id(&service_user.api_key_hash);
            match (presented_key_id, stored_key_id) {
                // Both keyed: cheap string compare selects the single candidate.
                (Some(a), Some(b)) if a != b => continue,
                // Presented key is keyed but this credential is legacy, or vice
                // versa: cannot match, and must not cost a bcrypt round.
                (Some(_), None) | (None, Some(_)) => continue,
                _ => {}
            }
            if crate::api_key::verify_against(presented, &service_user.api_key_hash) {
                return Some(service_user);
            }
        }
    }
    None
}

/// Bounded background updates of `service_user.last_used`.
///
/// The previous code spawned an unbounded fire-and-forget task per authenticated
/// request (A-23). A small semaphore gives the work backpressure: when the
/// budget is exhausted the update is skipped rather than queued without limit.
fn spawn_last_used_update(store: Arc<dyn CatalogStore + Send + Sync>, service_user_id: Uuid) {
    use std::sync::OnceLock;
    use tokio::sync::Semaphore;

    static BUDGET: OnceLock<Arc<Semaphore>> = OnceLock::new();
    let budget = BUDGET.get_or_init(|| Arc::new(Semaphore::new(32))).clone();

    let Ok(permit) = budget.try_acquire_owned() else {
        tracing::debug!("skipping service-user last_used update: update budget exhausted");
        return;
    };

    tokio::spawn(async move {
        let _permit = permit;
        if let Err(e) = store
            .update_service_user_last_used(service_user_id, Utc::now())
            .await
        {
            tracing::warn!(error = %e, "failed to update service user last_used");
        }
    });
}

/// Axum middleware: authenticate the request and attach session context.
///
/// Ordering matters. Public endpoints are resolved *before* any credential
/// verification so that an unauthenticated request to `/health` never reaches
/// bcrypt, and the public set is matched structurally (see [`crate::public_paths`])
/// so a resource named `config` cannot widen it.
pub async fn auth_middleware(
    State(store): State<Arc<dyn CatalogStore + Send + Sync>>,
    mut req: Request,
    next: Next,
) -> Response {
    let path = req.uri().path().to_string();
    let default_tenant = Uuid::nil();

    // NO_AUTH is a development affordance. `AppConfig::from_env_strict` refuses
    // to start the server with it enabled on a non-loopback bind address.
    if crate::config::no_auth_enabled() {
        if crate::public_paths::is_public_path(&path) {
            return next.run(req).await;
        }
        if !req.headers().contains_key(header::AUTHORIZATION) {
            let session = create_session(
                Uuid::nil(),
                "tenant_admin".to_string(),
                None,
                UserRole::TenantAdmin,
                86400,
            );
            req.extensions_mut().insert(session);
            req.extensions_mut()
                .insert(crate::auth::TenantId(default_tenant));
            return next.run(req).await;
        }
        // An Authorization header is present: fall through and verify it, so
        // tests and tools can exercise a specific identity even in NO_AUTH mode.
    }

    // Public endpoints, matched structurally rather than by suffix (A-11).
    //
    // B0o: this check used to sit *below* the API-key branch, which returned
    // unconditionally. A client that sets `X-API-Key` globally - the normal way
    // to configure an HTTP client - therefore could not reach `/v1/config`,
    // `/health` or the OAuth token endpoint at all, the opposite of the
    // documented ordering. Resolving public paths first also keeps an
    // unauthenticated `/health` probe away from bcrypt.
    if crate::public_paths::is_public_path(&path) {
        return next.run(req).await;
    }

    // Service-user API key.
    if let Some(api_key_header) = req.headers().get("X-API-Key") {
        let Ok(api_key) = api_key_header.to_str() else {
            return (StatusCode::UNAUTHORIZED, "Malformed X-API-Key header").into_response();
        };

        match authenticate_api_key(&store, api_key).await {
            Some(service_user) if service_user.is_valid() => {
                let session = create_session(
                    service_user.id,
                    service_user.name.clone(),
                    Some(service_user.tenant_id),
                    service_user.role.clone(),
                    86400,
                );
                req.extensions_mut().insert(session);
                req.extensions_mut()
                    .insert(crate::auth::TenantId(service_user.tenant_id));
                spawn_last_used_update(store.clone(), service_user.id);
                return next.run(req).await;
            }
            Some(service_user) => {
                audit_auth_event(
                    &store,
                    service_user.tenant_id,
                    Some(service_user.id),
                    service_user.name.clone(),
                    pangolin_core::audit::AuditAction::ApiKeyRejected,
                    serde_json::json!({ "reason": "inactive or expired", "path": path }),
                )
                .await;
                return (StatusCode::UNAUTHORIZED, "Invalid or expired API key").into_response();
            }
            None => {
                audit_auth_event(
                    &store,
                    default_tenant,
                    None,
                    "unknown",
                    pangolin_core::audit::AuditAction::ApiKeyRejected,
                    serde_json::json!({ "reason": "no matching key", "path": path }),
                )
                .await;
                return (StatusCode::UNAUTHORIZED, "Invalid or expired API key").into_response();
            }
        }
    }

    let auth_header = req
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|h| h.to_str().ok())
        .map(|s| s.to_string());

    // Root basic auth.
    if let Some(header_val) = auth_header.as_deref() {
        if let Some(encoded) = header_val.strip_prefix("Basic ") {
            if let Some((username, password)) = STANDARD
                .decode(encoded)
                .ok()
                .and_then(|d| String::from_utf8(d).ok())
                .and_then(|s| {
                    s.split_once(':')
                        .map(|(u, p)| (u.to_string(), p.to_string()))
                })
            {
                if let Some((root_user, root_pass)) = crate::config::root_credentials() {
                    // Constant-time comparison: `==` on &str leaks the length of
                    // the matching prefix through timing (A-14).
                    let user_ok = crate::config::constant_time_eq(&username, &root_user);
                    let pass_ok = crate::config::constant_time_eq(&password, &root_pass);
                    if user_ok && pass_ok {
                        let session = create_session(
                            Uuid::nil(),
                            "root".to_string(),
                            None,
                            UserRole::Root,
                            3600,
                        );
                        req.extensions_mut().insert(session);
                        req.extensions_mut()
                            .insert(crate::auth::TenantId(default_tenant));
                        req.extensions_mut().insert(crate::auth::RootUser);
                        apply_root_tenant_override(&mut req, default_tenant);
                        // Read what the audit needs *before* awaiting: holding a
                        // `&Request` across an await makes the middleware future
                        // non-Send, because `Request<Body>` is not `Sync`.
                        let assumed_tenant = req
                            .headers()
                            .get("X-Pangolin-Tenant")
                            .and_then(|v| v.to_str().ok())
                            .and_then(|v| Uuid::parse_str(v).ok());
                        audit_root_impersonation(
                            &store,
                            assumed_tenant,
                            default_tenant,
                            "root",
                            &path,
                        )
                        .await;
                        return next.run(req).await;
                    }
                    audit_auth_event(
                        &store,
                        default_tenant,
                        None,
                        username,
                        pangolin_core::audit::AuditAction::LoginFailed,
                        serde_json::json!({ "scheme": "basic", "path": path }),
                    )
                    .await;
                    return (StatusCode::UNAUTHORIZED, "Invalid credentials").into_response();
                }
            }
        }
    }

    let Some(token) = auth_header
        .as_deref()
        .and_then(|h| h.strip_prefix("Bearer "))
    else {
        return (
            StatusCode::UNAUTHORIZED,
            "Missing or invalid authorization header",
        )
            .into_response();
    };

    let secret = crate::config::jwt_secret();

    let claims = match verify_token(token, &secret) {
        Ok(c) => c,
        Err(e) => {
            tracing::debug!(error = %e, "rejected bearer token");
            return (StatusCode::UNAUTHORIZED, "Invalid token").into_response();
        }
    };

    // Revocation must fail *closed*. Previously an error from the store was
    // logged and ignored, so during a database blip every revoked token was
    // accepted again (A-13).
    //
    // B0o: the two nested `if let`s also failed open on the *shape* of the
    // claim. A token whose `jti` was present but not a UUID skipped the
    // revocation check entirely and was unrevocable for its whole lifetime, and
    // a token minted with no `jti` at all was likewise exempt. A malformed `jti`
    // is now a hard rejection, and the no-`jti` case is accepted only when the
    // operator has explicitly opted into legacy tokens.
    match claims.jti.as_deref() {
        Some(jti_str) => {
            let Ok(token_id) = uuid::Uuid::parse_str(jti_str) else {
                tracing::warn!(jti = %jti_str, "rejected a token with a malformed jti");
                return (StatusCode::UNAUTHORIZED, "Invalid token").into_response();
            };
            match store.is_token_revoked(token_id).await {
                Ok(true) => {
                    tracing::warn!(jti = %jti_str, "revoked token presented");
                    return (StatusCode::UNAUTHORIZED, "Token has been revoked").into_response();
                }
                Ok(false) => {}
                Err(e) => {
                    tracing::error!(
                        error = %e,
                        "token revocation check failed; rejecting the request (fail closed)"
                    );
                    return (
                        StatusCode::SERVICE_UNAVAILABLE,
                        "Unable to verify token status",
                    )
                        .into_response();
                }
            }
        }
        None => {
            if !crate::config::allow_tokens_without_jti() {
                tracing::warn!(
                    "rejected a token with no jti; it could never be revoked. Re-issue the \
                     token, or set PANGOLIN_ALLOW_TOKENS_WITHOUT_JTI=true during migration"
                );
                return (StatusCode::UNAUTHORIZED, "Invalid token").into_response();
            }
        }
    }

    let session = match claims.to_session() {
        Ok(s) => s,
        Err(e) => {
            tracing::debug!(error = %e, "token claims could not be converted to a session");
            return (StatusCode::UNAUTHORIZED, "Invalid session").into_response();
        }
    };

    if session.expires_at < Utc::now() {
        return (StatusCode::UNAUTHORIZED, "Token expired").into_response();
    }

    req.extensions_mut().insert(session.clone());

    let mut tenant_uuid = session.tenant_id.unwrap_or(default_tenant);

    let mut impersonating = false;
    if session.role == UserRole::Root {
        if let Some(override_uuid) = req
            .headers()
            .get("X-Pangolin-Tenant")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| Uuid::parse_str(s).ok())
        {
            if override_uuid != tenant_uuid {
                impersonating = true;
            }
            tenant_uuid = override_uuid;
        }
    }

    req.extensions_mut()
        .insert(crate::auth::TenantId(tenant_uuid));

    if session.role == UserRole::Root {
        req.extensions_mut().insert(crate::auth::RootUser);
    }

    if impersonating {
        tracing::warn!(
            user = %session.username,
            tenant = %tenant_uuid,
            "root session assumed another tenant's context"
        );
        audit_auth_event(
            &store,
            tenant_uuid,
            Some(session.user_id),
            session.username.clone(),
            pangolin_core::audit::AuditAction::TenantImpersonation,
            serde_json::json!({ "assumed_tenant": tenant_uuid.to_string(), "path": path }),
        )
        .await;
    }

    next.run(req).await
}

/// Audit a root basic-auth session that assumed a specific tenant context.
async fn audit_root_impersonation(
    store: &Arc<dyn CatalogStore + Send + Sync>,
    assumed: Option<Uuid>,
    default_tenant: Uuid,
    username: &str,
    path: &str,
) {
    let Some(assumed) = assumed else {
        return;
    };
    if assumed == default_tenant {
        return;
    }
    tracing::warn!(tenant = %assumed, "root basic-auth session assumed another tenant's context");
    audit_auth_event(
        store,
        assumed,
        None,
        username,
        pangolin_core::audit::AuditAction::TenantImpersonation,
        serde_json::json!({
            "assumed_tenant": assumed.to_string(),
            "scheme": "basic",
            "path": path,
        }),
    )
    .await;
}

/// Hash password using bcrypt
pub fn hash_password(password: &str) -> Result<String, String> {
    bcrypt::hash(password, bcrypt::DEFAULT_COST)
        .map_err(|e| format!("Failed to hash password: {}", e))
}

/// Verify password against hash
pub fn verify_password(password: &str, hash: &str) -> Result<bool, String> {
    bcrypt::verify(password, hash).map_err(|e| format!("Failed to verify password: {}", e))
}

/// Compile-time guard: axum requires the middleware future to be `Send`, and
/// the failure mode without this assertion is an opaque `Service` trait-bound
/// error pointing at the router rather than at the offending `.await`.
#[allow(dead_code)]
fn _assert_middleware_future_is_send() {
    fn require_send<T: Send>(_: T) {}
    let _ = |s: State<Arc<dyn CatalogStore + Send + Sync>>, r: Request, n: Next| {
        require_send(auth_middleware(s, r, n));
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_password_hashing() {
        let password = "test_password_123";
        let hash = hash_password(password).unwrap();

        assert!(verify_password(password, &hash).unwrap());
        assert!(!verify_password("wrong_password", &hash).unwrap());
    }

    #[test]
    fn test_jwt_generation_and_verification() {
        let secret = "test_secret_key_12345";
        let session = create_session(
            Uuid::new_v4(),
            "testuser".to_string(),
            None,
            UserRole::Root,
            3600,
        );

        let token = generate_token(session.clone(), secret).unwrap();
        let claims = verify_token(&token, secret).unwrap();
        let decoded_session = claims.to_session().unwrap();

        assert_eq!(decoded_session.user_id, session.user_id);
        assert_eq!(decoded_session.username, session.username);
        assert_eq!(decoded_session.role, session.role);
    }

    #[test]
    fn test_jwt_invalid_secret() {
        let secret = "test_secret_key_12345";
        let wrong_secret = "wrong_secret_key";
        let session = create_session(
            Uuid::new_v4(),
            "testuser".to_string(),
            None,
            UserRole::Root,
            3600,
        );

        let token = generate_token(session, secret).unwrap();
        assert!(verify_token(&token, wrong_secret).is_err());
    }

    #[test]
    fn test_session_creation() {
        let session = create_session(
            Uuid::new_v4(),
            "testuser".to_string(),
            Some(Uuid::new_v4()),
            UserRole::TenantAdmin,
            7200,
        );

        assert_eq!(session.username, "testuser");
        assert_eq!(session.role, UserRole::TenantAdmin);
        assert!(session.tenant_id.is_some());

        let duration = session.expires_at - session.issued_at;
        assert_eq!(duration.num_seconds(), 7200);
    }
}
