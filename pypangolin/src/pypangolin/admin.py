from typing import List, Dict, Any, Optional
from .models import AuditEvent, SystemStats, CatalogSummary, SearchResult
from .exceptions import ForbiddenError, PangolinError

class AuditClient:
    def __init__(self, client):
        self.client = client

    def list_events(self, limit: int = 100, offset: int = 0, 
                   user_id: str = None, resource_type: str = None, 
                   start_time: str = None, end_time: str = None) -> List[AuditEvent]:
        """List audit events."""
        params = {"limit": limit, "offset": offset}
        if user_id: params["user_id"] = user_id
        if resource_type: params["resource_type"] = resource_type
        if start_time: params["start_time"] = start_time
        if end_time: params["end_time"] = end_time
        
        data = self.client.get("/api/v1/audit", params=params)
        return [AuditEvent(**e) for e in data]

    def count(self) -> int:
        """Count total audit events."""
        data = self.client.get("/api/v1/audit/count")
        return data["count"]

    def get(self, event_id: str) -> AuditEvent:
        """Get a specific audit event."""
        data = self.client.get(f"/api/v1/audit/{event_id}")
        return AuditEvent(**data)

class SystemClient:
    def __init__(self, client):
        self.client = client

    def get_stats(self) -> SystemStats:
        """Get system dashboard statistics."""
        data = self.client.get("/api/v1/dashboard/stats")
        return SystemStats(**data)

    def get_catalog_summary(self, name: str) -> CatalogSummary:
        """Get summary of a specific catalog."""
        data = self.client.get(f"/api/v1/catalogs/{name}/summary")
        return CatalogSummary(**data)

    def get_settings(self) -> Dict[str, Any]:
        """Get system settings."""
        return self.client.get("/api/v1/config/settings")

    def update_settings(self, settings: Dict[str, Any]) -> Dict[str, Any]:
        """Update system settings."""
        return self.client.put("/api/v1/config/settings", json=settings)

class SearchClient:
    def __init__(self, client):
        self.client = client

    def query(self, q: str, tags: List[str] = None) -> List[SearchResult]:
        """Search for assets."""
        params = {"query": q}
        if tags:
            # B35: repeated keys (`?tags=a&tags=b`) cannot be deserialized into a
            # Vec by serde_urlencoded, which is what the server's `Query`
            # extractor uses - it 400'd. `tags[]` did not work either. The server
            # now takes one comma-separated value.
            params["tags"] = ",".join(tags)
        
        data = self.client.get("/api/v1/assets/search", params=params)
        return [SearchResult(**r) for r in data]

class TokenClient:
    def __init__(self, client):
        self.client = client

    def list_my_tokens(self, limit: int = None, offset: int = None) -> List[Dict[str, Any]]:
        params = {}
        if limit is not None: params['limit'] = limit
        if offset is not None: params['offset'] = offset
        return self.client.get("/api/v1/users/me/tokens", params=params)

    def generate(
        self,
        username: str = None,
        tenant_id: str = None,
        roles: list = None,
        expires_in_hours: int = 24,
    ) -> dict:
        """Mint a token.

        B_sdk2: this sent ``name``, ``user_id`` and ``expires_in_days``. The
        server's ``GenerateTokenRequest`` takes ``tenant_id``, ``username``,
        ``roles`` and ``expires_in_hours``, so all three were ignored and every
        token came back with the default 24-hour lifetime whatever the caller
        asked for. ``expires_in_days`` in particular meant a caller requesting
        30 days silently got one.

        The server request struct now uses ``deny_unknown_fields``, so sending
        the old shape is a loud 422 rather than a silent default.
        """
        payload = {
            "tenant_id": tenant_id or self.client._current_tenant_id,
            "expires_in_hours": expires_in_hours,
        }
        if username is not None:
            payload["username"] = username
        if roles is not None:
            payload["roles"] = roles
        return self.client.post("/api/v1/tokens", json=payload)

    def revoke(self, token_id: str):
        self.client.post(f"/api/v1/auth/revoke/{token_id}")

    def rotate(self, token_id: str) -> str:
        return self.client.post("/api/v1/tokens/rotate", json={"token_id": token_id})
