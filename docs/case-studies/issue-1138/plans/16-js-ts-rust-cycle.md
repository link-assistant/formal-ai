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
- [x] **L3 — the dogfood loop.** `./js` is iterated by hand and by agent;
  `formal-ai translate --from js --to ts --write` regenerates `./ts`; a
  mismatch between generated and committed `./ts` is a failing check, and the
  fix is either the source or the translator — never a hand-edit of the
  generated tree. Every translation failure the loop hits becomes a seed rule
  or a translator test in the same commit (the instruction's "forcing our
  development cycle to improve our own translation").
  Test: `tests/unit/issue_1138_dogfood_translation.rs` — after translating
  `./js` to `./ts`, the trees match; a deliberately corrupted rule fails with
  the refused construct named.
  First pass 2026-09-25: the whole committed corpus carried — 63 files,
  427,917 tokens, zero refusals (it is the plain web subset, and TypeScript
  is its syntactic superset, exactly the asymmetry L2's rules state). The
  committed `./ts` tree is the generated output; the check regenerates it
  byte-for-byte and proves the renderers follow the seed by corrupting the
  identifier class's ts carry and watching the refusal name
  `token_class identifier`.
- [x] **L4 — path-filtered CI with carry-forward and cycle enforcement.**
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
  Landed 2026-09-25 as `.github/workflows/layered-ci.yml` (four jobs:
  `changes`, `js`, `ts`, `rust`) with the tier predicates in
  `scripts/detect-code-changes.rs` (`js-changed`/`ts-changed`/`rust-changed`,
  cumulative downward, never upward — see the design note above). The js
  tier's `node --check` was measured green over the whole 63-file corpus
  before the leaf landed; the ts tier holds the js↔ts file-set parity; the
  rust tier runs the L3 dogfood gate in CI (delete `ts/**/*.ts`, regenerate,
  `git diff --exit-code -- ts/`). The shared tier stayed where it already
  lives — `release.yml` and the other spanning audits run ungated on every
  pull request synchronize, which the test pins so it cannot silently
  narrow. The same leaf repaired the L1 casualty found during the survey:
  the detector's `agentic_routing_changed` still matched the pre-L1
  `src/agentic_coding/` path, so the issue #1137 full four-client replay had
  been unreachable on pull requests since the restructure.
- [x] **L5 — round-trip verification from Rust.** The meta-language is a full
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
  Landed 2026-09-25 at the fidelity each quadrant owns (see the design note
  below): the corpus round trips run over every committed js and ts source
  through the pivot document medium, the declared-projection registry is
  `root_projection` policy rows in the projection seed with membership
  derived at check time, and the stale pending-leg pointers re-point to L7,
  which now owns the whole rust rendering quadrant.
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
- [x] **L7 — Rust rendering from the pivot.** The quadrant L5's survey
  measured as missing, answering the instruction's "we can test in CI/CD
  that we can get generated typescript and javascript back from Rust":
  every leg whose target is Rust source (meta → rust, js → rust, ts →
  rust), and the Rust → js/ts renders that compose through the pivot. The
  ladder, recorded so the next survey does not re-measure: raise the Rust
  leg's committed meta fidelity from `signature` (today's census) to
  `token_tree` — the meta-language engine already parses Rust losslessly
  (`reconstruct_source` is byte-identical), so the CST exists in memory and
  only the committed rendering is missing — then declare the grammar
  projection rules (rust meta kinds → es token classes) in the projection
  seed the way L2 declared the ES token classes, then the renderers, each
  refusing rather than guessing.
  Test: `tests/unit/issue_1138_rust_projection.rs` — for every owned Rust
  module, rust → meta at `token_tree` fidelity round-trips, and the
  ×Rust / Rust→ES legs it opens are CST-identical.
  Landed 2026-09-25, re-scoped by the design note below the way L5 was:
  the survey found the ladder's first rung is not `token_tree` but the
  lossless *network* serialization the engine already ships
  (`to_lino`/`from_lino` plus `reconstruct_text`), so the leaf lands the
  meta → rust leg live at `network` fidelity — every owned module
  round-trips byte-identically through the public translate surface, a
  runtime-verified check rather than a committed per-module tree, and a
  document in any other lino dialect is refused with the dialect named.
  The grammar-projection legs (rust → js/ts, js/ts → rust) need rule
  authoring, not wiring, and move to the new L8 leaf.
- [ ] **L8 — grammar projection rules.** The remainder of L7's quadrant:
  every leg that renders the source text of another grammar — rust → js,
  rust → ts, js → rust, ts → rust — through the translation-rule engine
  the meta-language dependency already ships (`TranslationRuleSet`: named
  rules, each a link query plus per-target templates, serialized as lino).
  The engine belongs to the dependency (the L6 policy); the rules are seed
  data here, discovered from the worker-parity corpus, and until a
  construct has a rule the renderer refuses it — the L2 doctrine restated
  across grammars. Opens with its own survey of the corpus's construct
  inventory; no rule row ships before that survey.
  Test: extends `tests/unit/issue_1138_rust_projection.rs` — each ruled
  construct renders, each unruled construct refuses by name, and the four
  legs flip live in the registry only as their rules land.

### Survey note 2026-09-25 — L8: the engine is measured (facts for the leaf's own survey)

- The dependency's `TranslationRuleSet` has a network-level entry point —
  `render_roots` over a parsed network for a target language, with
  `render_link` recursing per matched link — so rust → js/ts is rule
  authoring on top of the engine L7 already exercises, exactly as the
  leaf assumes.
- A `TranslationRule` is name + `LinkQuery` + reference-index captures +
  templates **keyed by target language**, so one rule set can carry both
  the `javascript` and `typescript` spellings per construct — the same
  row serves rust → js and rust → ts.
- Rule sets serialize as lino (`to_lino`/`from_lino`), so the rules are
  seed data here by construction. The leaf's open questions, measured
  2026-09-25 against the renderer source:
  - `LinkQuery` selects links by `by_type(LinkType)` plus `with_term` /
    `with_language`, or an s-expression with named captures — expressive
    enough to rule per node kind; the exact encoding of a tree-sitter
    kind on the parsed link (type vs term) is measured when the first
    rule is authored.
  - `render_roots` returns `None` only when *no* root has a template; an
    unruled link inside a ruled root falls back to `render_unclaimed`
    fallback rendering — **the dependency does not refuse unruled
    constructs**. So the leaf's refusal contract ("each unruled construct
    refuses by name") is Formal AI's own coverage pre-check: walk the
    network, require every link the roots reach to be claimed by a rule
    with a template for the target, and refuse naming the census kind
    otherwise — the es_meta carried/refused report pattern restated.
  - Template placeholders carry modes: `{name}` renders the captured link
    recursively, `{name:text}` / `{name:source}` splice captured source,
    `{name:term}` the link's term, `.` is the matched link itself;
    captures resolve from query captures by name, then reference indices.
  - `with_language_fallback` exists but is unnecessary here: each rule
    carries both the `javascript` and `typescript` templates.
- `data/seed/formal-language-projections.lino` is a different projection
  family (FOL ↔ natural-language statements, plan 04's territory); the
  L8 rules get their own seed surface, declared the way L2 declared the
  ES token classes and L5 the root rows.
- The construct inventory is already half-committed: the #673 census
  carries two tiers — 83 modules under `src/agentic_coding/` at
  `full_ast` fidelity with their node-kind histograms in the document,
  544 at `signature` — and the tree-sitter kind rides
  `metadata.term()` (the census groups by it), so an L8 rule is
  `LinkQuery::by_type(Syntax).with_term(kind)` plus templates.
  Aggregating the committed histograms: **114 distinct kinds** across the
  full_ast tier — the honest lower bound of the rule checklist; the
  corpus-wide number derives at check time the same way.

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

### Design note 2026-09-25 — L2g: the write mode and the agent tool (written before the code)

Survey facts the leaf is planned against:

- L2a–L2f landed (`c348d4634`, `d48f7fb48`): the ES quadrant is live,
  `formal-ai translate` prints a rendered target, refusals are
  seed-grounded, and the acceptance test already walks the committed
  `./js` corpus (recursive `*.js`) with zero refusals and CST-equal
  round trips. `./ts` holds only the README placeholder, which names
  `--write` as its regenerator.
- The agent surface that exists: the in-repo driver
  (`rust/src/agentic_coding/driver.rs`) advertises `DRIVER_TOOLS` and
  executes every planned call inside a sandboxed `AgentWorkspace`; the
  planner lowers typed answers into `PlannedToolCall { tool, arguments }`.
- The gap: `try_translation` answers backticked snippets, FOL statements
  and natural-language surfaces — a file-shaped request ("translate
  js/app.js to typescript") names no quoted surface, so it falls through
  to the Wiktionary arm and comes back a translation gap. The tool the
  instruction asked for would exist on no surface.

Decisions fixed before any of it is written:

- **The write mapping is a pure contract on repo-relative paths**, in
  `meta_translate` (alloc-pure, so the wasm surface keeps it): a path
  under `js/` ending `.js` maps to the same subpath under `ts/` ending
  `.ts`, and the reverse. Anything else is refused by name — a path with
  `..`, outside the two roots, or with an unowned extension is a wrong
  root, and a leg whose target is not a committed source tree (everything
  involving `meta` or `rust`) is an unsupported write, because the pivot
  document is not a tree and the rust write legs are owed by L3/L5.
- **The std half is one new module, `rust/src/translate_write.rs`**, shared
  by the CLI and the agent driver: `write_one` (a mapped file) and
  `write_tree` (the whole from-root) return a `WriteReport` whose
  rendering values feed the seed intents. Tree mode is transactional:
  every owned file is translated first and any refusal or invalid file
  writes nothing — a half-generated `./ts` is exactly the corrupt state
  L3's mismatch check exists to refuse. Non-owned files under the root
  (`.html`, `.css`, `.lino`, `.wasm`) are not the translator's to move;
  whether `ts/` also carries assets is L3/L4 policy, not this leaf.
- **CLI**: `--write`. With `--input`, one mapped file; without it, the
  whole from-root tree (the command L3's README already names).
  `--list` still answers first, and pending-leg honesty fires before any
  write. New diagnostics are seed intents in the existing translate pair
  (`meanings-translate-cycle.lino` carries the five lexemes, the
  responses file the `{placeholder}` texts):
  `translate_wrote_file`, `translate_wrote`, `translate_write_wrong_root`,
  `translate_write_unsupported`, `translate_write_refused`,
  `translate_source_missing`, `translate_source_invalid`,
  `translate_write_empty`.
- **The agent tool is three surfaces over the same library**: the
  registry record `tool tool_translate` in `data/seed/tools.lino`
  (mode thinking, inputs `from`/`to`/`path`/`write`, sources
  `rust:meta_translate` and the projection seed); the driver, where
  `translate` joins `DRIVER_TOOLS` with a schema and an execution arm
  that reads the path from the sandbox workspace, translates in-process,
  writes the mapped file when `write` is set, and reports the
  `WriteReport` text (`is_error` on refusal, wrong root or invalid); and
  the planner, where `try_translation` gains a source-tree arm — gated by
  the meaning-based translation-action check that already fires, plus a
  `js/`/`ts/` path token and a named ES target, so no new phrase table —
  answering through the pivot, with the shared-solver bridge lowering
  the same family to one `translate` tool call when the client
  advertises it (the write_program Ready-or-Defer shape).
- **Named tests, drafted failing first**: `issue_1138_translation_tool.rs`
  gains the mapping table, the `WriteReport` contract on temp
  directories (including the transactional refusal), and the new seed
  pins; a new `issue_1138_translate_agent_tool.rs` pins the registry
  record, the driver surface (`DRIVER_TOOLS`, the schema, the
  description), the planner lowering (one `translate` call with
  `{from, to, path, write}` against an advertised tool), and the solver
  family id.

Not in this leaf: committing the generated `./ts` tree, the
mismatch-is-red check, and the fix-the-source-or-the-translator rule —
that is L3 whole.

### Design note 2026-09-25 — L4: the cycle as CI structure (written before the code)

Survey facts the leaf is planned against:

- `release.yml` already runs a `detect-changes` job
  (`rust-script scripts/detect-code-changes.rs`, `fetch-depth: 0`) whose
  outputs gate individual jobs — the established per-job path mechanism this
  repository practices. Workflow-level `paths:` predicates also exist
  (`workflows.yml` on `.github/**`, `coding-ladder.yml` on both triggers),
  but per-tier layering needs the outputs form: a workflow-level predicate
  cannot express job-to-job gating, where the ts job must be skipped-by-js
  when js is red yet still run when only `ts/` changed.
- **Defect found while surveying, fixed in this leaf:** the detector's
  `agentic_routing_changed` matches `src/agentic_coding/`, and plan 16 L1
  moved the crate to `rust/src/agentic_coding/` — so the issue #1137 gate
  (full four-client replay on routing-changing pull requests) has been
  unreachable on every PR since the restructure, while the detector's own
  fixtures and the #1137 pin kept the stale spelling green. The fix is the
  prefix plus its pin; the full replay is additionally exercised once by
  `workflow_dispatch` (which sets `full-replay: true` on its own) before
  merge, because no PR run has ever taken that path under the L1 layout.
- No YAML crate sits in the dev-dependencies, and the repo convention audits
  workflows textually (`ci_gates.rs`, `ci-cd/workflow_fixtures.rs` with
  `job_block`); the leaf's test follows the convention rather than adding a
  parser dependency for one file.
- The js corpus is 63 files and every one of them passes `node --check`
  (measured 2026-09-25) — the js tier's fast check is real and green today,
  not ceremony. The ts tree is the 63 generated `.ts` files plus the README.

Decisions fixed before any of it is written:

1. **One new workflow, `.github/workflows/layered-ci.yml`, four jobs**:
   `changes` (the classifier — always runs), `js`, `ts`, `rust`. The checks
   that span all layers stay in their own ungated workflows: the shared tier
   is the audits whose inputs cross tier boundaries, and `release.yml`'s
   pull_request trigger carrying no `paths:` filter *is* the shared tier —
   the leaf's "shared job keyed on the union" reads, against this repository,
   as "the spanning jobs are not layer-gated", which the pipeline already
   satisfies and the test pins so it cannot silently narrow.
2. **Tier predicates are cumulative down the cycle, never up.** js tier:
   `js/**` + `data/seed/**` + the classifier and the layered workflow
   themselves; ts tier: `ts/**` + every js-tier input (the committed ts tree
   is a *function of* the js tree — a js-only edit that let the ts tier sleep
   would carry a stale-green tier past L3's regeneration contract); rust
   tier: `rust/**` + `js/**` + `ts/**` + `data/seed/**` + the binary action
   the rust job executes. `data/seed/**` in every tier is the plan's risk-3
   mitigation: a seed consumed by all three roots must not hide behind an
   untouched folder.
3. **Carry-forward is GitHub-native.** A tier whose predicate is false
   reports *skipped*, never red, so the tier's last green run stands — the
   instruction's "once JavaScript version stabilized, we don't reexecute its
   CI/CD". The workflow header states this so a skipped tier is read as
   carried, not ignored.
4. **Ordering is `needs`-structural.** ts `needs: [changes, js]` with
   `if: ts-changed && (needs.js.result == 'success' || needs.js.result ==
   'skipped')`: js red ⇒ ts reports skipped-by-js, not its own red; js
   skipped-by-path ⇒ ts still runs, because the js tier's last green carried
   forward. The rust job is the same over both js and ts. No `always()`
   anywhere — `always()` would let a tier report its own red over a red tier
   above it, which the instruction forbids.
5. **The classifier grows three outputs** (`js-changed`, `ts-changed`,
   `rust-changed`) with the cumulative rules and embedded fixtures proving
   the matrix: rust-only ⇒ js/ts asleep; ts-only ⇒ js asleep; js-only ⇒ all
   three awake; seed ⇒ all three; docs-only ⇒ none.
6. **Tier content is real and cheap where the tier admits it.** The js job
   runs `node --check` over every committed `js/**/*.js`. The ts job runs the
   js↔ts file-set parity (every `.js` has its `.ts` sibling and no extras) —
   the fast tier-2 rung, failing in seconds before the rust tier spends
   minutes. The rust job runs the L3 dogfood gate in CI, not only in the
   unit suite: delete `ts/**/*.ts`, regenerate with
   `formal-ai translate --from js --to ts --write`, then
   `git diff --exit-code -- ts/` — one diff catches byte drift, missing and
   extra files, and a transactional refusal (a refused regeneration writes
   nothing, so the tree stays deleted and the diff is the full red). The
   binary comes from `.github/actions/formal-ai-binary` (cached by source
   digest), so the tier costs one cached release build, not a second test
   suite.
7. **Named test** `rust/tests/unit/ci-cd/issue_1138_layered_ci.rs` (the
   directory spelling follows the existing `ci-cd` module; the leaf's
   `ci_cd` was a variance): the job graph and per-tier predicates, no
   cross-tier wiring (the js job's condition mentions no other tier's
   output), the skipped-by-js ordering clauses, the delete-regenerate-diff
   pins, the classifier wiring, and the shared tier staying ungated.

Not in this leaf: L5's round-trip projection; and narrowing the rust tier to
`rust/**` alone — it keeps `js/**` and `ts/**` today because the rust suite
audits both trees (the three-roots, translation-tool and dogfood tests read
them); the filter narrows when those audits migrate into the tiers that own
the trees.

### Design note 2026-09-25 — L5: the round trip the pivot can prove today (written before the code)

Survey facts the leaf is planned against:

- The leaf's sentence was written before the L2 design notes measured the
  census. rust → meta renders the self-AST census at `fidelity signature`
  (symbol names, line spans, node-kind counts) — a table of contents, not a
  CST. The meta → ES legs (`parse_document` → `render_source`) accept only
  `token_tree` pivot documents, so a census document cannot cross to ts or
  js: the literal chain "census lino → generated ts/js → back to lino" is
  mechanically impossible today. The leaf is executed at the fidelity each
  quadrant owns instead of faked at the fidelity the instruction dreams of.
- The engine itself parses Rust losslessly (`LinkNetwork::parse` +
  `reconstruct_source` is byte-identical; the census is a rendered summary,
  not the network's limit). But a Rust token tree still cannot render ES
  text without a grammar-to-grammar projection: no
  LinkNetwork↔PivotDocument bridge exists, and the ES tokenizer cannot lex
  Rust. That quadrant is L7, above.
- `parse_document` is documented as the inverse of `extract` +
  `render_document`, yet no corpus-wide check exercises the meta → ES legs
  through the document medium. The L2f acceptance proved js → ts → js by
  render + re-tokenize; it never parsed a rendered lino back. The serialize
  / parse pair is exactly what a pivot must guarantee and what L5 proves.
- The rust ↔ worker parity corpus is capability-level (recipe rows pair
  Rust handlers with `js/worker/` bundle files; 37 files), not a per-module
  projection manifest — so no per-module rust → es rows exist to declare,
  and a registry seeded with them would be invented coverage.
- The pending-leg pointers are stale or about to be: `(js|ts, rust)` still
  names L3, which landed as the dogfood loop and delivered no ES → Rust;
  `(meta, rust)` and `(rust, js|ts)` name L5, which this note scopes to
  verification only — landing it would stale them the same way. Pins live
  in three surfaces that must move together: `meta_translate.rs`'s
  `pending_leg`, the translation-tool test, and the two
  `rust → js  pending` example rows in `meanings-translate-cycle.lino`.

Decisions fixed before any of it is written:

1. **L5 proves the round trip over the whole committed ES corpus, through
   the document medium.** For every committed `js/**/*.js` and `ts/**/*.ts`:
   source → meta (`extract` + `render_document`), meta → ts and meta → js
   (`parse_document` + `render_source`), each rendered target → meta again
   (`extract`), and the token trees compare CST-equal to the source's
   pivot. That is the leaf's "lino → ts → lino and lino → js → lino round
   trips are CST-identical", with the lino being the rendered pivot of a
   committed ES source. CST equality keeps the L2 definition — kind, text
   and shape, spans ignored — plus `token_count`; the header fields
   `target` and `language` are provenance (where a document came from, not
   what it is) and are compared only on the identity legs. Corpus-wide is
   the point, not ceremony: the bundle files carry strings, escapes and
   template literals, so the sanitize/unescape pair is proven on the worst
   corpus the repository has, not on fixtures.
2. **The declared-projection registry is policy rows in the projection
   seed; membership is derived, never enumerated.**
   `data/seed/language-projection.lino` gains `root_projection` rows
   (from-root, to-root, fidelity, status `live`/`owed_by <leaf>`) stating
   exactly what is true: rust → meta at `signature`, live; the ES quadrant
   at `token_tree`, live; and every leg that renders source text of another
   grammar (meta → rust, js|ts → rust, rust → js|ts) `owed_by L7`. No row
   marks rust → es live at any fidelity — that projection does not exist,
   and a live row saying it would be invented coverage. Per-module
   membership is derived at check time — the census set under
   `data/meta/self-ast/` is the rust signature-projection set, the
   committed ES trees are the token-tree set — so the registry cannot drift
   from the trees it describes.
   `meta_translate` grows a registry reader over the seed so the data, not
   code, answers "which projections exist"; `pending_leg` stays the
   compile-time mirror, and the new test pins seed ⇄ code agreement (the
   two-surfaces pattern of `tools.lino` ⇄ `DRIVER_TOOLS`).
3. **The stale pointers are repaired by naming the owner leaf.** All three
   rendering legs re-point to L7 in one move — `meta_translate.rs`, the
   translation-tool test's three pins, and the two seed example rows — so
   `translate --list` never names a leaf that has already landed and
   disclaimed the leg.
4. **Named test, drafted failing first**: `tests/unit/issue_1138_round_trip_projection.rs`
   (the leaf's own name) — the corpus round trips of decision 1, the
   registry reader contract of decision 2, and the seed ⇄ `pending_leg`
   agreement of decision 3.

Not in this leaf: any Rust rendering (L7 whole), raising the census
fidelity, and a per-module worker-parity manifest — the capability-level
recipe rows stay the honest record of which Rust behaviors have js
counterparts until a projection that can compare them exists.

### Design note 2026-09-25 — L7: the rust rendering quadrant, measured (written before the code)

Survey facts the leaf is planned against (meta-language 0.58.2, the
self-maintained dependency, plus this repository's own surfaces):

- **The lossless rust serialization already ships.**
  `LinkNetwork::to_lino()` / `from_lino()` are an exact pair — the crate
  documents `from_lino(to_lino(n))` as isomorphic for any network,
  covering references, names, types, terms, definitions, languages,
  source spans, parse flags, and term registration. And the repo already
  proves the other direction per module: `self_ast::reconstruct_source`
  (issue #558) is `parse(rust) + reconstruct_text()`, byte-identical,
  verified by `self_ast::round_trips`. So `rust → meta` at full network
  fidelity and `meta → rust` are both off-the-shelf; the (Meta, Rust)
  leg is not missing machinery, only wiring.
- **`render_source(language)` is same-grammar only.** The dependency's
  source generation renders token/syntax links whose language label
  matches the target — it cannot translate a rust-parsed network into
  JavaScript. The genuinely missing engine is the grammar-to-grammar
  projection, and the dependency ships its general shape:
  `TranslationRuleSet` — named rules, each a `LinkQuery` match plus
  per-target templates, serialized to and from lino. Per the dependency
  policy, the rule *engine* belongs there; the *rules* are seed data
  here, discovered from the parity corpus.
- **The Meta root now has three lino dialects**, and a leg must say
  which it reads: the census (signature, what `--from rust --to meta`
  renders and what `data/meta/self-ast/` commits), the pivot token_tree
  (what the ES legs carry), and the network serialization (what
  `from_lino` accepts). A census or token_tree document fed to
  meta → rust is not a wrong guess — it is a named refusal.
- The L5 registry (root_projection rows + the seed ⇄ `pending_leg`
  agreement test) is the mechanism that flips legs live: one seed row
  and one code arm move together, and the test keeps them honest.

Decisions fixed before any of it is written:

1. **L7a — the (Meta, Rust) leg goes live at `network` fidelity, as a
   runtime-verified round trip, not a committed tree.** Wiring:
   `translate(Meta, Rust, …)` = `LinkNetwork::from_lino(source)` +
   `reconstruct_text()`; the honest-gap contract names the dialect —
   input that is census or token_tree lino is refused with the
   network-serialization expectation, not mis-parsed. The full-fidelity
   rust → meta direction is `parse + to_lino`, proven per owned module
   by round trip (`parse → to_lino → from_lino → reconstruct_text` is
   byte-identical) in the named test — nothing is committed, because the
   runtime comparison is the check and the census stays the committed
   fingerprint; committing the network lino of every module would add a
   ~14k-line-per-module derived tree with no additional guarantee. The
   seed's meta_to_rust row flips to `fidelity network, status live`,
   `pending_leg` drops the arm, and the fidelity vocabulary grows the
   `network` spelling.
2. **L7b — the registry vocabulary follows the ladder.** Fidelity
   spellings are `signature` (census), `token_tree` (ES pivot),
   `network` (full link serialization); the agreement test pins the
   closed set so a new spelling is a deliberate act. Rust → meta keeps
   rendering the census on the CLI (it is the documented, committed
   artifact); the network round trip is the L7 test's property, and a
   future CLI surface for it stays out of this leaf.
3. **L7c — the grammar projection (rust → js/ts and js/ts → rust) stays
   owed, by name, until rules exist.** The mechanism is
   `TranslationRuleSet` in the dependency; the seed of rules is authored
   here from the worker-parity corpus, and until a construct has a rule
   the renderer refuses it — the L2 doctrine restated across grammars.
   No rule row ships in this leaf; the leaf that authors them opens with
   its own survey of the corpus's construct inventory.
4. **Named test, drafted failing first**:
   `tests/unit/issue_1138_rust_projection.rs` — every owned Rust module
   (the census set, derived) round-trips byte-identically through the
   network serialization; `translate` meta → rust renders reconstructed
   source from a network document and refuses census/token_tree input
   by name; the leg table and the seed registry agree after the flip
   (the L5 agreement test re-runs green with the new row).

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
