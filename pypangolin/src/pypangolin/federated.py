from typing import List, Dict, Any, Optional
from .models import FederatedCatalog, FederatedCatalogConfig, SyncStats, View

class FederatedCatalogClient:
    def __init__(self, client):
        self.client = client

    def create(self, name: str, uri: str, warehouse: str = None, 
               credential: str = None, properties: Dict[str, str] = None) -> FederatedCatalog:
        """Create a federated catalog."""
        config = {
            "uri": uri,
            "warehouse": warehouse,
            "credential": credential,
            "properties": properties or {}
        }
        payload = {"name": name, "config": config}
        data = self.client.post("/api/v1/federated-catalogs", json=payload)
        return FederatedCatalog(**data)

    def list(self) -> List[FederatedCatalog]:
        """List all federated catalogs."""
        data = self.client.get("/api/v1/federated-catalogs")
        return [FederatedCatalog(**c) for c in data]

    def get(self, name: str) -> FederatedCatalog:
        """Get a specific federated catalog."""
        data = self.client.get(f"/api/v1/federated-catalogs/{name}")
        return FederatedCatalog(**data)

    def delete(self, name: str):
        """Delete a federated catalog."""
        self.client.delete(f"/api/v1/federated-catalogs/{name}")

    def test_connection(self, name: str) -> Dict[str, Any]:
        """Test connection to a federated catalog."""
        return self.client.post(f"/api/v1/federated-catalogs/{name}/test", json={})

    def sync(self, name: str) -> Dict[str, Any]:
        """Trigger a sync for a federated catalog."""
        return self.client.post(f"/api/v1/federated-catalogs/{name}/sync", json={})

    def get_stats(self, name: str) -> SyncStats:
        """Get sync stats for a federated catalog."""
        data = self.client.get(f"/api/v1/federated-catalogs/{name}/stats")
        return SyncStats(**data)

class ViewClient:
    def __init__(self, client, catalog_name: str):
        self.client = client
        self.catalog_name = catalog_name

    def create(self, namespace: str, name: str, sql: str, 
               schema: Dict[str, Any] = None, properties: Dict[str, str] = None) -> View:
        """Create a view."""
        payload = {
            "name": name,
            "sql": sql,
            "schema": schema,
            "properties": properties or {}
        }
        data = self.client.post(
            f"/v1/{self.catalog_name}/namespaces/{namespace}/views",
            json=payload
        )
        return View(**data)

    def get(self, namespace: str, name: str) -> View:
        """Get a view."""
        data = self.client.get(f"/v1/{self.catalog_name}/namespaces/{namespace}/views/{name}")
        return View(**data)
