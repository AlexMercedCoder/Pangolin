"""
Database connection assets with encrypted credential storage.
"""

from typing import Dict, Optional, Tuple
from cryptography.fernet import Fernet


class BaseConnectionAsset:
    """Base class for database connection assets with encrypted credentials."""
    
    ASSET_KIND = "DB_CONN_STRING"
    REQUIRED_DEPS = []  # Override in subclasses
    
    @staticmethod
    def _generate_key() -> str:
        """Generate a new Fernet encryption key."""
        return Fernet.generate_key().decode('utf-8')
    
    @staticmethod
    def _encrypt_value(value: str, key: str) -> str:
        """Encrypt a single value using Fernet."""
        f = Fernet(key.encode('utf-8'))
        return f.encrypt(value.encode('utf-8')).decode('utf-8')
    
    @staticmethod
    def _decrypt_value(encrypted_value: str, key: str) -> str:
        """Decrypt a single value using Fernet."""
        f = Fernet(key.encode('utf-8'))
        return f.decrypt(encrypted_value.encode('utf-8')).decode('utf-8')
    
    @classmethod
    def _encrypt_credentials(cls, credentials: Dict[str, str], 
                            encryption_key: Optional[str] = None) -> Tuple[Dict[str, str], str]:
        """
        Encrypt credentials dictionary.
        
        Returns:
            Tuple of (encrypted_credentials_dict, encryption_key)
        """
        if encryption_key is None:
            encryption_key = cls._generate_key()
        
        encrypted = {}
        for key, value in credentials.items():
            encrypted[f"encrypted_{key}"] = cls._encrypt_value(str(value), encryption_key)
        
        return encrypted, encryption_key
    
    @classmethod
    def _decrypt_credentials(cls, encrypted_creds: Dict[str, str], 
                            encryption_key: str) -> Dict[str, str]:
        """Decrypt credentials dictionary."""
        decrypted = {}
        for key, value in encrypted_creds.items():
            if key.startswith("encrypted_"):
                original_key = key.replace("encrypted_", "")
                decrypted[original_key] = cls._decrypt_value(value, encryption_key)
        
        return decrypted
    
    @classmethod
    def register(cls, client, catalog: str, namespace: str, name: str,
                connection_string: str, credentials: Dict[str, str],
                encryption_key: Optional[str] = None,
                store_key: bool = False,
                description: str = None,
                **extra_properties):
        """
        Register a database connection asset with encrypted credentials.
        
        Args:
            client: PangolinClient instance
            catalog: Catalog name
            namespace: Namespace name
            name: Asset name
            connection_string: Database connection string or endpoint
            credentials: Dictionary of credentials to encrypt (e.g., username, password)
            encryption_key: Optional encryption key. If None, one will be generated.
            store_key: Whether to store the encryption key in properties (default: True)
            description: Optional description
            **extra_properties: Additional properties to store
        
        Returns:
            Asset object
        """
        # Encrypt credentials
        encrypted_creds, key = cls._encrypt_credentials(credentials, encryption_key)
        
        # Build properties
        properties = {
            "connection_type": cls.__name__.replace("Asset", "").lower(),
            "connection_string": connection_string,
            **encrypted_creds,
            **extra_properties
        }
        
        # B_sdk4: `store_key` defaulted to True, so the encryption key was
        # written into the asset's properties *next to the ciphertext it
        # decrypts* - which reduces the encryption to obfuscation, since anyone
        # who can read the asset can read both halves. It now defaults to False;
        # callers who genuinely want the key co-located must ask for it, and get
        # told what that means.
        if store_key:
            import warnings

            warnings.warn(
                "store_key=True writes the encryption key into the same asset "
                "properties as the ciphertext it decrypts, so anyone who can "
                "read the asset can decrypt the credentials. Keep the key in a "
                "secrets manager and pass it via encryption_key= instead.",
                stacklevel=2,
            )
            properties["encryption_key"] = key
        
        # Add description if provided
        if description:
            properties["description"] = description
        
        # Register asset (POST - doesn't return properties)
        ns_client = client.catalogs.namespaces(catalog)
        ns_client.register_asset(
            namespace=namespace,
            name=name,
            kind=cls.ASSET_KIND,
            location=connection_string,
            properties=properties
        )
        
        # Fetch the asset to get properties (GET - includes properties)
        asset = cls._get_asset(client, catalog, namespace, name)
        
        return asset
    
    @classmethod
    def _get_asset(cls, client, catalog: str, namespace: str, name: str):
        """Retrieve asset from catalog using GET endpoint."""
        # Use the GET endpoint: /api/v1/catalogs/{catalog}/namespaces/{namespace}/assets/{name}
        endpoint = f"/api/v1/catalogs/{catalog}/namespaces/{namespace}/assets/{name}"
        return client.get(endpoint)
    
    @classmethod
    def _get_decrypted_credentials(cls, client, catalog: str, namespace: str, name: str,
                                   encryption_key: Optional[str] = None) -> Tuple[str, Dict[str, str]]:
        """
        Retrieve asset and decrypt credentials.
        
        Returns:
            Tuple of (connection_string, decrypted_credentials)
        """
        asset = cls._get_asset(client, catalog, namespace, name)
        properties = asset.get("properties", {})
        
        # Get encryption key
        if encryption_key is None:
            encryption_key = properties.get("encryption_key")
            if encryption_key is None:
                raise ValueError(
                    "No encryption key provided and none found in asset properties. "
                    "Please provide encryption_key parameter."
                )
        
        # Get connection string
        connection_string = properties.get("connection_string")
        if not connection_string:
            raise ValueError("No connection_string found in asset properties")
        
        # Decrypt credentials
        decrypted = cls._decrypt_credentials(properties, encryption_key)
        
        return connection_string, decrypted
    
    @classmethod
    def connect(cls, client, catalog: str, namespace: str, name: str,
               encryption_key: Optional[str] = None):
        """
        Retrieve asset and create connection object.
        
        Must be implemented by subclasses to return appropriate connection type.
        """
        raise NotImplementedError("Subclasses must implement connect()")
