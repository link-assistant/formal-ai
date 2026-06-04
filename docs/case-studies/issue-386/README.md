# Case study — Issue #386: "Unknown prompt: Отмени сортировку"

> Raw artifacts for this study live in [`raw-data/`](./raw-data/):
> [`issue-386.json`](./raw-data/issue-386.json) (full issue payload),
> [`issue-386-body.md`](./raw-data/issue-386-body.md) (the prefilled report the
> user submitted), [`issue-386-comments.json`](./raw-data/issue-386-comments.json)
> (empty — no comments at time of writing), and
> [`pr-387.json`](./raw-data/pr-387.json) (the draft PR this work updates).

## 0. One-paragraph summary

A user built a reverse-sorted file-listing program through a multi-turn dialog,
then sent the follow-up **"Отмени сортировку"** ("Cancel the sorting"). The
assistant replied `intent: unknown` and the user filed a prefilled bug report.
The report itself was the immediate subject of most of the actionable asks: it
wasted its limited URL budget on settings that were already at their shipped
default, repeated the worker on its own line, carried a multi-clause
attach-memory walkthrough, and printed a Reasoning Trace even when the dialog
above it had been trimmed to fit GitHub's URL cap. The deeper subject is the
refusal itself: the program-modification pipeline only understood **additive**
modifiers (add a path argument, add reverse-sort), so a **subtractive** request
("cancel/undo the sort") matched no modifier and fell through to `unknown`. This
PR fixes it with the architecture rethink the issue asked for (R-h): operations
**declare their inverse** in the seed (`cancel_reverse_sort` carries
`inverse "reverse_sort"`) and the subtractive substitution rules are **derived at
runtime** by mirroring the additive ones, so "cancel X" is the data-defined
inverse of "X" — no bespoke control flow, and the same fix lands natively and in
the wasm worker. Around the bug the issue layers a large architecture-vision ask
(self-describing seed data, a links-rooted semantic meta-language, tree-sitter
CST/AST, multiple virtual views of memory, no bare-text processing in code).

## 1. Timeline / sequence of events

All times from the report (`raw-data/issue-386-body.md`) and the issue payload.

1. **Earlier in the dialog** the user asks for a Rust program that lists files
   from a path argument, then refines it: *"Сделай сортировку результатов в
   обратном порядке"* ("Sort the results in reverse order"). The assistant
   correctly upgrades the program to reverse-sorted output — this is the
   reverse-sort feature delivered by epic #349 (closed via child PRs #355–#365;
   see [`../issue-365/README.md`](../issue-365/README.md)).
2. **The trigger turn.** The user sends **"Отмени сортировку"** ("Cancel the
   sorting"). The worker detects zero program modifiers, recovers no plan, and
   returns `intent: unknown` with the standard teach-me fallback.
3. **2026-06-01T17:47:37Z** — the user opens the prefilled "Report issue" link.
   The body (`raw-data/issue-386-body.md`) carries version `0.174.0`, a
   `Worker: wasm worker` line, Mode/Status/Diagnostics, a full User Context block
   in which **every** value equals its default, a trimmed dialog
   (`... omitted 5 earlier messages ...`), a Reasoning Trace, and a long
   attach-memory section.
4. **Issue #386 filed** with the title *"Unknown prompt: Отмени сортировку"* and
   a Description that enumerates the report-trimming asks, the two UI features
   (reset settings, copy conversation as Markdown), and the architecture vision.
5. **Draft PR #387** opened on branch `issue-386-0f7c7e8a730c`
   (`raw-data/pr-387.json`); this case study and the implemented fixes update it.

## 2. Requirements (every explicit and implicit ask)

Lettering is preserved across the PR description, commits, and this document.

### Report-trimming asks (the prefilled GitHub report)
- **R-a — Omit default-valued settings.** Do not print a setting whose value
  equals its shipped default: Mode (manual), Status (Manual mode), Diagnostics
  (off), Theme (auto), Guess probability (80%), Temperature (0.7), Follow-up
  probability (75%), Location (inference-only). Saves space for the dialog.
- **R-b — Drop the Reasoning Trace when the dialog is incomplete.** If earlier
  turns were trimmed to fit GitHub's URL cap, the trace no longer corresponds to
  a fully-shown dialog and must be omitted.
- **R-c — Shorten "Attach full memory (optional)".** Replace the multi-clause
  zip/Gist walkthrough with a short pointer to the docs.
- **R-d — Fold the worker into the version.** `0.174.0 (wasm)` instead of a
  separate `Worker: wasm worker` line.

### UI feature asks
- **R-e — Reset settings.** In the settings UI, reset each setting to its
  default individually, and all of them at once.
- **R-f — Copy a conversation as Markdown.** From the conversations list, copy
  the full dialog as Markdown. When diagnostics mode is on, reasoning steps are
  converted to Markdown and appended after each AI message.

### Process / documentation asks
- **R-g — Deep case study.** Download all logs/data into
  `docs/case-studies/issue-386/`, reconstruct the timeline, list every
  requirement, root-cause each problem, propose solution plans per requirement,
  survey existing components/libraries, and search online for additional facts.
  *(This document.)*
- **R-h — Architecture rethink.** Self-describing seed data; a links-rooted
  semantic meta-language (each symbol/word/statement a link); tree-sitter
  CST/AST as a dependency (with a Rust alternative if available); knowledge APIs
  with preserved, access-counted, restorable caches; multiple virtual views of
  memory (meanings/words/symbols/letters/nouns/verbs/phrases/SVO/statements);
  no bare-text processing in code (hardcoded examples allowed only in tests);
  use of and upgrade to `link-assistant/meta-expression` for translation.

### Cross-cutting meta-instructions
- **R-i — Fix everywhere.** If a defect exists in multiple places, fix all of
  them.
- **R-j — Add tracing where root-causing is blocked.** If data is insufficient,
  add debug/verbose output for the next iteration.
- **R-k — Report upstream.** If the issue touches another repo we can file
  issues on, do so with reproducible examples, workarounds, and fix suggestions.
- **R-l — Single PR, incremental commits.** One PR, but commit each finished
  part so intermediate progress is visible.
- **R-m — Nothing hardcoded in code; tests guard every feature.** Examples may
  be hardcoded in tests, never in production code.
- **R-n — Latest requirements override earlier ones on contradiction.**

## 3. Root-cause analysis

### 3.1 The report wasted its URL budget (R-a … R-d)

The prefilled report is built unconditionally: every environment and
user-context field is emitted regardless of whether it still holds its default,
the worker prints on its own line, the attach-memory section carries the full
walkthrough, and the Reasoning Trace is emitted whenever a trace exists — even
after the dialog above it has been trimmed to fit GitHub's ~8 KB URL cap. The
GitHub issue URL has a hard length budget, and the dialog is the single most
valuable payload for diagnosis, so every default-valued line is budget spent on
zero information. **Root cause: the report builder had no notion of "equals
default ⇒ omit" and no coupling between dialog-completeness and the trace.**

### 3.2 The refusal — "Отмени сортировку" (the title bug)

This is the substantive defect. The program-modification pipeline is fully
data-driven and was designed (correctly) so that *adding* a capability is data,
not code:

- **Trigger phrases** live in `data/seed/operation-vocabulary.lino`. The
  `reverse_sort` operation lists Russian triggers `в обратном порядке`,
  `сортиров+обратн`, `отсортир+обратн` (lines 143–166). There is **no**
  `cancel` / `undo` / `remove-sort` operation anywhere in the vocabulary — a
  repository-wide search for `cancel|undo|отмен` in `data/seed/` returns only
  unrelated Wiktionary cache bodies.
- **Substitution rules** live in `data/seed/program-plan-rules.lino`. Every rule
  is **additive**: `request:modifier -> path_argument` upgrades
  `list_files → list_files_arg`; `request:modifier -> reverse_sort` upgrades
  `list_files → list_files_reverse_sort` (and the `_arg` variants). There is
  **no** rule that *removes* `reverse_sort` from a task slug.
- **Modifier discovery** is derived from those rule conditions:
  `program_plan::modifier_slugs()` (`src/program_plan.rs:67`) scans the rules for
  `request:modifier -> <slug>` and returns exactly `{path_argument,
  reverse_sort}`. `detected_program_modifiers()`
  (`src/intent_formalization.rs:582`) intersects the vocabulary detections with
  that set.

So for "Отмени сортировку": the vocabulary surfaces no `cancel`/subtractive
operation, the intersection with `modifier_slugs()` is empty, and — one step
earlier — the bare-follow-up gate `looks_like_bare_program_artifact_follow_up`
(`src/program_coreference.rs`) lists only *additive* verbs and nouns, so the turn
is not even recognized as a program follow-up. It clears no gate, surfaces no
modifier, and falls through to `unknown`. **Root cause: the modifier ontology is
additive-only — there is no concept of a subtractive/cancel operation, in neither
the coreference gate, the vocabulary, nor the rules.**

One subtlety is worth recording, because the first draft of this study got it
wrong: the program *does* already accumulate state across turns.
`active_program_context()` (`src/intent_formalization.rs:458`, added for issue
#357's bare-coreference follow-ups) walks the history in reverse and re-parses
the most recent turn that `write_program_parameters()` recognizes — which is the
previous **rendered assistant answer**. That answer's task slug already encodes
the full accumulated program, so the recovered base task for the cancel turn is
`list_files_arg_reverse_sort`, not a bare `list_files`; applying the current
turn's modifier to it is sufficient. There is no missing "multi-turn state"
model — the cancel turn simply never reached this recovery, because the
additive-only ontology rejected it upstream. The single root cause therefore
manifests in three layers: the follow-up gate, the substitution-rule /
`modifier_slugs()` set, and rule-synthesis decomposition — "cancel X" had a home
in none of them.

This is exactly the shape of the reverse-sort capability that took epic #349 ten
child PRs (#355–#365) to land, so the risk was real: a hand-written subtractive
rule could regress the additive pipeline #349 hardened. This PR removes that risk
by **not hand-writing the subtractive rule at all**. The inverse rules are
*derived* from the very additive rules they mirror (§4 R-b-bug), so they cannot
drift from the additive behavior, and the additive pipeline runs unchanged. The
accumulated multi-turn program state already exists (`active_program_context()`,
above), so no new state model was needed. The fix is therefore **landed in this
PR** — sort cancellation works natively and in the wasm worker — rather than
deferred, and the combined additive + subtractive suite (including the four-turn
Russian repro) guards against regression.

### 3.3 Why the architecture invites this class of bug (R-h)

The refusal is not a one-off: it is the predictable result of an ontology that
enumerates *additive* operations only. Any request expressed as the **negation**
or **removal** of a previously-applied operation has no home in the current
model, so it routes to `unknown`. The user's architecture ask (R-h) is the
generalization of this observation: reasoning over a links-rooted semantic
meta-language with explicit, self-describing operations (including inverses)
would let "cancel X" be derived from "X" rather than separately enumerated. This
PR implements exactly that inverse-derivation for the sort operation (§4
R-b-bug): the seed declares the inverse and the subtractive rules are derived,
dissolving this specific instance of the bug class. The broader links-rooted
meta-language remains the staged follow-up vision in §6.

## 4. Solution plans (per requirement) and what was implemented

### R-a — Omit default-valued settings · **done**
The report builder now consults `PREFERENCE_DEFAULTS` (the single source of
truth) and the same `settingIsDefault(key, value)` helper used by the reset
feature, emitting a User-Context / Environment line only when its value differs
from the shipped default. Mode/Status (manual), Diagnostics (off), Theme (auto),
Guess probability (80%), Temperature (0.7), Follow-up probability (75%), and
inference-only Location are all suppressed at defaults. Locked by
`tests/e2e/tests/issue-386.spec.js` →
*"a fresh-default report omits default settings…"*.

### R-b — Drop the Reasoning Trace on a trimmed dialog · **done**
The trace section is gated on dialog completeness: when the URL fitter drops
earlier turns (the body then contains an `omitted` marker), the
`## Reasoning Trace` section is not emitted. Locked by
*"the reasoning trace is dropped once earlier turns are trimmed to fit"*
(asserts `href.length <= 8192`, body contains `omitted`, body has no
`## Reasoning Trace`).

### R-c — Shorten the attach-memory section · **done**
The multi-clause zip/Gist walkthrough is replaced by a one-line pointer to
`docs/upload-memory.md`. Locked by *"the attach-memory section is a short
pointer to the docs"*.

### R-d — Fold the worker into the version · **done**
The standalone `**Worker**` line is gone; the version renders as
`<version> (wasm)`. Locked by the `\*\*Version\*\*: .*\(wasm\)` /
`not.toContain('**Worker**')` assertions.

### R-e — Reset settings · **done**
A reset bar at the top of the settings panel exposes a per-setting reset and a
reset-all. It is driven by a `settingDescriptors` registry whose keys all map
into `PREFERENCE_DEFAULTS`; `modifiedSettings` filters to non-default settings
via `settingIsDefault`, `resetSetting(d)` writes `PREFERENCE_DEFAULTS[d.key]`,
and `resetAllSettings()` resets every modified one. Reset-all is disabled and an
empty-state row shows when nothing is modified. New i18n keys
`settings.resetHeading/resetAll/resetOne/resetNone` across all four locales.
Locked by the *"reset settings to default"* describe (two tests).

### R-f — Copy a conversation as Markdown · **done**
A `conversationToMarkdown(events, conversationId, options)` helper reconstructs
the full dialog from persisted events: each `kind: "message"` becomes
`### <author>` + content; for assistant messages, when `includeReasoning` is on,
the buffered `kind: "reasoning"` events are appended as `#### <reasoningLabel>`
+ a numbered list. A copy button on each conversation row calls
`handleCopyConversation`, which sets `includeReasoning` from
`diagnosticsModeRef.current` — so diagnostics-on exports fold reasoning in,
diagnostics-off omit it. New i18n keys
`conversation.copyMarkdown/copyMarkdownDone/copyMarkdownTitle` across all
locales. Locked by the *"copy a conversation as Markdown"* describe (two tests:
with/without diagnostics).

### R-b-bug — Resolve "Отмени сортировку" · **done**
Implemented as the inverse-derivation architecture the issue asked for (R-h), not
a hand-written special case. Each piece is pure data plus a generic mechanism:
1. **Declare the inverse in seed data.** `data/seed/operation-vocabulary.lino`
   gains a `cancel_reverse_sort` operation carrying `inverse "reverse_sort"` and
   trigger phrases in all four languages (en `cancel/undo/remove the sorting`, ru
   `отмени/убери/отмените сортировку`, `без сортировки`, hi `सॉर्ट हटाओ`,
   `सॉर्ट रद्द करो`, zh `取消排序`, `撤销排序`, …). No code lists these phrases.
2. **Parse the declaration.** `OperationTrigger.inverse_of` and
   `OperationVocabulary::inverse_pairs()` (`src/seed/operation_vocabulary.rs`)
   expose every declared `(cancel_op, base_op)` pair.
3. **Derive the subtractive rules at runtime.** `derive_inverse_rules()`
   (`src/program_plan.rs`) mirrors each additive base rule that fires on
   `request:modifier -> reverse_sort` (single-link rewrite) into its inverse —
   firing on `request:modifier -> cancel_reverse_sort` with the task add/remove
   swapped. The four additive base rules yield two derived rules
   (`cancel_reverse_sort__reverse_sort_list_files` and `…_list_files_arg`,
   6 rules total), so `modifier_slugs()` now reports `cancel_reverse_sort` as a
   first-class modifier. Adding a new cancellable operation is pure seed data —
   no new control flow here (R-h/R-m).
4. **Recognize the follow-up.** The bare-coreference gate
   (`src/program_coreference.rs`) gains the sort nouns and the
   cancel/undo/remove/revert verbs (+ ru/hi/zh forms), so "Отмени сортировку" is
   recognized as a program follow-up and bound to the active artifact — whose
   accumulated task, `list_files_arg_reverse_sort`, is recovered by re-parsing the
   prior rendered answer (§3.2). Lowering it with `cancel_reverse_sort` resolves
   to `list_files_arg` — sort removed, path argument kept.
5. **Verify the cancel actually removes the sort.** Rule synthesis
   (`src/rule_synthesis.rs`) decomposes `cancel_reverse_sort` as `operation
   cancel` / `operation_modifier reverse_sort` and flips the render check: a
   cancel must leave **no** descending order — the exact inverse of
   reverse_sort's "must be descending" check — so a cancel that silently failed
   to remove the sort fails verification.
6. **Cross-runtime parity.** The same derivation is mirrored in the web worker
   (`src/web/formal_ai_worker.js`); `experiments/issue-386-js-cancel-sort.mjs`
   proves the four-turn Russian dialog yields the unsorted program in the wasm
   runtime too.

Locked by `tests/integration/issue_386_cancel_sort.rs` (the four-turn Russian
repro, the full diagnostic reasoning chain, and the same flow across all four
languages), the `program_plan` / `operation_vocabulary` / `program_coreference`
unit tests added alongside the code, and `tests/e2e/tests/issue-386.spec.js` (the
cancel flow plus the inverse-rule reasoning chain rendered in the browser).

### R-g — Case study · **done** (this document + `raw-data/`).

### R-h — Architecture rethink · **core principle delivered as a codebase-wide sweep; infra-blocked subsystems staged (§6)**
The rethink's governing principle — behavior **derived from self-describing seed
data** instead of hardcoded control flow — is applied throughout this PR. The
cancel-sort fix (§4 R-b-bug) is its first concrete, regression-free instance
(operations declare an `inverse`; the subtractive rules are *derived*, never
written); the same principle then drives a codebase-wide sweep that converts the
hardcoded recognizers — calculation and currency cues, the translation and
define-in-Links gates, unit and calendar vocabulary, the intent question/statement
split, the coding-catalog aliases, and finally the Wikidata entity/property tables
the translation formalizer matched against (`ITEM_LABELS`/`PROPERTY_PATTERNS`, now
projected from `data/seed/meanings-wikidata.lino` by semantic role) — into
seed-defined meanings queried by semantic role, each conversion landed as its own
commit and ledgered in `changelog.d` (R-m: the raw-word lookup tables this sweep
targeted are gone from production code; hardcoded text now lives only in tests). The remaining sub-systems — a fully links-rooted
semantic meta-language, tree-sitter CST/AST, knowledge-API caches, multiple
virtual memory views, and adoption of `meta-expression` for translation — are an
ongoing program rather than a one-commit change. Several need a product /
runtime decision before they can land safely: a native tree-sitter dependency
cannot load in the wasm worker that serves the web/desktop/VS Code surfaces;
live knowledge-API access and a web-search provider from the browser require an
endpoint, credentials, and a cache-eviction policy; swapping the translation
pipeline onto `meta-expression` changes an external dependency. Each must be
decomposed into an independently-verifiable change so it cannot regress the ~180
i18n keys or the reasoning pipeline. The full staged plan is §6, tracked on the
vision track (issue #244, `../issue-244/README.md`); the principle itself now
lives in the codebase rather than being deferred.

### R-i / R-l / R-m / R-n — satisfied by construction
Defaults are read from the single `PREFERENCE_DEFAULTS` source (no hardcoded
setting list duplicated across report + reset + UI); examples are hardcoded only
in `tests/e2e/tests/issue-386.spec.js`; each finished part is a separate commit
on the single PR #387; later asks override earlier ones where they conflict.

### R-j — Tracing
The existing reasoning trace already surfaces the `unknown` path with
`trace:formalization`, `trace:fallback:unknown`, and the detected-modifier set,
which was sufficient to root-cause the refusal (§3.2) without new instrumentation.

### R-k — Upstream reports
No external-repository defect was identified: the refusal is local to this
repo's seed ontology, not a bug in `lino-i18n`, `meta-expression`, or
`tree-sitter`. The R-h adoption of `meta-expression`/`tree-sitter` is a feature
integration, not a bug report, so no upstream issue is warranted at this stage.

## 5. Existing components / libraries reviewed

- **`PREFERENCE_DEFAULTS` (in-repo).** The single source of default settings
  values; reused by the report-trimming, the reset feature, and `settingIsDefault`
  so there is exactly one definition of "default" (R-m).
- **`lino-i18n@0.1.1` (in-repo dependency).** Owns the four-locale catalog and
  the parity check (`check-i18n-catalog.mjs`); the new reset/copy keys plug into
  the existing REQUIRED_KEYS contract.
- **Program-modification pipeline (in-repo, epic #349 / #355–#365).** The
  data-driven `operation-vocabulary.lino` + `program-plan-rules.lino` +
  `program_plan::modifier_slugs()` machinery is the template the cancel-sort fix
  (R-b-bug) mirrors: `derive_inverse_rules()` reuses the additive rules as the
  source of their own inverses, keeping modifier discovery fully data-driven.
- **[tree-sitter](https://tree-sitter.github.io/tree-sitter) (external, R-h).**
  Incremental parser producing concrete syntax trees; the Rust binding
  [`tree-sitter`](https://crates.io/crates/tree-sitter) plus grammar crates
  (`tree-sitter-rust`, etc.) give native CST access. Candidate dependency for the
  "programming language → CST → semantic meta-language" path.
- **[`link-assistant/meta-expression`](https://github.com/link-assistant/meta-expression)
  (external, R-h).** The user asks to adopt its best practices and latest version
  for translation requests; relevant to the links-rooted meta-language and the
  translation pipeline.
- **[`link-foundation/relative-meta-logic`](https://github.com/link-foundation/relative-meta-logic)
  (external, R-h).** Cited as the model for defining each term formally in terms
  of other terms — the self-describing-seed-data target.

## 6. Architecture gap analysis (R-h) — staged plan

The vision is recorded here so it can be decomposed into child issues on the
existing vision track (#244). Item 4 (operations with inverses) is **delivered in
this PR** for the sort operation; the rest is the staged roadmap. Each item is
independently shippable and testable:

1. **Self-describing seed data.** Every seed term defined via other terms (per
   `relative-meta-logic`), so the ontology is recursive and inspectable rather
   than a flat list of bare strings.
2. **Links-rooted semantic meta-language.** Each symbol/word/statement is a link;
   reasoning happens over links, not over text. Natural language → semantic
   meta-language; programming language → CST → semantic meta-language.
3. **tree-sitter CST/AST dependency** for the programming-language path, with a
   native Rust binding where available.
4. **Operation ontology with inverses.** Operations carry their inverses, so
   "cancel X" is *derived* from "X" — directly dissolving the §3.2 root cause
   instead of enumerating every negation. **Delivered for the sort operation by
   this PR (§4 R-b-bug); the next step is to generalize the same `inverse`
   declaration to every operation.**
5. **Multiple virtual memory views** (meanings, words, symbols, letters, nouns,
   verbs, noun/verb phrases, SVO, statements) over the same links store.
6. **Knowledge-API cache discipline.** Preserve all API requests; count accesses;
   evict raw API responses first (restorable on demand) while preserving the
   reasoning steps that formed the request and followed it. **First down-payment
   in this PR:** `data/seed/meanings-wikidata.lino` pre-caches the Wikidata
   entity/property vocabulary the formalizer needs, each meaning rooted in the
   external knowledge base through its language-independent Q-id/P-id, so the code
   references the concept rather than a raw English word.
7. **No bare-text processing in code.** Production code reasons in the
   meta-language; hardcoded text lives only in tests (R-m). **Delivered
   incrementally across this PR** by the seed-role sweep (§4 R-h): the
   `changelog.d` ledger records each converted recognizer, the translation
   formalizer's Wikidata entity/property tables — the last raw-word lookup tables
   in production code — being the deepest and most recent.

## 7. Verification

- `cargo test --all-features` — the full suite passes, including
  `tests/integration/issue_386_cancel_sort.rs` (four-turn Russian repro, the full
  diagnostic reasoning chain, and the cancel flow across en/ru/hi/zh) and the new
  `program_plan` / `seed::operation_vocabulary` / `program_coreference` unit tests
  (inverse-pair exposure, derived-rule shape, round-trip inverse, gate
  recognition in every language).
- `cargo fmt --all -- --check` and `cargo clippy --all-targets --all-features
  -- -D warnings` — clean.
- `rust-script scripts/check-file-size.rs` — passes after the three oversized
  files (`src/seed/meanings.rs`, `src/seed/roles.rs`, and
  `tests/unit/specification/code_generation.rs`) were split under the 1000-line
  guard. The seed-lexicon gates in `src/seed/meanings/tests.rs` (every meaning
  self-describing, covering en/ru/hi/zh, every word form described, every meaning
  reaching the `link` root) and the formalizer's seed projection
  (`item_entries()` / `predicate_patterns()` over the Wikidata meanings) are part
  of `cargo test -p formal-ai --lib` (378 tests).
- `node experiments/issue-386-meanings-mirror.mjs` — the browser worker's
  embedded `MEANINGS_LINO` is byte-identical to the 32 canonical seed files
  (12625 lines), so the new Wikidata vocabulary and every other meaning reason the
  same in Rust and JS.
- `node experiments/issue-386-js-cancel-sort.mjs` — the web worker mirror yields
  the unsorted program for the same four-turn dialog (cross-runtime parity).
- `node tests/e2e/scripts/check-i18n-catalog.mjs`,
  `check-language-test-coverage.mjs` (covers en/ru/hi/zh),
  `check-language-change-parity.mjs`, `check-multilingual-intent-coverage.mjs`,
  and `check-web-tdz.mjs` — pass.
- `tests/e2e/tests/issue-386.spec.js` — four describe blocks (trimmed report,
  reset settings, copy conversation, cancel a program modification) pass under
  `playwright.local.config.js`.
