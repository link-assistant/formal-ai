# Requirements — Issue #550

Every requirement extracted from [formal-ai#550][issue] (mirrored as
[hive-mind#1963][hm-issue]), traced to its fix and verification. Requirement IDs
(P1–P5, M1–M8) are this case study's own labels.

## A. The five reported defects

| ID | Verbatim requirement | Fix | Verified by |
|---|---|---|---|
| **P1** | "Animation gradient does not span 2 paragraphs/steps of thinking. No it looks broken, because each paragraph/line/step has its own gradient. The gradient is to create a feeling that there is more content, so it should be applied to full scrolled container of all thinking steps." | Move the `mask-image` fade from the per-line `.thinking-preview-previous` to the container `.thinking-preview-collapsed:has(.thinking-preview-previous)` as one `linear-gradient(to bottom, transparent 0, #000 1.4em)`; drop the per-line mask. | `tests/e2e/tests/issue-1963.spec.js` — P1 describe (container masks across both steps; lone step never masked) |
| **P2** | "Thinking steps are not fully written, some parts are omitted." | Raise the detail cap 120 → 600 in `truncate_thinking_detail` (`src/thinking.rs`) and `thinkingDetailText` (`src/web/app.js`), kept in sync. | `tests/unit/issue_1963.rs` (4 tests, en/ru/hi/zh) + e2e detail check |
| **P3** | "When where is no yet message body and only thinking steps we the width of message broken, that causes bad feeling, as UI have sudden jump in width when message starts displaying." | Remove the `.pending .message-body { width: 116px }` clamp (and the dead `.typing::after`) so the pending body uses the natural settled width. | `tests/e2e/tests/issue-1963.spec.js` — P3 describe (pending body matches settled width, no 116px) |
| **P4** | "We still have theming issues in all `services` box" | Add complete dark rules for every services/update surface in both `:root[data-theme="dark"]` and `@media (prefers-color-scheme: dark)`. | `tests/e2e/tests/issue-1963.spec.js` — P4 describe (service cards, action buttons, update state go dark) |
| **P5** | "Not all buttons on top menu are reacting to hover as other buttons, so it is partial." | One shared hover + active-hover + `:focus-visible` ring for every topbar control, light and dark. | `tests/e2e/tests/issue-1963.spec.js` — P5 describe (each control adopts shared hover; focus ring) |

## B. Meta-requirements (codebase-wide & strategic)

| ID | Verbatim requirement | Disposition |
|---|---|---|
| **M1** | "double check all the UI issues (even out of scope of that I described), if the issue is one place it should be fixed in all places. So all our UI are have uniform behavior" | **Honored.** P2 fixed in both Rust + JS; P4/P5 fixed in both dark layers. The systemic duplication root cause is recorded in `best-practices.md`. |
| **M2** | "we should reuse our own react.js components to easily support everything" | **Honored.** A reusable `ToolbarButton` component (authored in `src/web/app/main.jsx`, bun-built into `src/web/app.js`) now renders all 11 topbar controls (source-code/download/report/memory/diagnostics/mode/mode-toggle/mobile-menu/sidebar-toggle) through Chakra's `chakra.*` styled factory, so the shared treatment is enforced by markup, not copied per element. CSS-side, P4/P5 also consolidate onto the `--fa-*` token system. |
| **M3** | "must fully transition to https://chakra-ui.com and JSX, so we can ensure everything is nice and polished" | **Done — full Chakra UI + JSX migration shipped in this PR.** `src/web/app/main.jsx` is authored in modern JSX and bundled by the **bun bundler** (`build:web`); the app mounts under `<ChakraProvider value={chakraSystem}>`, renders through Chakra's `chakra.*` styled factory, and is themed by the `--fa-*` → Chakra `semanticTokens` bridge in `src/web/app/theme.js` (styles in JavaScript). React, ReactDOM, `@chakra-ui/react`, and `@emotion/react` are bundled into `app.js`. An earlier draft claimed the Emotion runtime was "CSP-blocked" by #479 and that `app.js` was "served un-transpiled"; **both premises were false** — the application page `src/web/app/index.html` carries no CSP (the `style-src 'self'` policy is only on the marketing pages, which never load `app.js`), and `app.js` is already bun-built. Pixel-identity verified via the JSX-equivalence harness + the full Playwright suite (332/1), and the migration is pinned end-to-end by `tests/e2e/tests/issue-550-chakra-migration.spec.js` (React mount + live Emotion runtime + `chakra.*` ToolbarButton). ADR in `solution-plans.md` / README §6. |
| **M4** | "download all logs and data related about the issue to this repository … compile that data to `./docs/case-studies/issue-{id}` folder, and use it to do deep case study analysis (also … search online for additional facts and data) … reconstruct timeline/sequence of events, list of each and all requirements … find root causes of the each problem, and propose possible solutions and solution plans for each requirement … check known existing components/libraries." | **Done** — this `docs/case-studies/issue-550/` folder (README timeline + root causes, this file, `solution-plans.md`, `proposed-issues.md`, `best-practices.md`, `raw-data/`, `screenshots/`). Online research cited in README §6. |
| **M5** | "If there is not enough data to find actual root cause, add debug output and verbose mode if not present, that will allow us to find root cause on next iteration." | **N/A this iteration** — all five root causes were located directly in the stylesheet / naturalizer source (README §3); no additional instrumentation was required. Noted for completeness. |
| **M6** | "If issue related to any other repository/project, where we can report issues on GitHub, please do so. Each issue must contain reproducible examples, workarounds and suggestions for fix the issue in code." | **Assessed — none needed.** Every defect is in formal-ai's own first-party code (its stylesheet and naturalizer), not a dependency. Reasoning in `proposed-issues.md`. The product tracker is `formal-ai#550`; the fix is `formal-ai#551`. |
| **M7** | "double check to fully apply requirements to entire codebase, so if we have issue in multiple places, it should be fixed in all them." | **Honored** — same as M1; see the "fix it in all places" section of `best-practices.md` (P2 across two runtimes; P4/P5 across two dark layers). |
| **M8** | "Please plan and execute everything in this single pull request … until it is each and every requirement fully addressed, and everything is totally done." | **Honored** — one PR (`formal-ai#551`) carries all five fixes + tests + changelog + this case study. |

## C. Traceability — requirement → files touched

| Requirement | Source files | Test files |
|---|---|---|
| P1 | `src/web/styles.css` | `tests/e2e/tests/issue-1963.spec.js` |
| P2 | `src/thinking.rs`, `src/web/app.js` (+ mirror `tests/source/thinking.rs`) | `tests/unit/issue_1963.rs`, `tests/unit/mod.rs` |
| P3 | `src/web/styles.css` | `tests/e2e/tests/issue-1963.spec.js` |
| P4 | `src/web/styles.css` (`--fa-*` tokens) | `tests/e2e/tests/issue-1963.spec.js` |
| P5 | `src/web/styles.css` (`--fa-*` tokens) | `tests/e2e/tests/issue-1963.spec.js` |
| M2 | `src/web/app/main.jsx` (`ToolbarButton`) → `src/web/app.js` | `tests/e2e/tests/demo.spec.js`, `issue-1963.spec.js` |
| M3 | `src/web/app/main.jsx` (JSX, `<ChakraProvider>`, `chakra.*`), `src/web/app/theme.js` (`--fa-*`→`semanticTokens` bridge), `tsconfig.json` (bun JSX), `package.json`/`release.yml` (`build:web` + diff gate), `src/web/styles.css` (`--fa-*` tokens) → `src/web/app.js` | `tests/e2e/tests/issue-550-chakra-migration.spec.js` (React mounts from the bun-built bundle, Emotion runtime live, ToolbarButton renders via `chakra.*`) + full Playwright suite (332/1) + `experiments/verify-jsx-equivalence.mjs` |
| M4 | `docs/case-studies/issue-550/**` (this folder) | — |
| release | `changelog.d/20260621_133148_issue_550_ui_ux_polish.md` | CI changelog gate |

> **Note on test-file IDs.** The shipped test files are named `issue_1963.rs` /
> `issue-1963.spec.js` and their `describe` titles say "Issue #1963" because the
> canonical fix was authored against the tracking mirror. They verify exactly the P1–P5
> defects of `#550`. See README §8 for the full ID mapping.

[issue]: https://github.com/link-assistant/formal-ai/issues/550
[hm-issue]: https://github.com/link-assistant/hive-mind/issues/1963
