import os
import yaml
from pathlib import Path
from typing import Dict, Optional, Any
import click

CONFIG_DIR = Path.home() / ".pangolin"
CONFIG_FILE = CONFIG_DIR / "profiles.yaml"

DEFAULT_CONFIG = {
    "profiles": {
        "default": {
            "url": "http://localhost:8080",
            "username": "admin",
            "token": None,
            "tenant_id": None
        }
    },
    "active_profile": "default"
}

def load_config() -> Dict[str, Any]:
    if not CONFIG_FILE.exists():
        return DEFAULT_CONFIG
    
    try:
        with open(CONFIG_FILE, "r") as f:
            return yaml.safe_load(f) or DEFAULT_CONFIG
    except Exception as e:
        click.echo(f"Warning: Failed to load config file: {e}", err=True)
        return DEFAULT_CONFIG

def save_config(config: Dict[str, Any]) -> None:
    """Write the config, which holds bearer tokens, with owner-only permissions.

    B_sdk4: this wrote ``~/.pangolin/profiles.yaml`` at the process umask -
    typically 0644 - so every local account could read the stored JWT. The
    directory is created 0700 and the file chmod'd to 0600, and the mode is set
    *before* the secret is written so there is no window where the file exists
    world-readable.
    """
    CONFIG_DIR.mkdir(parents=True, exist_ok=True)
    try:
        os.chmod(CONFIG_DIR, 0o700)
    except OSError:  # pragma: no cover - e.g. Windows
        pass

    # Create (or truncate) with the right mode before anything is written.
    fd = os.open(CONFIG_FILE, os.O_WRONLY | os.O_CREAT | os.O_TRUNC, 0o600)
    with os.fdopen(fd, "w") as f:
        yaml.safe_dump(config, f)

    try:
        os.chmod(CONFIG_FILE, 0o600)
    except OSError:  # pragma: no cover - e.g. Windows
        pass

def get_active_profile(config: Optional[Dict[str, Any]] = None) -> Dict[str, Any]:
    if config is None:
        config = load_config()
    
    active_name = config.get("active_profile", "default")
    profiles = config.get("profiles", {})
    
    if active_name not in profiles:
        # Fallback to default if active profile is missing
        return DEFAULT_CONFIG["profiles"]["default"]
        
    return profiles[active_name]

def update_profile(profile_name: str, updates: Dict[str, Any]) -> None:
    config = load_config()
    if "profiles" not in config:
        config["profiles"] = {}
        
    if profile_name not in config["profiles"]:
        config["profiles"][profile_name] = {}
        
    config["profiles"][profile_name].update(updates)
    save_config(config)

def set_active_profile(profile_name: str) -> None:
    config = load_config()
    if profile_name not in config.get("profiles", {}):
        raise click.ClickException(f"Profile '{profile_name}' does not exist.")
    
    config["active_profile"] = profile_name
    save_config(config)
