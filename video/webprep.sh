#!/usr/bin/env bash
# Turn the full-quality renders into the copies the website serves.
#
#   ./webprep.sh            # all five
#   ./webprep.sh <slug>     # just one
#
# A 1080p high-quality render is ~12 MB. The gallery plays these in a card a few
# hundred pixels wide, so 720p at CRF 28 is visually identical there and roughly
# a fifth of the bytes — which matters because these are committed to the repo
# and served as static files.
#
# Deliberately strips audio (-an): the series is silent, and an empty audio
# track only invites a browser to treat the element as needing a sound decision.
# faststart moves the moov atom to the front so playback starts before the whole
# file arrives.

set -uo pipefail
cd "$(dirname "$0")"

SRC="renders"
DEST="../website/videos"
CRF="${CRF:-28}"
HEIGHT="${HEIGHT:-720}"
POSTER_AT="${POSTER_AT:-2.4}"

slugs=("$@")
if [ ${#slugs[@]} -eq 0 ]; then
  slugs=(what-is-pangolin architecture-and-features whats-new-in-0-6-0 production-readiness getting-started)
fi

mkdir -p "$DEST"
missing=()

for slug in "${slugs[@]}"; do
  src="$SRC/$slug.mp4"
  if [ ! -s "$src" ]; then
    echo "!!! $slug has no render at $src - skipping"
    missing+=("$slug")
    continue
  fi

  ffmpeg -nostdin -loglevel error -y -i "$src" \
    -vf "scale=-2:$HEIGHT" \
    -c:v libx264 -profile:v high -pix_fmt yuv420p \
    -crf "$CRF" -preset slow -movflags +faststart -an \
    "$DEST/$slug.mp4"

  ffmpeg -nostdin -loglevel error -y -ss "$POSTER_AT" -i "$src" \
    -vf "scale=-2:$HEIGHT" -frames:v 1 -q:v 4 "$DEST/$slug.jpg"

  printf "%-28s %7s -> %7s  + poster\n" "$slug" \
    "$(du -h "$src" | cut -f1)" "$(du -h "$DEST/$slug.mp4" | cut -f1)"
done

echo
if [ ${#missing[@]} -eq 0 ]; then
  echo "ALL WEB ASSETS WRITTEN"
else
  echo "NO RENDER FOR: ${missing[*]}"
fi
