# Issue 509 Case Study

Issue: <https://github.com/link-assistant/formal-ai/issues/509>
PR: <https://github.com/link-assistant/formal-ai/pull/590>

## 1. Summary

Issue #509 asks FormalAI to extract information from dialog history. The issue
comment clarifies the requirement: natural-language queries should work over the
dialog history and the broader associative memory, using natural-language access
rather than SQL-like query languages.

The browser already had a local IndexedDB recall path for cross-conversation
queries, but the Rust solver path used by the library, CLI, and OpenAI-compatible
chat-completion adapter only handled narrow conversation memory requests: user
name recall, previous-question recall, and conversation summaries. Prompts such
as "When did I mention Rust?" therefore fell through to concept lookup or the
unknown fallback.

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
search request against those events. Because the current prompt still looked
like a concept question, the router could answer from the static Rust concept
record instead of the dialog history supplied by the caller.

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

The Rust adapter does not receive cross-conversation IDs, so "other
conversation" query forms are accepted and recorded as `other_conversations`
scope while searching the provided prior dialog turns. The browser keeps its
existing persisted IndexedDB search behavior for actual cross-conversation
memory.

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

Regression and seed verification:

- `cargo test --test unit solve_with_history_ -- --nocapture`
  (`raw-data/history-regression-after-fix.log`)
- `cargo test --test unit reference_closure -- --nocapture`
  (`raw-data/reference-closure.log`)
- `cargo test --test unit total_closure -- --nocapture`
  (`raw-data/total-closure.log`)
- `cargo test --test unit data_files -- --nocapture`
  (`raw-data/data-files.log`)
- `cargo test --test unit check_file_size -- --nocapture`
  (`raw-data/file-size.log`)
- `cargo test --test unit -- --nocapture`
  (`raw-data/unit-full.log`)

Final result: full unit target passed with 1207 passed, 0 failed, 1 ignored.
