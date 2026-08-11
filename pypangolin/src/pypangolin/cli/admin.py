import click
from rich.console import Console
from rich.table import Table
from ..client import PangolinClient

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
        # B_cli/B_sdk: a failed command must exit non-zero. These blanket
        # handlers printed the error and returned 0, so a script or CI job
        # could not tell a working command from a broken one - which is
        # exactly why the TypeErrors above went unnoticed for so long.
        raise SystemExit(1)

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
        # B_cli/B_sdk: a failed command must exit non-zero. These blanket
        # handlers printed the error and returned 0, so a script or CI job
        # could not tell a working command from a broken one - which is
        # exactly why the TypeErrors above went unnoticed for so long.
        raise SystemExit(1)

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
        # B_cli/B_sdk: a failed command must exit non-zero. These blanket
        # handlers printed the error and returned 0, so a script or CI job
        # could not tell a working command from a broken one - which is
        # exactly why the TypeErrors above went unnoticed for so long.
        raise SystemExit(1)

# --- Users ---
@admin.command()
@click.option('--username', required=True)
@click.option('--password', required=True)
@click.option('--email', required=True)
# B_sdk1: the default was 'TenantUser'. `UserRole` is kebab-case, so the
# server only accepts 'root', 'tenant-admin' or 'tenant-user' - every
# `create-user` without an explicit --role 422'd.
@click.option(
    '--role',
    default='tenant-user',
    type=click.Choice(['root', 'tenant-admin', 'tenant-user'], case_sensitive=False),
)
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
        # B_cli/B_sdk: a failed command must exit non-zero. These blanket
        # handlers printed the error and returned 0, so a script or CI job
        # could not tell a working command from a broken one - which is
        # exactly why the TypeErrors above went unnoticed for so long.
        raise SystemExit(1)

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
        # B_cli/B_sdk: a failed command must exit non-zero. These blanket
        # handlers printed the error and returned 0, so a script or CI job
        # could not tell a working command from a broken one - which is
        # exactly why the TypeErrors above went unnoticed for so long.
        raise SystemExit(1)

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
             client.warehouses.create_s3(
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
        # B_cli/B_sdk: a failed command must exit non-zero. These blanket
        # handlers printed the error and returned 0, so a script or CI job
        # could not tell a working command from a broken one - which is
        # exactly why the TypeErrors above went unnoticed for so long.
        raise SystemExit(1)

@admin.command()
@click.pass_context
def list_warehouses(ctx):
    """List warehouses"""
    client = get_client(ctx)
    try:
        warehouses = client.warehouses.list()
        table = Table(title="Warehouses")
        table.add_column("Name", style="green")
        # B_sdk3: this rendered `w.id`, but `Warehouse` has no `id` field, so
        # the command raised AttributeError whenever it found a warehouse - and
        # only *appeared* to work on an empty list.
        table.add_column("Storage", style="cyan")

        for w in warehouses:
            table.add_row(w.name, w.storage_config.get("type", "-"))
        console.print(table)
    except Exception as e:
        console.print(f"[red]Error listing warehouses:[/red] {e}")
        # B_cli/B_sdk: a failed command must exit non-zero. These blanket
        # handlers printed the error and returned 0, so a script or CI job
        # could not tell a working command from a broken one - which is
        # exactly why the TypeErrors above went unnoticed for so long.
        raise SystemExit(1)

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
        client.catalogs.create(name=name, warehouse=warehouse, type=type)
        console.print(f"[green]Catalog created successfully:[/green] {name}")
    except Exception as e:
         console.print(f"[red]Error creating catalog:[/red] {e}")
         # B_cli/B_sdk: a failed command must exit non-zero. These blanket
         # handlers printed the error and returned 0, so a script or CI job
         # could not tell a working command from a broken one - which is
         # exactly why the TypeErrors above went unnoticed for so long.
         raise SystemExit(1)

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
        # B_cli/B_sdk: a failed command must exit non-zero. These blanket
        # handlers printed the error and returned 0, so a script or CI job
        # could not tell a working command from a broken one - which is
        # exactly why the TypeErrors above went unnoticed for so long.
        raise SystemExit(1)

# --- Governance ---
@admin.command()
@click.option('--user-id', required=True, help='UUID of the user to grant to')
@click.option('--action', 'actions', required=True, multiple=True,
              help='Action to grant; repeat for several')
@click.option('--scope-type', required=True,
              type=click.Choice(['tenant', 'catalog', 'namespace', 'asset', 'tag']))
@click.option('--catalog-id', help='Catalog UUID (catalog, namespace and asset scopes)')
@click.option('--namespace', help='Namespace (namespace and asset scopes)')
@click.option('--asset-id', help='Asset UUID (asset scope)')
@click.option('--tag-name', help='Tag name (tag scope)')
@click.pass_context
def grant_permission(ctx, user_id, actions, scope_type, catalog_id, namespace, asset_id, tag_name):
    """Grant a permission to a user.

    B_sdk3: this called `grant(username=..., action=..., resource=...)`, none of
    which are parameters of `PermissionClient.grant` - so it raised a TypeError
    on *every* invocation. The blanket `except Exception` printed the TypeError
    as a red line and the command still exited 0, which is why it survived.
    """
    client = get_client(ctx)
    try:
        client.permissions.grant(
            user_id=user_id,
            actions=list(actions),
            scope_type=scope_type,
            catalog_id=catalog_id,
            namespace=namespace,
            asset_id=asset_id,
            tag_name=tag_name,
        )
        console.print(
            f"[green]Granted {', '.join(actions)} on {scope_type} to {user_id}[/green]"
        )
    except Exception as e:
        console.print(f"[red]Error granting permission:[/red] {e}")
        # B_cli/B_sdk: a failed command must exit non-zero. These blanket
        # handlers printed the error and returned 0, so a script or CI job
        # could not tell a working command from a broken one - which is
        # exactly why the TypeErrors above went unnoticed for so long.
        raise SystemExit(1)

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
        # B_cli/B_sdk: a failed command must exit non-zero. These blanket
        # handlers printed the error and returned 0, so a script or CI job
        # could not tell a working command from a broken one - which is
        # exactly why the TypeErrors above went unnoticed for so long.
        raise SystemExit(1)

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
        # B_cli/B_sdk: a failed command must exit non-zero. These blanket
        # handlers printed the error and returned 0, so a script or CI job
        # could not tell a working command from a broken one - which is
        # exactly why the TypeErrors above went unnoticed for so long.
        raise SystemExit(1)
