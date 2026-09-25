# Plan 16 — The js → ts → rust development cycle

Status: planned 2026-09-21 from the maintainer's instruction that day; not yet
reconciled against plan 14's wave order (it is appended after wave D because it
was opened six days into the pull request, to buy iteration speed the Rust-only
cycle cannot give). Every leaf below is delivered **in this pull request**.
Scope extended 2026-09-24 (below): the cycle is one face of full three-root
parity, not a browser-worker convenience.

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

## The scope extension of 2026-09-24

Quoted, not paraphrased, from the same pull request's review thread:

> Full partity between Rust, JavaScript, TypeScript for client and backend,
> and all translatable to each other via meta language.

> That is not about browser worker, JavaScript and TypeScript code must be
> fully implemented the entire backend server and all other logic, not only
> browseer.

> Make sure we have it in requirements and our development guidelines and
> architect notes. As that may speed up iteration drammatically as JavaScript
> executes faster than Rust compiles.

> We should have function or script for translation in any direction via meta
> language.

What this changes in the plan: the js and ts trees are not thin iterated
convenience copies — they are full implementation roots for the client **and
the backend server and all other logic**, so L2's translation product and L3's
dogfood loop aim at whole-system coverage, not at the browser worker subset.
The requirement side is the standing doctrine of 2026-09-24 (R992-R996,
`REQUIREMENTS.md`), which supersedes the 2026-08-04 interfacing-only-JavaScript
boundary while keeping the worker shrink ratchet in force until the parity
migration replaces the mirrored modules; the architect note is
`docs/architect-notes/2026-09-24-three-roots-full-parity-via-the-meta-language.md`,
and the any-direction dispatcher (library function plus `formal-ai translate`
subcommand, rust → meta live, every other leg naming its owing leaf) is
delivered as `rust/src/meta_translate.rs` with
`rust/tests/unit/issue_1138_translation_tool.rs`.

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
- [x] **L6 — dependency policy wired to the cycle.** When the translator or
  the layer check needs a feature a self-maintained dependency has not
  released, the workaround is `[patch]`-style source installs, and a separate
  GitHub issue per dependency lists every patch applied and what blocks
  using the latest release. Issues filed for dependencies are written
  generally (not Formal-AI-specific) so the general case moves into the
  dependency and Formal AI's own code shrinks. CONTRIBUTING carries the
  policy. As of 2026-09-21 the manifest carries no `[patch]` or source-install
  entry, so there is not yet a first issue to link; the gate below forces the
  link the moment one lands, and this plan will carry it.
  Test: `tests/unit/ci_cd/issue_1138_dependency_patches.rs` — every
  `[patch]`/source-install entry in `Cargo.toml` has an open issue referenced
  by URL in a comment beside it, and every referenced issue exists.

### Design note 2026-09-25 — L2's material legs (written before any L2 code)

Survey the same day, so the leaf is planned against measured facts and not
against hope:

- The crate holds no JavaScript parser. The worker mirrors and the
  module-list machinery read `./js` as text; nothing tokenizes it. The pivot
  the cycle leans on — the self-AST census — projects Rust at
  `fidelity signature` (symbol names and line spans), which is a table of
  contents, not a CST.
- `./js` today is 63 ES-module files of the plain web subset (no TS-only
  syntax, no JSX); `./ts` is a README placeholder. `release.yml` and
  `coverage.yml` carry no path predicates at all, and no per-layer job
  exists for L4 to filter — L4 stays ordered after L2/L3 exactly as listed.

The syntactic fact that shapes the first rung: TypeScript is a syntactic
superset of the committed `./js` corpus. That makes the two directions
asymmetric, and the translator must say so rather than flatten them:

- `js → ts` is a token-faithful carry. Zero refused constructs here is a
  property of the corpus, not a promise — the report still enumerates every
  construct class it carried, and the first TS-incompatible construct that
  lands in `./js` is refused by name.
- `ts → js` is the real translation: type annotations, interfaces, type
  aliases, generics, enums, `as`/`satisfies` either strip under a rule or
  are refused; nothing is guessed.

Engine/data split, per the founding doctrine: the tokenizer and token-tree
normalizer are engine code — one general ES lexer serving both roots — while
the projection (which token class each construct maps to in each target, and
which are refused) lives as per-language projection rules in the
meta-language seed, discovered from the Rust↔worker parity corpus, never
hard-coded per test. The CST diff is a comparison of normalized token trees,
so "the translation landed perfectly" is a checkable statement.

Leaf split for implementation, each landing with its named test failing
first, per repo convention:

- **L2a** — the ES tokenizer and token tree.
  `tests/unit/issue_1138_js_tokenizer.rs`: the committed `./js` corpus
  tokenizes with balanced structure; regex-versus-division and
  template-literal edges pinned by fixtures.
- **L2b** — the js → meta extractor (token tree into a pivot document),
  extending the census vocabulary from `fidelity signature` toward
  `fidelity token_tree` for the JS root.
- **L2c** — the meta → ts renderer plus the first seed projection rules.
- **L2d** — the ts → meta extractor: type-construct recognition with the
  refused list populated from real files, not from imagination.
- **L2e** — the meta → js renderer (strips exactly what the seed rules say).
- **L2f** — the `TranslationReport` (translated, refused, CST diff) and the
  L2 acceptance test: js → ts → js over the committed `./js` tree reports
  zero refused constructs and a null CST diff on the identity round trip.
- **L2g** — `--write` mode and the agent `translate` tool, so the dogfood
  loop of L3 has something to turn.

### Design note 2026-09-25 — the L2b–L2f batch (written before the code)

L2a landed (`rust/src/es_tokenizer.rs`, the engine lexer). The rest of L2
is one batch because the legs are mutually defining — the renderer cannot
be tested without the extractor's document, and the acceptance test needs
both renderers. Decisions fixed before any of it is written:

- **The pivot document is a lino `token_tree` document** rendered from the
  L2a token tree: `target`, `language` (javascript/typescript), `engine
  es_tokenizer`, `fidelity token_tree`, `token_count`, then the tree itself
  (`leaf <kind> "<text>"`, `group <delim>`, `template` with `chunk` and
  `interpolation` children, indentation = nesting). Values are
  double-quoted with the `sanitize_lino_value` escape set (`\` `"` newline
  tab CR), so the shared `seed::parser` reads them back exactly; the
  extractor writes, a pivot parser reads, and `js → meta` and `meta → js`
  are inverse by construction.
- **CST equality is token-tree equality, spans ignored.** Spans name bytes
  of one text; a rendered target is a different text. A round trip is
  perfect exactly when kind, text, and shape survive.
- **The renderers emit canonical spacing** — leaf tokens joined by single
  spaces, group delimiters spaced, template chunks verbatim between
  backticks. Because the tokenizer's regex/division decision is a pure
  function of the previous significant token, re-tokenizing canonically
  spaced output repeats every decision the original made; no token can
  merge because every pair is separated. Rendered code is not promised to
  be *runnable* (ASI is not a token-tree fact); it is promised to
  re-tokenize to the same tree.
- **Projection rules live in the seed, not the engine.**
  `data/seed/language-projection.lino` carries, per token class, which
  targets carry it (`carries_to js|ts`), and per recognizable
  TypeScript-only construct, a signature (a short leaf/group sequence)
  with `refuses_to js`. The engine is a generic matcher over whatever the
  seed declares — `projection_rules_from(lino_text)` is exposed so a test
  can prove the behavior moves with the data.
- **Refusal is total, not partial.** A refused construct or token class
  produces *no* target, and the report names every refused construct;
  dropping just the refused tokens would silently change the tree. The
  `translate` CLI reports refusals through a new `translate_refused`
  response intent, grounded like the other nine.
- **Signature recognition is conservative and documented.** The seed's
  signatures match only sequences that cannot be valid plain JavaScript
  (`interface`/`enum` declarations, `type X =` aliases, `x satisfies T`).
  Two candidates are deliberately excluded and the seed says why: generic
  parameter lists (`a < b > c` is legal JavaScript) and `as` casts
  (`export { x as y }` is plain ES module syntax, and the committed js
  corpus uses it); they carry silently until a fidelity raise or a real
  `./ts` corpus owes the recognizer.
- **Live legs after the batch:** js↔meta, ts↔meta, js→ts, ts→js. Rust legs
  stay pending (L3/L5) exactly as before; only the L2 rows of the leg
  table change.

### Design note 2026-09-25 — the shipped crate carries its own data (L1 remediation)

`cargo package` failed on the first green tip after L1 (Build Package in
`release.yml`). Root cause, measured before the fix was chosen: **230
distinct `include_str!` targets in `rust/src` (2.3 MiB) live outside the
package root** (`data/seed`, `data/meta`, `js/worker`, one script), read
through `../`-escaping literals. A package cannot pack or read files
outside its root: `cargo package` verification extracts the `.crate` as
the package root and the escaping literals then point one level above it,
so the lib cannot compile from the archive — and this crate is published
to crates.io, where the `.crate` is all a consumer ever gets. The
manifest's `include = ["../data/**", ...]` rows could never fix this:
cargo packs those files at `data/**` *inside* the extracted root, while
the sources look for them one level above it; no single literal can
resolve in both layouts.

Scope was measured, not assumed: verification compiles **lib + bins
only** (proved with a probe crate whose broken test/example includes
package cleanly), and the manifest's `include` list already excludes
`tests/` and `examples/` from the archive — so only `rust/src`'s 230
targets are packaging-relevant. Test and example includes that read
`docs/`, `vscode/`, or `.github/` are repository self-audits and keep
reading the repository, which is the honest division: the library is
shipped, the audits are not.

Fix (the mirror pattern this repo already practices in both directions —
`data/meta/self-ast/` mirrors `rust/src` for audits, `js/seed-files.js`
embeds 118 seeds for the browser):

- `rust/embedded/` is a committed byte mirror of every repo file a
  `rust/src` `include_str!` names, at its repository-relative subpath
  (`rust/embedded/data/seed/...`, `rust/embedded/js/worker/...`) — 231
  files, 2.3 MiB, of which 167 are seeds.
  `scripts/mirror-package-data.rs` regenerates it (`--write`) and gates
  it (`--check`): byte-equality against the source files, no
  unreferenced mirror files, and — the invariant that keeps the class
  from returning — **no `include_str!` in `rust/src` resolving outside
  the package root**. The step runs in
  `scripts/regenerate-derived-artifacts.sh`, so the existing
  derived-artifacts `--check` CI catches a stale mirror.
- The 254 include sites across 66 source files are rewritten once,
  mechanically, to plain relative paths into `rust/embedded/` — identical
  bytes, identical behavior, resolvable in-repo and in-archive.
- The manifest's dead `../`-escaping include rows are replaced by
  `/embedded/**`.

Sized against the limits before landing: the `.crate` grows by the
mirror's ~2.3 MiB raw (lino text compresses ~4×) on top of today's
2.1 MiB compressed, far under the 10 MiB crates.io ceiling that
`scripts/check-crate-package-size.rs` enforces.


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
