# Issue 712: phrasing-general tool routing

Issue [#712](https://github.com/link-assistant/formal-ai/issues/712) reported four classes of requests that worked only with a narrow verb: URL fetching, web search, file editing, and declarative file creation.

## Reproduction

The Agent CLI was connected to a local release-mode `formal-ai serve --agent-mode` endpoint. Formal AI authored `tests/issue_712.rs`; its first run is preserved in [`red-regression.log`](red-regression.log). All four tests failed on v0.289.0: each planner result was `None` except the new-file request, which emitted `read`.

The raw self-coding sessions are preserved as:

- [`agent-create-regression-test.jsonl`](agent-create-regression-test.jsonl)
- [`agent-create-fix.jsonl`](agent-create-fix.jsonl)
- [`agent-apply-fix.jsonl`](agent-apply-fix.jsonl)

The model-authored production patches are replayable from:

- [`experiments/issue_712_intent_routing.patch`](../../../experiments/issue_712_intent_routing.patch)
- [`experiments/issue_712_edit_correction.patch`](../../../experiments/issue_712_edit_correction.patch)

## Root cause

- The advertised fetch capability only accepted explicit HTTP-fetch cues, even though URL navigation was already recognized elsewhere.
- Search intent lacked the reported `google`, source-oriented `say`, and request-oriented `need` cues.
- The edit parser required a file target cue such as `in`; a leading edit action adjacent to a path was not accepted, and the first action could describe the file rather than the replacement clause.
- The write parser did not recognize `contents:`, allowing the file-read router to claim the request.

## Verification

The fix adds the missing semantic surfaces and makes the parsers reuse their existing structural evidence. [`green-regression.log`](green-regression.log) records the focused unit matrix passing. `tests/integration/issue_712_intent_routing.rs` additionally boots the real server and covers Chat Completions, Responses, and Gemini, including a whole-issue matrix. The release workflow drives `new file: notes.txt, contents: hello` through the real Agent CLI and asserts that `notes.txt` is written with `hello`.
