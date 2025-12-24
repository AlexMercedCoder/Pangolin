from .base import BaseAsset

class HudiAsset(BaseAsset):
    @classmethod
    def register(cls, client, catalog, namespace, name, location, **kwargs):
        return super().register(client, catalog, namespace, name, location, kind="HUDI_TABLE", **kwargs)

    @classmethod
    def write(cls, *args, **kwargs):
        raise NotImplementedError("Writing Hudi tables from Python directly is not yet supported. Please use Spark.")
