#!/usr/bin/env bash
#
# Every relative Markdown link must resolve.
#
# The documentation carried 274 broken links: files moved between `docs/`
# subdirectories and the links that pointed at them were never updated, plus a
# set of references into a `planning/` directory that is not in this repository
# at all. A link that 404s is worse than no link - it sends a reader looking for
# something that does not exist, and it makes the surrounding text look
# maintained when it is not.
#
# Excluded, deliberately:
#   * pangolin_docs.md / pypangolin_docs.md - generated concatenations whose
#     inner links are relative to the original files' locations and cannot
#     resolve in the flattened document. They carry a header saying so.
#   * pangolin/target/ - build artefacts, including vendored copies of README
#     files from packaged crates.
#   * node_modules/ - not ours.
#
# Usage: scripts/check_doc_links.sh   (run from the repository root)

set -uo pipefail

broken=0

while IFS= read -r file; do
    dir=$(dirname "$file")
    # Markdown links to a .md target, relative only. Anchors and URLs are out of
    # scope: an anchor needs heading parsing and a URL needs the network, and
    # both are a different check from "does this file exist".
    grep -oE '\]\((\.\./|\./)?[a-zA-Z0-9_./-]+\.md\)' "$file" 2>/dev/null \
        | sed 's/](//; s/)$//' \
        | while IFS= read -r link; do
            if [[ ! -f "$dir/$link" ]]; then
                echo "::error file=$file::broken link -> $link"
                echo "  $file -> $link"
            fi
        done
done < <(
    find . -name '*.md' \
        -not -path './node_modules/*' \
        -not -path '*/node_modules/*' \
        -not -path './pangolin/target/*' \
        -not -name 'pangolin_docs.md' \
        -not -name 'pypangolin_docs.md'
) > /tmp/pangolin_doc_links.$$ 2>&1

if [[ -s /tmp/pangolin_doc_links.$$ ]]; then
    cat /tmp/pangolin_doc_links.$$
    broken=$(grep -c -- '->' /tmp/pangolin_doc_links.$$ || true)
    rm -f /tmp/pangolin_doc_links.$$
    echo
    echo "error: $((broken / 2)) broken documentation link(s)." >&2
    echo "       Either point them at the file that exists, or unlink the text" >&2
    echo "       if the target is not in this repository." >&2
    exit 1
fi

rm -f /tmp/pangolin_doc_links.$$
echo "every relative documentation link resolves"
