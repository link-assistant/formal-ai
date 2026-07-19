// Issue #557: Apple "Liquid Glass" for the Chakra UI `glass` skin.
//
// The user asked us to build the glass skin on top of Chakra UI using
// rdev/liquid-glass-react (https://github.com/rdev/liquid-glass-react). We
// verified no ready-made "Chakra UI + liquid-glass-react" integration exists on
// npm, so this module is the careful in-house bridge (to be extracted into its
// own package later, per the issue).
//
// The library renders a genuine refractive glass element via an SVG
// feDisplacementMap warp layer. It has two quirks we tame here:
//   1. It assumes Tailwind utility classes (`bg-black`, `opacity-0`,
//      `pointer-events-none`, `mix-blend-overlay`, `text-white`, …). We inject a
//      tiny scoped stylesheet that defines exactly those utilities so the glass
//      renders correctly without pulling Tailwind into the bundle.
//   2. It centres itself with `translate(-50%, -50%)` from `top/left: 50%` and
//      sizes to its child + padding — it is built for fixed-size floating
//      controls. We therefore use it as a *decorative backing layer*: a
//      non-interactive, absolutely-positioned glass pane sized to fill its host,
//      with the real (accessible, test-targeted) Chakra button sitting on top.
//      This keeps the DOM/behaviour of interactive controls identical across
//      skins (E2E-safe) while showing authentic liquid glass behind them.
//
// Displacement refraction is Chromium-first; Safari/Firefox degrade gracefully
// to the blur + tint layers, which is acceptable for a progressive-enhancement
// skin. When the glass skin is inactive this module renders nothing.

import React from "react";
import LiquidGlass from "liquid-glass-react";

// The bundler's JSX factory is pinned to `h` (React.createElement) via
// jsconfig.json — see src/web/app/main.jsx. Any JSX in this module compiles to
// `h(tag, props, …children)`, so `h`/`Fragment` must be in scope here too.
const { createElement: h, Fragment, useRef } = React;

// The Tailwind-ish utilities liquid-glass-react references internally. Scoped
// under `.fa-glass-backing` so they can never leak onto the rest of the app.
const GLASS_UTILITY_CSS = `
.fa-glass-backing .relative { position: relative; }
.fa-glass-backing .absolute { position: absolute; }
.fa-glass-backing .bg-black { background-color: #000; }
.fa-glass-backing .opacity-0 { opacity: 0; }
.fa-glass-backing .opacity-20 { opacity: 0.2; }
.fa-glass-backing .opacity-100 { opacity: 1; }
.fa-glass-backing .pointer-events-none { pointer-events: none; }
.fa-glass-backing .cursor-pointer { cursor: pointer; }
.fa-glass-backing .mix-blend-overlay { mix-blend-mode: overlay; }
.fa-glass-backing .transition-all { transition-property: all; }
.fa-glass-backing .duration-150 { transition-duration: 150ms; }
.fa-glass-backing .ease-in-out { transition-timing-function: cubic-bezier(0.4, 0, 0.2, 1); }
.fa-glass-backing .text-white { color: #fff; }
.fa-glass-backing { pointer-events: none; }
.fa-glass-backing, .fa-glass-backing * { pointer-events: none !important; }
`;

let glassUtilitiesInjected = false;

// Inject the scoped utility stylesheet exactly once, lazily, on first use.
function ensureGlassUtilities() {
  if (glassUtilitiesInjected) {
    return;
  }
  if (typeof document === "undefined") {
    return;
  }
  const style = document.createElement("style");
  style.id = "fa-liquid-glass-utilities";
  style.textContent = GLASS_UTILITY_CSS;
  document.head.appendChild(style);
  glassUtilitiesInjected = true;
}

// Map the user's glass configuration (opacity/blur/refraction from settings)
// onto liquid-glass-react props. Kept pure so it is easy to unit-test.
export function glassConfigToProps(config = {}) {
  const opacity = Number.isFinite(config.opacity) ? config.opacity : 0.65;
  const blur = Number.isFinite(config.blur) ? config.blur : 0.11;
  const refraction = Number.isFinite(config.refraction) ? config.refraction : 60;
  // More opaque glass refracts less and blurs a touch more, mirroring how a
  // frostier pane scatters rather than bends light.
  const opacityFactor = 1 - Math.min(Math.max(opacity, 0), 1);
  return {
    displacementScale: Math.round(refraction * (0.6 + opacityFactor * 0.8)),
    blurAmount: blur,
    saturation: 150,
    aberrationIntensity: 2,
    elasticity: 0.12,
  };
}

// A non-interactive liquid-glass pane that fills its (positioned) host. `width`
// and `height` are the fixed pixel size of the control it backs; `radius`
// matches the control's corner radius. Renders nothing outside the glass skin.
export function GlassBacking({
  width,
  height,
  radius = 999,
  config,
  active = true,
}) {
  const hostRef = useRef(null);
  if (!active) {
    return null;
  }
  ensureGlassUtilities();
  const props = glassConfigToProps(config);
  return (
    <span
      ref={hostRef}
      className="fa-glass-backing"
      aria-hidden="true"
      style={{
        position: "absolute",
        top: 0,
        left: 0,
        width,
        height,
        borderRadius: radius,
        pointerEvents: "none",
        // liquid-glass-react sizes its refractive canvas to child + internal
        // padding, so it renders a few pixels TALLER than the host control and
        // adds a large `0 12px 40px` drop shadow. Without clipping, that orb +
        // halo bleeds past the button and reads as a "wild" floating chip
        // (Issue #557 review). `overflow: hidden` clamps the backing to the
        // exact rounded footprint of the control it decorates.
        overflow: "hidden",
        // Sits behind the host control's own content (the icon/label), which is
        // painted in normal flow above this negative layer. The host sets
        // `isolation: isolate` so this layer stays clamped to the control.
        zIndex: -1,
      }}
    >
      <LiquidGlass
        {...props}
        cornerRadius={radius}
        padding="0px"
        mode="standard"
        mouseContainer={hostRef}
        style={{
          position: "absolute",
          top: "50%",
          left: "50%",
          pointerEvents: "none",
        }}
      >
        <span style={{ display: "block", width, height }} />
      </LiquidGlass>
    </span>
  );
}

export default GlassBacking;
