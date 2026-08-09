#!/usr/bin/env node
// Regenerate every video project from the shared template.
//
//   node build.mjs
//
// Each video becomes a self-contained HyperFrames project under out/<slug>/ so
// it can be checked and rendered independently:
//
//   npx hyperframes check  out/<slug>
//   npx hyperframes render out/<slug> --quality high -o renders/<slug>.mp4
//
// Editing brand.mjs, template.mjs or content.mjs and re-running this script
// rebuilds all five consistently. Nothing under out/ is hand-edited.

import { mkdirSync, writeFileSync, rmSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

import { videos } from "./content.mjs";
import { renderComposition, compositionDuration } from "./template.mjs";

const here = dirname(fileURLToPath(import.meta.url));
const outDir = join(here, "out");

const CLI_PIN = "0.7.103";

rmSync(outDir, { recursive: true, force: true });
mkdirSync(outDir, { recursive: true });

const manifest = [];

for (const video of videos) {
  const dir = join(outDir, video.slug);
  mkdirSync(dir, { recursive: true });

  writeFileSync(join(dir, "index.html"), renderComposition(video), "utf8");

  writeFileSync(
    join(dir, "hyperframes.json"),
    JSON.stringify(
      {
        $schema: "https://hyperframes.heygen.com/schema/hyperframes.json",
        registry:
          "https://raw.githubusercontent.com/heygen-com/hyperframes/main/registry",
        paths: {
          blocks: "compositions",
          components: "compositions/components",
          assets: "assets",
        },
        media: { autoProxy: true },
        authoringSkill: "general-video",
      },
      null,
      2,
    ) + "\n",
    "utf8",
  );

  writeFileSync(
    join(dir, "package.json"),
    JSON.stringify(
      {
        name: `pangolin-video-${video.slug}`,
        private: true,
        type: "module",
        scripts: {
          dev: `npx --yes hyperframes@${CLI_PIN} preview`,
          check: `npx --yes hyperframes@${CLI_PIN} check`,
          render: `npx --yes hyperframes@${CLI_PIN} render`,
        },
      },
      null,
      2,
    ) + "\n",
    "utf8",
  );

  const duration = compositionDuration(video);
  manifest.push({
    slug: video.slug,
    title: video.title,
    series: video.series,
    scenes: video.scenes.length,
    duration,
  });
  console.log(
    `built ${video.slug.padEnd(26)} ${String(duration).padStart(6)}s  ${video.scenes.length} scenes`,
  );
}

writeFileSync(
  join(here, "manifest.json"),
  JSON.stringify(manifest, null, 2) + "\n",
  "utf8",
);

const total = manifest.reduce((a, v) => a + v.duration, 0);
console.log(`\n${manifest.length} videos, ${Math.round(total)}s total`);
