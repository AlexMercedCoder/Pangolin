// Pangolin video brand system.
//
// The website is a light canvas built on the pangolin orange #df8d53 with a
// cool blue #53a5df secondary. Video inverts the canvas — tech subject, and a
// dark field lets the brand orange actually read at 30fps instead of washing
// out — but keeps the brand hues exact and tints every neutral toward the
// orange so nothing goes dead gray.
//
// Type: Archivo Black carries the claims, JetBrains Mono carries the evidence.
// That is the whole series in a pairing — Pangolin's story this release is a
// project walking back its own marketing, so the display face states and the
// machine face substantiates. Both are pre-bundled by the renderer, so no
// build-time font fetch and no cloud fail-closed risk.

export const palette = {
  bg: "#16110d", // near-black, warmed toward the accent hue
  bgLift: "#1e1711", // panel fill
  panel: "#241c15", // card fill
  rule: "#3b2e23", // hairline structure
  fg: "#f7f1ea", // warm off-white
  muted: "#c3b0a0", // secondary copy — still AA on bg
  // Metadata copy. Sits over the warm glow in places, so it is pitched well
  // above the 4.5:1 floor rather than at it — the glow lifts the local
  // background and a borderline value fails the contrast pass mid-scene.
  dim: "#b6a496",
  accent: "#df8d53", // brand primary
  accentDeep: "#ac5a20", // brand accent
  blue: "#53a5df", // brand secondary — the "spec / data" register
  ok: "#7cd0a4",
  warn: "#e8b04b",
  bad: "#f08a72",
};

export const fonts = {
  display: '"Archivo Black", "Montserrat", sans-serif',
  mono: '"JetBrains Mono", "Source Code Pro", monospace',
};

export const canvas = { w: 1920, h: 1080, fps: 30 };

// Status keyword -> pill treatment. Dark text on a light chip so every pill
// clears AA regardless of which status it lands on.
export const statusTone = {
  solid: palette.ok,
  good: palette.ok,
  ready: palette.ok,
  new: palette.blue,
  partial: palette.warn,
  beta: palette.warn,
  missing: palette.bad,
  untested: palette.bad,
  none: palette.bad,
};

export function toneFor(status) {
  const key = String(status).toLowerCase().split(/[^a-z]+/).filter(Boolean)[0];
  return statusTone[key] || palette.muted;
}

export const css = `
  * { margin: 0; padding: 0; box-sizing: border-box; }
  html, body {
    width: ${canvas.w}px; height: ${canvas.h}px;
    overflow: hidden; background: ${palette.bg};
  }
  body { font-family: ${fonts.mono}; color: ${palette.fg}; }

  #root { position: relative; width: ${canvas.w}px; height: ${canvas.h}px; overflow: hidden; }

  /* Full-bleed fill lives on a child, never on the composition root — the
     producer can drop the root element's own background and render black. */
  .field { position: absolute; inset: 0; background: ${palette.bg}; }

  .deco { position: absolute; pointer-events: none; }

  .glow-warm {
    top: -420px; right: -300px; width: 1500px; height: 1500px; border-radius: 50%;
    background: radial-gradient(circle, rgba(223,141,83,0.30) 0%, rgba(223,141,83,0.09) 42%, rgba(223,141,83,0) 68%);
  }
  .glow-cool {
    bottom: -560px; left: -380px; width: 1400px; height: 1400px; border-radius: 50%;
    background: radial-gradient(circle, rgba(83,165,223,0.22) 0%, rgba(83,165,223,0.06) 45%, rgba(83,165,223,0) 70%);
  }

  /* Vertical hairlines — structural rhythm across the frame. */
  .rules { inset: 0; display: flex; justify-content: space-between; padding: 0 200px; }
  .rules span { display: block; width: 2px; height: 100%; background: ${palette.rule}; opacity: 0.55; }

  /* The scale fan: overlapping arcs, the pangolin's armour read as geometry.
     Shapes rather than ghost type — decorative text fights the contrast pass. */
  .fan { bottom: -300px; right: -180px; width: 1100px; height: 1100px; }

  .grain {
    inset: 0; opacity: 0.16; mix-blend-mode: overlay;
    background-image: url("data:image/svg+xml;utf8,<svg xmlns='http://www.w3.org/2000/svg' width='160' height='160'><filter id='n'><feTurbulence type='fractalNoise' baseFrequency='0.85' numOctaves='3' stitchTiles='stitch'/></filter><rect width='160' height='160' filter='url(%23n)' opacity='0.5'/></svg>");
  }

  /* ---- Persistent chrome ---- */
  .chrome { position: absolute; inset: 0; }

  .topbar {
    position: absolute; top: 0; left: 0; right: 0; height: 88px;
    display: flex; align-items: center; justify-content: space-between;
    padding: 0 96px; border-bottom: 2px solid ${palette.rule};
  }
  .brandmark { display: flex; align-items: center; gap: 18px; }
  .brandmark i {
    display: block; width: 16px; height: 16px; border-radius: 50%;
    background: ${palette.accent};
  }
  .brandmark b {
    font-family: ${fonts.mono}; font-weight: 700; font-size: 26px;
    letter-spacing: 0.34em; color: ${palette.fg};
  }
  .topmeta {
    font-family: ${fonts.mono}; font-weight: 400; font-size: 22px;
    letter-spacing: 0.2em; color: ${palette.dim}; text-transform: uppercase;
  }
  .topmeta em { font-style: normal; color: ${palette.accent}; font-weight: 700; }

  .botbar {
    position: absolute; bottom: 0; left: 0; right: 0; height: 84px;
    display: flex; align-items: center; justify-content: space-between;
    padding: 0 96px; border-top: 2px solid ${palette.rule};
  }
  .serieslabel {
    font-family: ${fonts.mono}; font-size: 21px; letter-spacing: 0.24em;
    color: ${palette.dim}; text-transform: uppercase;
  }
  .ticker { display: flex; gap: 12px; align-items: center; }
  .ticker span {
    display: block; width: 68px; height: 6px; border-radius: 3px;
    background: ${palette.rule};
  }
  .ticker span.on { background: ${palette.accent}; }

  /* ---- Scene shell ---- */
  .scene {
    position: absolute; left: 0; right: 0; top: 88px; bottom: 84px;
    padding: 72px 96px 64px;
    display: flex; flex-direction: column; justify-content: center;
  }

  .kicker {
    display: flex; align-items: center; gap: 20px;
    font-family: ${fonts.mono}; font-weight: 700; font-size: 24px;
    letter-spacing: 0.3em; text-transform: uppercase; color: ${palette.accent};
    margin-bottom: 26px;
  }
  .kicker s { display: block; width: 84px; height: 3px; background: ${palette.accent}; text-decoration: none; }

  h2.head {
    font-family: ${fonts.display}; font-weight: 400; font-size: 74px;
    line-height: 1.04; letter-spacing: -0.015em; color: ${palette.fg};
    max-width: 1500px;
  }

  .idx {
    position: absolute; top: 56px; right: 96px;
    font-family: ${fonts.display}; font-weight: 400; font-size: 210px;
    line-height: 1; color: transparent;
    -webkit-text-stroke: 3px ${palette.rule};
  }

  /* ---- statement ---- */
  .stmt { max-width: 1560px; }
  .stmt .big {
    font-family: ${fonts.display}; font-weight: 400; font-size: 92px;
    line-height: 1.06; letter-spacing: -0.02em; color: ${palette.fg};
  }
  .stmt .big u { text-decoration: none; color: ${palette.accent}; }
  .stmt .sub {
    margin-top: 40px; max-width: 1280px;
    font-family: ${fonts.mono}; font-weight: 400; font-size: 34px;
    line-height: 1.5; color: ${palette.muted};
  }
  .stmt .bar { width: 200px; height: 5px; background: ${palette.accent}; margin-bottom: 44px; }

  /* ---- list ---- */
  .rows { margin-top: 48px; display: flex; flex-direction: column; gap: 26px; max-width: 1620px; }
  .row { display: flex; align-items: flex-start; gap: 28px; }
  .row i {
    flex: none; display: block; width: 16px; height: 16px; margin-top: 16px;
    background: ${palette.accent}; transform: rotate(45deg);
  }
  .row p {
    font-family: ${fonts.mono}; font-weight: 400; font-size: 33px;
    line-height: 1.42; color: ${palette.fg};
  }
  .row p k { font-style: normal; font-weight: 700; color: ${palette.accent}; }
  .row p q { quotes: none; font-style: normal; color: ${palette.muted}; }

  /* ---- table ---- */
  .trows { margin-top: 48px; display: flex; flex-direction: column; gap: 22px; max-width: 1640px; }
  .trow { display: flex; align-items: center; gap: 28px; }
  .trow b {
    font-family: ${fonts.mono}; font-weight: 400; font-size: 33px; color: ${palette.fg};
    white-space: nowrap;
  }
  .trow s {
    flex: 1; display: block; height: 2px; text-decoration: none;
    background-image: linear-gradient(to right, ${palette.rule} 0 12px, transparent 12px 24px);
    background-size: 24px 2px;
  }
  .pill {
    flex: none; display: block; padding: 9px 26px; border-radius: 999px;
    font-family: ${fonts.mono}; font-weight: 700; font-size: 24px;
    letter-spacing: 0.14em; text-transform: uppercase; color: ${palette.bg};
    min-width: 190px; text-align: center;
  }

  /* ---- code ---- */
  .codepanel {
    margin-top: 48px; max-width: 1560px; padding: 48px 56px;
    background: ${palette.panel}; border: 3px solid ${palette.accentDeep}; border-radius: 10px;
  }
  .codeline {
    font-family: ${fonts.mono}; font-weight: 400; font-size: 34px; line-height: 1.7;
    color: ${palette.fg}; white-space: pre;
  }
  .codeline.cmt { color: ${palette.dim}; }
  .codeline.key { color: ${palette.accent}; font-weight: 700; }
  .codeline.val { color: ${palette.blue}; }

  /* ---- open / close ---- */
  .open { display: flex; align-items: center; justify-content: space-between; gap: 80px; }
  .open .lede { max-width: 1120px; min-width: 0; }
  .wordmark {
    font-family: ${fonts.display}; font-weight: 400; font-size: 176px;
    line-height: 0.94; letter-spacing: -0.035em; color: ${palette.fg};
  }
  .wordmark u { text-decoration: none; color: ${palette.accent}; }
  .tagline {
    margin-top: 34px; font-family: ${fonts.mono}; font-weight: 400; font-size: 37px;
    line-height: 1.42; color: ${palette.muted}; max-width: 1080px;
  }
  .specs { flex: none; display: flex; flex-direction: column; gap: 22px; min-width: 340px; }
  .spec { border-left: 4px solid ${palette.accent}; padding-left: 24px; }
  .spec dt {
    font-family: ${fonts.mono}; font-weight: 400; font-size: 20px; letter-spacing: 0.24em;
    text-transform: uppercase; color: ${palette.dim}; margin-bottom: 6px;
  }
  .spec dd {
    font-family: ${fonts.mono}; font-weight: 700; font-size: 32px; color: ${palette.fg};
  }

  .close .big {
    font-family: ${fonts.display}; font-weight: 400; font-size: 84px; line-height: 1.08;
    letter-spacing: -0.02em; color: ${palette.fg}; max-width: 1500px;
  }
  .close .big u { text-decoration: none; color: ${palette.accent}; }
  .close .repolink { display: inline-block; margin-top: 46px; }
  .close .repo {
    display: block;
    font-family: ${fonts.mono}; font-weight: 700; font-size: 36px; color: ${palette.fg};
  }
  .close .ul { display: block; width: 100%; height: 5px; background: ${palette.accent}; margin-top: 14px; }
`;
