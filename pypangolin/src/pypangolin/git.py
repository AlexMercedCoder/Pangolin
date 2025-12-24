from typing import List, Dict, Any, Optional
from .models import Branch, Tag, Commit, MergeOperation, Conflict
from .exceptions import NotFoundError, PangolinError

class BranchClient:
    def __init__(self, client):
        self.client = client

    def create(self, name: str, from_branch: str = "main", catalog_name: str = None) -> Branch:
        """Create a new branch."""
        payload = {
            "name": name,
            "from_branch": from_branch,
            "catalog": catalog_name
        }
        data = self.client.post("/api/v1/branches", json=payload)
        return Branch(**data)

    def list(self, catalog_name: str = None) -> List[Branch]:
        """List all branches, optionally filtered by catalog."""
        params = {}
        if catalog_name:
            params["catalog"] = catalog_name
        data = self.client.get("/api/v1/branches", params=params)
        return [Branch(**b) for b in data]

    def get(self, name: str) -> Branch:
        """Get details of a specific branch."""
        data = self.client.get(f"/api/v1/branches/{name}")
        return Branch(**data)

    def list_commits(self, branch_name: str, catalog_name: str = None) -> List[Commit]:
        """List commits for a branch."""
        params = {}
        if catalog_name:
            params["catalog"] = catalog_name
        data = self.client.get(f"/api/v1/branches/{branch_name}/commits", params=params)
        return [Commit(**c) for c in data]

    def merge(self, source_branch: str, target_branch: str, catalog_name: str = None) -> MergeOperation:
        """Trigger a merge between two branches."""
        payload = {
            "source_branch": source_branch,
            "target_branch": target_branch,
            "catalog": catalog_name
        }
        data = self.client.post("/api/v1/branches/merge", json=payload)
        return MergeOperation(**data)

    def rebase(self, branch_name: str, base_branch: str, catalog_name: str = None) -> Branch:
        """Rebase a branch onto another base branch."""
        payload = {
            "base_branch": base_branch,
            "catalog": catalog_name
        }
        data = self.client.post(f"/api/v1/branches/{branch_name}/rebase", json=payload)
        return Branch(**data)

class TagClient:
    def __init__(self, client):
        self.client = client

    def create(self, name: str, commit_id: str, catalog_name: str = None) -> Tag:
        """Create a tag on a specific commit."""
        payload = {
            "name": name,
            "commit_id": commit_id,
            "catalog": catalog_name
        }
        data = self.client.post("/api/v1/tags", json=payload)
        return Tag(**data)

    def list(self, catalog_name: str = None) -> List[Tag]:
        """List tags."""
        params = {}
        if catalog_name:
            params["catalog"] = catalog_name
        data = self.client.get("/api/v1/tags", params=params)
        return [Tag(**t) for t in data]

    def delete(self, name: str, catalog_name: str = None):
        """Delete a tag."""
        params = {}
        if catalog_name:
            params["catalog"] = catalog_name
        self.client.delete(f"/api/v1/tags/{name}", params=params) # Assuming delete supports params in client.py

class MergeOperationClient:
    def __init__(self, client):
        self.client = client
        
    def list(self, catalog_name: str) -> List[MergeOperation]:
        """List merge operations for a catalog."""
        # Note: API route is /api/v1/catalogs/:catalog_name/merge-operations
        data = self.client.get(f"/api/v1/catalogs/{catalog_name}/merge-operations")
        return [MergeOperation(**m) for m in data]
        
    def get(self, operation_id: str) -> MergeOperation:
        data = self.client.get(f"/api/v1/merge-operations/{operation_id}")
        return MergeOperation(**data)
        
    def list_conflicts(self, operation_id: str) -> List[Conflict]:
        data = self.client.get(f"/api/v1/merge-operations/{operation_id}/conflicts")
        return [Conflict(**c) for c in data]
        
    def resolve_conflict(self, conflict_id: str, resolution: str, chosen_content: Any = None):
        """Resolve a conflict. resolution: 'source', 'target', or 'custom'"""
        payload = {
            "resolution": resolution,
            "content": chosen_content
        }
        self.client.post(f"/api/v1/conflicts/{conflict_id}/resolve", json=payload)
        
    def complete(self, operation_id: str):
        """Complete a merge operation after resolving conflicts."""
        self.client.post(f"/api/v1/merge-operations/{operation_id}/complete")
        
    def abort(self, operation_id: str):
        """Abort a merge operation."""
        self.client.post(f"/api/v1/merge-operations/{operation_id}/abort")
