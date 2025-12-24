from .base import BaseAsset

class LanceAsset(BaseAsset):
    @classmethod
    def register(cls, client, catalog, namespace, name, location, **kwargs):
        # Register as "LANCE" (SCREAMING_SNAKE_CASE expected by API)
        return super().register(client, catalog, namespace, name, location, kind="LANCE", **kwargs)

    @classmethod
    def write(cls, client, catalog, namespace, name, data, location, **kwargs):
        try:
            import lancedb
        except ImportError:
            raise ImportError("Please install 'lancedb' (pip install pypangolin[lance]) to use LanceAsset")
            
        # 1. Write Data
        # lancedb.connect(uri).create_table(...) or similar
        # For simple file writing, pylance might be used, but lancedb is the primary interface.
        # Assuming location is a URI.
        
        # Simplified write logic for lancedb
        db = lancedb.connect(location)
        db.create_table(name, data=data, mode="overwrite") # or append
        
        # 2. Register
        return cls.register(client, catalog, namespace, name, location, **kwargs)
    
    @classmethod
    def read(cls, client, catalog, namespace, name):
        """Read Lance dataset from registered asset location."""
        try:
            import lancedb
        except ImportError:
            raise ImportError("Please install 'lancedb' (pip install pypangolin[lance]) to use LanceAsset")
        
        asset = client.get(f"/api/v1/catalogs/{catalog}/namespaces/{namespace}/assets/{name}")
        location = asset.get("location")
        
        # Connect to lancedb and read the table
        db = lancedb.connect(location)
        table = db.open_table(name)
        return table.to_pandas()

