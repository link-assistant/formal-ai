# Issue #1184 — White-Box Derivation for Every Answer

Issue [#1184](https://github.com/link-assistant/formal-ai/issues/1184) (E148,
part of the #1183 umbrella): the answer carries its derivation as links, and
`formal-ai explain <answer-id>` prints it. This case study records the audit,
the plan, and what landed.

## Audit — what the tree already provided

- `EventLog` (`rust/src/event_log.rs`) appends content-addressed events, and
  the answering routes already log the online trail:
  `web_search:request` (payload is the query text, `live_search.rs:50`,
  `solver_handler_how.rs:91`, `document_originality.rs:58`) and
  `source:http` in **two spellings** — `SourceCapture::trace_payload`
  (`<url> fetched_at=… sha256=… cached=…`, `source_fetch.rs:207`) and the
  coding synthesis runtime's semicolon form
  (`url=…;fetched_at=…;sha256=…;catalog_match=…`,
  `coding/synthesis_runtime.rs:69,188`). The log is per-call and projected
  into `--thinking` narration and `evidence_links`, never persisted by
  answer id.
- `VerifiedAnswer` (`rust/src/solver_handlers/verifiable_task.rs:30`) already
  carries the right `derivation_id` convention — content-addressed,
  `verifiable_task.rs:606` — but only on the verifiable-task route. The
  obligation ledger already keys checks as
  `"<VerifiedAnswer::derivation_id>:<slug>"` (`obligation_ledger.rs:53`).
- `SymbolicAnswer` (`rust/src/engine.rs:55`) carries `evidence_links` as
  untyped strings and no stable id, so nothing outside the verifiable-task
  path could be keyed.
- No `Explain` CLI variant existed; the word "explain" is already the
  in-chat self-explanation recipe (`agentic_coding/explain.rs`,
  `EXPLAIN_KEYWORDS`), which must not regress.

## Plan

1. Project, don't duplicate: the record is a second projection of the *same*
   `EventLog` the thinking trace curates, so the two cannot disagree (R6).
2. Name the missing stages now (`formalize:fragment`, `decompose:part`,
   `recompose:bind`, `render:emit`, `verify:evidence`) as public constants,
   so the E128/E129/E131/E132 landings append one vocabulary.
3. Persist one lino file per answer under `data/cache/derivations/` (the
   `sources-registry` `cache_path` convention), keyed by the content-addressed
   answer id; absent stages are omitted from the file and printed as
   "not recorded" (R5).
4. Add the `Explain` CLI subcommand distinct from the self-explanation
   recipe: it takes a `answer_<16 hex>` id and reads the store, it never
   inspects the running codebase (R7).

## Delivery

| Piece | File |
| --- | --- |
| Record projection, lino round-trip, durable store, explain renderer | `rust/src/derivation.rs` |
| CLI subcommand runner (`--format text` / `--format links`) | `rust/src/cli_explain.rs` |
| `Explain` variant + dispatch | `rust/src/main.rs` (`mod cli_explain`, `Command::Explain`, match arm) |
| Record shape and stage vocabulary as data | `data/seed/derivation-schema.lino` (+ embedded mirror) |
| Tests (projection, id stability, "not recorded", thinking agreement, round-trip, R7 disjointness, quoting) | `rust/tests/unit/issue_1184_derivation_records.rs` |

Integration sites left for the wiring pass (each one line, listed so the
wiring cannot miss them):

- `rust/src/lib.rs`: `pub mod derivation;` (the test and `cli_explain.rs`
  import `formal_ai::derivation`).
- `rust/src/engine.rs`: add `pub derivation_id: String` to `SymbolicAnswer`
  (with `#[serde(default)]` so old serialized answers still parse), set at
  construction via `derivation::answer_derivation_id(&answer)`; this also
  delivers R8 (Serve JSON) because the struct derives `Serialize`.
- Persist hook: the answering surface that owns the `EventLog` at return
  time calls `Derivation::record_for(&log, &answer.derivation_id).persist(root)`
  so records exist for `explain` to read.

## Honest residuals

- R9 three-roots parity (js/ts translation + `cross-runtime-synthesis.json`
  cases) is the parity pass's follow-up; the shape is declared as seed data
  so the translation has one contract.
- The live online variant of the test (fetch, answer, `explain` the id)
  lands with the E128 (#1163) stage events it cites.
- No local build or test run was performed for this shard (fleet
  constraint); CI carries verification.
