from .main import cli

# B_sdk5: `[project.scripts]` needs a callable to point at. `cli` is the Click
# group; `main` is the conventional name for the console-script entry point.
main = cli

__all__ = ["cli", "main"]

if __name__ == "__main__":
    cli()
