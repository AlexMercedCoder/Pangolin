#!/usr/bin/env bash
#
# Verify the environment-variable reference against the source of truth.
#
# B43: `docs/environment-variables.md` documented `PANGOLIN_HOST`,
# `PANGOLIN_PORT` and `PANGOLIN_STORE_TYPE` - none of which any code reads - and
# omitted roughly twenty variables that are read. Anyone configuring a
# deployment from that page set variables that did nothing and missed the ones
# that mattered, which is exactly how B9's `PANGOLIN_STORE_TYPE` in the compose
# files survived.
#
# Hand-maintained documentation of a machine-readable fact drifts. This script
# re-derives the set from the code and fails when the docs and the code
# disagree, so the drift is caught in CI rather than by an auditor.
#
# Usage: scripts/check_env_var_docs.sh   (run from the pangolin/ workspace root)

set -euo pipefail

DOC="../docs/environment-variables.md"

if [[ ! -f "$DOC" ]]; then
    echo "error: $DOC not found; run this from the pangolin/ workspace root" >&2
    exit 1
fi

# Every PANGOLIN_* name the server actually reads. Test-only knobs
# (PANGOLIN_TEST_*) are deliberately excluded: they configure the test harness,
# not a deployment.
mapfile -t IN_CODE < <(
    grep -rhoE 'PANGOLIN_[A-Z0-9_]+' pangolin_api/src pangolin_store/src --include='*.rs' \
        | grep -v '^PANGOLIN_TEST_' \
        | sort -u
)

# The doc deliberately names a few variables that do *not* exist, to warn
# readers off them. Those lines carry a `<!-- not-a-variable -->` marker so this
# check can tell "documented as real" from "documented as a trap".
mapfile -t IN_DOCS < <(
    grep -vF '<!-- not-a-variable -->' "$DOC" \
        | grep -ohE 'PANGOLIN_[A-Z0-9_]+' \
        | grep -v '^PANGOLIN_TEST_' \
        | sort -u
)

missing=()
for name in "${IN_CODE[@]}"; do
    if ! printf '%s\n' "${IN_DOCS[@]}" | grep -qx "$name"; then
        missing+=("$name")
    fi
done

phantom=()
for name in "${IN_DOCS[@]}"; do
    if ! printf '%s\n' "${IN_CODE[@]}" | grep -qx "$name"; then
        phantom+=("$name")
    fi
done

status=0

if (( ${#missing[@]} )); then
    echo "error: read by the server but absent from $DOC:" >&2
    printf '  %s\n' "${missing[@]}" >&2
    status=1
fi

if (( ${#phantom[@]} )); then
    echo "error: documented in $DOC but read by nothing:" >&2
    printf '  %s\n' "${phantom[@]}" >&2
    status=1
fi

if (( status == 0 )); then
    echo "environment-variable reference matches the code (${#IN_CODE[@]} variables)"
fi

exit "$status"
