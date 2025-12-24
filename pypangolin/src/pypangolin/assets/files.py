from .base import BaseAsset
import os

def write_dataframe(location, data, format="parquet", **kwargs):
    """
    detects if 'data' is pandas/polars/pyarrow and writes nicely
    """
    # 1. Pandas
    if hasattr(data, "to_parquet") and hasattr(data, "to_csv"): # Pandas & Polars share these usually
        # We need to distinguish or just use the method
        if format == "parquet":
            data.to_parquet(location, **kwargs)
        elif format == "csv":
            data.to_csv(location, **kwargs)
        elif format == "json":
            data.to_json(location, **kwargs)
        else:
            raise ValueError(f"Unknown format: {format}")
    else:
        # 2. PyArrow Table
        # Check if pyarrow table
        if "pyarrow" in str(type(data)):
             import pyarrow.parquet as pq
             if format == "parquet":
                 pq.write_table(data, location, **kwargs)
             else:
                 raise NotImplementedError("PyArrow table write only supported for Parquet currently.")
        else:
             raise ValueError("Data must be a Pandas/Polars DataFrame or PyArrow Table")

class ParquetAsset(BaseAsset):
    @classmethod
    def register(cls, client, catalog, namespace, name, location, **kwargs):
        return super().register(client, catalog, namespace, name, location, kind="PARQUET_TABLE", **kwargs)

    @classmethod
    def write(cls, client, catalog, namespace, name, data, location, **kwargs):
        write_dataframe(location, data, format="parquet", **kwargs)
        return cls.register(client, catalog, namespace, name, location, **kwargs)
    
    @classmethod
    def read(cls, client, catalog, namespace, name):
        """Read Parquet file from registered asset location."""
        import pandas as pd
        asset = client.get(f"/api/v1/catalogs/{catalog}/namespaces/{namespace}/assets/{name}")
        location = asset.get("location")
        return pd.read_parquet(location)

class CsvAsset(BaseAsset):
    @classmethod
    def register(cls, client, catalog, namespace, name, location, **kwargs):
        return super().register(client, catalog, namespace, name, location, kind="CSV_TABLE", **kwargs)

    @classmethod
    def write(cls, client, catalog, namespace, name, data, location, **kwargs):
        write_dataframe(location, data, format="csv", **kwargs)
        return cls.register(client, catalog, namespace, name, location, **kwargs)
    
    @classmethod
    def read(cls, client, catalog, namespace, name):
        """Read CSV file from registered asset location."""
        import pandas as pd
        asset = client.get(f"/api/v1/catalogs/{catalog}/namespaces/{namespace}/assets/{name}")
        location = asset.get("location")
        return pd.read_csv(location)

class JsonAsset(BaseAsset):
    @classmethod
    def register(cls, client, catalog, namespace, name, location, **kwargs):
        return super().register(client, catalog, namespace, name, location, kind="JSON_TABLE", **kwargs)

    @classmethod
    def write(cls, client, catalog, namespace, name, data, location, **kwargs):
        write_dataframe(location, data, format="json", **kwargs)
        return cls.register(client, catalog, namespace, name, location, **kwargs)
    
    @classmethod
    def read(cls, client, catalog, namespace, name):
        """Read JSON file from registered asset location."""
        import pandas as pd
        asset = client.get(f"/api/v1/catalogs/{catalog}/namespaces/{namespace}/assets/{name}")
        location = asset.get("location")
        return pd.read_json(location)

