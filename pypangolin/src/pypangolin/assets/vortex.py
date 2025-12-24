from .base import BaseAsset

class VortexAsset(BaseAsset):
    @classmethod
    def register(cls, client, catalog, namespace, name, location, **kwargs):
        return super().register(client, catalog, namespace, name, location, kind="VORTEX", **kwargs)

    @classmethod
    def write(cls, *args, **kwargs):
        raise NotImplementedError("Writing Vortex files from Python directly via pypangolin is not yet supported. Please use appropriate libraries.")
