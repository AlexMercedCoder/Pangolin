from abc import ABC, abstractmethod
from ..client import PangolinClient

class BaseAsset(ABC):
    @classmethod
    @abstractmethod
    def register(cls, client: PangolinClient, catalog: str, namespace: str, name: str, location: str, **kwargs):
        pass

    @classmethod
    def write(cls, *args, **kwargs):
        raise NotImplementedError("This asset type does not support writing data yet.")
