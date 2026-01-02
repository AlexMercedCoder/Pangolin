# Pangolin CLI Documentation

The `pypangolin` library includes a powerful command-line interface (CLI) that replicates the functionality of the official Rust CLIs (`pangolin-admin` and `pangolin-user`). It allows you to manage tenants, warehouses, catalogs, users, and execute common data engineering workflows directly from your terminal.

## Installation

The CLI is installed automatically with `pypangolin`. Ensure you have the optional dependencies if needed (though core requirements are usually sufficient).

```bash
pip install pypangolin
```

## Usage Entry Point

The CLI is accessed via the `pypangolin.cli.main` module:

```bash
python -m pypangolin.cli.main [OPTIONS] COMMAND [ARGS]...
```

For convenience, you might want to create an alias:

```bash
alias pypangolin='python -m pypangolin.cli.main'
```

## Global Options

- `--debug`: Enable debug logging for verbose output.
- `--profile TEXT`: Specify a profile to use for the command, overriding the active profile.
- `--help`: Show help text and exit.

## Command Groups

The CLI is organized into several command groups:

- [Admin Commands](admin.md): Administrative tasks like managing tenants, users, warehouses, and catalogs.
- [User Commands](user.md): User workflows like exploring catalogs, generating code, and managing branches/tags.
- [Profile Management](profiles.md): Managing connection profiles (URL, credentials, tenants).

## Quick Start

1.  **Check Profiles**:
    ```bash
    pypangolin profiles
    ```

2.  **Login**:
    ```bash
    pypangolin user login --username admin --password password
    ```

3.  **List Tenants** (Admin):
    ```bash
    pypangolin admin list-tenants
    ```
