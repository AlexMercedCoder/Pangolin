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
            # Axum/serde_urlencoded often requires array notation or repeated keys handling
            # Trying tags[] for repeated keys
            params["tags[]"] = tags 
        
        data = self.client.get("/api/v1/assets/search", params=params)
        return [SearchResult(**r) for r in data]

class TokenClient:
    def __init__(self, client):
        self.client = client

    def list_my_tokens(self) -> List[Dict[str, Any]]:
        return self.client.get("/api/v1/users/me/tokens")

    def generate(self, name: str, user_id: str = None, tenant_id: str = None, expires_in_days: int = 30) -> str:
        payload = {
            "name": name, 
            "expires_in_days": expires_in_days,
            "user_id": user_id,
            "tenant_id": tenant_id or self.client._current_tenant_id
        }
        return self.client.post("/api/v1/tokens", json=payload)

    def revoke(self, token_id: str):
        self.client.post(f"/api/v1/auth/revoke/{token_id}")

    def rotate(self, token_id: str) -> str:
        return self.client.post("/api/v1/tokens/rotate", json={"token_id": token_id})
