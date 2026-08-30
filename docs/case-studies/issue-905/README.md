# Issue 905: tool-result evidence gates completion

Issue [#905](https://github.com/link-assistant/formal-ai/issues/905) showed
two external Agent CLI sessions failing to create `hello.txt`, then reporting
that the change was complete and verified. The request also exposed a parsing
defect: `containing exactly: Hello World` became the payload
`exactly: Hello World`.

## Evidence inventory

Authenticated snapshots under [`raw-data/`](raw-data/) retain the complete
issue, its comments, PR 927 metadata, all three PR comment streams, and the two
original provider logs. The issue and comments contain no image attachments;
this is a protocol/state defect, so there is no visual before/after evidence.

[`test-evidence/regression-red.log`](test-evidence/regression-red.log) is the
before-state: six focused assertions fail on the parser, failed-write recovery,
nonzero/explicit failure handling, and evidence mismatch. The green and related
suite logs in the same directory preserve the after-state.

## Root cause

Three independent mistakes formed one dishonest state transition:

1. The content parser chose the shorter `containing …` marker when
   `containing exactly …` began at the same byte, leaving `exactly:` in the
   payload.
2. `Progress::scan` appended every tool result to `completed` regardless of
   structured status, explicit `is_error`, or provider-owned failure text.
   Anthropic and Responses adapters also discarded their error metadata.
3. The general planner emitted its success sentence after the expected number
   of calls. It never required a successful verification result or compared
   observed output with the request-derived expected bytes.

## Implementation

`ChatMessage` now preserves `is_error` (including the `isError` alias), and the
Anthropic and Responses adapters project their native error fields into that
shared signal. Tool-result normalization recognizes explicit error envelopes,
nonzero exit/status fields, and raw nonzero exit markers.

Progress separates observed attempts from successful step satisfaction. On the
first rejected write for a concrete path, the general planner reads that
path and retries the write once; a second failure for the same path terminates
with the observed error. If a write-only client rejects the auxiliary plan
file, the planner still attempts the user's primary file once. Failed
verification terminates honestly. Successful
literal-file verification reaches completion only when normalized stdout equals
the request-derived content. Output vocabulary alone is not a failure signal,
so a user may legitimately request the exact bytes `failed` or `error`.

The seed now includes the longer `containing exactly …` surface, and equal-start
markers select the longest match. Mismatch and unverified responses are seeded
for English, Russian, Hindi, Chinese, and Spanish.

## Reproduction

Run the focused and neighboring checks with:

```sh
cargo test --test unit issue_905 -- --nocapture
cargo test --test unit agentic_general_planner -- --nocapture
cargo test --test unit issue_680 -- --nocapture
cargo test --test unit issue_681 -- --nocapture
cargo test --test unit agentic_surfaces -- --nocapture
python3 scripts/audit-total-closure.py
```

The focused suite covers the exact reported prompt, twenty varied parser
phrases, explicit Qwen-style errors, Codex-style nonzero status, bounded retry,
wrong evidence, matching evidence, failure vocabulary as requested content, and
Anthropic/Responses protocol parity.

## Same-task self-application

The five reviewed smallest leaves are: exact-marker parsing, protocol metadata
preservation, success-only progress accounting, bounded read/write recovery,
and the evidence-honesty invariant. Formal AI served the `formal-ai` model to
the external Agent CLI for the fifth leaf. Session
`ses_034e9dafeffe7nxeTkFhmHLmZN` wrote and read back
[`tool-result-evidence-invariant.lino`](self-hosting-authorship/tool-result-evidence-invariant.lino)
in four chat rounds. The generated artifact is byte-for-byte equal to the
canonical [`data/meta` invariant](../../../data/meta/tool-result-evidence-invariant.lino).
This is one of five leaves, or 20% same-task Agent-CLI authorship.

The replayable harness is
[`experiments/issue_905_self_authoring/run.sh`](../../../experiments/issue_905_self_authoring/run.sh),
and raw client/server traces are under
[`self-hosting-authorship/`](self-hosting-authorship/).

Moving the general-change state machine into its own size-compliant module also
changed the planner-derived self-healing fixture, and issue #1066 has moved code
out of `src/agentic_coding/planner.rs` and back into it several times since, so
the link counts the fixture pins describe a file that keeps changing. Session
`ses_faf5f322effeicCLioH0QBuTdQ` reran the canonical self-healing recipe through
Formal AI and the real Agent CLI, wrote the refreshed fixture, and read it back
successfully. The byte-identical artifact and trace are retained under
[`self-hosting-fixture-refresh/`](self-hosting-fixture-refresh/); the replay
harness is
[`experiments/issue_905_self_healing_refresh/run.sh`](../../../experiments/issue_905_self_healing_refresh/run.sh).

An earlier whole-issue `solve` attempt is retained separately under
[`self-hosting-attempt/`](self-hosting-attempt/). It selected an unsupported
hosted fallback model and received HTTP 401 before Formal AI executed. It is
documented as failed infrastructure evidence and is not counted as authorship.
