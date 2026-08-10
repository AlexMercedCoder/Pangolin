#!/bin/bash
#
# Build and push the three published images: API, CLI and UI.
#
# The version is read from the workspace manifest rather than written here. It
# used to be hardcoded, and was left at 0.3.0 while the project shipped 0.4.0,
# 0.5.0 and 0.5.1 - so a release built from this script would have overwritten
# the 0.3.0 tags and published nothing under its own version. `latest` moved,
# which is why `alexmerced/pangolin-api:latest` and `:0.3.0` are the same image.
#
# Usage:
#   scripts/build_docker_sequential.sh              # build and push $VERSION and latest
#   scripts/build_docker_sequential.sh --dry-run    # print what would run
#   VERSION=0.7.1 scripts/build_docker_sequential.sh
#
# Requires `docker login` and a buildx builder capable of linux/amd64 and
# linux/arm64.

set -euo pipefail

cd "$(dirname "$0")/.."

DRY_RUN=""
if [[ "${1:-}" == "--dry-run" ]]; then
    DRY_RUN="echo [dry-run]"
fi

# Single source of truth: the workspace manifest, which
# `scripts/bump_version.sh` keeps in step with the SDK, UI and Helm chart.
VERSION="${VERSION:-$(grep -m1 '^version' pangolin/Cargo.toml | sed 's/.*"\(.*\)".*/\1/')}"

if [[ ! "$VERSION" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
    echo "error: could not read a semantic version from pangolin/Cargo.toml (got '$VERSION')" >&2
    exit 1
fi

echo "Publishing version $VERSION"
echo

# Refuse to silently overwrite a tag that is already published. Re-pushing an
# existing release tag replaces what users pulled yesterday with something else
# under the same name, which is the one thing a version number is supposed to
# prevent. Set ALLOW_OVERWRITE=1 to proceed anyway.
if [[ -z "$DRY_RUN" && "${ALLOW_OVERWRITE:-}" != "1" ]]; then
    for repo in pangolin-api pangolin-cli pangolin-ui; do
        code=$(curl -s -o /dev/null -w "%{http_code}" \
            "https://hub.docker.com/v2/repositories/alexmerced/$repo/tags/$VERSION" || echo 000)
        if [[ "$code" == "200" ]]; then
            echo "error: alexmerced/$repo:$VERSION is already published." >&2
            echo "       Bump the version, or set ALLOW_OVERWRITE=1 if you are certain." >&2
            exit 1
        fi
    done
    echo "None of the three tags exist yet; proceeding."
    echo
fi

build() {
    local name="$1" dockerfile="$2" context="$3" step="$4"
    echo "--- Building ${name} (${step}/3) ---"
    $DRY_RUN docker buildx build \
        --platform linux/amd64,linux/arm64 \
        -t "alexmerced/${name}:latest" \
        -t "alexmerced/${name}:${VERSION}" \
        --push \
        -f "$dockerfile" \
        "$context"
}

build pangolin-api  pangolin/Dockerfile        pangolin     1
build pangolin-cli  pangolin/Dockerfile.tools  pangolin     2
build pangolin-ui   pangolin_ui/Dockerfile     pangolin_ui  3

echo
echo "All three images published at ${VERSION} and latest."
