from typing import List, Dict, Any, Optional
from .models import Role, Permission, ServiceUser, BusinessMetadata, AccessRequest
from .exceptions import NotFoundError, ForbiddenError, PangolinError

class RoleClient:
    def __init__(self, client):
        self.client = client

    def create(self, name: str, description: str = None) -> Role:
        """Create a new role."""
        payload = {
            "name": name, 
            "tenant-id": self.client._current_tenant_id,
            "description": description
        }
        data = self.client.post("/api/v1/roles", json=payload)
        return Role(**data)

    def list(self) -> List[Role]:
        """List all roles."""
        data = self.client.get("/api/v1/roles")
        return [Role(**r) for r in data]

    def get(self, role_id: str) -> Role:
        """Get role details."""
        data = self.client.get(f"/api/v1/roles/{role_id}")
        return Role(**data)
        
    def delete(self, role_id: str):
        """Delete a role."""
        self.client.delete(f"/api/v1/roles/{role_id}")

class PermissionClient:
    def __init__(self, client):
        self.client = client

    def grant(self, role_id: str, action: str, scope_type: str, scope_id: str = None) -> Permission:
        """Grant a permission to a role."""
        payload = {
            "role_id": role_id,
            "action": action,
            "scope": {
                "type": scope_type,
                "id": scope_id
            }
        }
        data = self.client.post("/api/v1/permissions", json=payload)
        return Permission(**data)

    def revoke(self, permission_id: str):
        """Revoke a permission."""
        self.client.delete(f"/api/v1/permissions/{permission_id}")

    def assign_role(self, user_id: str, role_id: str):
        """Assign a role to a user."""
        self.client.post(f"/api/v1/users/{user_id}/roles", json={"role_id": role_id})

    def revoke_role(self, user_id: str, role_id: str):
        """Revoke a role from a user."""
        self.client.delete(f"/api/v1/users/{user_id}/roles/{role_id}")
        
    def get_user_roles(self, user_id: str) -> List[Role]:
        """Get roles assigned to a user."""
        data = self.client.get(f"/api/v1/users/{user_id}/roles")
        return [Role(**r) for r in data]

class ServiceUserClient:
    def __init__(self, client):
        self.client = client

    def create(self, name: str, role: str = "tenant-user") -> ServiceUser:
        """Create a service user."""
        payload = {"name": name, "role": role}
        data = self.client.post("/api/v1/service-users", json=payload)
        return ServiceUser(**data)

    def list(self) -> List[ServiceUser]:
        """List service users."""
        data = self.client.get("/api/v1/service-users")
        return [ServiceUser(**u) for u in data]

    def rotate_key(self, service_user_id: str) -> ServiceUser:
        """Rotate API key for a service user."""
        data = self.client.post(f"/api/v1/service-users/{service_user_id}/rotate")
        return ServiceUser(**data)
        
    def delete(self, service_user_id: str):
        """Delete a service user."""
        self.client.delete(f"/api/v1/service-users/{service_user_id}")

class BusinessMetadataClient:
    def __init__(self, client):
        self.client = client

    def upsert(self, asset_id: str, tags: List[str] = [], properties: Dict[str, Any] = {}, description: str = None, discoverable: bool = False) -> BusinessMetadata:
        """Upsert business metadata."""
        payload = {
            "tags": tags,
            "properties": properties,
            "description": description,
            "discoverable": discoverable
        }
        data = self.client.post(f"/api/v1/assets/{asset_id}/metadata", json=payload)
        return BusinessMetadata(**data["metadata"])

    def get(self, asset_id: str) -> BusinessMetadata:
        """Get metadata for an asset."""
        data = self.client.get(f"/api/v1/assets/{asset_id}/metadata")
        return BusinessMetadata(**data["metadata"])

    def delete(self, asset_id: str, key: str):
        """Delete metadata from an asset."""
        self.client.delete(f"/api/v1/assets/{asset_id}/metadata", params={"key": key})
        
    def request_access(self, asset_id: str, motivation: str) -> AccessRequest:
        """Request access to an asset."""
        payload = {"asset_id": asset_id, "motivation": motivation}
        data = self.client.post(f"/api/v1/assets/{asset_id}/access-requests", json=payload)
        return AccessRequest(**data)
        
    def list_requests(self) -> List[AccessRequest]:
        """List access requests."""
        data = self.client.get("/api/v1/access-requests")
        return [AccessRequest(**r) for r in data]
