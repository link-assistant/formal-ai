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
| C. Chakra color-mode tokens **[shipped]** | The `--fa-*` tokens are bridged 1:1 into Chakra `semanticTokens` by [`src/web/app/theme.js`](../../../src/web/app/theme.js) (`fa.surface.card` → `var(--fa-surface-card)`), so a Chakra component can consume the same themed values. styles.css stays the single source of truth for the hex values; Chakra references the vars. See the M2/M3 section below. |

**Chosen for this PR: B** (built on A). The token system is the idiomatic precedent
already used by `landing.css` and by the `--code-*`/`--hljs-*` tokens inside
`.markdown-body`; this change brings `styles.css` in line with them.

---

## P5 — uniform topbar hover/focus

**Goal.** Every header control gives the same hover and keyboard-focus feedback.

| Option | Assessment |
|---|---|
| **A. One shared selector list for all topbar controls** **[shipped]** | Group every control (`report`/`source-code`/`download`/`memory` buttons, `sidebar-toggle`, `mobile-menu-toggle`, `mode`/`diagnostics` toggles) under one `:hover`, one active-state `:hover`, and one `:focus-visible` ring, in light + dark — now driven by the `--fa-control-hover-*` / `--fa-focus-ring` tokens. Removes the per-button drift that caused "partial". |
| **B. A reusable `ToolbarButton` React component** **[shipped]** | The structural home for the shared treatment (M2). A single `ToolbarButton` (anchor/button variants, icon + localized label, a11y + menu-priority props) now renders **all 11** topbar controls, so a control can no longer be added with the wrong markup or a missing class. Authored in JSX and rendered through the Chakra styled factory (`chakra.a` / `chakra.button`), part of the full Chakra/JSX migration (M2/M3). |

**Chosen: A and B together.** The CSS shared-selector layer makes the *treatment*
uniform; the `ToolbarButton` component makes the *markup* uniform by construction. Both
shipped in this PR.

---

## M2 / M3 — component reuse & Chakra UI + JSX migration

This is the strategic requirement. The reviewer's standing instruction was explicit:
*"My requirement is translation to Chakra, and using modern JSX, with styles in JavaScript.
Use bun bundler for building. … we do it in this PR … Nothing can be deferred or delayed to
other pull requests."* This PR delivers the **full** migration: the front end is authored in
JSX, bundled by the bun bundler, mounted under `<ChakraProvider>`, and rendered through
Chakra's styled factory with styles driven from JavaScript via a token-bridged theme system.

> **Retraction.** An earlier draft of this section argued the Emotion-runtime Chakra + JSX
> layer was *"architecturally blocked"* by issue [#479][issue-479]'s `style-src 'self'` CSP
> and by `app.js` being *"served raw, not transpiled."* **Both premises were false and are
> withdrawn.** Verified facts: (1) the application page
> [`src/web/app/index.html`](../../../src/web/app/index.html) carries **no Content-Security-Policy**
> at all — the `style-src 'self'` CSP is present only on the marketing pages
> (`src/web/index.html`, `src/web/docs/index.html`, `src/web/download/index.html`), which do
> not load `app.js`, so Emotion's runtime `<style>` injection on the app page was never CSP-
> constrained; and (2) `app.js` is **not** served verbatim — `package.json`'s `build:web`
> already `bun build`s it (the same gate applies to `vendor.bundle.js` / `ocr.bundle.js`), so
> a transpile step for the app entry was a one-line config change, not an architectural
> barrier. The "block" did not exist; the migration ships.

### What shipped, decomposed

Chakra UI is three separable things; **all three** are now delivered:

1. **A semantic design-token theme system** (single source of colour per mode). — **Shipped.**
   The `--fa-*` token palette (P4-option-B) is bridged 1:1 into Chakra `semanticTokens` by
   [`src/web/app/theme.js`](../../../src/web/app/theme.js): `faVar("--fa-surface-card")` makes
   `fa.surface.card` resolve to `var(--fa-surface-card)`, so styles.css stays the single
   source of truth for the hex values while Chakra components reference the themed vars.
2. **A reusable component library + JSX authoring.** — **Shipped.** `src/web/app/main.jsx` is
   authored in JSX and the reusable `ToolbarButton` primitive (P5-option-B) renders all 11
   topbar controls through `chakra.a` / `chakra.button`.
3. **The Emotion CSS-in-JS runtime + `<ChakraProvider>`.** — **Shipped.** `main.jsx` mounts
   `createRoot(...).render(<ChakraProvider value={chakraSystem}><App /></ChakraProvider>)`;
   React, ReactDOM, `@chakra-ui/react` and `@emotion/react` are bundled into `app.js`.

### How the migration stays pixel-identical (the real ADR)

The genuine engineering problem was not *whether* to adopt Chakra but *how to do it without
regressing the pixel-tested UI*. Three deliberate choices make the conversion provably
behaviour-preserving:

* **Classic JSX runtime bound to `h`.** [`tsconfig.json`](../../../tsconfig.json) pins bun's
  JSX transform to `{jsx: "react", jsxFactory: "h", jsxFragmentFactory: "Fragment"}` with
  `h === React.createElement`. So the JSX in `main.jsx` compiles back to the *same*
  `h(tag, props, …children)` calls the file used before the migration. This let an AST
  codemod ([`experiments/codemod-h-to-jsx.mjs`](../../../experiments/codemod-h-to-jsx.mjs))
  convert 446 of 453 render calls to JSX with byte-identical compiled output, and made the
  conversion verifiable: [`experiments/verify-jsx-equivalence.mjs`](../../../experiments/verify-jsx-equivalence.mjs)
  compiles `main.jsx` before/after with `bun build … --packages external`, canonicalises
  both through `@babel/generator {compact:true}`, and asserts the only differences are the
  three hand-converted, semantics-preserving residuals (a dynamic SVG tag aliased as
  `const Shape = tag`, and two spread-children → array-children list renders, both keyed).
* **`preflight: false`.** Chakra does **not** inject its global CSS reset, so it cannot
  restyle existing elements (margins, box-sizing, headings).
* **`globalCss: {}`.** Chakra's default `html`/`body`/`*` global rules are removed, so the
  provider emits only CSS-variable definitions in `@layer tokens`; cascade layers rank below
  unlayered author styles, so even those cannot win over styles.css.
* **`chakra.*` low-level primitives, not recipes.** `chakra.a` / `chakra.button` /
  `chakra.div` / … render the plain element with `className` preserved and no recipe styles,
  so styles.css governs and computed styles are unchanged.

The net effect: mounting `<ChakraProvider>` and converting the tree to JSX + `chakra.*`
leaves the rendered DOM and computed styles byte-identical — confirmed by the full Playwright
suite (332 passed / 1 skipped) and the two static guards (`check-web-tdz`,
`check-web-hardcoded-ui-strings`), which parse the bun-compiled output.

### Library evaluation

| Library / feature | Relevance | Verdict |
|---|---|---|
| **Chakra UI v3** (`@chakra-ui/react`) | Issue's stated target. Token system, component model, `chakra.*` styled factory, and the Emotion runtime are all adopted. | **Shipped** (`<ChakraProvider>` + `theme.js` bridge + `chakra.*` primitives). |
| **Emotion** (`@emotion/react`) | Chakra's runtime CSS-in-JS engine; bundled into `app.js`. The app page has no CSP, so its runtime style injection is unconstrained. | **Shipped** (transitively via Chakra). |
| **bun bundler** | Compiles `main.jsx` (classic JSX runtime) → `app.js` via `build:web`. | **Shipped** (the requested build tool). |
| **CSS custom properties** (native) | The `--fa-*` palette: single source of colour, bridged into Chakra `semanticTokens`, fixes the P4/P5 duplication root cause. | **Shipped** (`--fa-*` palette + `theme.js`). |
| **Reusable JSX component** | `ToolbarButton` consolidates all 11 topbar controls through `chakra.*`. | **Shipped** (`ToolbarButton`). |
| **`mask-image` + `linear-gradient`** ([MDN][mdn-mask]) | The P1 scroll-fade mechanism; fading a scroll edge is a documented standard use. | **Baseline** (Chrome/Edge 120+, Firefox 53+, Safari 15.4+; `-webkit-` for older Safari). **Shipped.** |
| **`:has()` selector** ([MDN][mdn-has]) | Scopes the P1 fade to "container that has a previous step", so a lone step isn't masked. | **Baseline** (Dec 2023). **Shipped.** |
| **`:focus-visible`** ([MDN][mdn-fv]) | Keyboard-only focus ring for P5 without rings on mouse click; [W3C WCAG C45][wcag-c45]. | **Baseline** (2019). **Shipped.** |
| **Playwright** (already a dependency) | Behavioral regression for all five defects plus the migration (computed colors, widths, hover/focus). | **Shipped** (`issue-1963.spec.js`; full suite 332/1). |
| Tailwind | Utility CSS could also centralize tokens, but the issue explicitly names Chakra; a second system would fragment the styling story. | Rejected. |

---

## Summary

* **Shipped now ([formal-ai#551][pr]):** P1, P2, P3, P4, P5 — each minimal and in-place,
  with regression tests; **plus** the full M2/M3 migration — the front end is authored in
  **JSX**, bundled by the **bun bundler**, mounted under **`<ChakraProvider>`**, rendered
  through Chakra's **`chakra.*`** styled factory, and themed from JavaScript via the
  `--fa-*` → Chakra `semanticTokens` bridge in `theme.js`. The earlier "CSP-blocked"
  rationale was based on two false premises (the app page has no CSP; `app.js` is already
  bun-built) and is retracted above.
* **Equivalence proof:** the classic JSX runtime (`jsxFactory: h`) makes JSX compile back to
  the original `h()` calls, so the conversion is verifiable byte-for-byte
  (`verify-jsx-equivalence.mjs`); `preflight:false` + `globalCss:{}` + `chakra.*` primitives
  keep computed styles unchanged. The full Playwright suite (332/1) and both static guards
  confirm the DOM and behaviour are unchanged.

[pr]: https://github.com/link-assistant/formal-ai/pull/551
[issue-479]: https://github.com/link-assistant/formal-ai/issues/479
[mdn-mask]: https://developer.mozilla.org/en-US/docs/Web/CSS/mask-image
[mdn-has]: https://developer.mozilla.org/en-US/docs/Web/CSS/:has
[mdn-fv]: https://developer.mozilla.org/en-US/docs/Web/CSS/:focus-visible
[wcag-c45]: https://www.w3.org/WAI/WCAG21/Techniques/css/C45
[chakra-tokens]: https://chakra-ui.com/docs/theming/semantic-tokens
