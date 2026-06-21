# Best Practices & Lessons Learned — Issue #550

What this case study teaches, framed so the *class* of defect cannot recur — not just
the five instances.

## 1. The systemic root cause: no design tokens

Four of the five defects (P1, P3, P4, P5) live in `src/web/styles.css`, and the single
fact that connects P4 and P5 is that **the stylesheet has zero CSS custom properties.**
Light and dark are *manually duplicated hex* across three layers (light base,
`:root[data-theme="dark"]`, `@media (prefers-color-scheme: dark)`), and every
interactive treatment is written per element.

Consequences observed:
* **P4** — a new surface (the #549 services/update panel) was added with light rules
  only; nobody hand-duplicated the dark rules, so it rendered light-on-dark.
* **P5** — new topbar buttons were added without copying the older buttons' hover rules,
  so feedback was partial.

> **Resolution (shipped in this PR).** A `--fa-*` semantic-token palette is now defined
> once per theme in `styles.css`: light values in `:root{}`; each dark layer overrides
> **only the values**. Tokens cover surfaces (`--fa-surface-card`, `--fa-surface-raised`),
> borders, text (`--fa-text-body`, `--fa-text-muted`, `--fa-text-strong`), accents, the
> services/update panel, and the shared control interaction set
> (`--fa-control-hover-*`, `--fa-control-active-hover-*`, `--fa-focus-ring`). A new element
> now *inherits* correct theming by consuming a token instead of re-deriving hex, which
> removes the P4/P5 root cause. This matches the precedent already set by `landing.css` and
> by the `--code-*`/`--hljs-*` tokens in `.markdown-body`. It is also the CSP-safe substrate
> a future Chakra theme maps onto (`solution-plans.md` M2/M3 ADR). The change is
> value-preserving — every token equals the exact prior hex — so the exact-RGB regression
> assertions in `issue-1963.spec.js` stayed green.

## 2. "Fix it in all places" requires knowing where the places are

The issue's M1/M7 ("if the issue is one place it should be fixed in all places") only
works if you can enumerate the duplication. Here that meant:
* P2 lives in **two runtimes** — the Rust core (`src/thinking.rs`) and the JS worker
  (`src/web/app.js`). Both were changed and cross-referenced with a "keep both constants
  in sync" comment.
* P4/P5 live in **two dark layers** — the explicit `data-theme` override and the
  `prefers-color-scheme` fallback. Both were changed.

> **Recommendation.** When a value is intentionally duplicated across runtimes or theme
> layers, leave a comment at each site pointing at the others, so the next editor changes
> all of them. Better: collapse the duplication (single token / single shared constant)
> so there is only one place.

## 3. Reuse selectors before reaching for new components

P5's fix grouped every topbar control under one selector list instead of adding yet
another per-button rule — the CSS-level expression of "reuse our own components" (M2):
one rule, uniform behavior, no drift. This PR then took the component-level step too: a
single `ToolbarButton` (`src/web/app.js`) now renders all 11 topbar controls, so the
shared markup/classes/a11y attributes are guaranteed by construction — a new control
cannot be added with the wrong shape. The component is built on raw `React.createElement`
(no JSX/Chakra runtime), so it ships under the strict CSP today.

> **Recommendation.** Consolidate at *both* layers: a shared selector removes style drift,
> and a shared component removes markup drift. Neither alone is sufficient — the selector
> can't stop a malformed button, and the component can't stop a one-off `:hover` override.

## 4. Test at the layer where the defect lives

* P2 is logic (a character cap) → **Rust unit tests** (`tests/unit/issue_1963.rs`) that
  are fast and deterministic.
* P1/P3/P4/P5 are *computed style and interaction* → **Playwright** tests
  (`issue-1963.spec.js`) that read `getComputedStyle`, real hover/focus states, and
  dark-mode colors. A unit test could not have caught "the mask is on the wrong element"
  or "this button doesn't change on hover"; only a rendered DOM can.

> **Recommendation.** Match the test medium to the defect medium. Pixel/computed-style
> regressions need a browser; pure logic does not. A test that cannot fail proves
> nothing — validate the spec fails when the fix is stashed.

## 5. Gotcha: the `semantic_grounding` scanner and "P<number>" tokens

A real trap hit during this work, worth recording so it doesn't recur:

* formal-ai's `tests/unit/semantic_grounding.rs` scans `src/**` and `data/seed/**` (roots
  `["data/seed", "src"]`) for Wikidata-style IDs via the regex `\b[QLP][0-9]+\b`. Any
  `Q`/`L`/`P` followed by digits is treated as a Wikidata entity/lexeme/property that
  **must** have a checked-in cache file (`<ID>.lino` + `<ID>.json`).
* Writing the problem label **"P2"** in a comment in `src/thinking.rs` makes the scanner
  think property **P2** is referenced → it demands `P2.json`/`P2.lino`, which don't exist
  → grounding tests fail.

> **Recommendation.** In scanned source (`src/**`, `data/seed/**`), refer to the issue's
> problems as **"problem 2"**, never **"P2"**. `.js`/`.css`/`tests/**`/`docs/**` are not
> scanned, so the bare `P<n>` label is safe there (and is used freely in *this* case
> study) — but the safest habit is to avoid the `letter+digits` shape in any first-party
> source comment.

## 6. Respect the canonical branch / placeholder convention

formal-ai auto-creates a `[WIP]` solution-draft PR per issue, on a branch whose first
commit is a `.gitkeep` placeholder ("Initial commit with task details"). The repo's
convention (seen in the merged PR for the predecessor issue) is: *placeholder commit →
real work → "Revert 'Initial commit with task details'" → merge*.

> **Recommendation.** Adopt the existing draft PR and its branch instead of opening a
> second PR (one PR per issue); add the real work, then revert the placeholder so the
> `.gitkeep` ends up byte-identical to `main`. Don't leave orphan branches. **When a
> parallel solver shares the branch, re-fetch before every push and never force-push** —
> reconcile by adopting the canonical remote, not by overwriting it.

## 7. Don't perturb generated artifacts

CI runs `git diff --exit-code` on the committed `vendor.bundle.js` / `ocr.bundle.js`. The
fixes deliberately touched only hand-written `app.js` / `styles.css` / Rust and left the
bundle *entry* files alone, so a rebuild produced no bundle diff.

> **Recommendation.** Before committing web changes, rebuild and confirm the generated
> bundles are unchanged (or regenerate and commit them intentionally) — never let an
> incidental bundle diff ride along with a logic fix.

## 8. Make the repro production-faithful

`experiments/issue-1963-harness.html` links the **shipped** `styles.css` (not a
hand-copied snippet), so the before/after renders in `screenshots/` reflect real
production CSS. A repro that forks the styles can "prove" a fix that production doesn't
actually have.

> **Recommendation.** Repro harnesses should import the real stylesheet/component, so the
> harness and production cannot drift.

## 9. Gotcha: the `language-test-coverage` gate wants *every* language

Another hidden CI contract, in the same family as #5 — caught only because the *code*
commit (not the placeholder revert) was validated in CI:

* formal-ai's `check:language-test-coverage` (run in the Lint job) diffs the PR against
  `main`; if any **language-facing** file changed (`src/web/app.js`,
  `src/web/i18n*.{js,lino}`, `src/language.rs`, `src/solver*.rs`, `data/seed/**`,
  `src/translation/**`, …), the PR's *added test lines* must cover **all** supported UI
  languages (en, ru, hi, zh), detected by language name or by script (Cyrillic → ru,
  Devanagari → hi, Han → zh).
* The P2 fix touched `src/web/app.js` (the `thinkingDetailText` cap), so an English-only
  Rust test would trip the gate: *"Missing: ru, hi, zh."*

> **Recommendation.** When a fix touches language-facing code, add regression tests in
> *every* supported language — and make them meaningful, not box-ticking. Here the
> multilingual tests double as a real check that the 600-char cap counts Unicode scalar
> values, not bytes (a Han detail of 208 chars / 624 bytes must survive whole). One gate,
> two correctness wins.

> **Meta-lesson.** Validate the *code* commit in CI, not only the `.gitkeep` revert. A
> gate that fires on a language-facing diff is invisible when the head being tested is the
> revert (which touches only `.gitkeep`); push the code commit as head first, let CI
> exercise it, then add the revert last (see #6).
