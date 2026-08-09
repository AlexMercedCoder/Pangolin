#!/usr/bin/env bash
# Render every video, one at a time, and extract a poster for each.
#
#   ./render.sh                 # all five
#   ./render.sh production-readiness   # just one
#
# Renders are serialised deliberately: each one already parallelises across
# workers, and a headless Chrome render can wedge. Every attempt runs under a
# hard timeout and gets exactly one retry before the slug is recorded as failed
# and the script moves on, so one bad render never blocks the other four.

set -uo pipefail
cd "$(dirname "$0")"

TIMEOUT="${RENDER_TIMEOUT:-900}"
POSTER_AT="${POSTER_AT:-2.4}"
OUT="renders"
mkdir -p "$OUT"

slugs=("$@")
if [ ${#slugs[@]} -eq 0 ]; then
  slugs=(what-is-pangolin architecture-and-features whats-new-in-0-6-0 production-readiness getting-started)
fi

failed=()

for slug in "${slugs[@]}"; do
  mp4="$OUT/$slug.mp4"
  ok=0

  for attempt in 1 2; do
    echo ">>> render $slug (attempt $attempt/2)"
    rm -f "$mp4"
    if timeout --kill-after=30s "$TIMEOUT" \
        npx hyperframes render "out/$slug" --quality high --output "$PWD/$mp4" --quiet; then
      if [ -s "$mp4" ]; then ok=1; break; fi
      echo "!!! $slug produced no output"
    else
      code=$?
      [ $code -eq 124 ] && echo "!!! $slug timed out after ${TIMEOUT}s" || echo "!!! $slug exited $code"
    fi
  done

  if [ $ok -eq 1 ]; then
    ffmpeg -nostdin -loglevel error -y -ss "$POSTER_AT" -i "$mp4" \
      -frames:v 1 -q:v 3 "$OUT/$slug.jpg"
    dur=$(ffprobe -v error -show_entries format=duration -of csv=p=0 "$mp4")
    size=$(du -h "$mp4" | cut -f1)
    echo "<<< $slug OK  ${dur}s  $size  + poster"
  else
    failed+=("$slug")
    echo "<<< $slug FAILED after 2 attempts - skipping"
  fi
done

echo
if [ ${#failed[@]} -eq 0 ]; then
  echo "ALL RENDERS OK"
else
  echo "FAILED: ${failed[*]}"
fi
