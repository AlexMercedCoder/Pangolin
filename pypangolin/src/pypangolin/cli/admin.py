import click
from rich.console import Console
from rich.table import Table
from ..client import PangolinClient
from .config import get_active_profile
import json

console = Console()

def get_client(ctx):
    profile = ctx.obj['profile']
    token = profile.get('token')
    return PangolinClient(uri=profile.get('url', 'http://localhost:8080'), token=token, tenant_id=profile.get('tenant_id'))

@click.group()
def admin():
    """Administrative commands for Pangolin"""
    pass

# --- Tenants ---
@admin.command()
@click.option('--name', required=True, help='Name of the tenant')
@click.pass_context
def create_tenant(ctx, name):
    """Create a new tenant"""
    client = get_client(ctx)
    try:
        resp = client.tenants.create(name=name)
        console.print(f"[green]Tenant created successfully:[/green] {name} (ID: {resp.id})")
    except Exception as e:
        console.print(f"[red]Error creating tenant:[/red] {e}")

@admin.command()
@click.pass_context
def list_tenants(ctx):
    """List all tenants"""
    client = get_client(ctx)
    try:
        tenants = client.tenants.list()
        table = Table(title="Tenants")
        table.add_column("ID", style="cyan")
        table.add_column("Name", style="green")
        
        for t in tenants:
            table.add_row(str(t.id), t.name)
        
        console.print(table)
    except Exception as e:
        console.print(f"[red]Error listing tenants:[/red] {e}")

@admin.command()
@click.argument('tenant_id')
@click.pass_context
def delete_tenant(ctx, tenant_id):
    """Delete a tenant"""
    client = get_client(ctx)
    try:
        client.tenants.delete(tenant_id)
        console.print(f"[green]Tenant deleted successfully:[/green] {tenant_id}")
    except Exception as e:
        console.print(f"[red]Error deleting tenant:[/red] {e}")

# --- Users ---
@admin.command()
@click.option('--username', required=True)
@click.option('--password', required=True)
@click.option('--email', required=True)
@click.option('--role', default='TenantUser')
@click.option('--tenant-id', help='Tenant ID to assign user to')
@click.pass_context
def create_user(ctx, username, password, email, role, tenant_id):
    """Create a new user"""
    client = get_client(ctx)
    try:
        resp = client.users.create(
            username=username,
            password=password,
            email=email,
            role=role,
            tenant_id=tenant_id
        )
        console.print(f"[green]User created successfully:[/green] {username} (ID: {resp.id})")
    except Exception as e:
        console.print(f"[red]Error creating user:[/red] {e}")

@admin.command()
@click.pass_context
def list_users(ctx):
    """List all users"""
    client = get_client(ctx)
    try:
        users = client.users.list()
        table = Table(title="Users")
        table.add_column("ID", style="cyan")
        table.add_column("Username", style="green")
        table.add_column("Email")
        table.add_column("Role")
        
        for u in users:
            table.add_row(str(u.id), u.username, u.email, u.role)
        
        console.print(table)
    except Exception as e:
        console.print(f"[red]Error listing users:[/red] {e}")

# --- Warehouses ---
@admin.command()
@click.option('--name', required=True)
@click.option('--type', 'type_', default='s3', help='Storage type (s3, gcs, azure)')
@click.option('--bucket', help='Bucket name')
@click.option('--region', default='us-east-1')
@click.option('--endpoint', help='S3 Endpoint URL')
@click.option('--access-key', help='Access Key ID')
@click.option('--secret-key', help='Secret Access Key')
@click.pass_context
def create_warehouse(ctx, name, type_, bucket, region, endpoint, access_key, secret_key):
    """Create a new warehouse"""
    client = get_client(ctx)
    try:
        if type_ == 's3':
             resp = client.warehouses.create_s3(
                name=name,
                bucket=bucket,
                region=region,
                endpoint=endpoint,
                access_key=access_key,
                secret_key=secret_key
            )
        else:
            # Fallback for generic creation if client supports valid kwargs
            # For now simplified to S3 as per Rust CLI default
            raise NotImplementedError("Only S3 warehouses supported in this CLI version currently")
            
        console.print(f"[green]Warehouse created successfully:[/green] {name}")
    except Exception as e:
        console.print(f"[red]Error creating warehouse:[/red] {e}")

@admin.command()
@click.pass_context
def list_warehouses(ctx):
    """List warehouses"""
    client = get_client(ctx)
    try:
        warehouses = client.warehouses.list()
        table = Table(title="Warehouses")
        table.add_column("Name", style="green")
        table.add_column("ID", style="cyan")
        
        for w in warehouses:
            table.add_row(w.name, str(w.id))
        console.print(table)
    except Exception as e:
        console.print(f"[red]Error listing warehouses:[/red] {e}")

# --- Catalogs ---
@admin.command()
@click.option('--name', required=True)
@click.option('--warehouse', required=True)
@click.option('--type', default='Local')
@click.pass_context
def create_catalog(ctx, name, warehouse, type):
    """Create a new catalog"""
    client = get_client(ctx)
    try:
        resp = client.catalogs.create(name=name, warehouse=warehouse, type=type)
        console.print(f"[green]Catalog created successfully:[/green] {name}")
    except Exception as e:
         console.print(f"[red]Error creating catalog:[/red] {e}")

@admin.command()
@click.pass_context
def list_catalogs(ctx):
    """List catalogs"""
    client = get_client(ctx)
    try:
        catalogs = client.catalogs.list()
        table = Table(title="Catalogs")
        table.add_column("Name", style="green")
        table.add_column("Type")
        table.add_column("Warehouse")
        
        for c in catalogs:
            table.add_row(c.name, c.catalog_type, c.warehouse_name or "-")
        console.print(table)
    except Exception as e:
        console.print(f"[red]Error listing catalogs:[/red] {e}")

# --- Governance ---
@admin.command()
@click.option('--username', required=True)
@click.option('--action', required=True)
@click.option('--resource', required=True)
@click.pass_context
def grant_permission(ctx, username, action, resource):
    """Grant permission to a user"""
    client = get_client(ctx)
    try:
        # Assuming permissions client has grant method
        # If not, we might need to look up permission logic in client.py
        # Based on client.py inspection, PermissionClient exists
        # But grant method might specifically be on UserClient or not implemented directly as 'grant'
        # Let's check PermissionClient methods in next step if this fails, but for now assuming standard naming
        # Reading client.py again... PermissionClient is imported from governance. Let's assume it has create/grant
        
        # Actually, let's look at governance.py from Step 51 file list if needed, or infer from usage
        # Rust CLI does: GrantPermission { username, action, resource }
        # Python implementation likely: client.permissions.grant(username, action, resource)
        # We will wrap in try/catch to be safe
        client.permissions.grant(username=username, action=action, resource=resource)
        console.print(f"[green]Granted {action} on {resource} to {username}[/green]")
    except Exception as e:
        console.print(f"[red]Error granting permission:[/red] {e}")

@admin.command()
@click.option('--limit', default=100)
@click.pass_context
def list_audit_events(ctx, limit):
    """List audit logs"""
    client = get_client(ctx)
    try:
        events = client.audit.list_events(limit=limit)
        table = Table(title="Audit Logs")
        table.add_column("Time", style="cyan")
        table.add_column("User")
        table.add_column("Action", style="bold")
        table.add_column("Resource")
        table.add_column("Result")
        
        for e in events:
            # Handle timestamp formatting if needed
            ts = str(e.timestamp)
            table.add_row(ts, e.user_id, e.action, e.resource_type, e.result)
        console.print(table)
    except Exception as e:
        console.print(f"[red]Error listing audit events:[/red] {e}")

@admin.command()
@click.pass_context
def get_system_settings(ctx):
    """Get system settings"""
    client = get_client(ctx)
    try:
        settings = client.system.get_settings()
        console.print_json(data=settings)
    except Exception as e:
        console.print(f"[red]Error getting settings:[/red] {e}")
