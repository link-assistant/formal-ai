Part of the E127 umbrella (#1183); affects 22 of the 34 parity probes.

## Evidence

`rust/src/solver_handlers/web_requests.rs` (the `format!` arms around lines 240-300: default, latest-news, open-research, in en and ru) answers with a fixed paragraph: "Web search requested for `…`. In the browser demo formal-ai defaults to the DuckDuckGo Instant Answer endpoint … Provider: duckduckgo (default) …". Every arm only describes what "the browser demo" would do; none executes a search. In the CLI this is the final answer (22 of 34 parity probes, #1171): no search is executed, no page is fetched, nothing is answered. It is also the fallback for requests that are not searches at all (rewriting, classification, extraction, SQL, arithmetic).

## Root cause (verified on `main` d209aac64)

- The paragraph is built by `answer_web_search_query` (`rust/src/solver_handlers/web_requests.rs:196-300`). Its arms (default, latest-news, open-research; en and ru text) only describe what "the browser demo" would do.
- For the probed prompts it is reached from the **unknown-reasoning fallback**: when no handler claims a prompt, `rust/src/solver_unknown_reasoning.rs:109-123` picks a focus phrase and `return answer_web_search_query(prompt, focus, kind, log)` with `WebSearchQueryKind::UnknownReasoningFallback`. So every class without its own handler (rewriting, classification, extraction, SQL, arithmetic, poems, advice …) ends in a description of a search that never ran.
- A real executed search already exists: `rust/src/solver_handlers/web_requests/live_search.rs` (`try_web_search_with_client` → `crate::search_fusion::execute_search_fusion`, provider captures, RRF fusion, a `web_search_unavailable` answer when offline). It is only reached from `solver_dispatch.rs:209` for prompts whose intent is explicitly "search the web".

## Requirements

- **R1** The unknown-reasoning fallback runs the executed search (`try_web_search_with_client` / `execute_search_fusion`) for its focus instead of `answer_web_search_query`, then formalizes the fetched pages (#1163) and answers from them with citations; offline it answers with the existing `web_search_unavailable` response plus what the offline cache holds.
- **R2** `answer_web_search_query`'s descriptive text is no longer an answer anywhere; it survives only as `--thinking` narration of the planned search (or is deleted).
- **R3** A request that does not need outside knowledge never reaches the fallback: each class gets its handler through #1175 (routing) and the class issues (#1172, #1174, #1176, #1177, #1178, #1186).
- **R4** The answer states what was searched, which pages were read (URL + hash, through #1184's derivation), and which statements the answer rests on.
- **R5** Three-roots parity: the browser worker's fallback follows the same rule (translated code, parity case).

## Tests

- `rust/tests/unit/issue_1173_fallback_executes_search.rs`: with a mocked `SourceTransport` returning captured provider responses, the 22 probe prompts that returned the paragraph now produce either a class handler's answer or an executed-search answer with at least one source; none contains "Web search requested for".
- Offline: the same prompts with `offline = true` produce `web_search_unavailable` (or a class answer), never the paragraph.
- Command: `RUSTUP_TOOLCHAIN=1.98.1 cargo test --test unit issue_1173_fallback_executes_search`.

## Definition of done

- [ ] Requirement shard `docs/requirements/issue-1173-no-canned-search-answer.md`; `rust-script scripts/assemble-requirements.rs --write`; traceability rows.
- [ ] Case study `docs/case-studies/issue-1173/` with the 22 probe outputs before and after.
- [ ] Changelog fragment in `changelog.d/`; `RUSTUP_TOOLCHAIN=1.98.1 rust-script scripts/run-ci-gates.rs --stage rust` green.

## Depends on / blocks

- Depends on #1163 (E128) to turn fetched pages into statements the answer can use.
- Works with #1175 (E140): routing removes the misrouted prompts; this issue fixes what the fallback does for the prompts that remain.
- Blocks the class rows of #1171 (E136) that currently show the paragraph.

## What to do

- When a request needs outside knowledge, execute the search (the `web-search` dependency / `source_research.rs`), fetch and formalize the results (E128), and answer from them with citations. The browser path does the same through the worker.
- A request that does not need outside knowledge (rewrite this, classify this, compute this) must never fall to search; route by the formalized request (E131 (#1166), plan 10).
- The mechanism description may appear in `--thinking` output, never as the answer.

## How to test

The 13 affected probes from E136 (#1171) each produce an answer with at least one fetched source or a computation trace; none contains "Web search requested for".



