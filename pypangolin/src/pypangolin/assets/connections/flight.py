"""
Arrow Flight connection assets (Dremio).
"""

from .base import BaseConnectionAsset
from typing import Optional


class DremioAsset(BaseConnectionAsset):
    """Dremio connection asset using dremioframe."""
    
    REQUIRED_DEPS = ["dremioframe"]
    
    @classmethod
    def connect(cls, client, catalog: str, namespace: str, name: str,
               encryption_key: Optional[str] = None):
        """
        Create and return a dremioframe DremioClient object.
        
        Returns:
            dremioframe.DremioClient object
        """
        try:
            from dremioframe import DremioClient
        except ImportError:
            raise ImportError(
                "dremioframe is required for Dremio connections. "
                "Install with: pip install 'pypangolin[dremio]'"
            )
        
        connection_string, creds = cls._get_decrypted_credentials(
            client, catalog, namespace, name, encryption_key
        )
        
        # Parse connection string to extract hostname and port
        # Format: grpc+tls://hostname:port or grpc://hostname:port
        hostname = "data.dremio.cloud"
        port = 443
        tls = True
        
        if "://" in connection_string:
            protocol, rest = connection_string.split("://", 1)
            tls = "tls" in protocol.lower()
            
            if ":" in rest:
                hostname, port_str = rest.rsplit(":", 1)
                port = int(port_str)
            else:
                hostname = rest
        
        # Build connection parameters
        conn_params = {
            "hostname": hostname,
            "port": port,
            "tls": tls
        }
        
        # Add authentication
        if "token" in creds:
            conn_params["pat"] = creds["token"]  # Personal Access Token
        elif "username" in creds and "password" in creds:
            conn_params["username"] = creds["username"]
            conn_params["password"] = creds["password"]
        
        # Optional parameters
        if "disable_certificate_verification" in creds:
            value = creds["disable_certificate_verification"]
            if isinstance(value, str):
                conn_params["disable_certificate_verification"] = value.lower() == "true"
            else:
                conn_params["disable_certificate_verification"] = bool(value)
        
        if "project_id" in creds:
            conn_params["project_id"] = creds["project_id"]
        
        return DremioClient(**conn_params)
