# Issue #1587: request constraints and dialogue facts

Issue: <https://github.com/link-assistant/formal-ai/issues/1587>
Pull request: <https://github.com/link-assistant/formal-ai/pull/1595>

The initial branch contained development logs and a placeholder PR, with no
implementation. All issue comments, PR conversation comments, inline comments
and reviews were inspected. The only issue comment concerned an earlier
unverified Formal AI session; there were no additional product requirements or
required sub-issues. The related #1588 fix was merged from main before drafting.

Both protocol adapters extracted the last user prompt and earlier user/assistant
turns. System text was used only to remove an echoed prompt; developer messages
were dropped. History reached the solver, but ordinary personal assertions had
no general retrieval route. This explains the language/label examples and the
Wikipedia definition returned for the colour question.

Reproduce with a fresh server using `POST /v1/chat/completions`:

```json
{"messages":[{"role":"system","content":"Always answer in French."},{"role":"user","content":"What is 2 + 2?"}]}
```

Before the fix this returned `2 + 2 = 4` without acknowledging the system
constraint. The retained before-test fails on that exact behavior. Now it
returns `Cannot satisfy language constraint (Always answer in French).`
French is not a supported response language in this grammar. The other original
cases produce `2 + 2 = 4` under the English instruction, `yes`, and `green`.

The request context formalizes supported English/Russian instruction grammar
from shared data. System constraints outrank developer constraints; the last
constraint of a kind at the same rank wins. Unsupported output clauses and
unparsed qualifications are refused with the source instruction included.
Supported English/Russian language constraints use the existing scoped solver
override. JSON uses an object with an `answer` string. Labels and word limits
are validated against the composed answer; impossible combinations get an
explicit refusal. Binary comparisons use exact decimal-string ordering, so
large integers and close decimals cannot silently round to the wrong label.

Dialogue facts are possessive assertions such as “My favourite food is lentil
soup.” Retrieval matches normalized subjects, takes the latest user assertion,
and reports its history turn. Assistant and tool text cannot supply user facts.
A supported recall question with no matching assertion is reported as unavailable
rather than replaced with a general definition. This is a bounded assertion/recall grammar, not a
claim of unrestricted language understanding or arbitrary JSON-schema support.

| Acceptance | Implementation and verification |
| --- | --- |
| All four original examples and held-out variants | Shared fixture `data/parity/issue-1587-request-context.json`; exact answers or explicitly named refusals in both runtime tests |
| System/developer language, format, labels, length | Shared `data/meta/request-context.json`; source role/index and disposition in `request_constraint` steps |
| Earlier user facts | Original colour case, different subjects, multiple facts and misleading assistant text; `request_fact` provenance |
| Request isolation | Concurrent and repeated calls, memory-recording test, unchanged ten-test #1559 suite; no new persistent store |
| All final answer paths | Chat Completions, Responses `instructions`, retained/refused symbolic answers and agentic final answers compose constraints |
| Runtime parity | Generated TypeScript; shared fixtures registered in the native protocol unit suite; execution remains unverified under the main-only native-test policy |

Local verification uses focused JavaScript tests and all JavaScript local gates.
The new regression was observed failing before integration and passing afterward.
Native compilation/testing is reserved for hosted CI; the repository policy
permits only native type/lint checks on PRs and executes native tests on main.
Current-head CI results and immutable run links
are recorded on the PR; a passing type check alone is not native execution proof.

Formal AI was used for inspection and supplied-command execution. Its unsupplied
implementation attempt returned an unsupported-task answer without editing.
The implementation is externally authored, with zero accepted independent
reasoning or authorship credit. The frozen eight-unit plan, complete failed
attempt, supplied procedures and verification transcripts are retained under
`experiments/issue-1587/`. The driver timed out a combined test command at its
60-second boundary; the unchanged isolation suite then passed when run alone
without imposing that driver deadline.

The final local gate run completed all 138 entries. Its two failures were the
abbreviation gate (a retained log filename, subsequently renamed and rechecked)
and the dependency audit. All executable local gates now pass except that audit.
The unchanged desktop and VS Code lockfiles contain picomatch versions affected
by [GHSA-fc5q-39xc-vjpf](https://github.com/advisories/GHSA-fc5q-39xc-vjpf).
On 2026-11-17 the advisory listed no patched release and the registry's latest
was 4.0.7. The gate remains closed without suppressing the advisory or replacing
an unrelated dependency with an unreviewed implementation. Green CI and native
runtime execution remain unfinished acceptance evidence.
