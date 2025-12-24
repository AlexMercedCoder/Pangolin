"""
SQL database connection assets (PostgreSQL, MySQL, Redshift).
"""

from .base import BaseConnectionAsset
from typing import Optional


class PostgreSQLAsset(BaseConnectionAsset):
    """PostgreSQL database connection asset."""
    
    REQUIRED_DEPS = ["psycopg2"]
    
    @classmethod
    def connect(cls, client, catalog: str, namespace: str, name: str,
               encryption_key: Optional[str] = None):
        """
        Create and return a psycopg2 connection object.
        
        Returns:
            psycopg2.connection object
        """
        try:
            import psycopg2
        except ImportError:
            raise ImportError(
                "psycopg2 is required for PostgreSQL connections. "
                "Install with: pip install 'pypangolin[postgres]'"
            )
        
        connection_string, creds = cls._get_decrypted_credentials(
            client, catalog, namespace, name, encryption_key
        )
        
        # Build connection parameters
        conn_params = {}
        
        # Parse connection string for host, port, database
        # Format: postgresql://host:port/database
        if "://" in connection_string:
            parts = connection_string.split("://")[1]
            if "/" in parts:
                host_port, database = parts.split("/", 1)
                if ":" in host_port:
                    host, port = host_port.split(":")
                    conn_params["port"] = int(port)
                else:
                    host = host_port
                conn_params["host"] = host
                conn_params["database"] = database
        
        # Add credentials
        if "username" in creds:
            conn_params["user"] = creds["username"]
        if "password" in creds:
            conn_params["password"] = creds["password"]
        
        # Add any additional parameters from credentials
        for key, value in creds.items():
            if key not in ["username", "password"]:
                conn_params[key] = value
        
        return psycopg2.connect(**conn_params)


class MySQLAsset(BaseConnectionAsset):
    """MySQL database connection asset."""
    
    REQUIRED_DEPS = ["mysql-connector-python"]
    
    @classmethod
    def connect(cls, client, catalog: str, namespace: str, name: str,
               encryption_key: Optional[str] = None):
        """
        Create and return a mysql.connector connection object.
        
        Returns:
            mysql.connector.connection object
        """
        try:
            import mysql.connector
        except ImportError:
            raise ImportError(
                "mysql-connector-python is required for MySQL connections. "
                "Install with: pip install 'pypangolin[mysql]'"
            )
        
        connection_string, creds = cls._get_decrypted_credentials(
            client, catalog, namespace, name, encryption_key
        )
        
        # Build connection parameters
        conn_params = {}
        
        # Parse connection string for host, port, database
        # Format: mysql://host:port/database
        if "://" in connection_string:
            parts = connection_string.split("://")[1]
            if "/" in parts:
                host_port, database = parts.split("/", 1)
                if ":" in host_port:
                    host, port = host_port.split(":")
                    conn_params["port"] = int(port)
                else:
                    host = host_port
                conn_params["host"] = host
                conn_params["database"] = database
        
        # Add credentials
        if "username" in creds:
            conn_params["user"] = creds["username"]
        if "password" in creds:
            conn_params["password"] = creds["password"]
        
        # Add any additional parameters
        for key, value in creds.items():
            if key not in ["username", "password"]:
                conn_params[key] = value
        
        return mysql.connector.connect(**conn_params)


class RedshiftAsset(BaseConnectionAsset):
    """Amazon Redshift connection asset (Postgres-compatible)."""
    
    REQUIRED_DEPS = ["psycopg2"]
    
    @classmethod
    def connect(cls, client, catalog: str, namespace: str, name: str,
               encryption_key: Optional[str] = None):
        """
        Create and return a psycopg2 connection object for Redshift.
        
        Returns:
            psycopg2.connection object
        """
        try:
            import psycopg2
        except ImportError:
            raise ImportError(
                "psycopg2 is required for Redshift connections. "
                "Install with: pip install 'pypangolin[redshift]'"
            )
        
        connection_string, creds = cls._get_decrypted_credentials(
            client, catalog, namespace, name, encryption_key
        )
        
        # Build connection parameters (same as PostgreSQL)
        conn_params = {}
        
        if "://" in connection_string:
            parts = connection_string.split("://")[1]
            if "/" in parts:
                host_port, database = parts.split("/", 1)
                if ":" in host_port:
                    host, port = host_port.split(":")
                    conn_params["port"] = int(port)
                else:
                    host = host_port
                conn_params["host"] = host
                conn_params["database"] = database
        
        if "username" in creds:
            conn_params["user"] = creds["username"]
        if "password" in creds:
            conn_params["password"] = creds["password"]
        
        for key, value in creds.items():
            if key not in ["username", "password"]:
                conn_params[key] = value
        
        return psycopg2.connect(**conn_params)
