# Plan 16 — The js → ts → rust development cycle

Status: planned 2026-09-21 from the maintainer's instruction that day; not yet
reconciled against plan 14's wave order (it is appended after wave D because it
was opened six days into the pull request, to buy iteration speed the Rust-only
cycle cannot give). Every leaf below is delivered **in this pull request**.

## The instruction this plan answers

Quoted, not paraphrased (2026-09-21):

> We also can try to make sure our meta-language <-> rust and meta-language
> <-> JavaScript/TypeScript conversions are fully functional, if so, we can
> have in addition to Rust source code the JavaScript/TypeScript source code,
> which we can iterate first, so for example, we can start by changing only
> JavaScript code, after that we translate it to TypeScript, and apply changes
> programmatically, as our Formal AI should have not only ability to do rules
> based language translation itself, but also we should have a callable tool
> option/command for that, and also a function as library to call. So, we start
> by iterating quikly with JavaScript, translate it to TypeScript using our
> Formal AI system, check that translation landed perfectly, if something
> wrong - fix it (so at each pull request, we directly forcing our development
> cycle to care to improve our own translation between languages, making meta
> language and all our dependencies even better, yet it also allow us to
> iterate faster). So CI/CD for js is only executed when we have changes in
> root of repository ./js folder, and for ts is ./ts, and for rust is ./rust,
> that means once JavaScript version stabalized, we don't reexecute it CI/CD,
> if we have got success on ./js folder on last commit with ./js changes, the
> same for ./ts and the same for ./rust. So we go gradually from js -> ts ->
> rust. JavaScript will give us fast iteration and easy running of selected
> tests while we iterating, ts will give us a better formalization, and rust
> will give us maximum formalization with maximun runtime performance. This
> vision must be reflected in all our related docs, and be fully delivered in
> this pull request. As otherwise it may take ages to complete with current
> speed, we doing this pull request already for 6 days. Also as we do support
> meta-language as full CST-like intermediate language, we actully can test in
> CI/CD that we can get generated typescript and javascript back from Rust, so
> we always check for round trip translations, but we should be able to switch
> between languages as needed during coding process. Also make sure we fail on
> ts only if all js checks pass, and fail on rust only if all js and ts checks
> pass, so we always enforce that development cycle in using our CI/CD for
> pull request.

Plus the two dependency directives of the same day, which this plan carries
because the translation cycle is their first consumer:

> If you see missing features or released versions of our software in Rust,
> you can use as a temporary workaround installing from source code, and if
> needed you may apply patches to that code. That is for all dependencies that
> we maintain ourselves, and for each such dependency we need to create
> separate issue so we list all patches we applied, and any other issues, that
> making us unable to use latest released package (if any).

> We also need to make sure that in all our dependencies we create issues that
> are not specific to Formal AI only, so we can simplify code of Formal AI,
> moving most of general cases to our maintained dependencies.

## Root cause this plan attacks

The pull request iterates through the Rust compiler even for changes whose
semantics a script runtime could check in milliseconds, and the meta-language
already carries both a Rust and a browser-worker projection
(`mirror_runtimes` construction stage) without either direction being a
callable product. The fix is not another mirror convention: it is making the
meta-language the CST-style pivot with **generated, committed, verified**
JavaScript and TypeScript trees, and a CI structure that only re-checks the
layer that changed.

## Leaves

Each leaf follows the repo convention: named test drafted failing first, then
implemented, box ticked in the landing commit.

- [ ] **L1 — repository layout.** `./js`, `./ts`, `./rust` as the three
  source roots. `./rust` holds what is today the crate root (`Cargo.toml`,
  `src/`, `tests/`, `benches/`, `examples/`); `./js` and `./ts` hold the
  generated trees plus the hand-iterated files the cycle starts from (today's
  `src/web/` worker mirrors and `seed_loader` move under `./js` as the first
  citizens). A top-level map document states which tree is authoritative for
  which module while the cycle is partial. Docs: VISION, ROADMAP,
  CONTRIBUTING, README.
  Test: `tests/unit/issue_1138_three_source_roots.rs` — layout, no path
  outside its root claims to be a source of any layer, and every existing
  reference to `src/web` resolves inside `./js`.
- [ ] **L2 — translation as a product.** One library entry point
  (`translate(source_tree, target_language) -> TranslationReport`), one CLI
  command (`formal-ai translate --from js --to ts [--write]`), and one agent
  tool (`translate`) so the agent CLI can call it mid-session. The report
  lists every construct it translated, every construct it refused, and the
  CST diff between source and rendered target, so "the translation landed
  perfectly" is a checkable statement, not a hope. Rules live in the
  meta-language seed (per-language projection rules), discovered from the
  existing Rust↔worker parity corpus — nothing hard-coded per test.
  Test: `tests/unit/issue_1138_translation_tool.rs` — js→ts and ts→js over
  the committed `./js` tree report zero refused constructs and a null CST
  diff on the identity round trip.
- [ ] **L3 — the dogfood loop.** `./js` is iterated by hand and by agent;
  `formal-ai translate --from js --to ts --write` regenerates `./ts`; a
  mismatch between generated and committed `./ts` is a failing check, and the
  fix is either the source or the translator — never a hand-edit of the
  generated tree. Every translation failure the loop hits becomes a seed rule
  or a translator test in the same commit (the instruction's "forcing our
  development cycle to improve our own translation").
  Test: `tests/unit/issue_1138_dogfood_translation.rs` — after translating
  `./js` to `./ts`, the trees match; a deliberately corrupted rule fails with
  the refused construct named.
- [ ] **L4 — path-filtered CI with carry-forward and cycle enforcement.**
  Workflows gain folder predicates: the js checks run only when `./js`
  changed, ts only when `./ts` changed, rust only when `./rust` (plus
  workflow/config) changed. Each layer's checks reuse the **last green run of
  that layer** when its folder is untouched, so a stabilized layer costs
  nothing. Enforcement order is structural: the ts job is a dependent job
  that reports failure only after the js checks for the same commit are green
  (js red ⇒ ts reports skipped-by-js, not its own red), and the rust job
  depends on js+ts the same way. Failures of docs/CI-audit jobs that span all
  layers stay in a shared job keyed on the union of the three paths plus
  `docs/`, `data/`, `.github/`.
  Test: `tests/unit/ci_cd/issue_1138_layered_ci.rs` — workflow files parse;
  every job declares the intended path filter; the dependency graph proves
  the js→ts→rust gating order; no layer's check runs when only another
  layer's folder changed.
- [ ] **L5 — round-trip verification from Rust.** The meta-language is a full
  CST-like intermediate language, so the CI proves it: Rust modules →
  meta-language (the self-AST extraction that already feeds
  `data/meta/self-ast/`) → generated TypeScript and JavaScript → back to
  meta-language → compared CST-equal to the original. Committed `./ts` and
  `./js` trees are checked against this projection so drift is a red check,
  and a developer can switch between the three languages mid-change knowing
  the pivot, not a copy-paste, carries the change.
  Test: `tests/unit/issue_1138_round_trip_projection.rs` — for every Rust
  module with a declared projection, lino→ts→lino and lino→js→lino round
  trips are CST-identical.
- [ ] **L6 — dependency policy wired to the cycle.** When the translator or
  the layer check needs a feature a self-maintained dependency has not
  released, the workaround is `[patch]`-style source installs, and a separate
  GitHub issue per dependency lists every patch applied and what blocks
  using the latest release. Issues filed for dependencies are written
  generally (not Formal-AI-specific) so the general case moves into the
  dependency and Formal AI's own code shrinks. CONTRIBUTING carries the
  policy; the first such issue is linked from this plan.
  Test: `tests/unit/ci_cd/issue_1138_dependency_patches.rs` — every
  `[patch]`/source-install entry in `Cargo.toml` has an open issue referenced
  by URL in a comment beside it, and every referenced issue exists.

## Risks

1. **The restructure churns every path.** Mitigation: L1 lands as one commit
   with `git mv` history preserved, and CI keeps passing at each leaf (the
   branch is red-zero-tolerance before L1 starts — the CI-red fix batch
   lands first).
2. **Generated trees reviewed as if hand-written.** Mitigation: generated
   files carry a first-line marker and the audit gates
   (`source_test_placement`, warning-band, CodeQL) learn to attribute them to
   the generating layer, not to hand-authorship.
3. **Carry-forward hides a layer broken by a shared runtime change** (for
   example a lino seed consumed by all three). Mitigation: L4's shared job
   keyed on the union path set; the seed files live in `data/` which is in
   every layer's filter.
4. **Translation perfectionism stalls the cycle.** Mitigation: the translator
   refuses constructs rather than guessing; refused constructs are tracked as
   issues, and the js tree avoids them until a rule exists (honest gaps, no
   fake coverage — same doctrine as capability routing).
