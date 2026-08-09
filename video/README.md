# Pangolin video workspace

The five short explainers embedded on the website's [Videos section](../website/index.html#videos),
built with [HyperFrames](https://hyperframes.heygen.com) — video rendered from HTML.

All five are generated from **one template**. Nothing under `out/` is hand-edited;
change the source files and rebuild.

| File          | What it holds                                                                          |
| ------------- | -------------------------------------------------------------------------------------- |
| `brand.mjs`   | Palette, type pairing, and the whole stylesheet                                        |
| `template.mjs`| The six scene archetypes, their motion, and the composition shell                      |
| `content.mjs` | The five scripts — the only file with copy in it                                       |
| `build.mjs`   | Generates `out/<slug>/` — one self-contained HyperFrames project per video             |
| `render.sh`   | Renders each project in turn and extracts a poster                                     |
| `manifest.json` | Generated: slug, title, scene count and exact duration per video                     |

## Rebuilding

```bash
node build.mjs                        # regenerate all five projects
npx hyperframes check out/<slug>      # lint + runtime + layout + motion + contrast
./render.sh                           # render all five, one at a time, + posters
./render.sh production-readiness      # or just one
```

Rendered MP4s and posters land in `renders/`. The copies the site actually serves
live in `../website/videos/` — `render.sh` does not write there, so copy them
across deliberately after reviewing a render:

```bash
cp renders/*.mp4 renders/*.jpg ../website/videos/
```

## The videos

| Slug                        | Title                     | Length |
| --------------------------- | ------------------------- | ------ |
| `what-is-pangolin`          | What is Pangolin?         | ~36s   |
| `architecture-and-features` | Architecture & Features   | ~39s   |
| `whats-new-in-0-6-0`        | What's New in 0.6.0       | ~38s   |
| `production-readiness`      | Production Readiness      | ~40s   |
| `getting-started`           | Getting Started           | ~40s   |

All are silent — no audio track — so they can autoplay muted, loop on hover, or
play on click without an audio decision.

## Accuracy

Every claim in `content.mjs` is traceable to the repository at 0.6.0:
`README.md`'s maturity table, `CHANGELOG.md`'s 0.6.0 entry and its "Still
outstanding" list, `SECURITY.md`'s advisory and known-gaps section, and
`docs/operations/backend-parity.md`.

**If the project's maturity changes, these videos become wrong.** The
production-readiness video in particular hard-codes status pills — `Missing`,
`Untested`, `Beta`. Re-check `content.mjs` against the README's maturity table
whenever that table moves, and re-render.

## Design notes

The website is a light canvas; the videos are dark. That is deliberate — the
brand orange `#df8d53` reads as a highlight on dark and washes out on light at
30fps under H.264. The brand hues are unchanged and every neutral is tinted
toward the orange, so nothing goes dead gray.

Type is Archivo Black against JetBrains Mono: the display face carries the
claims, the machine face carries the evidence. That split is the point of this
release — a project walking back its own marketing — so the two registers stay
visually separate throughout. Both families are pre-bundled by the renderer, so
there is no build-time font fetch and no fail-closed risk in a cloud render.

A known non-blocking lint warning, `timeline_track_too_dense`, fires on all five:
each is a six-scene monolith rather than six mounted sub-compositions. That is
the intended shape here — the scenes are template-generated and never edited by
hand, so the readability the warning protects does not apply.
