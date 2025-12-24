from .base import BaseAsset

class ParquetAsset(BaseAsset):
    @classmethod
    def register(cls, client, catalog, namespace, name, location, **kwargs):
        # TODO
        pass

    @classmethod
    def write(cls, client, catalog, namespace, name, data, location, **kwargs):
        # 1. Write Data (using pyarrow/polars/pandas)
        # TODO: Detect dataframe library and write
        
        # 2. Register
        cls.register(client, catalog, namespace, name, location)
