"""
NoSQL database connection assets (MongoDB).
"""

from .base import BaseConnectionAsset
from typing import Optional


class MongoDBAsset(BaseConnectionAsset):
    """MongoDB database connection asset."""
    
    REQUIRED_DEPS = ["pymongo"]
    
    @classmethod
    def connect(cls, client, catalog: str, namespace: str, name: str,
               encryption_key: Optional[str] = None):
        """
        Create and return a pymongo MongoClient object.
        
        Returns:
            pymongo.MongoClient object
        """
        try:
            from pymongo import MongoClient
        except ImportError:
            raise ImportError(
                "pymongo is required for MongoDB connections. "
                "Install with: pip install 'pypangolin[mongodb]'"
            )
        
        connection_string, creds = cls._get_decrypted_credentials(
            client, catalog, namespace, name, encryption_key
        )
        
        # MongoDB connection string format: mongodb://[username:password@]host:port/database
        # Build full connection string with credentials
        if "username" in creds and "password" in creds:
            # Insert credentials into connection string
            if "://" in connection_string:
                protocol, rest = connection_string.split("://", 1)
                # Check if credentials already in string
                if "@" not in rest:
                    connection_string = f"{protocol}://{creds['username']}:{creds['password']}@{rest}"
        
        return MongoClient(connection_string)
