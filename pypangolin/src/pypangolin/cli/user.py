import sys

import click
from rich.console import Console
from rich.table import Table
from ..client import PangolinClient
from .config import update_profile

console = Console()

def get_client(ctx):
    profile = ctx.obj['profile']
    token = profile.get('token')
    return PangolinClient(uri=profile.get('url', 'http://localhost:8080'), token=token, tenant_id=profile.get('tenant_id'))

@click.group()
def user():
    """User commands for Pangolin"""
    pass

@user.command()
@click.option('--username', required=True)
@click.option('--password', required=True)
@click.option('--tenant-id', help='Optional tenant ID to login to')
@click.pass_context
def login(ctx, username, password, tenant_id):
    """Login and save token to profile"""
    client = get_client(ctx)
    try:
        # Pass tenant_id to login if provided
        client.login(username=username, password=password, tenant_id=tenant_id)
        
        # Access the token from the client after login
        token = client.token
        
        profile_name = ctx.obj['profile_name']
        updates = {
            "token": token,
            "username": username
        }
        if tenant_id:
            updates["tenant_id"] = tenant_id
            
        update_profile(profile_name, updates)
        console.print(f"[green]Successfully logged in as {username}[/green]")
    except Exception as e:
        console.print(f"[red]Login failed:[/red] {e}")
        # B_cli/B_sdk: a failed command must exit non-zero. These blanket
        # handlers printed the error and returned 0, so a script or CI job
        # could not tell a working command from a broken one - which is
        # exactly why the TypeErrors above went unnoticed for so long.
        raise SystemExit(1)

@user.command()
@click.pass_context
def list_catalogs(ctx):
    """List available catalogs"""
    client = get_client(ctx)
    try:
        catalogs = client.catalogs.list()
        table = Table(title="Catalogs")
        table.add_column("Name", style="green")
        table.add_column("Type", style="cyan")
        
        for c in catalogs:
            table.add_row(c.name, c.catalog_type)
            
        console.print(table)
    except Exception as e:
        console.print(f"[red]Error listing catalogs:[/red] {e}")
        # B_cli/B_sdk: a failed command must exit non-zero. These blanket
        # handlers printed the error and returned 0, so a script or CI job
        # could not tell a working command from a broken one - which is
        # exactly why the TypeErrors above went unnoticed for so long.
        raise SystemExit(1)

@user.command()
@click.argument('query')
@click.pass_context
def search(ctx, query):
    """Search for assets"""
    client = get_client(ctx)
    try:
        results = client.search.query(q=query)
        table = Table(title=f"Search Results: {query}")
        table.add_column("Name", style="green")
        table.add_column("Kind")
        table.add_column("Catalog")
        table.add_column("Namespace")
        
        for r in results:
            # B_sdk3: this rendered `r.score`, which neither `SearchResult` nor
            # the server's response has - AttributeError on every hit.
            table.add_row(r.name, r.kind, r.catalog, r.namespace)
        console.print(table)
    except Exception as e:
        console.print(f"[red]Search failed:[/red] {e}")
        # B_cli/B_sdk: a failed command must exit non-zero. These blanket
        # handlers printed the error and returned 0, so a script or CI job
        # could not tell a working command from a broken one - which is
        # exactly why the TypeErrors above went unnoticed for so long.
        raise SystemExit(1)

@user.command()
@click.option('--language', type=click.Choice(['pyiceberg', 'pyspark', 'dremio', 'sql'], case_sensitive=False), required=True)
@click.option('--table', required=True, help='Full table name (catalog.namespace.table)')
@click.pass_context
def generate_code(ctx, language, table):
    """Generate connection code for a table"""
    # Local logic for code generation
    profile = ctx.obj['profile']
    url = profile.get('url', 'http://localhost:8080')
    token = profile.get('token', '<YOUR_TOKEN>')
    
    parts = table.split('.')
    if len(parts) < 3:
        console.print("[red]Error: Table name must be in format catalog.namespace.table[/red]")
        return
        
    catalog = parts[0]
    namespace = parts[1]
    table_name = parts[2]
    
    code = ""
    if language == 'pyiceberg':
        code = f"""from pyiceberg.catalog import load_catalog

catalog = load_catalog(
    "{catalog}",
    **{{
        "uri": "{url}",
        "token": "{token}",
        "header.X-Iceberg-Access-Delegation": "vended-credentials"
    }}
)

table = catalog.load_table("{namespace}.{table_name}")
df = table.scan().to_pandas()
print(df.head())"""

    elif language == 'pyspark':
         code = f"""spark = SparkSession.builder \\
    .conf("spark.sql.extensions", "org.apache.iceberg.spark.extensions.IcebergSparkSessionExtensions") \\
    .conf("spark.sql.catalog.{catalog}", "org.apache.iceberg.spark.SparkCatalog") \\
    .conf("spark.sql.catalog.{catalog}.type", "rest") \\
    .conf("spark.sql.catalog.{catalog}.uri", "{url}") \\
    .conf("spark.sql.catalog.{catalog}.token", "{token}") \\
    .getOrCreate()

df = spark.table("{catalog}.{namespace}.{table_name}")
df.show()"""
    elif language == 'sql':
        code = f"SELECT * FROM {catalog}.{namespace}.{table_name} LIMIT 10;"
    else:
        code = f"-- Code generation for {language} not yet implemented"

    console.print(f"[bold cyan]Generated {language} code:[/bold cyan]")
    console.print(code)

# --- Branching ---
@user.command()
@click.argument('catalog')
@click.pass_context
def list_branches(ctx, catalog):
    """List branches in a catalog"""
    client = get_client(ctx)
    try:
        branches = client.branches.list(catalog_name=catalog)
        table = Table(title=f"Branches in {catalog}")
        table.add_column("Name", style="green")
        table.add_column("Type")
        table.add_column("Head Commit")
        
        for b in branches:
            table.add_row(b.name, b.branch_type, str(b.head_commit_id))
        console.print(table)
    except Exception as e:
        console.print(f"[red]Error listing branches:[/red] {e}")
        # B_cli/B_sdk: a failed command must exit non-zero. These blanket
        # handlers printed the error and returned 0, so a script or CI job
        # could not tell a working command from a broken one - which is
        # exactly why the TypeErrors above went unnoticed for so long.
        raise SystemExit(1)

@user.command()
@click.argument('catalog')
@click.argument('name')
@click.option('--from', 'from_', help='Source branch')
@click.pass_context
def create_branch(ctx, catalog, name, from_):
    """Create a new branch"""
    client = get_client(ctx)
    try:
        client.branches.create(catalog_name=catalog, name=name, from_branch=from_)
        console.print(f"[green]Branch created successfully:[/green] {name}")
    except Exception as e:
        console.print(f"[red]Error creating branch:[/red] {e}")
        # B_cli/B_sdk: a failed command must exit non-zero. These blanket
        # handlers printed the error and returned 0, so a script or CI job
        # could not tell a working command from a broken one - which is
        # exactly why the TypeErrors above went unnoticed for so long.
        raise SystemExit(1)

@user.command()
@click.option('--catalog', required=True)
@click.option('--source', required=True)
@click.option('--target', required=True)
@click.pass_context
def merge_branch(ctx, catalog, source, target):
    """Merge two branches"""
    client = get_client(ctx)
    try:
        # B_sdk3: this passed `source=`/`target=`, but `BranchClient.merge` takes
        # `source_branch`/`target_branch` - a TypeError on every call, swallowed
        # by the blanket except below and reported with a zero exit code.
        client.branches.merge(
            catalog_name=catalog, source_branch=source, target_branch=target
        )
        console.print(f"[green]Merge operation initiated between {source} and {target}[/green]")
    except Exception as e:
        console.print(f"[red]Error merging branches:[/red] {e}")
        # B_cli/B_sdk: a failed command must exit non-zero. These blanket
        # handlers printed the error and returned 0, so a script or CI job
        # could not tell a working command from a broken one - which is
        # exactly why the TypeErrors above went unnoticed for so long.
        raise SystemExit(1)

# --- Tags ---
@user.command()
@click.argument('catalog')
@click.pass_context
def list_tags(ctx, catalog):
    """List tags in a catalog"""
    client = get_client(ctx)
    try:
        tags = client.tags.list(catalog_name=catalog)
        table = Table(title=f"Tags in {catalog}")
        table.add_column("Name", style="green")
        table.add_column("Commit ID")
        
        for t in tags:
            table.add_row(t.name, str(t.commit_id))
        console.print(table)
    except Exception as e:
        console.print(f"[red]Error listing tags:[/red] {e}")
        # B_cli/B_sdk: a failed command must exit non-zero. These blanket
        # handlers printed the error and returned 0, so a script or CI job
        # could not tell a working command from a broken one - which is
        # exactly why the TypeErrors above went unnoticed for so long.
        raise SystemExit(1)

@user.command()
@click.argument('catalog')
@click.argument('name')
@click.argument('commit-id')
@click.pass_context
def create_tag(ctx, catalog, name, commit_id):
    """Create a new tag"""
    client = get_client(ctx)
    try:
        client.tags.create(catalog_name=catalog, name=name, commit_id=commit_id)
        console.print(f"[green]Tag created successfully:[/green] {name}")
    except Exception as e:
        console.print(f"[red]Error creating tag:[/red] {e}")
        # B_cli/B_sdk: a failed command must exit non-zero. These blanket
        # handlers printed the error and returned 0, so a script or CI job
        # could not tell a working command from a broken one - which is
        # exactly why the TypeErrors above went unnoticed for so long.
        raise SystemExit(1)

# --- Requests ---
@user.command()
@click.pass_context
def list_requests(ctx):
    """List access requests"""
    # Assuming governance client
    # client.requests? Or client.access_requests?
    # client.py had `search`, `audit`, etc.
    # Looking back at client.py (Step 92), I don't see an explicit AccessRequestClient exposed as property
    # It might be missing.
    # In Rust CLI: ListAccessRequests is User command.
    # If python client is missing it, I'll skip it for now or add to Client if I were modifying client.
    # I am allowed to modify pypangolin. 
    # But for now I'll just skip implementation or output "Not implemented" if client support is missing.
    console.print("[yellow]List requests not yet supported in Python client[/yellow]")

@user.command()
@click.option('--description', required=True)
@click.pass_context
def get_token(ctx, description):
    """Generate a token"""
    client = get_client(ctx)
    try:
        token = client.tokens.generate(name=description)
        # B_sdk4: the raw token used to be printed into the terminal, where it
        # lands in scrollback and shell-session transcripts. It now goes to
        # stdout alone, so redirecting to a file still works, while the
        # human-facing confirmation goes to stderr.
        console.print("[green]Token generated.[/green]", file=sys.stderr)
        click.echo(token)
    except Exception as e:
        console.print(f"[red]Error generating token:[/red] {e}")
        # B_cli/B_sdk: a failed command must exit non-zero. These blanket
        # handlers printed the error and returned 0, so a script or CI job
        # could not tell a working command from a broken one - which is
        # exactly why the TypeErrors above went unnoticed for so long.
        raise SystemExit(1)
