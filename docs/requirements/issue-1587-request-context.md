## Issue #1587 Request Constraints and Dialogue Facts

Source: [issue #1587](https://github.com/link-assistant/formal-ai/issues/1587).
The four original reproductions and held-out variants are shared fixtures in
`data/parity/issue-1587-request-context.json`. Unsupported constraints must be
named explicitly; the issue permits this outcome instead of an incorrect answer.

| ID | Requirement | Status / Evidence |
| --- | --- | --- |
| R1587-1 | Formalize system/developer messages as request-local output language, format, label and length constraints; compose these with the last user task, or explicitly name an unmet constraint. | Implemented in `js/server/request-context.mjs` and `rust/src/protocol/request_context.rs`, with shared grammar/prose, role precedence, supported language overrides and final-answer validation. Exact-answer and refusal fixtures run in `rust/tests/web/issue-1587-request-context.test.mjs` and `rust/tests/unit/protocol/issue_1587_request_context.rs`. |
| R1587-2 | Retrieve facts from earlier user turns of the supplied dialogue, preserving provenance and excluding assistant claims. | Implemented: possessive assertions are matched by normalized subject; latest matching user fact wins, including multiple statements per turn. The shared original-colour, food, multi-fact and latest-fact cases exercise retrieval. Missing facts produce an explicit unavailable-context answer. |
| R1587-3 | Keep request constraints/facts isolated; preserve issue #1559 standing-requirement scoping. | Implemented with local context values and the existing scoped worker override (a copied native solver configuration). The JavaScript test exercises concurrent requests, persistent memory recording and agentic finals; native tests repeat constrained/unconstrained requests. Existing `rust/tests/web/issue-1559-standing-requirement-scope.test.mjs` remains unchanged. |
| R1587-4 | Expose applied, superseded and unmet system constraints in thinking_steps and regress all four reports in both runtimes. | Implemented: each constraint records role, message index, kind, disposition and source text; recalled facts name their history turn. Both runtime tests consume the same fixture. JavaScript execution passes; native tests are registered in the existing protocol unit suite, but native execution remains unverified under the main-only native-test policy. |
