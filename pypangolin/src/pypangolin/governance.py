from typing import List, Dict, Any
from .models import Role, Permission, ServiceUser, BusinessMetadata, AccessRequest

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

    def grant(
        self,
        user_id: str,
        actions,
        scope_type: str,
        catalog_id: str = None,
        namespace: str = None,
        asset_id: str = None,
        tag_name: str = None,
    ) -> Permission:
        """Grant a permission to a user.

        B_sdk1: this used to emit ``{"type": ..., "id": ...}`` as the scope.
        ``PermissionScope`` is an internally-tagged enum whose variants carry
        ``catalog_id`` / ``namespace`` / ``asset_id`` / ``tag_name`` - there is
        no ``id`` field on any variant - so every call 422'd. The variant fields
        are now named explicitly, and only the ones that variant defines are
        sent.

        ``actions`` accepts a single action or a list.
        """
        if isinstance(actions, str):
            actions = [actions]

        # Note the mixed casing, verified against the server's own output:
        # `PermissionScope` carries `#[serde(rename_all = "kebab-case",
        # tag = "type")]`, and for an enum `rename_all` renames the *variants*,
        # not the fields of its struct variants. So the tag values are
        # kebab-case ("catalog", "namespace") while the fields stay snake_case
        # ("catalog_id", "asset_id", "tag_name"). Guessing either way round is
        # how B_sdk1 happened.
        scope = {"type": scope_type}
        if scope_type == "catalog":
            scope["catalog_id"] = catalog_id
        elif scope_type == "namespace":
            scope["catalog_id"] = catalog_id
            scope["namespace"] = namespace
        elif scope_type == "asset":
            scope["catalog_id"] = catalog_id
            scope["namespace"] = namespace
            scope["asset_id"] = asset_id
        elif scope_type == "tag":
            scope["tag_name"] = tag_name
        elif scope_type != "tenant":
            raise ValueError(
                f"unknown scope type {scope_type!r}; expected one of "
                "tenant, catalog, namespace, asset, tag"
            )

        payload = {
            "user-id": user_id,
            "actions": list(actions),
            "scope": scope,
        }
        data = self.client.post("/api/v1/permissions", json=payload)
        return Permission(**data)

    def revoke(self, permission_id: str):
        """Revoke a permission."""
        self.client.delete(f"/api/v1/permissions/{permission_id}")

    def assign_role(self, user_id: str, role_id: str):
        """Assign a role to a user."""
        self.client.post(f"/api/v1/users/{user_id}/roles", json={"role-id": role_id})

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

    def list(self, limit: int = None, offset: int = None) -> List[ServiceUser]:
        """List service users."""
        params = {}
        if limit is not None: params['limit'] = limit
        if offset is not None: params['offset'] = offset
        data = self.client.get("/api/v1/service-users", params=params)
        return [ServiceUser(**u) for u in data]

    def get(self, service_user_id: str) -> ServiceUser:
        """Get service user details."""
        data = self.client.get(f"/api/v1/service-users/{service_user_id}")
        return ServiceUser(**data)

    def update(self, service_user_id: str, name: str = None, description: str = None, active: bool = None) -> ServiceUser:
        """Update service user."""
        payload = {}
        if name: payload["name"] = name
        if description: payload["description"] = description
        if active is not None: payload["active"] = active
        
        self.client.put(f"/api/v1/service-users/{service_user_id}", json=payload)
        # Re-fetch to return object (PUT returns 200 OK usually, but pattern varies. Assuming standard)
        return self.get(service_user_id)

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

    def update(self, asset_id: str, tags: List[str] = [], properties: Dict[str, Any] = {}, description: str = None, discoverable: bool = False) -> BusinessMetadata:
        """Update business metadata (alias for upsert)."""
        return self.upsert(asset_id, tags, properties, description, discoverable)

    def get(self, asset_id: str) -> BusinessMetadata:
        """Get metadata for an asset."""
        data = self.client.get(f"/api/v1/assets/{asset_id}/metadata")
        return BusinessMetadata(**data["metadata"])

    def delete(self, asset_id: str):
        """Delete *all* business metadata for an asset.

        B_sdk2: this used to take a ``key`` and pass it as a query parameter,
        implying a per-key delete. The server takes no such parameter and
        deletes the asset's entire metadata record - description, tags,
        properties and the ``discoverable`` flag. Callers reaching for
        "remove one property" were silently wiping everything. Removing the
        parameter makes the signature tell the truth; use :meth:`upsert` with
        the desired properties to change individual keys.
        """
        self.client.delete(f"/api/v1/assets/{asset_id}/metadata")
        
    def request_access(self, asset_id: str, reason: str = None) -> AccessRequest:
        """Request access to an asset.

        B_sdk2: the payload sent ``motivation``; the server reads ``reason``
        (``CreateAccessRequestPayload``), so the justification was dropped on
        every request and reviewers saw an empty reason with no error anywhere.
        The asset id also does not belong in the body - it is in the path.
        """
        payload = {"reason": reason}
        data = self.client.post(f"/api/v1/assets/{asset_id}/access-requests", json=payload)
        return AccessRequest(**data)
        
    def list_requests(self) -> List[AccessRequest]:
        """List access requests."""
        data = self.client.get("/api/v1/access-requests")
        return [AccessRequest(**r) for r in data]
