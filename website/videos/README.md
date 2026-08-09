# Site video assets

Rendered MP4s and posters served by `../index.html#videos`.

**Do not edit these by hand.** They are build products of `../../video/` — one
HyperFrames project per video, all generated from a single template. To change
a video, edit `../../video/content.mjs` (copy), `brand.mjs` (look) or
`template.mjs` (scene shapes), then:

```bash
cd ../../video
node build.mjs
./render.sh <slug>
cp renders/<slug>.mp4 renders/<slug>.jpg ../website/videos/
```

Every file here is silent — no audio track — so the page can autoplay them
muted without an audio decision. See `../../video/README.md` for the accuracy
rules; the production-readiness video hard-codes maturity statuses that go
stale when the README's maturity table moves.
