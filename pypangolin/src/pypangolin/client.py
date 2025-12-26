import requests
from typing import Optional, List, Dict, Any, Union
from .models import Tenant, Warehouse, Catalog, Namespace, Asset, User
from .auth import login
from .git import BranchClient, TagClient, MergeOperationClient
from .governance import RoleClient, PermissionClient, ServiceUserClient, BusinessMetadataClient
from .admin import AuditClient, SystemClient, SearchClient, TokenClient
from .federated import FederatedCatalogClient, ViewClient
from .exceptions import (
    PangolinError, AuthenticationError, AuthorizationError, 
    NotFoundError, ConflictError, ValidationError
)

class PangolinClient:
    def __init__(self, uri: str, username: str = None, password: str = None, token: str = None, tenant_id: str = None):
        self.uri = uri.rstrip("/")
        self._token = token
        self._current_tenant_id: Optional[str] = tenant_id
        
        if not self._token and username and password:
            self._token = login(self.uri, username, password, tenant_id=tenant_id)
            
    def login(self, username: str, password: str, tenant_id: str = None):
        """Authenticate and store token."""
        self._token = login(self.uri, username, password, tenant_id=tenant_id)
        if tenant_id:
            self._current_tenant_id = tenant_id

    @property
    def token(self) -> Optional[str]:
        return self._token
        
    @property
    def tenants(self):
        return TenantClient(self)

    @property
    def warehouses(self):
        return WarehouseClient(self)
        
    @property
    def catalogs(self):
        return CatalogClient(self)
        
    @property
    def users(self):
        return UserClient(self)

    @property
    def branches(self):
        return BranchClient(self)

    @property
    def tags(self):
        return TagClient(self)

    @property
    def merge_operations(self):
        return MergeOperationClient(self)

    @property
    def roles(self):
        return RoleClient(self)

    @property
    def permissions(self):
        return PermissionClient(self)

    @property
    def service_users(self):
        return ServiceUserClient(self)

    @property
    def metadata(self):
        return BusinessMetadataClient(self)

    @property
    def audit(self):
        return AuditClient(self)

    @property
    def system(self):
        return SystemClient(self)

    @property
    def search(self):
        return SearchClient(self)

    @property
    def tokens(self):
        return TokenClient(self)

    @property
    def federated_catalogs(self):
        return FederatedCatalogClient(self)

    def set_tenant(self, tenant_id: str):
        """Set the active tenant context for subsequent requests"""
        self._current_tenant_id = tenant_id

    def _request(self, method: str, path: str, **kwargs) -> Any:
        url = f"{self.uri}{path}"
        headers = kwargs.pop("headers", {})
        
        if self._token:
            headers["Authorization"] = f"Bearer {self._token}"
            
        if self._current_tenant_id:
            headers["X-Pangolin-Tenant"] = self._current_tenant_id
            
        try:
            response = requests.request(method, url, headers=headers, **kwargs)
        except requests.RequestException as e:
            raise PangolinError(f"Connection failed: {str(e)}")
            
        if response.status_code >= 400:
            self._handle_error(response)
            
        if response.status_code == 204:
            return None
            
        try:
            return response.json()
        except ValueError:
            return response.text
            
    def _handle_error(self, response: requests.Response):
        msg = f"{response.reason}: {response.text}"
        if response.status_code == 401:
            raise AuthenticationError(msg, response.status_code, response.text)
        elif response.status_code == 403:
            raise AuthorizationError(msg, response.status_code, response.text)
        elif response.status_code == 404:
            raise NotFoundError(msg, response.status_code, response.text)
        elif response.status_code == 409:
            raise ConflictError(msg, response.status_code, response.text)
        elif response.status_code == 422:
            raise ValidationError(msg, response.status_code, response.text)
        else:
            raise PangolinError(msg, response.status_code, response.text)

    def get(self, path: str, **kwargs):
        return self._request("GET", path, **kwargs)

    def post(self, path: str, json: Any = None, **kwargs):
        return self._request("POST", path, json=json, **kwargs)
        
    def put(self, path: str, json: Any = None, **kwargs):
        return self._request("PUT", path, json=json, **kwargs)
        
    def delete(self, path: str, **kwargs):
        return self._request("DELETE", path, **kwargs)


class TenantClient:
    def __init__(self, client: PangolinClient):
        self.client = client
    
    def list(self) -> List[Tenant]:
        data = self.client.get("/api/v1/tenants")
        return [Tenant(**t) for t in data]
        
    def create(self, name: str) -> Tenant:
        data = self.client.post("/api/v1/tenants", json={"name": name})
        return Tenant(**data)
        
    def get(self, tenant_id: str) -> Tenant:
        data = self.client.get(f"/api/v1/tenants/{tenant_id}")
        return Tenant(**data)
        
    def switch(self, name_or_id: str):
        """Convenience to find tenant and set context"""
        # Try to find by ID or Name
        tenants = self.list()
        tenant = next((t for t in tenants if t.id == name_or_id or t.name == name_or_id), None)
        
        if not tenant:
            raise NotFoundError(f"Tenant '{name_or_id}' not found")
            
        self.client.set_tenant(tenant.id)
        return tenant

    def update(self, tenant_id: str, properties: Dict[str, str]) -> Tenant:
        """Update tenant properties."""
        payload = {"properties": properties}
        data = self.client.put(f"/api/v1/tenants/{tenant_id}", json=payload)
        return Tenant(**data)

    def delete(self, tenant_id: str):
        """Delete a tenant."""
        self.client.delete(f"/api/v1/tenants/{tenant_id}")


class WarehouseClient:
    def __init__(self, client: PangolinClient):
        self.client = client
        
    def list(self) -> List[Warehouse]:
        data = self.client.get("/api/v1/warehouses")
        return [Warehouse(**w) for w in data]
        
    def create_s3(self, name: str, bucket: str, region: str = "us-east-1", 
                 access_key: str = None, secret_key: str = None, prefix: str = None,
                 endpoint: str = None,
                 vending_strategy: Union[str, Dict] = "AwsStatic",
                 **kwargs) -> Warehouse:
        
        storage_config = {
            "s3.bucket": bucket,
            "s3.region": region,
        }
        if access_key: storage_config["s3.access-key-id"] = access_key
        if secret_key: storage_config["s3.secret-access-key"] = secret_key
        if prefix: storage_config["prefix"] = prefix
        if endpoint: storage_config["s3.endpoint"] = endpoint
        
        # Merge extra config from kwargs
        storage_config.update(kwargs)

        if access_key and secret_key and vending_strategy == "AwsStatic":
            vending_strategy = {
                "AwsStatic": {
                    "access_key_id": access_key,
                    "secret_access_key": secret_key
                }
            }
        elif isinstance(vending_strategy, str):
             pass 

        payload = {
            "name": name,
            "storage_config": storage_config,
            "vending_strategy": vending_strategy
        }
        
        data = self.client.post("/api/v1/warehouses", json=payload)
        return Warehouse(**data)

    def update(self, name: str, storage_config: Dict[str, str] = None, 
               vending_strategy: Union[str, Dict] = None) -> Warehouse:
        """Update warehouse configuration."""
        payload = {}
        if storage_config:
            payload["storage_config"] = storage_config
        if vending_strategy:
            payload["vending_strategy"] = vending_strategy
        data = self.client.put(f"/api/v1/warehouses/{name}", json=payload)
        return Warehouse(**data)

    def delete(self, name: str):
        """Delete a warehouse."""
        self.client.delete(f"/api/v1/warehouses/{name}")


class UserClient:
    def __init__(self, client: PangolinClient):
        self.client = client
        
    def create(self, username: str, email: str, role: str, tenant_id: str = None, password: str = None) -> User:
        payload = {
            "username": username,
            "email": email,
            "role": role, # "TenantAdmin", "TenantUser", "Root"
            "tenant_id": tenant_id,
        }
        if password:
            payload["password"] = password
            
        data = self.client.post("/api/v1/users", json=payload)
        return User(**data)
    
    def list(self) -> List[User]:
        data = self.client.get("/api/v1/users")
        return [User(**u) for u in data]


class CatalogClient:
    def __init__(self, client: PangolinClient):
        self.client = client
        
    def list(self) -> List[Catalog]:
        data = self.client.get("/api/v1/catalogs")
        return [Catalog(**c) for c in data]
    
    def get(self, name: str) -> Catalog:
        data = self.client.get(f"/api/v1/catalogs/{name}")
        return Catalog(**data)
    
    def create(self, name: str, warehouse: str, type: str = "Local") -> Catalog:
        payload = {
            "name": name,
            "warehouse_name": warehouse,
            "catalog_type": type # "Local", "Federated"
        }
        
        data = self.client.post("/api/v1/catalogs", json=payload)
        return Catalog(**data)
    
    def update(self, name: str, properties: Dict[str, str]) -> Catalog:
        """Update catalog properties."""
        payload = {"properties": properties}
        data = self.client.put(f"/api/v1/catalogs/{name}", json=payload)
        return Catalog(**data)

    def delete(self, name: str):
        """Delete a catalog."""
        self.client.delete(f"/api/v1/catalogs/{name}")
    
    def namespaces(self, catalog_name: str):
        return NamespaceClient(self.client, catalog_name)

    def views(self, catalog_name: str):
        """Get view client for this catalog."""
        return ViewClient(self.client, catalog_name)


class NamespaceClient:
    def __init__(self, client: PangolinClient, catalog_name: str):
        self.client = client
        self.catalog_name = catalog_name
        
    def create(self, namespace: List[str], properties: Dict[str, str] = None) -> Namespace:
        payload = {
            "namespace": namespace,
            "properties": properties or {}
        }
        data = self.client.post(f"/v1/{self.catalog_name}/namespaces", json=payload)
        return Namespace(**data)
        
    def list(self, parent: str = None) -> List[List[str]]:
        params = {}
        if parent: params["parent"] = parent
        data = self.client.get(f"/v1/{self.catalog_name}/namespaces", params=params)
        return data["namespaces"] # returns Vec<Vec<String>>

    def register_asset(self, namespace: str, name: str, kind: str, location: str, properties: Dict[str, str] = None) -> Dict[str, Any]:
        """
        Register a generic asset (non-Iceberg) in the catalog.
        """
        payload = {
            "name": name,
            "kind": kind,
            "location": location,
            "properties": properties or {}
        }
        # Note: The API path for asset registration is under /api/v1/catalogs, not /v1 (Iceberg)
        # We need to construct the full path carefully.
        # Based on search results: /api/v1/catalogs/:catalog_name/namespaces/:namespace/assets
        return self.client.post(f"/api/v1/catalogs/{self.catalog_name}/namespaces/{namespace}/assets", json=payload)
