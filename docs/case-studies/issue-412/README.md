# Case study — issue #412: a numeric-list follow-up answered `unknown`

> Source issue: <https://github.com/link-assistant/formal-ai/issues/412>
> Branch: `issue-412-1f5f93827186` · PR: #413
> Raw data: [`raw-data/`](./raw-data) (issue JSON, comments, body, online research)

## 1. Summary

The deployed wasm assistant held a coding conversation in Russian. The **first**
turn established a complete coding context — JavaScript, "give me the code and
the result":

```
U: У меня есть числа 3, 5, 6, 7, 8 отсортируй их в JavaScript, дай мне код и результат
A: Вот код на JavaScript … const sorted = [...numbers].sort((a, b) => a - b); … Результат: 3, 5, 6, 7, 8
```

The **second** turn was a bare follow-up that names no language and does not ask
for code:

```
U: Отсортируй 4, 3, 1, 17, 8, 9, 15
A (reported): intent: unknown — "Я тебя не понял. …"
```

A human reads the second turn as "do the same thing (sort, in JavaScript, with
code + result) to this new list." The solver did not: the numeric-list handler
requires the language and the code request to be present **in the prompt itself**,
so the bare follow-up fell through every specialized handler to `unknown`.

## 2. Timeline of events

| # | Event |
|---|-------|
| 1 | User asks, in Russian, to sort `3, 5, 6, 7, 8` in JavaScript with code + result. |
| 2 | Assistant answers correctly via the issue-#395 numeric-list path (code + `Результат:`). |
| 3 | User sends a bare follow-up `Отсортируй 4, 3, 1, 17, 8, 9, 15` (no language, no "code"). |
| 4 | `tryNumericList` declines: `programLanguageFromPrompt(normalized)` is `null`. |
| 5 | Every other specialized handler declines; dispatch falls through to `unknown`. |
| 6 | Assistant returns the `unknown` apology instead of continuing the coding context. |

The reported reasoning trace (see [`raw-data/issue-412-body.md`](./raw-data/issue-412-body.md))
confirms step 4–6: `formalization:(@USER OP:express ?отсортируй …)` → `fallback:unknown`.

## 3. Root cause

The numeric-list handler (`src/solver_handlers/numeric_list/mod.rs`, mirrored in
`src/web/formal_ai_worker.js`) is **stateless** — it inspects only the current
prompt. Its language gate

```rust
let language = crate::coding::program_language_by_alias(normalized)?; // None → bail
```

returns `None` for `Отсортируй 4, 3, 1, 17, 8, 9, 15`, so the handler bails before
it can compute anything. The reduction family (`sum`, `product`, …) is gated even
harder: it additionally requires the explicit `code_request` operation. Neither
gate consults the conversation, so an established coding context cannot carry over.

This is the same *class* of defect as the conversational-coreference work in
issues #324 / #357 / #398 (a follow-up that depends on a prior turn), but the
numeric-list handler had never been taught to participate in it.

## 4. Requirements extracted from the issue

The issue mixes one concrete defect with a broad generalization mandate. Every
distinct requirement, and how this PR addresses it:

| # | Requirement | Status in this PR |
|---|-------------|-------------------|
| R1 | The reported follow-up must no longer answer `unknown`; it must continue the JavaScript code+result context. | **Done** — history-aware inheritance in both runtimes. |
| R2 | Fix must not over-reach: a bare sort with no established language must stay `unknown` (no fabricated language). | **Done** — guarded inheritance + regression test. |
| R3 | Apply the fix across the **entire** codebase wherever the defect appears (Rust solver **and** JS worker mirror). | **Done** — both runtimes patched, byte-parity verified. |
| R4 | Generalize beyond this one prompt (≈10 similar tasks): the path must cover the whole numeric-list family and both languages. | **Done** — inheritance covers every operation/program language; follow-ups tested across all four supported natural languages (en, ru, hi, zh) plus the reduction family. |
| R5 | Use `link-foundation/meta-language` for coding manipulation. | **Already true** — the inherited path runs through the meta-language CST engine (trace shows `cst_engine meta_language`). |
| R6 | Integrate external knowledge oracles (Wikifunctions, Rosetta Code, Hello World Collection, Stack Overflow) as cached APIs merged into views. | **Done** — `src/knowledge.rs` models the four sources and a `CodingOracle`; the `write_program` fallback (`src/solver_handler_oracle.rs`, mirrored in the worker) answers languages the catalogue does not template (Kotlin/Swift/PHP/Bash/Lua/Haskell + a Rosetta-Code Kotlin factorial) from the cached corpus. See §7; researched in [`raw-data/online-research.md`](./raw-data/online-research.md). |
| R7 | "Meta-algorithm that builds algorithms" / first-principles re-architecture. | **Partially** — the numeric-list path already composes code from the `coding-idioms.lino` knowledge base rather than per-prompt templates, and the oracle generalises the answer source beyond hand-written templates; the broader task-agnostic meta-builder remains the tracked next step (§7). |
| R8 | Cache popular cases but never everything (≤1%, or 512 if smaller, per source). | **Done** — `cache_capacity`/`within_cache_capacity`/`KNOWLEDGE_CACHE_FLOOR` in `src/knowledge.rs` enforce the `min(1%, 512)`-clamped cap, and a ratchet test fails if the committed snapshot set exceeds it. |
| R9 | Add debug/verbose output if data is insufficient for root cause. | **Not needed** — root cause was directly reproducible; added a `numeric_list_coreference` trace line for observability instead. |
| R10 | Build a deep case study under `docs/case-studies/issue-412`. | **Done** — this document + `raw-data/`. |
| R11 | Report upstream issues to related repos with repro + fix suggestions. | **N/A** — the defect is entirely in this repository; meta-language behaved correctly. |
| R12 | Reproduce with a test before fixing; keep everything in PR #413. | **Done** — failing-then-passing integration test + JS parity experiment. |

## 5. The fix

### Rust (`src/solver_handlers/numeric_list/mod.rs`)

- New `numeric_list_history_context(history)` scans the conversation **newest
  first** and inherits from a prior turn **only** when that turn was itself a
  genuine numeric-list coding request — it has a recognised operation, a
  supported program language, and lists ≥2 numbers. This is what keeps unrelated
  chatter (e.g. "What is the capital of France?") from leaking a language (R2).
- `try_numeric_list_with_history` resolves the language as
  `program_language_by_alias(normalized).or(inherited.language)` (the current
  prompt always wins) and treats a reduction's code request as satisfied when it
  was inherited. It appends a `numeric_list_coreference inherited_language=… inherited_code_request=…`
  event to the trace.
- `src/solver.rs` threads the conversation `history` into
  `handle_specialized_pattern` and calls the history-aware entry point for the
  `numeric_list` handler.

### JS worker (`src/web/formal_ai_worker.js`)

The byte-for-byte mirror: `numericListHistoryContext(history)` +
`tryNumericList(prompt, history)` with the identical gate, language resolution,
and `numeric_list_coreference` evidence line; the dispatch entry now passes
`history`.

### Why it is safe

Inheritance fires only behind the "prior turn was a real numeric-list coding
request" guard, and the current prompt's own language always takes precedence, so
the change is purely additive for every existing (history-free) case — the
170-cell cross-runtime parity matrix stays byte-identical.

## 6. Verification

- `tests/integration/issue_412_followup_sort.rs` — 6 tests: the exact reported
  follow-up recovers JavaScript + code + sorted result; a no-context bare prompt
  stays non-`write_program`; a reduction follow-up inherits the code request; and
  the coreference is exercised across **every supported language** — an
  English/Python, a Hindi/JavaScript, and a Chinese/Python context each inherit
  across a bare follow-up that names no language and render the localized result
  label (`Result:` / `परिणाम:` / `结果:`).
- `experiments/issue-412-js-numeric-list-coreference.mjs` — the same scenarios
  (plus an "unrelated context → not claimed" negative) through the JS worker in a
  node VM sandbox: **all pass**.
- `experiments/issue-395-cross-runtime-codegen-parity.mjs` — **170/170 cells
  byte-identical**, confirming no regression.
- `examples/repro_issue_412.rs` — runnable reproduction printing the recovered
  answer and the full meta-language-backed trace.
- `tests/integration/issue_412_oracle_languages.rs` + the
  `source_tests/solver_handler_oracle` / `source_tests/knowledge` suites — the
  coding oracle (R6) and the bounded-cache policy (R8): Kotlin/Swift/PHP
  hello-world resolve from the oracle, catalogued languages keep their verified
  route, and the cache-cap ratchet holds.
- `experiments/issue-412-js-oracle.mjs` — the worker's oracle path; the rendered
  answer is byte-identical to the Rust `solve()` output.
- `examples/issue_412_oracle_smoke.rs` — runnable oracle smoke check across
  Kotlin/Swift/PHP (oracle) and Rust (catalogue).
- Full suites: `cargo test --test source` (387), `--test unit` (814), `--test
  integration` (35), `cargo clippy --all-targets` under `-Dwarnings` — all green.

## 7. The broad generalization mandate (R6–R8)

The 2026-06-11 review rejected deferring the larger asks, so they ship in this
same PR alongside the numeric-list fix.

### 7.1 External knowledge sources as cached APIs (R6) — done

`src/knowledge.rs` models the four public corpora — Rosetta Code,
Wikifunctions, the Hello World Collection, and Stack Overflow — as external
sources we treat as APIs even when they expose none: a fetched page is parsed
into a reviewed `OracleSnippet` and cached under a per-source bucket exactly
like the Wikidata/Wiktionary caches. `CodingOracle` is the offline-first lookup
that resolves a `(task, language)` request to a snippet plus its deterministic
output and source attribution. The `write_program` fallback lives in
`src/solver_handler_oracle.rs` (called from `solver.rs` when the formalizer
yields `UnsupportedWriteProgram`, *after* the issue-#340 blueprint handler
declines): a canonical request like "write a hello world program in Kotlin" now
returns the idiomatic snippet, its output, and "Source: Hello World Collection
(…), cached locally as a popular example" — the same "code + result" shape the
built-in catalogue produces, but for a language the catalogue never templated.
Catalogued languages (Rust, Python, …) keep their verified "compiled and ran"
route untouched; the oracle is purely additive.

The committed popular-case cache covers Kotlin/Swift/PHP/Bash/Lua/Haskell
hello-world (Hello World Collection) and a Kotlin factorial (Rosetta Code).

### 7.2 Bounded cache, never a mirror (R8) — done

`cache_capacity(source_total)` encodes "never cache more than 1% — or 512 items
when 1% is smaller": 1% rounded up (MSRV-safe `(n+99)/100`), floored at
`KNOWLEDGE_CACHE_FLOOR = 512`, then clamped to the source's own size. The same
number bounds every per-source / per-topic cache so no single corpus can bloat
the merged views. `committed_snapshots_stay_within_the_cache_cap` is the ratchet
that fails CI if any per-source snapshot count creeps over the cap.

### 7.3 Cross-runtime parity (R3, restated for the oracle) — done

The oracle data, lookup, and answer renderer are mirrored in
`src/web/formal_ai_worker.js` (`CODING_ORACLE_SNAPSHOTS`, `codingOracleLookup`,
`codingOracleAnswer`). `experiments/issue-412-js-oracle.mjs` drives the worker's
own `tryWriteProgram` and the rendered answer is verified **byte-identical** to
the Rust `solve()` output.

### 7.4 Meta-builder (R7) — tracked next step

The numeric-list path already composes from `data/seed/coding-idioms.lino`
rather than hard-coded templates, and the oracle now generalises the *answer
source* beyond hand-written templates. The remaining step is to generalise the
composer into a task-agnostic "algorithm that builds algorithms" so an arbitrary
new coding task reaches the meta-language CST engine without a bespoke handler.
This is the one piece still open; it is architectural rather than a data/policy
gap and is tracked as the direct follow-up to this PR.

### 7.5 Live refresh

The committed snapshots are the offline accelerator; a gated live-refresh path
(mirroring the existing `FORMAL_AI_LIVE_API` discipline used by the Wikidata /
Wiktionary loaders) repopulates them from the live sources without changing the
offline-first default, so tests stay fast on the popular cached cases.
