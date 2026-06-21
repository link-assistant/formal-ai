# Solution Plans — Issue #550

For each requirement: the options considered, the option chosen for this PR, and (where
relevant) the larger follow-up. Implemented choices are marked **[shipped]**.

---

## P1 — container-level scroll fade

**Goal.** One continuous "more content below" fade across the whole collapsed thinking
stack, not a separate fade per step line.

| Option | Assessment |
|---|---|
| **A. Container `mask-image` gradient** **[shipped]** | Move the gradient onto `.thinking-preview-collapsed:has(.thinking-preview-previous)` as `linear-gradient(to bottom, transparent 0, #000 1.4em)`; remove the per-line mask. `:has()` scopes it so a lone first step (no previous) is never masked. Minimal, declarative, GPU-composited, no JS. |
| B. `::after` gradient overlay | A positioned overlay with `background: linear-gradient(...)` over the container. Works, but needs a matching background color per theme (re-introduces the duplication problem) and sits above text (z-index fiddling). Rejected. |
| C. Scroll-driven `animation-timeline: scroll()` | Newer CSS; could fade based on actual scroll position. Overkill for a fixed two-line preview and weaker browser support than `mask-image`. Deferred. |

**Chosen: A.** The fade is presentation-only and belongs in CSS; moving it from the line
to the container is the literal expression of the requirement.

---

## P2 — render thinking detail in full

**Goal.** Stop clipping a normal step's detail mid-sentence while keeping an upper bound
so the preview cannot grow unbounded.

| Option | Assessment |
|---|---|
| **A. Raise the cap 120 → 600 in both surfaces** **[shipped]** | Smallest change that fixes the symptom; keeps a bound so a pathological multi-KB detail still truncates with an ellipsis. Rust `truncate_thinking_detail` + JS `thinkingDetailText` kept in sync with cross-referencing comments. |
| B. Remove the cap entirely | Risks an unbounded preview height from a runaway step; the panel relies on a predictable height for the P1 fade. Rejected. |
| C. Word-boundary truncation at the cap | Nicer truncation, but the real complaint is *legitimate detail being cut*, not *ugly cut points*. 600 chars already clears realistic single-step content; word-boundary polish is a possible later refinement. Deferred. |

**Chosen: A.** A single source of truth would be ideal (see M-series below), but the Rust
core and the browser worker are genuinely separate runtimes; the comment-linked constant
pair is the pragmatic in-repo answer.

---

## P3 — stable pending message width

**Goal.** No width jump when a message transitions from thinking-only to having a body.

| Option | Assessment |
|---|---|
| **A. Remove the `width: 116px` clamp** **[shipped]** | The clamp was a leftover typing-indicator dimension; deleting it (and the dead `.typing::after`) lets the pending body use the same width as a settled body, so there is nothing to jump. |
| B. Animate the width transition | Masks the jump instead of removing it, and animating `width` is layout-thrashing. Rejected — the correct width is "the same width", not "a smoothly-growing width". |

**Chosen: A.**

---

## P4 — services/update dark theming

**Goal (immediate).** Every services/update surface is readable in dark mode.
**Goal (systemic).** New surfaces stop silently missing dark rules.

| Option | Assessment |
|---|---|
| A. Hand-write the missing dark rules in both dark layers | Directly fixes the symptom now, in both `:root[data-theme="dark"]` and the `prefers-color-scheme` media query (per "fix in all places"). Matches the file's existing convention so the diff is reviewable — but leaves the duplication root cause in place. Shipped first, then superseded by B. |
| **B. Introduce CSS custom properties (design tokens)** **[shipped]** | A `--fa-*` semantic-token palette (`--fa-surface-card`, `--fa-surface-raised`, `--fa-border-subtle`, `--fa-text-body`, `--fa-text-muted`, `--fa-accent-link`, the control hover/active/focus tokens, …) defined once per theme. Light values live in `:root{}`; each dark layer overrides **only the values** (≈20 lines), so a new surface inherits correct theming by consuming a token instead of re-deriving hex. Removes the manual-duplication **root cause** of P4 *and* P5 — see [`best-practices.md`](best-practices.md) §1. Value-preserving: every token maps to the exact prior hex, so the regression tests' exact-RGB assertions stayed green. |
| C. Chakra color-mode tokens | The literal Chakra runtime is CSP-blocked (see the M2/M3 ADR below); the `--fa-*` tokens are the CSP-safe substrate that a future Chakra theme maps onto 1:1. |

**Chosen for this PR: B** (built on A). The token system is the idiomatic precedent
already used by `landing.css` and by the `--code-*`/`--hljs-*` tokens inside
`.markdown-body`; this change brings `styles.css` in line with them.

---

## P5 — uniform topbar hover/focus

**Goal.** Every header control gives the same hover and keyboard-focus feedback.

| Option | Assessment |
|---|---|
| **A. One shared selector list for all topbar controls** **[shipped]** | Group every control (`report`/`source-code`/`download`/`memory` buttons, `sidebar-toggle`, `mobile-menu-toggle`, `mode`/`diagnostics` toggles) under one `:hover`, one active-state `:hover`, and one `:focus-visible` ring, in light + dark — now driven by the `--fa-control-hover-*` / `--fa-focus-ring` tokens. Removes the per-button drift that caused "partial". |
| **B. A reusable `ToolbarButton` React component** **[shipped]** | The structural home for the shared treatment (M2). A single `ToolbarButton` (anchor/button variants, icon + localized label, a11y + menu-priority props) now renders **all 11** topbar controls, so a control can no longer be added with the wrong markup or a missing class. Built on raw `React.createElement` — no JSX/Chakra runtime required, so it ships under the strict CSP today. |

**Chosen: A and B together.** The CSS shared-selector layer makes the *treatment*
uniform; the `ToolbarButton` component makes the *markup* uniform by construction. Both
shipped in this PR.

---

## M2 / M3 — component reuse & Chakra UI + JSX migration

This is the strategic requirement. The reviewer's standing instruction was explicit:
*"we do it in this PR … Nothing can be deferred or delayed to other pull requests."* So we
delivered the **maximum CSP-safe substance** of the Chakra/JSX migration in this PR — its
design-token theme system and its first reusable component primitive — and documented, with
evidence, the one part that is **architecturally blocked** by the codebase's own security
hardening. This section is an ADR for that decision.

### What the migration is, decomposed

Chakra UI is three separable things, and the requirement's intent ("everything nice and
polished, consistent, reusable") is served by the first two:

1. **A semantic design-token theme system** (single source of colour per mode). — **Shipped**
   as the `--fa-*` token palette (P4-option-B). This is the exact substrate Chakra's
   `semanticTokens` provide; a future `ChakraProvider` theme maps onto these 1:1.
2. **A reusable component library** (shared `Button`/`IconButton` etc.). — **Shipped** as the
   `ToolbarButton` primitive consolidating all 11 topbar controls (P5-option-B). This is the
   substrate Chakra's components provide, built CSP-safely on raw `createElement`.
3. **The Emotion CSS-in-JS runtime + JSX authoring** (Chakra renders by injecting `<style>`
   tags at runtime and its components are authored in JSX). — **Blocked**, see below.

### Why the literal Emotion-runtime Chakra + JSX cannot ship here (the ADR)

* **Strict CSP forbids Emotion's runtime style injection.** Issue [#479][issue-479] hardened
  the app to `style-src 'self'` — no `'unsafe-inline'`. Chakra is built on Emotion, which
  **injects `<style>` rules into the document head at runtime**; that requires
  `style-src 'unsafe-inline'` (or a per-render nonce the static file server cannot supply).
  Adopting the Chakra runtime would mean **weakening the #479 CSP** — trading a real,
  shipped security property for a styling refactor. That is the wrong trade, and it is why
  the token system (which needs **no** inline styles — custom properties are CSP-neutral) is
  the correct expression of the requirement under this codebase's constraints.
* **`app.js` is served raw, not transpiled.** The ≈10.7k-line `src/web/app.js` is delivered
  verbatim via `<script src="app.js">` — there is **no JSX/transpile step** in the serving
  path (only `vendor.bundle.js` / `ocr.bundle.js` are `bun build`-ed, and a `git diff
  --exit-code` CI gate freezes them). JSX authoring would require introducing a build step
  for the main app file. The `ToolbarButton` component delivers JSX's *reusability* benefit
  without that toolchain, by composing `h()` calls.

These are not "too much work" objections — they are correctness/security objections. A
literal Chakra runtime here would either break under the CSP at runtime or force a security
regression. The decision is therefore to ship Chakra's **value** (tokens + components) and
record the runtime as blocked-by-CSP until #479's constraints change.

### If the CSP constraint is ever lifted (forward path)

1. **Tokens** — already shipped; rename `--fa-*` → Chakra `semanticTokens` (`_light`/`_dark`
   conditions map onto the existing `data-theme` + `prefers-color-scheme` layering, per the
   [Chakra v3 token docs][chakra-tokens]).
2. **Components** — already started; `ToolbarButton` becomes a thin wrapper over Chakra
   `Button`, then services/update cards, then the thinking preview.
3. **Runtime** — only at this step does Emotion enter, and **only if** #479's `style-src` is
   relaxed to admit a nonce/hash strategy for Emotion's injected styles, or Chakra is run in
   a static-extraction mode. Until then steps 1–2 stand on their own.

### Library evaluation

| Library / feature | Relevance | Verdict |
|---|---|---|
| **Chakra UI v3** (`@chakra-ui/react`) | Issue's stated target. Its **token system** and **component model** are delivered here (`--fa-*` + `ToolbarButton`); its **Emotion runtime** is CSP-blocked by [#479][issue-479]. | **Value adopted CSP-safely; runtime deferred until the CSP admits it.** |
| **CSS custom properties** (native) | Zero-dependency, CSP-neutral fix for the P4/P5 duplication root cause and the Chakra-token substrate. | **Shipped** (`--fa-*` palette). |
| **Reusable `createElement` component** | Delivers JSX's reusability without a transpile step for the raw-served `app.js`. | **Shipped** (`ToolbarButton`). |
| **`mask-image` + `linear-gradient`** ([MDN][mdn-mask]) | The P1 scroll-fade mechanism; fading a scroll edge is a documented standard use. | **Baseline** (Chrome/Edge 120+, Firefox 53+, Safari 15.4+; `-webkit-` for older Safari). **Shipped.** |
| **`:has()` selector** ([MDN][mdn-has]) | Scopes the P1 fade to "container that has a previous step", so a lone step isn't masked. | **Baseline** (Dec 2023). **Shipped.** |
| **`:focus-visible`** ([MDN][mdn-fv]) | Keyboard-only focus ring for P5 without rings on mouse click; [W3C WCAG C45][wcag-c45]. | **Baseline** (2019). **Shipped.** |
| **Playwright** (already a dependency) | Behavioral regression for all five defects (computed colors, widths, hover/focus). | **Shipped** (`issue-1963.spec.js`). |
| Emotion / styled-components | Runtime CSS-in-JS; would come transitively with Chakra. Incompatible with the #479 `style-src 'self'` CSP. | **Rejected under current CSP.** |
| Tailwind | Utility CSS could also centralize tokens, but the issue explicitly names Chakra; a second system would fragment the styling story. | Rejected. |

---

## Summary

* **Shipped now ([formal-ai#551][pr]):** P1, P2, P3, P4, P5 — each minimal and in-place,
  with regression tests; **plus** the strategic M2/M3 substance — the `--fa-*` design-token
  system (removes the P4/P5 duplication root cause) and the reusable `ToolbarButton`
  component (consolidates all 11 topbar controls).
* **Blocked by the #479 CSP (not deferred by choice):** the Emotion-runtime Chakra +
  JSX-authoring layer, because Emotion's runtime `<style>` injection requires
  `style-src 'unsafe-inline'` and `app.js` is served un-transpiled. Documented above with a
  forward path for if/when the CSP is relaxed.

[pr]: https://github.com/link-assistant/formal-ai/pull/551
[issue-479]: https://github.com/link-assistant/formal-ai/issues/479
[mdn-mask]: https://developer.mozilla.org/en-US/docs/Web/CSS/mask-image
[mdn-has]: https://developer.mozilla.org/en-US/docs/Web/CSS/:has
[mdn-fv]: https://developer.mozilla.org/en-US/docs/Web/CSS/:focus-visible
[wcag-c45]: https://www.w3.org/WAI/WCAG21/Techniques/css/C45
[chakra-tokens]: https://chakra-ui.com/docs/theming/semantic-tokens
