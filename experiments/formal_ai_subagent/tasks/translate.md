TASK (tag TRANSLATE): move the JS → Rust translation via meta-language toward full automation, and report upstream what blocks it.

Facts (2026-10-08):
- `scripts/translate-js-rust.mjs` drives `selfTranslate` from link-foundation/meta-language main, pinned at `679a3b3c` (PR #196, merged but unreleased).
- Scope is 128 modules (`js/agentic/crate/*.mjs` plus the meta reasoner). 249 of 2367 items translate, 2118 are carried, and 1 module translates fully.
- The output is committed only as fixtures (`rust/tests/fixtures/js-rust-translation/`); `rust/src` is still hand-ported.
- Refusal census, from `data/meta/js-rust-translation.lino`: `import {…}` 405, JSDoc type 198, `null` 162, unknown sibling or imported name 114, arrow function 96, function value 95, object without `$` tag 77, `new` 74, `.find()` 59, `.split()` 53, regular expression 52, `.map()` 48, …

Do:
1. **Upstream issues.** In link-foundation/meta-language, read the open issues and recent PRs first (`gh issue list -R link-foundation/meta-language --state all --search …`), and do not file duplicates. File one issue per top refusal construct that has no issue yet. Each issue gives:
   - the construct;
   - its count here;
   - a minimal JS input and the Rust it should produce;
   - a link to this repository's census at the pinned commit.

   Also ask in one issue for a release that contains PR #196. Collect the URLs.
2. **Measure the wider scope.** Run the translator over the modules outside the current scope with `--why`, or by temporarily widening it in a sandbox copy: the agentic planner `js/agentic/*.mjs`, `js/server/*.mjs` and the worker. Commit only a measured report (`docs/case-studies/pull-request-1188/js-rust-translation-scope.md`) with per-root translated/carried counts and the top refusals. Do not widen the committed scope if the ratchet would record mostly carried items without a plan.
3. **Translation-friendly JS where it costs nothing.** In `js/agentic/crate/*.mjs`, rewrite constructs the portable core already supports as equivalents: for example a `for` loop for `.find()` or `.map()` where meta-language translates loops, or an explicit sentinel where it refuses `null`, but only when the JS stays idiomatic and every test passes. Then run `node scripts/translate-js-rust.mjs --write --fetch <module>`; the translated counts may only rise.
4. **Rows.** Update the R994, R1000 and R992 notes with the new numbers and the upstream issue links, honestly: the loop stays partial until generated Rust replaces the hand twins.

Rules: never run cargo or any script that calls it. `--compile` (rustc on projections) is a Rust build, so leave it to CI. Disk is under 7 GB, so delete the fetched upstream tarball afterwards. Claims go in claims.md. Ledger rows T190–T199.
