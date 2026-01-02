import click
import logging
from .config import load_config, get_active_profile
from .admin import admin
from .user import user
from .config import set_active_profile, load_config as load_full_config

@click.group()
@click.option('--debug/--no-debug', default=False, help='Enable debug logging')
@click.option('--profile', help='Profile to use (overrides active profile)')
@click.pass_context
def cli(ctx, debug, profile):
    """Pangolin Data Catalog CLI"""
    # Configure logging
    log_level = logging.DEBUG if debug else logging.INFO
    logging.basicConfig(
        level=log_level,
        format='%(message)s', # Clean output for CLI
        handlers=[logging.StreamHandler()]
    )
    
    # Load configuration
    ctx.ensure_object(dict)
    config = load_full_config()
    
    # Determine profile to use
    profile_name = profile or config.get("active_profile", "default")
    
    ctx.obj['config'] = config
    ctx.obj['profile_name'] = profile_name
    ctx.obj['profile'] = get_active_profile(config) if not profile else config.get("profiles", {}).get(profile, {})

    if debug:
        click.echo(f"Using profile: {profile_name}")

@cli.command()
@click.argument('profile_name')
def use(profile_name):
    """Switch the active profile"""
    try:
        set_active_profile(profile_name)
        click.echo(f"Switched to profile: {profile_name}")
    except Exception as e:
        click.echo(f"Error: {e}", err=True)

@cli.command()
def profiles():
    """List all available profiles"""
    config = load_full_config()
    active = config.get("active_profile", "default")
    for name, _ in config.get("profiles", {}).items():
        prefix = "* " if name == active else "  "
        click.echo(f"{prefix}{name}")

# Register command groups
cli.add_command(admin)
cli.add_command(user)

if __name__ == "__main__":
    cli()
