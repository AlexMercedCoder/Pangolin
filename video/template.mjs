// The one template every video in the series is generated from.
//
// Six scene archetypes — open, statement, list, table, code, close — share a
// persistent chrome (top bar, progress ticker) and a persistent decorative
// field (two glows, hairline rules, an arc fan, grain). Each video differs only
// in which archetypes it strings together and what copy it carries, which is
// what makes the five read as one series.

import { palette, fonts, canvas, css, toneFor } from "./brand.mjs";

const SCENE_SECONDS = {
  open: 4.0,
  statement: 5.5,
  close: 3.6,
};

function sceneDuration(scene) {
  if (SCENE_SECONDS[scene.type]) return SCENE_SECONDS[scene.type];
  if (scene.type === "list") return round2(4.2 + 1.15 * scene.rows.length);
  if (scene.type === "table") return round2(4.0 + 1.0 * scene.rows.length);
  if (scene.type === "code") return round2(3.6 + 0.85 * scene.lines.length);
  throw new Error(`unknown scene type: ${scene.type}`);
}

const round2 = (n) => Math.round(n * 100) / 100;

// Escape everything, then re-open the three inline tags the copy is allowed to
// use: <u> accent on display faces, <k> accent-bold and <q> muted on mono rows.
function copy(text) {
  return String(text)
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/&lt;(\/?)(u|k|q)&gt;/g, "<$1$2>");
}

// ---------------------------------------------------------------- scenes ----

// A wordmark wraps at spaces but a single long token cannot, so it will run
// straight under the specs column at the default size. Size the open title to
// its longest unbreakable token instead of assuming eight characters.
const WORDMARK_MAX = 176;
const WORDMARK_TRACK = 0.72; // approximate cap advance per em in Archivo Black
const LEDE_WIDTH = 1120;

function wordmarkSize(markup) {
  const longest = markup
    .replace(/<\/?[a-z]+>/g, "")
    .split(/\s+/)
    .reduce((max, word) => Math.max(max, word.length), 0);
  if (!longest) return WORDMARK_MAX;
  return Math.min(WORDMARK_MAX, Math.floor(LEDE_WIDTH / (longest * WORDMARK_TRACK)));
}

function renderScene(scene, i) {
  const id = `sc${i}`;
  const inner = (html) => `<div class="inner">${html}</div>`;

  const kicker = scene.kicker
    ? `<div class="kicker"><s></s>${copy(scene.kicker)}</div>`
    : "";
  const head = scene.head ? `<h2 class="head">${copy(scene.head)}</h2>` : "";

  switch (scene.type) {
    case "open":
      return inner(`
        <div class="open">
          <div class="lede">
            <div class="wordmark" style="font-size:${wordmarkSize(scene.wordmark)}px">${copy(scene.wordmark)}</div>
            <p class="tagline">${copy(scene.tagline)}</p>
          </div>
          <div class="specs">
            ${scene.specs
              .map(
                ([k, v]) =>
                  `<dl class="spec"><dt>${copy(k)}</dt><dd>${copy(v)}</dd></dl>`,
              )
              .join("")}
          </div>
        </div>`);

    case "statement":
      return inner(`
        <div class="stmt">
          ${kicker}
          <div class="bar"></div>
          <div class="big">${copy(scene.big)}</div>
          <p class="sub">${copy(scene.sub)}</p>
        </div>`);

    case "list":
      return inner(`
        ${kicker}${head}
        <div class="rows">
          ${scene.rows
            .map((r) => `<div class="row"><i></i><p>${copy(r)}</p></div>`)
            .join("")}
        </div>`);

    case "table":
      return inner(`
        ${kicker}${head}
        <div class="trows">
          ${scene.rows
            .map(
              ([label, status]) => `<div class="trow">
                <b>${copy(label)}</b><s></s>
                <span class="pill" style="background:${toneFor(status)}">${copy(status)}</span>
              </div>`,
            )
            .join("")}
        </div>`);

    case "code":
      return inner(`
        ${kicker}${head}
        <div class="codepanel">
          ${scene.lines
            .map(
              ([tone, text]) =>
                `<div class="codeline ${tone}">${copy(text)}</div>`,
            )
            .join("")}
        </div>`);

    case "close":
      return inner(`
        <div class="close">
          <div class="big">${copy(scene.big)}</div>
          <div>
            <span class="repolink">
              <span class="repo">${copy(scene.repo)}</span>
              <span class="ul"></span>
            </span>
          </div>
        </div>`);
  }
  throw new Error(`unknown scene type: ${scene.type}`);
}

// ---------------------------------------------------------------- motion ----

// Every entrance is authored as a timeline instruction at an absolute position,
// never as a CSS initial transform — pairing the two is the one lint failure
// this composition shape reliably walks into.
function renderMotion(scene, i, start, dur) {
  const s = `#sc${i}`;
  const at = (o) => round2(start + o);
  const out = [];
  const push = (line) => out.push(`  ${line}`);

  switch (scene.type) {
    case "open":
      push(`tl.from("${s} .wordmark", { y: 74, opacity: 0, duration: 0.85, ease: "power4.out" }, ${at(0.15)});`);
      push(`tl.from("${s} .tagline", { y: 34, opacity: 0, duration: 0.7, ease: "power2.out" }, ${at(0.45)});`);
      push(`tl.from("${s} .spec", { x: 56, opacity: 0, duration: 0.6, stagger: 0.11, ease: "back.out(1.4)" }, ${at(0.6)});`);
      break;

    case "statement":
      push(`tl.from("${s} .kicker", { x: -44, opacity: 0, duration: 0.5, ease: "power3.out" }, ${at(0.1)});`);
      push(`tl.fromTo("${s} .bar", { scaleX: 0, transformOrigin: "left center" }, { scaleX: 1, duration: 0.6, ease: "expo.out", immediateRender: false }, ${at(0.28)});`);
      push(`tl.from("${s} .big", { y: 46, opacity: 0, duration: 0.75, ease: "power3.out" }, ${at(0.38)});`);
      push(`tl.from("${s} .sub", { y: 28, opacity: 0, duration: 0.65, ease: "power2.out" }, ${at(0.72)});`);
      break;

    case "list":
      push(`tl.from("${s} .kicker", { x: -44, opacity: 0, duration: 0.5, ease: "power3.out" }, ${at(0.1)});`);
      push(`tl.from("${s} .head", { y: 40, opacity: 0, duration: 0.65, ease: "power3.out" }, ${at(0.24)});`);
      push(`tl.from("${s} .row", { x: 54, opacity: 0, duration: 0.55, stagger: 0.13, ease: "power2.out" }, ${at(0.5)});`);
      break;

    case "table":
      push(`tl.from("${s} .kicker", { x: -44, opacity: 0, duration: 0.5, ease: "power3.out" }, ${at(0.1)});`);
      push(`tl.from("${s} .head", { y: 40, opacity: 0, duration: 0.65, ease: "power3.out" }, ${at(0.24)});`);
      push(`tl.from("${s} .trow b", { x: -34, opacity: 0, duration: 0.5, stagger: 0.1, ease: "power2.out" }, ${at(0.5)});`);
      push(`tl.fromTo("${s} .trow s", { scaleX: 0, transformOrigin: "left center" }, { scaleX: 1, duration: 0.55, stagger: 0.1, ease: "power2.inOut", immediateRender: false }, ${at(0.62)});`);
      push(`tl.from("${s} .pill", { scale: 0.7, opacity: 0, duration: 0.5, stagger: 0.1, ease: "back.out(2)" }, ${at(0.78)});`);
      break;

    case "code":
      push(`tl.from("${s} .kicker", { x: -44, opacity: 0, duration: 0.5, ease: "power3.out" }, ${at(0.1)});`);
      push(`tl.from("${s} .head", { y: 40, opacity: 0, duration: 0.65, ease: "power3.out" }, ${at(0.24)});`);
      push(`tl.from("${s} .codepanel", { y: 40, opacity: 0, duration: 0.6, ease: "power3.out" }, ${at(0.48)});`);
      push(`tl.from("${s} .codeline", { x: -30, opacity: 0, duration: 0.45, stagger: 0.14, ease: "power2.out" }, ${at(0.72)});`);
      break;

    case "close":
      push(`tl.from("${s} .big", { y: 48, opacity: 0, duration: 0.75, ease: "power3.out" }, ${at(0.12)});`);
      push(`tl.from("${s} .repo", { y: 24, opacity: 0, duration: 0.55, ease: "power2.out" }, ${at(0.55)});`);
      push(`tl.fromTo("${s} .ul", { scaleX: 0, transformOrigin: "left center" }, { scaleX: 1, duration: 0.7, ease: "expo.out", immediateRender: false }, ${at(0.7)});`);
      break;
  }

  // Exit on the inner wrapper, never on the clip itself — the framework owns
  // clip visibility and animating it races with the runtime. The zero-duration
  // set on the clip boundary is the hard kill: without it a seek that lands
  // past the fade can restore stale visibility state.
  push(`tl.to("${s} .inner", { opacity: 0, y: -26, duration: 0.45, ease: "power2.in" }, ${at(dur - 0.5)});`);
  push(`tl.set("${s} .inner", { opacity: 0 }, ${at(dur)});`);
  return out.join("\n");
}

// ------------------------------------------------------------- composition --

export function renderComposition(video) {
  let cursor = 0;
  const scenes = video.scenes.map((scene, i) => {
    const dur = sceneDuration(scene);
    const entry = { scene, i, start: round2(cursor), dur };
    cursor = round2(cursor + dur);
    return entry;
  });
  const total = round2(cursor);

  const clips = scenes
    .map(
      ({ scene, i, start, dur }) =>
        `      <section id="sc${i}" class="clip scene" data-start="${start}" data-duration="${dur}" data-track-index="1">
${renderScene(scene, i)}
      </section>`,
    )
    .join("\n");

  const ticker = scenes
    .map((_, i) => `<span id="tk${i}"${i === 0 ? ' class="on"' : ""}></span>`)
    .join("");

  // Progress ticker: a zero-duration set at each scene boundary. backgroundColor
  // is on the animatable allowlist and these are not clip elements, so the
  // state is reproducible from any seek in either direction.
  const tickerMotion = scenes
    .flatMap(({ i, start }) => {
      const lines = [
        `  tl.set("#tk${i}", { backgroundColor: "${palette.accent}" }, ${start});`,
      ];
      if (i > 0) {
        lines.unshift(
          `  tl.set("#tk${i - 1}", { backgroundColor: "${palette.rule}" }, ${start});`,
        );
      }
      return lines;
    })
    .join("\n");

  // Ambient loops. Finite repeat counts derived with floor so the last cycle
  // lands inside data-duration rather than overshooting it.
  const reps = (cycle) => Math.max(0, Math.floor(total / cycle) - 1);

  const motion = scenes
    .map(({ scene, i, start, dur }) => renderMotion(scene, i, start, dur))
    .join("\n");

  const fanArcs = [0, 1, 2, 3, 4, 5, 6]
    .map((n) => {
      const r = 190 + n * 82;
      return `<circle cx="550" cy="550" r="${r}" fill="none" stroke="${palette.accent}" stroke-width="3" stroke-opacity="${(0.26 - n * 0.028).toFixed(3)}" stroke-dasharray="${420 + n * 60} 4000" transform="rotate(-58 550 550)"/>`;
    })
    .join("");

  return `<!doctype html>
<html lang="en">
  <head>
    <meta charset="UTF-8" />
    <meta name="viewport" content="width=${canvas.w}, height=${canvas.h}" />
    <title>Pangolin 0.6.0 — ${video.title}</title>
    <script src="https://cdn.jsdelivr.net/npm/gsap@3.14.2/dist/gsap.min.js"></script>
    <style>
${css}
    </style>
  </head>
  <body>
    <div
      id="root"
      data-composition-id="main"
      data-start="0"
      data-duration="${total}"
      data-width="${canvas.w}"
      data-height="${canvas.h}"
      data-fps="${canvas.fps}"
    >
      <!-- Persistent field. Not clips: these exist for the whole composition. -->
      <div class="field"></div>
      <div class="deco glow-warm" id="glowWarm"></div>
      <div class="deco glow-cool" id="glowCool"></div>
      <div class="deco rules" id="hairRules">
        <span></span><span></span><span></span><span></span><span></span><span></span><span></span>
      </div>
      <svg class="deco fan" id="fan" viewBox="0 0 1100 1100" aria-hidden="true" data-layout-allow-overflow>${fanArcs}</svg>
      <div class="deco grain"></div>

      <!-- Persistent chrome. -->
      <div class="chrome">
        <div class="topbar">
          <div class="brandmark"><i></i><b>PANGOLIN</b></div>
          <div class="topmeta"><em>v0.6.0</em> &nbsp;·&nbsp; Alpha</div>
        </div>
        <div class="botbar">
          <div class="serieslabel">${copy(video.series)}</div>
          <div class="ticker">${ticker}</div>
        </div>
      </div>

${clips}
    </div>

    <script>
      window.__timelines = window.__timelines || {};
      const tl = gsap.timeline({ paused: true });

      // --- ambient field ---
  tl.to("#glowWarm", { scale: 1.14, duration: 5.5, ease: "sine.inOut", yoyo: true, repeat: ${reps(11)} }, 0);
  tl.to("#glowCool", { scale: 1.18, duration: 6.5, ease: "sine.inOut", yoyo: true, repeat: ${reps(13)} }, 0.8);
  tl.to("#fan", { rotation: 14, transformOrigin: "50% 50%", duration: ${total}, ease: "none" }, 0);
  tl.to("#hairRules", { opacity: 0.55, duration: 4, ease: "sine.inOut", yoyo: true, repeat: ${reps(8)} }, 0);

      // --- progress ticker ---
${tickerMotion}

      // --- scenes ---
${motion}

      window.__timelines["main"] = tl;
    </script>
  </body>
</html>
`;
}

export function compositionDuration(video) {
  return round2(
    video.scenes.reduce((acc, scene) => acc + sceneDuration(scene), 0),
  );
}
