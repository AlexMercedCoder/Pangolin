import requests
from typing import Optional, List, Dict, Any, Union
from .models import Tenant, Warehouse, Catalog, Namespace, Asset
from .auth import login
from .exceptions import (
    PangolinError, AuthenticationError, AuthorizationError, 
    NotFoundError, ConflictError, ValidationError
)

class PangolinClient:
    def __init__(self, uri: str, username: str = None, password: str = None, token: str = None):
        self.uri = uri.rstrip("/")
        self._token = token
        self._current_tenant_id: Optional[str] = None
        
        if not self._token and username and password:
            self._token = login(self.uri, username, password)
            
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


class WarehouseClient:
    def __init__(self, client: PangolinClient):
        self.client = client
        
    def list(self) -> List[Warehouse]:
        data = self.client.get("/api/v1/warehouses")
        return [Warehouse(**w) for w in data]
        
    def create_s3(self, name: str, bucket: str, region: str = "us-east-1", 
                 access_key: str = None, secret_key: str = None, prefix: str = None,
                 vending_strategy: Union[str, Dict] = "AwsStatic") -> Warehouse:
        
        storage_config = {
            "s3.bucket": bucket,
            "s3.region": region,
        }
        if access_key: storage_config["s3.access-key-id"] = access_key
        if secret_key: storage_config["s3.secret-access-key"] = secret_key
        if prefix: storage_config["prefix"] = prefix

        if access_key and secret_key and vending_strategy == "AwsStatic":
            vending_strategy = {
                "AwsStatic": {
                    "access_key_id": access_key,
                    "secret_access_key": secret_key
                }
            }
        elif isinstance(vending_strategy, str):
            # Fallback for other string enums if they exist and are unit variants (e.g. "None")
             pass 

        payload = {
            "name": name,
            "storage_config": storage_config,
            "vending_strategy": vending_strategy
        }
        
        data = self.client.post("/api/v1/warehouses", json=payload)
        return Warehouse(**data)


class CatalogClient:
    def __init__(self, client: PangolinClient):
        self.client = client
        
    def list(self) -> List[Catalog]:
        data = self.client.get("/api/v1/catalogs")
        return [Catalog(**c) for c in data]
    
    def get(self, name: str) -> Catalog:
        data = self.client.get(f"/api/v1/catalogs/{name}")
        return Catalog(**data)
    
    def create(self, name: str, warehouse: str, type: str = "pantry") -> Catalog:
        payload = {
            "name": name,
            "warehouse_name": warehouse,
            "catalog_type": type # "pantry", "glue", etc. (backend uses 'catalog_type' or 'type'?)
            # Backend struct CreateCatalogRequest: { name, warehouse_name, catalog_type }
        }
        # Checking backend struct: create_catalog expects CreateCatalogRequest
        # struct: pub struct CreateCatalogRequest { pub name: String, pub warehouse_name: Option<String>, pub catalog_type: CatalogType ... }
        
        data = self.client.post("/api/v1/catalogs", json=payload)
        # Note: the backend might return the Created Catalog struct
        return Catalog(**data)
    
    def namespaces(self, catalog_name: str):
        return NamespaceClient(self.client, catalog_name)


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
