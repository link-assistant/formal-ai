# Issue 509 Case Study

Issue: <https://github.com/link-assistant/formal-ai/issues/509>
PR: <https://github.com/link-assistant/formal-ai/pull/590>

## 1. Summary

Issue #509 asks FormalAI to extract information from dialog history. The issue
comment clarifies the requirement: natural-language queries should work over the
dialog history and the broader associative memory, using natural-language access
rather than SQL-like query languages.

The browser already had a local IndexedDB recall path for cross-conversation
queries, but the Rust solver path used by the library, CLI, and API adapters only
handled narrow conversation memory requests: user name recall, previous-question
recall, and conversation summaries. Prompts such as "When did I mention Rust?"
therefore fell through to concept lookup or the unknown fallback, and the local
HTTP server did not consult the persisted `FORMAL_AI_MEMORY_PATH` event log for
natural-language memory queries.

## 2. Collected Data

| Data | File |
| --- | --- |
| Issue metadata and comments | `raw-data/issue-509.json`, `raw-data/issue-509-comments.json` |
| PR metadata, comments, reviews | `raw-data/pr-590.json`, `raw-data/pr-590-comments.json`, `raw-data/pr-590-review-comments.json`, `raw-data/pr-590-reviews.json` |
| Before-fix reproducer | `raw-data/reproduction-before-fix.log` |
| Focused verification after the fix | `raw-data/focused-after-fix.log` |
| Existing history-regression verification | `raw-data/history-regression-after-fix.log` |
| Seed and size checks | `raw-data/reference-closure.log`, `raw-data/total-closure.log`, `raw-data/data-files.log`, `raw-data/file-size.log` |
| Full unit test run | `raw-data/unit-full.log` |

## 3. Root Cause

`solve_with_history` records prior chat messages as `prior_turn:user` and
`prior_turn:assistant` events, but no handler interpreted a natural-language
search request against those events. The HTTP server also opened the same
`SyncStore` used by `/v1/memory`, yet `/v1/chat/completions`, `/v1/responses`,
and `/v1/messages` routed requests directly through the solver without passing
those persisted `MemoryEvent`s into recall. Because the current prompt still
looked like a concept question, the router could answer from the static Rust
concept record instead of the dialog history or persisted memory supplied by the
caller.

## 4. Implemented Design

The conversation-memory handler was extracted to
`src/solver_handlers/conversation_memory.rs` to keep the main handler module
under the 1000-line source limit.

The new `conversation_recall` branch:

- recognizes recall prompts through seed roles in
  `data/seed/meanings-conversation.lino`;
- extracts the searched term from the seed ellipsis slot;
- scans `prior_turn:user` and `prior_turn:assistant` events using the shared
  prompt normalizer;
- records `filter:memory_query`, `filter:memory_scope`,
  `filter:memory_matches`, and `memory_match` evidence events;
- returns role-labelled matching turns, or a no-match answer, before concept
  lookup can claim the prompt.

The same recall recognizer now also has a persisted-memory path:

- `answer_memory_recall` scans portable `MemoryEvent` records from `demo_memory`
  or full-memory bundles, grouped by `conversationId` / `conversationTitle`;
- trigger prompts are skipped so a recalled search never matches itself;
- `other_conversations` excludes the current request-history group when the
  caller supplies one, while still searching the persisted memory store;
- `/v1/chat/completions`, `/v1/responses`, and `/v1/messages` pass
  `SyncStore::open().events()` into memory-aware protocol helpers before falling
  back to the normal solver;
- `formal-ai memory query --prompt ...` exposes the same natural-language recall
  directly over saved memory files.

## 5. Verification

Reproduction before the fix:

- `cargo test --test unit conversation_history -- --nocapture`
- Saved output: `raw-data/reproduction-before-fix.log`
- Result: failed. Recall prompts routed to `concept_lookup` or `unknown`, and
  the chat-completion path answered from the static Rust concept.

Focused verification after the fix:

- `cargo test --test unit conversation_history -- --nocapture`
- Saved output: `raw-data/focused-after-fix.log`
- Result: passed.

Persisted-memory verification after the expansion:

- `cargo test --test unit persisted_memory -- --nocapture`
- `cargo test --test integration cli_memory_query_answers_natural_language_recall_from_persisted_memory -- --nocapture`
- Result: passed. Coverage includes the reusable memory helper,
  `/v1/chat/completions`, `/v1/responses`, `/v1/messages`, and the CLI
  `memory query` command.

Regression and seed verification:

- `cargo test --test unit conversation_history -- --nocapture`
- `cargo test --test unit reference_closure -- --nocapture`
- `cargo test --test unit total_closure -- --nocapture`
- `cargo test --test unit data_files -- --nocapture`
- `cargo test --test unit check_file_size -- --nocapture`
- `npm --prefix tests/e2e run check:language-test-coverage`
- `npm --prefix tests/e2e run check:language-parity`
- `cargo fmt --all -- --check`
- `cargo clippy --all-targets --all-features`
- `cargo test --all-features --verbose`

Final result: full all-features test target passed with 1215 passed, 0 failed,
1 ignored.
