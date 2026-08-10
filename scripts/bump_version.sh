#!/usr/bin/env bash
#
# Set one version across every artifact that carries one.
#
# Improvement #8. From 0.6.0 the server, both CLIs, the Python SDK, the UI and
# the Helm chart are meant to share a version number - but the process was
# manual, and it had already drifted one day after that release: the SDK's
# `__version__` said 0.1.0 against a 0.6.0 package (B38) and the UI lockfile
# said 0.1.0 against a 0.6.0 package.json (B46). A property that has to be
# maintained by hand in five places is not a property; it is a coincidence
# waiting to end.
#
# The SDK's `__version__` is deliberately not in the list: it is read from the
# installed distribution metadata at import time, so there is one fewer place to
# forget.
#
# Usage:
#   scripts/bump_version.sh 0.7.0          # apply
#   scripts/bump_version.sh 0.7.0 --check  # verify everything already agrees

set -euo pipefail

if [[ $# -lt 1 ]]; then
    echo "usage: $0 <version> [--check]" >&2
    exit 2
fi

VERSION="$1"
CHECK_ONLY="${2:-}"

if [[ ! "$VERSION" =~ ^[0-9]+\.[0-9]+\.[0-9]+(-[0-9A-Za-z.-]+)?$ ]]; then
    echo "error: '$VERSION' is not a semantic version" >&2
    exit 2
fi

cd "$(dirname "$0")/.."
ROOT="$PWD"

# file:line-matcher pairs. Each matcher is a sed expression that rewrites only
# the version-bearing line, so an unrelated `0.6.0` elsewhere in the file is
# left alone.
apply() {
    local file="$1" pattern="$2" replacement="$3"
    if [[ ! -f "$file" ]]; then
        echo "error: $file not found" >&2
        return 1
    fi
    sed -i -E "s|$pattern|$replacement|" "$file"
}

read_version() {
    local file="$1" pattern="$2"
    grep -oE "$pattern" "$file" | head -1 | grep -oE '[0-9]+\.[0-9]+\.[0-9]+(-[0-9A-Za-z.-]+)?'
}

declare -a FILES=(
    "pangolin/Cargo.toml"
    "pypangolin/pyproject.toml"
    "pangolin_ui/package.json"
    "deployment_assets/helm/pangolin/Chart.yaml"
)

if [[ "$CHECK_ONLY" == "--check" ]]; then
    status=0
    for f in "${FILES[@]}"; do
        found=$(read_version "$ROOT/$f" '^\s*"?(version|appVersion)"?\s*[:=]\s*"?[0-9]+\.[0-9]+\.[0-9]+' || true)
        if [[ "$found" != "$VERSION" ]]; then
            echo "::error::$f is at ${found:-<none>}, expected $VERSION"
            status=1
        fi
    done
    # The Helm chart carries two.
    app=$(grep -oE '^appVersion:\s*"?[0-9]+\.[0-9]+\.[0-9]+' \
        "$ROOT/deployment_assets/helm/pangolin/Chart.yaml" | grep -oE '[0-9]+\.[0-9]+\.[0-9]+' || true)
    if [[ "$app" != "$VERSION" ]]; then
        echo "::error::Chart.yaml appVersion is at ${app:-<none>}, expected $VERSION"
        status=1
    fi
    # Inter-crate path dependencies pin a version requirement; a stale one
    # fails the build rather than merely looking untidy.
    stale=$(grep -hoE 'path = "\.\./pangolin_[a-z_]+", version = "[0-9]+\.[0-9]+\.[0-9]+[^"]*"' \
        "$ROOT"/pangolin/*/Cargo.toml | grep -v "\"$VERSION\"" || true)
    if [[ -n "$stale" ]]; then
        echo "::error::workspace path dependencies are not at $VERSION:"
        echo "$stale"
        status=1
    fi

    # The UI lockfile drifted from package.json once already (B46).
    lock=$(grep -m1 -oE '"version":\s*"[0-9]+\.[0-9]+\.[0-9]+' \
        "$ROOT/pangolin_ui/package-lock.json" | grep -oE '[0-9]+\.[0-9]+\.[0-9]+' || true)
    if [[ "$lock" != "$VERSION" ]]; then
        echo "::error::pangolin_ui/package-lock.json is at ${lock:-<none>}, expected $VERSION"
        status=1
    fi
    if [[ $status -eq 0 ]]; then
        echo "every artifact is at $VERSION"
    fi
    exit $status
fi

# Rust workspace: `[workspace.package] version`, which every crate inherits.
apply "pangolin/Cargo.toml" \
    '^version = "[0-9]+\.[0-9]+\.[0-9]+.*"' \
    "version = \"$VERSION\""

# Path dependencies between workspace crates also pin a version requirement.
# Missing these is not cosmetic: cargo refuses to resolve
# `pangolin_core = "^0.6.0"` against a workspace now at 0.7.0, so the build
# breaks outright. Only `path = ...` lines are touched, leaving third-party
# requirements alone.
for manifest in pangolin/*/Cargo.toml; do
    sed -i -E "s|(\{ path = \"\.\./pangolin_[a-z_]+\", version = \")[0-9]+\.[0-9]+\.[0-9]+[^\"]*|\1$VERSION|g" \
        "$manifest"
done

# Python SDK.
apply "pypangolin/pyproject.toml" \
    '^version = "[0-9]+\.[0-9]+\.[0-9]+.*"' \
    "version = \"$VERSION\""

# UI. The lockfile carries the same version twice near the top and is what
# drifted last time, so it is rewritten rather than left to `npm install`.
apply "pangolin_ui/package.json" \
    '"version": "[0-9]+\.[0-9]+\.[0-9]+.*"' \
    "\"version\": \"$VERSION\""
if [[ -f "pangolin_ui/package-lock.json" ]]; then
    python3 - "$VERSION" <<'PY'
import json, sys, collections
version = sys.argv[1]
path = "pangolin_ui/package-lock.json"
with open(path) as f:
    lock = json.load(f, object_pairs_hook=collections.OrderedDict)
lock["version"] = version
# npm mirrors the root package version under packages[""].
if "packages" in lock and "" in lock["packages"]:
    lock["packages"][""]["version"] = version
with open(path, "w") as f:
    json.dump(lock, f, indent=2)
    f.write("\n")
PY
fi

# Helm chart: the chart's own version and the app it deploys.
apply "deployment_assets/helm/pangolin/Chart.yaml" \
    '^version: [0-9]+\.[0-9]+\.[0-9]+.*' \
    "version: $VERSION"
apply "deployment_assets/helm/pangolin/Chart.yaml" \
    '^appVersion: "?[0-9]+\.[0-9]+\.[0-9]+.*"?' \
    "appVersion: \"$VERSION\""

# Cargo.lock records the workspace crates' versions; refresh it without
# touching dependency resolution.
if command -v cargo >/dev/null 2>&1; then
    (cd pangolin && cargo update --workspace --offline >/dev/null 2>&1) || true
fi

echo "set every artifact to $VERSION"
echo
echo "Verify with: scripts/bump_version.sh $VERSION --check"
