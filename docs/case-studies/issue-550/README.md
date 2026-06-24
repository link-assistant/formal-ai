# Case Study — Issue #550 "Unexpected UI/UX behavior"

Deep analysis of [link-assistant/formal-ai#550][issue], the five reported UI/UX
defects, their shared root cause, the fix shipped in [#551][pr], and the strategic
work delivered with it: the **full Chakra UI + JSX migration** — the front end is now
authored in JSX, bundled by the bun bundler, mounted under `<ChakraProvider>`, and
themed from JavaScript via a `--fa-*` → Chakra `semanticTokens` bridge, with a reusable
`ToolbarButton` rendered through Chakra's styled factory (see §6).

> **Mirror.** This product-repo issue is mirrored to the tracking repo as
> [link-assistant/hive-mind#1963][hm-issue]. The fix commits and tests landed on the
> formal-ai branch `issue-550-c636b0e4075d` (PR [#551][pr]); because the canonical fix
> was authored against the mirror, the shipped **test files and changelog reference
> `#1963`** (`tests/unit/issue_1963.rs`, `tests/e2e/tests/issue-1963.spec.js`). This
> case study uses the product ID `#550` throughout and maps the two IDs explicitly in
> [§8](#8-how-this-pr-delivers). A sibling case study exists in the tracking repo at
> `docs/case-studies/issue-1963/`; this document is the formal-ai-native record the
> issue's M4 requirement asks for (`docs/case-studies/issue-{id}` in *this* repo).

Companion documents in this folder:

| File | Contents |
|---|---|
| [`requirements.md`](requirements.md) | Every requirement (P1–P5 defects + M1–M8 meta), verbatim, traced to fix + test. |
| [`solution-plans.md`](solution-plans.md) | Options A/B/C per problem, the shipped choice, and the Chakra/JSX migration record (JSX + bun bundler + `<ChakraProvider>` + token bridge — all shipped; the earlier "CSP-blocked" claim retracted). |
| [`best-practices.md`](best-practices.md) | Nine lessons so the *class* of defect cannot recur (incl. CI gotchas). |
| [`proposed-issues.md`](proposed-issues.md) | Upstream/third-party assessment (conclusion: none warranted). |
| [`raw-data/`](raw-data/) | Issue/PR JSON + predecessor issues (#488/#541) + v0.214.0 code snapshots (the "download all logs and data" requirement). |
| [`screenshots/`](screenshots/) | `before/` and `after/` renders of every affected surface (light + dark). |

---

## 1. Summary

Issue #550 reports **five UI/UX defects** in the formal-ai chat surface, plus a set of
**codebase-wide and strategic requirements** (fix every duplicate, reuse components,
migrate to Chakra UI + JSX, compile a case study, file upstream issues if any).

| # | Defect (short) | Layer | Root cause |
|---|---|---|---|
| **P1** | Reasoning fade repeats per step instead of spanning the whole collapsed stack | CSS | `mask-image` gradient on the per-line element, not the container |
| **P2** | Thinking step detail is clipped mid-sentence ("some parts are omitted") | Logic (Rust + JS) | A 120-char cap in both `truncate_thinking_detail` and `thinkingDetailText` |
| **P3** | Message width jumps when the body appears after thinking-only | CSS | A leftover `.pending .message-body { width: 116px }` typing-indicator clamp |
| **P4** | The `services` / auto-update box is unreadable in dark mode | CSS | New surface shipped with light rules only; no dark rules in either dark layer |
| **P5** | Only some top-menu buttons react to hover | CSS | Per-button hover rules; newer controls never got them |

The single fact connecting **P4 and P5** is that `src/web/styles.css` uses **zero CSS
custom properties** — light and dark are *manually duplicated hex* across three layers,
and every interactive treatment is hand-written per element, so each new surface or
button silently misses rules the older ones already have (full analysis in
[`best-practices.md` §1](best-practices.md)).

**Shipped in [#551][pr]:** all five defects fixed in-place, with Rust unit tests (P2)
and Playwright behavioral tests (P1/P3/P4/P5). **Plus the full M2/M3 migration:** the
front end (`src/web/app/main.jsx`) is now authored in **JSX**, bundled by the **bun
bundler** into `src/web/app.js`, mounted under **`<ChakraProvider>`**, and rendered
through Chakra's **`chakra.*`** styled factory. A `--fa-*` semantic design-token system
in `styles.css` (collapses the three-layer hex duplication that is the P4/P5 root cause
into one source of truth per theme) is bridged 1:1 into Chakra `semanticTokens` by
`src/web/app/theme.js`, so styles are driven from JavaScript while styles.css stays the
single source of truth for colour. A reusable `ToolbarButton` renders all 11 topbar
controls, so the shared markup/treatment is uniform by construction.

> **Retraction.** An earlier draft claimed Chakra's Emotion runtime was "CSP-blocked"
> by [#479][issue-479] and that `app.js` was "served un-transpiled." **Both premises
> were false and are withdrawn:** the application page `src/web/app/index.html` carries
> **no CSP** (the `style-src 'self'` policy is only on the marketing pages, which don't
> load `app.js`), and `app.js` is **already bun-built** from `main.jsx` by `build:web`.
> The migration ships in full — see §6 and [`solution-plans.md`](solution-plans.md).

---

## 2. Timeline / sequence of events

Reconstructed from the formal-ai git history (commit hashes are verifiable) and the
issue/PR metadata in [`raw-data/`](raw-data/).

| When | Event | Evidence |
|---|---|---|
| 2026-06-15 | **`9d4ab641` "Add visible thinking preview"** introduces the collapsed reasoning stack — the per-line `mask-image` fade (→ **P1**) and the 120-char detail cap (→ **P2**). | `git log -S"thinking-preview-previous"` |
| 2026-06-17 | **`5ce4a4b2` "Surface thinking on top … (issue #488)"** extends the preview; the per-line fade and cap persist. | same |
| 2026-06-20 | **`151865c7` "fix(web): close dark-theme gaps on primary widgets (#541 R1)"** — the dark-mode pass that the issue's Chakra ask grew out of; it fixed *primary* widgets but left the pending-width relic (**P3**) and the newer buttons' hover gap (**P5**). | `git log -S"prefers-color-scheme"` |
| 2026-06-20 | **PR #549 "add auto update flow"** (`2c0f465e feat(desktop): add auto update flow`) adds the desktop services/update panel with **light rules only** → **P4**. | repo history |
| 2026-06-21 | **v0.214.0 released** (`2febec94`). The five rough edges are now all live together in one screenshot. | repo history |
| 2026-06-21 11:56 | **hive-mind#1963** filed (tracking mirror). | [hm-issue][hm-issue] |
| 2026-06-21 13:07:31Z | **formal-ai#550** filed by `konard` with the annotated screenshot and the five-point list. | `raw-data/issue-550.json` |
| 2026-06-21 13:08:48Z | **formal-ai#551** draft PR opened on `issue-550-c636b0e4075d` (placeholder `.gitkeep` commit `2d07cc5b`). | `raw-data/pr-551.json` |
| 2026-06-21 | **`e58966dc`** fix → **`c4f1d8e9`** ru/hi/zh tests → **`5a90a4f7`** revert placeholder. | branch history |
| 2026-06-21 13:55:46Z | `konard` posts the deep-analysis comment on #550 (root causes, repros, workarounds, fixes), linking the tracking-repo case study. | `raw-data/issue-550-comments.json` |

**Reading of the timeline.** None of the five is a fresh bug — each is a *residual
rough edge* from features shipped quickly (#488 thinking preview; #541 theme pass;
#549 auto-update panel). They cluster because the stylesheet has no shared token layer
to make new code inherit correct behavior.

---

## 3. Root-cause map

| # | Symptom (verbatim) | Where | Root cause | Fix |
|---|---|---|---|---|
| **P1** | "each paragraph/line/step has its own gradient … should be applied to full scrolled container" | `src/web/styles.css` | The fade `mask-image` sat on the per-line `.thinking-preview-previous`. | Move the gradient to `.thinking-preview-collapsed:has(.thinking-preview-previous)` as one `linear-gradient(to bottom, transparent 0, #000 1.4em)`; drop the per-line mask. `:has()` keeps a lone first step unmasked. |
| **P2** | "Thinking steps are not fully written, some parts are omitted." | `src/thinking.rs`, `src/web/app/main.jsx` | `truncate_thinking_detail` (Rust) and `thinkingDetailText` (JS) both capped step detail at **120 chars**, clipping at the data layer before CSS ever ran. | Raise the cap to **600** in both runtimes, kept in sync with cross-referencing comments. Still bounded (ellipsis) so a runaway step can't blow up the preview height the P1 fade relies on. |
| **P3** | "sudden jump in width when message starts displaying" | `src/web/styles.css` | A leftover `.pending .message-body { width: 116px }` (an old typing-dots dimension) forced the thinking-only bubble to 116px, then it snapped to the full column when the body arrived. | Remove the clamp (and the dead `.typing::after { content: "…" }`) so the pending body uses the same natural width as a settled body — nothing to jump. |
| **P4** | "We still have theming issues in all `services` box" | `src/web/styles.css` | The #549 services/update panel shipped with light rules only; neither `:root[data-theme="dark"]` nor `@media (prefers-color-scheme: dark)` had rules for it → light-on-light text in dark mode. | Add complete dark rules for every services/update surface (`.desktop-service`, `-dot/-state/-token/-url/-actions/-start`, `.desktop-update-actions`, `-install`, `-update-state`, `.desktop-services-error`) in **both** dark layers. |
| **P5** | "Not all buttons on top menu are reacting to hover … so it is partial." | `src/web/styles.css` | Hover was written per button; controls added later (`report`, `source-code`, `download`, `memory`, `sidebar-toggle`) never got matching rules. | One shared hover + active-state hover + `:focus-visible` ring for **every** topbar control, in light *and* dark. |

---

## 4. Current-state inventory ("reuse, don't rebuild")

What already exists that the fix builds on, rather than introducing parallel systems:

* **Thinking preview component** (`src/web/app/main.jsx`) — the collapsed/expanded stack
  with `.thinking-preview-collapsed` / `.thinking-preview-previous` classes. P1 only moves
  a CSS property between two existing elements; no new component.
* **The naturalizer pair** — `src/thinking.rs` (`truncate_thinking_detail`) and
  `src/web/app/main.jsx` (`thinkingDetailText`) are the two existing truncation surfaces.
  P2 edits the constant in both; it does not add a third path.
* **Three theming layers already in the stylesheet** — light base,
  `:root[data-theme="dark"]`, and `@media (prefers-color-scheme: dark)
  :root:not([data-theme="light"])`. P4/P5 extend the *existing* layers (the dark fixes
  go into both dark layers, matching the file's convention).
* **i18n catalog** — every UI string is a `t("key")` entry; no fix introduces a
  hard-coded string (verified by the `check:web-hardcoded-ui` guard).
* **Playwright e2e harness** (`tests/e2e/`) — already a dependency; P1/P3/P4/P5 reuse it
  via a fixture-injection strategy that boots the real app so shipped CSS applies.
* **Native web-platform features** — `mask-image`, `:has()`, `:focus-visible` are all
  Baseline (see §6); no polyfill or library is added for the fix.

---

## 5. Requirements summary

Full detail with verbatim quotes and traceability in [`requirements.md`](requirements.md).

* **P1–P5** — the five reported defects. All fixed and tested (this PR).
* **M1 / M7** — "fix it in all places / uniform behavior." Honored: P2 in both runtimes;
  P4/P5 in both dark layers; audit recorded in `best-practices.md`.
* **M2** — "reuse our own react.js components." Honored: a reusable `ToolbarButton`
  component now renders all 11 topbar controls, and CSS consolidates onto the `--fa-*`
  tokens. Shared markup *and* shared treatment, by construction.
* **M3** — "fully transition to Chakra UI + JSX." **Done.** `src/web/app/main.jsx` is
  authored in JSX and bundled by the bun bundler; the app mounts under `<ChakraProvider>`,
  renders through Chakra's `chakra.*` styled factory, and is themed via the `--fa-*` →
  Chakra `semanticTokens` bridge in `theme.js`. React/ReactDOM/Chakra/Emotion are bundled
  into `app.js` (full record in §6 / `solution-plans.md`).
* **M4** — "download data → `docs/case-studies/issue-{id}` → deep analysis + online
  research." **This folder.**
* **M5** — "add debug/verbose if root cause unclear." N/A — all five root causes were
  located directly in source; noted for completeness.
* **M6** — "file upstream issues if any." Assessed — none needed; all defects are
  first-party formal-ai code (`proposed-issues.md`).
* **M8** — "everything in this single PR." Honored — [#551][pr] carries all fixes +
  tests + changelog + this case study.

---

## 6. Solution shape — the five fixes and the full Chakra/JSX migration

**The five fixes (this PR):** minimal, in-place fixes for P1–P5, each tested at the layer
where the defect lives (Rust units for P2 logic; Playwright for P1/P3/P4/P5 computed
styles and interaction).

**The strategic M2/M3 work (also this PR).** The maintainer's standing instruction was
explicit: *"My requirement is translation to Chakra, and using modern JSX, with styles in
JavaScript. Use bun bundler for building."* All three of Chakra's separable layers ship:

1. **Design-token theme system → shipped.** A `--fa-*` semantic-token palette defined once
   per theme in `styles.css` — light values in `:root{}`, each dark layer overriding only
   the values (~20 lines). Tokens cover surfaces, borders, text, accents, the
   services/update panel, and the shared control interaction set (`--fa-control-hover-*`,
   `--fa-control-active-hover-*`, `--fa-focus-ring`). This collapses the three-layer hex
   duplication that is the **P4/P5 root cause** into one source of truth, matching the
   precedent already set by `landing.css` and the `--code-*`/`--hljs-*` tokens in
   `.markdown-body`. Value-preserving (every token = exact prior hex), so the exact-RGB
   regression assertions stayed green. `src/web/app/theme.js` bridges each token 1:1 into a
   Chakra `semanticToken` (`fa.surface.card` → `var(--fa-surface-card)`), so Chakra
   components reference the same themed values — styles driven from JavaScript, styles.css
   still the single source of truth for colour.
2. **JSX authoring + reusable component model → shipped.** `src/web/app/main.jsx` is
   authored in JSX and bundled by the bun bundler. A single `ToolbarButton` component
   renders all 11 topbar controls through Chakra's `chakra.a` / `chakra.button` styled
   factory, so shared markup/classes/a11y/menu-priority are guaranteed by construction.
3. **Emotion CSS-in-JS runtime + `<ChakraProvider>` → shipped.** `main.jsx` mounts
   `createRoot(...).render(<ChakraProvider value={chakraSystem}><App /></ChakraProvider>)`;
   React, ReactDOM, `@chakra-ui/react`, and `@emotion/react` are bundled into `app.js`.

> **Retraction of the earlier "CSP-blocked" ADR.** A prior draft of this section argued
> Chakra's Emotion runtime could not ship because (a) issue [#479][issue-479]'s
> `style-src 'self'` CSP forbids Emotion's runtime `<style>` injection, and (b) `app.js`
> is served un-transpiled. **Both premises were false.** Verified: the application page
> [`src/web/app/index.html`](../../../src/web/app/index.html) carries **no CSP** — the
> `style-src 'self'` policy is only on the marketing pages (`src/web/index.html`,
> `src/web/docs/index.html`, `src/web/download/index.html`), which never load `app.js`; and
> `app.js` is **already bun-built** from `main.jsx` (`package.json` `build:web`, gated by
> `git diff --exit-code` in `.github/workflows/release.yml`). There was no architectural
> block; the migration ships in full.

**How the migration stays pixel-identical.** [`tsconfig.json`](../../../tsconfig.json) pins
bun's JSX transform to the classic runtime (`jsxFactory: h`, `h === React.createElement`),
so the JSX compiles back to the *same* `h()` calls the file used before. An AST codemod
([`experiments/codemod-h-to-jsx.mjs`](../../../experiments/codemod-h-to-jsx.mjs)) converted
the render calls, and [`experiments/verify-jsx-equivalence.mjs`](../../../experiments/verify-jsx-equivalence.mjs)
proves equivalence: it compiles `main.jsx` before/after with `bun build --packages external`,
canonicalises both through `@babel/generator {compact:true}`, and confirms the only
differences are three hand-converted, semantics-preserving residuals. `theme.js` sets
`preflight: false` and `globalCss: {}` so Chakra emits no reset/global rules, and the tree
uses `chakra.*` low-level primitives (no recipes), so computed styles are unchanged —
confirmed by the full Playwright suite (332 passed / 1 skipped) and both static guards.

### Library / feature research (M4)

| Feature | Role | Status |
|---|---|---|
| [`mask-image` + `linear-gradient`][mdn-mask] | P1 container scroll-fade. Fading a scroll container's edge is a documented standard use of gradient masks. | **Baseline** (unprefixed Chrome/Edge 120+, Firefox 53+, Safari 15.4+; `-webkit-` kept for older Safari). **Shipped.** |
| [`:has()` selector][mdn-has] | Scopes the P1 fade to "a container that *has* a previous step," so a lone step isn't masked. | **Baseline** (newly available Dec 2023, all core browsers). **Shipped.** |
| [`:focus-visible`][mdn-fv] | P5 keyboard-only focus ring without rings on mouse click — the [W3C WCAG technique C45][wcag-c45] for focus indication. | **Baseline** (since 2019). **Shipped.** |
| [Chakra UI v3 semantic tokens][chakra-tokens] | The M3 target. Token theme, component model, `chakra.*` styled factory, and the Emotion runtime are all adopted; `--fa-*` is bridged into `semanticTokens` by `theme.js`. | **Shipped** (`<ChakraProvider>` + bridge + primitives). |
| Emotion (`@emotion/react`) | Chakra's runtime CSS-in-JS engine; bundled into `app.js`. The app page has no CSP, so its runtime style injection is unconstrained. | **Shipped** (transitively via Chakra). |
| bun bundler | Compiles `main.jsx` (classic JSX runtime) → `app.js` via `build:web`. | **Shipped** (the requested build tool). |
| CSS custom properties (native) | The `--fa-*` palette: single source of colour, bridged into Chakra `semanticTokens`; removes the P4/P5 duplication root cause. | **Shipped** (`--fa-*` palette + `theme.js`). |
| Reusable JSX component | `ToolbarButton` consolidates all 11 topbar controls through `chakra.*`. | **Shipped** (`ToolbarButton`, 11 controls). |
| Playwright (already a dep) | Behavioral regression for all five defects plus the migration. | **Shipped** (`issue-1963.spec.js`; full suite 332/1). |
| Tailwind | Could also centralize tokens, but the issue names Chakra; a second system would fragment styling. | **Rejected.** |

---

## 7. Constraints & CI contracts

The fix had to satisfy several non-obvious repo contracts (each cost real debugging —
see [`best-practices.md`](best-practices.md)):

* **`semantic_grounding`** scans `src/**` and `data/seed/**` for `\b[QLP][0-9]+\b`
  Wikidata IDs. Writing the label **"P2"** in a `src/**` comment makes it demand a
  `P2.json`/`P2.lino` cache file → test fails. Source comments say **"problem 2"**, not
  "P2". (`.js`/`.css`/`tests/**`/`docs/**` are not scanned — verified at
  `tests/unit/semantic_grounding.rs:260`, roots `["data/seed", "src"]` — so the `P1–P5`
  labels in *this* case study are safe.)
* **`check:language-test-coverage`** requires that when a language-facing file changes
  (here `src/web/app.js`), the PR's added test lines cover **en, ru, hi, zh**. Satisfied
  by `tests/unit/issue_1963.rs` (the Han test doubles as proof the cap counts Unicode
  scalar values, not bytes).
* **Bundle diff gate** — `git diff --exit-code` on `vendor.bundle.js` / `ocr.bundle.js` /
  `app.js` (`.github/workflows/release.yml`). `app.js` is now a generated artifact
  (bun-built from `src/web/app/main.jsx` by `build:web`), so the source edit and the
  rebuilt `app.js` must be committed together for the gate to pass.
* **`check:web-hardcoded-ui`**, **`check:i18n`**, **`check:language-parity`**,
  **`check:intent-coverage`**, **`check:web-tdz`** — all six web guards pass.
* **Placeholder convention** — adopt the draft PR's branch; land real work; revert the
  `.gitkeep` "Initial commit" last so it ends byte-identical to `main`.

---

## 8. How this PR delivers

PR [#551][pr] on branch `issue-550-c636b0e4075d`. Commits:
`e58966dc` (fix) → `c4f1d8e9` (ru/hi/zh tests) → `5a90a4f7` (revert placeholder) →
this case study.

| # | Files | Test |
|---|---|---|
| P1 | `src/web/styles.css` (`.thinking-preview-collapsed:has(.thinking-preview-previous)` container mask; per-line mask removed) | `tests/e2e/tests/issue-1963.spec.js` — "(P1) … one continuous gradient" |
| P2 | `src/thinking.rs` (`truncate_thinking_detail`, cap 600), `src/web/app.js` (`thinkingDetailText`, cap 600), `tests/source/thinking.rs` (mirror) | `tests/unit/issue_1963.rs` (4 tests, en/ru/hi/zh) + e2e detail check |
| P3 | `src/web/styles.css` (`width: 116px` clamp + dead `.typing::after` removed) | `tests/e2e/tests/issue-1963.spec.js` — "(P3) … no 116px clamp" |
| P4 | `src/web/styles.css` (full dark rules for services/update surfaces in both dark layers) | `tests/e2e/tests/issue-1963.spec.js` — "(P4) … dark-themed in dark mode" |
| P5 | `src/web/styles.css` (shared hover + active-hover + `:focus-visible` for all topbar controls, light + dark) | `tests/e2e/tests/issue-1963.spec.js` — "(P5) … every topbar control reacts to hover" |
| test wiring | `tests/unit/mod.rs` (`mod issue_1963;`), `tests/e2e/playwright.local.config.js` (testMatch) | — |
| repro | `experiments/issue-1963-harness.html` (links the **shipped** `styles.css`) | — |
| release | `changelog.d/20260621_133148_issue_550_ui_ux_polish.md` (`bump: patch`, 5 `### Fixed` bullets → #550) | CI changelog gate |
| case study | `docs/case-studies/issue-550/**` (this folder) | — |

**ID mapping.** Product issue **#550** ⇄ tracking mirror **#1963**. The fix's test
files and changelog were authored against the mirror, so they carry `1963` in their
names/titles; the changelog fragment and this case study carry `550`. Both refer to the
same five defects and the same commits.

`git diff --stat main...HEAD` (non-doc): `src/web/styles.css` +275, `src/web/app.js`
+11, `src/thinking.rs` +9, `tests/e2e/tests/issue-1963.spec.js` +368,
`tests/unit/issue_1963.rs` +138, plus mirror test / wiring / changelog / harness.

---

## 9. Before / after evidence

Renders of every affected surface in light and dark are in
[`screenshots/before/`](screenshots/before/) and
[`screenshots/after/`](screenshots/after/). The annotated issue screenshot is
[`screenshot-main.png`](screenshot-main.png).

| Surface | Before | After |
|---|---|---|
| Collapsed thinking (P1) | `before/thinking-collapsed-{light,dark}.png` | `after/thinking-collapsed-{light,dark}.png` |
| Expanded thinking (P2) | `before/thinking-expanded-{light,dark}.png` | `after/thinking-expanded-{light,dark}.png` |
| Pending vs body (P3) | `before/pending-{light,dark}.png`, `before/pending-context-*` | `after/pending-{light,dark}.png`, `after/pending-context-*` |
| Services / update (P4) | `before/services-{light,dark}.png` | `after/services-{light,dark}.png` |
| Topbar hover (P5) | `before/topbar-{light,dark}{,-hover}.png` | `after/topbar-{light,dark}{,-hover}.png` |
| Full message (regression) | `before/message-{light,dark}.png` | `after/message-{light,dark}.png` |

---

## 10. Acceptance criteria

* ✅ **P1** — one continuous fade across the collapsed stack; lone first step unmasked.
  (`issue-1963.spec.js` P1)
* ✅ **P2** — step detail renders in full to 600 chars in both runtimes; bound preserved;
  Unicode-scalar counting verified. (`issue_1963.rs`, 4/4 pass)
* ✅ **P3** — pending body width equals settled body width (no 116px clamp).
  (`issue-1963.spec.js` P3)
* ✅ **P4** — every services/update surface is dark in dark mode, in both dark layers.
  (`issue-1963.spec.js` P4)
* ✅ **P5** — all topbar controls share hover + a `:focus-visible` ring, light + dark.
  (`issue-1963.spec.js` P5)
* ✅ **M1/M4/M6/M7/M8** — duplicates fixed in all places; this case study compiled with
  online research; upstream assessed (none); single PR.
* ✅ **M2** — reusable `ToolbarButton` component renders all 11 topbar controls through
  Chakra's `chakra.*` styled factory; CSS consolidated onto `--fa-*` tokens.
  (`demo.spec.js`, `issue-1963.spec.js`)
* ✅ **M3** — **full Chakra UI + JSX migration shipped.** `src/web/app/main.jsx` is
  authored in JSX, bundled by the bun bundler, mounts under `<ChakraProvider>`, renders via
  Chakra's `chakra.*` factory, and is themed by the `--fa-*` → `semanticTokens` bridge in
  `theme.js`; React/ReactDOM/Chakra/Emotion are bundled into `app.js`. The earlier
  "CSP-blocked" claim was retracted — the app page carries no CSP and `app.js` is already
  bun-built (see §6). Pixel-identity verified by the JSX-equivalence harness and the full
  Playwright suite (332/1).

[issue]: https://github.com/link-assistant/formal-ai/issues/550
[pr]: https://github.com/link-assistant/formal-ai/pull/551
[issue-479]: https://github.com/link-assistant/formal-ai/issues/479
[hm-issue]: https://github.com/link-assistant/hive-mind/issues/1963
[mdn-mask]: https://developer.mozilla.org/en-US/docs/Web/CSS/mask-image
[mdn-has]: https://developer.mozilla.org/en-US/docs/Web/CSS/:has
[mdn-fv]: https://developer.mozilla.org/en-US/docs/Web/CSS/:focus-visible
[wcag-c45]: https://www.w3.org/WAI/WCAG21/Techniques/css/C45
[chakra-tokens]: https://chakra-ui.com/docs/theming/semantic-tokens
