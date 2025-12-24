"""
Cloud data warehouse connection assets (Snowflake, Synapse, BigQuery).
"""

from .base import BaseConnectionAsset
from typing import Optional


class SnowflakeAsset(BaseConnectionAsset):
    """Snowflake data warehouse connection asset."""
    
    REQUIRED_DEPS = ["snowflake-connector-python"]
    
    @classmethod
    def connect(cls, client, catalog: str, namespace: str, name: str,
               encryption_key: Optional[str] = None):
        """
        Create and return a Snowflake connection object.
        
        Returns:
            snowflake.connector.connection object
        """
        try:
            import snowflake.connector
        except ImportError:
            raise ImportError(
                "snowflake-connector-python is required for Snowflake connections. "
                "Install with: pip install 'pypangolin[snowflake]'"
            )
        
        connection_string, creds = cls._get_decrypted_credentials(
            client, catalog, namespace, name, encryption_key
        )
        
        # Snowflake connection parameters
        conn_params = {}
        
        # Extract account from connection string or credentials
        if "account" in creds:
            conn_params["account"] = creds["account"]
        
        if "username" in creds:
            conn_params["user"] = creds["username"]
        if "password" in creds:
            conn_params["password"] = creds["password"]
        
        # Optional parameters
        for key in ["warehouse", "database", "schema", "role"]:
            if key in creds:
                conn_params[key] = creds[key]
        
        return snowflake.connector.connect(**conn_params)


class SynapseAsset(BaseConnectionAsset):
    """Azure Synapse connection asset."""
    
    REQUIRED_DEPS = ["pyodbc"]
    
    @classmethod
    def connect(cls, client, catalog: str, namespace: str, name: str,
               encryption_key: Optional[str] = None):
        """
        Create and return a pyodbc connection object for Synapse.
        
        Returns:
            pyodbc.Connection object
        """
        try:
            import pyodbc
        except ImportError:
            raise ImportError(
                "pyodbc is required for Synapse connections. "
                "Install with: pip install 'pypangolin[synapse]'"
            )
        
        connection_string, creds = cls._get_decrypted_credentials(
            client, catalog, namespace, name, encryption_key
        )
        
        # Build ODBC connection string
        # Format: Driver={ODBC Driver 17 for SQL Server};Server=tcp:server.database.windows.net,1433;Database=dbname;Uid=user;Pwd=password;
        
        conn_str_parts = []
        
        if "driver" in creds:
            conn_str_parts.append(f"Driver={{{creds['driver']}}}")
        else:
            conn_str_parts.append("Driver={ODBC Driver 17 for SQL Server}")
        
        if "server" in creds:
            conn_str_parts.append(f"Server={creds['server']}")
        if "database" in creds:
            conn_str_parts.append(f"Database={creds['database']}")
        if "username" in creds:
            conn_str_parts.append(f"Uid={creds['username']}")
        if "password" in creds:
            conn_str_parts.append(f"Pwd={creds['password']}")
        
        conn_str = ";".join(conn_str_parts)
        
        return pyodbc.connect(conn_str)


class BigQueryAsset(BaseConnectionAsset):
    """Google BigQuery connection asset."""
    
    REQUIRED_DEPS = ["google-cloud-bigquery"]
    
    @classmethod
    def connect(cls, client, catalog: str, namespace: str, name: str,
               encryption_key: Optional[str] = None):
        """
        Create and return a BigQuery Client object.
        
        Returns:
            google.cloud.bigquery.Client object
        """
        try:
            from google.cloud import bigquery
        except ImportError:
            raise ImportError(
                "google-cloud-bigquery is required for BigQuery connections. "
                "Install with: pip install 'pypangolin[bigquery]'"
            )
        
        connection_string, creds = cls._get_decrypted_credentials(
            client, catalog, namespace, name, encryption_key
        )
        
        # BigQuery uses service account JSON or application default credentials
        # For now, support project_id parameter
        project = creds.get("project_id")
        
        if "credentials_json" in creds:
            # If credentials JSON is provided, use it
            import json
            from google.oauth2 import service_account
            
            creds_dict = json.loads(creds["credentials_json"])
            credentials = service_account.Credentials.from_service_account_info(creds_dict)
            return bigquery.Client(project=project, credentials=credentials)
        else:
            # Use application default credentials
            return bigquery.Client(project=project)
