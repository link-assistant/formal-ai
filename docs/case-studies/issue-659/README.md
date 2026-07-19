# Issue 659: hardcoded-language burn-down and self-hosted learning

Issue [#659](https://github.com/link-assistant/formal-ai/issues/659) asks for a
ratchet that prevents new hardcoded natural language in Rust production code
while allowing the existing debt to be removed incrementally. Pull-request
feedback then asked Formal AI to execute the task through an Agent CLI, retain
what it learned, and demonstrate that the implementation generalizes beyond
one prompt. The requirement-by-requirement mapping is in `requirements.md` and
the complete GitHub records are preserved under `raw-data/github/`.

## Root cause and reproducer

The initial scanner classified a literal only when it contained whitespace and
ended in sentence punctuation. That correctly found 774 existing sentence
literals after merging current `main`, but it missed unpunctuated user-facing
phrases such as `format!("Try again")`, `.push_str("Please wait")`, and
`return "Need details"`. A test containing those three exact output positions
failed with an empty detection set before the implementation changed.

The scanner now retains lexer context. Punctuated prose is detected globally;
an unpunctuated multi-word literal is detected only in an output-producing
position (`format!`, `push_str`, or `return`). Negative fixtures exclude code
fragments and internal value lists. This exposed another 548 existing rows, so
the committed baseline contains 1,322 sorted path-and-literal entries. Both set
differences are errors: an unlisted detection is new debt, and an allowlist row
with no matching source is stale debt. The gate therefore cannot be passed by
silently growing the baseline or hiding completed migration.

The issue's exact `Sorry, I can't do that.` fixture fails check mode. The same
fixture passes only after an explicit allowlist row is supplied, and then fails
again when that source disappears. Inline `rust-script` tests preserve this
new-debt/admitted/stale sequence.

## Real migration

Eight English fallback literals in `src/engine_responses.rs` duplicated values
already grounded in `data/seed/multilingual-responses.lino`. The implementation
now uses the existing `seed::response_for` lookup instead, removing those rows
from source rather than merely admitting them. CI and the contributor checks
run `rust-script scripts/check-hardcoded-language.rs` on every change.

## Formal AI executing issue 659's task

The observed failure modes, baseline counts, and migration result are persisted
as an associative Links network in
`data/meta/issue-659-hardcoded-language-learning.lino`. The generalized
learning-report descriptor table ranks that network and derives
`hardcoded-language-learning-report.lino`; there is no issue-specific branch in
the planner.

`experiments/agent_cli_e2e/run_issue_659_learning.sh` starts Formal AI's HTTP
server and supplies it as the model provider to both the real
`@link-assistant/agent` and `opencode` CLIs. The live task deliberately uses
different wording from the in-process regression. Each CLI must write the full
report, each report must contain the persisted observations and lessons, and
the two files must be byte-identical. Their streams, diagnostics, server trace,
and derived artifact are committed under `agent-cli-evidence/`; CI repeats the
same command.

The report remains `awaiting_human_review` and does not promote its own lessons.
Its named promotion gate is
`hardcoded_language_fixture_context_gate_and_agent_cli_e2e_pass`. A unit test
also compares the committed Agent CLI artifact with the current renderer byte
for byte, so code or evidence drift cannot pass unnoticed.

## Verification

The focused verification sequence is:

```sh
rust-script --test scripts/check-hardcoded-language.rs
rust-script scripts/check-hardcoded-language.rs
cargo test --test unit issue_659_hardcoded_language_learning -- --nocapture
cargo build --release --bin formal-ai
experiments/agent_cli_e2e/run_issue_659_learning.sh
```

Red and green test logs are retained in `raw-data/verification/`. The external
transcripts are evidence of the network boundary and real CLI tool loop; the
byte-reproducibility test connects that live evidence back to the deterministic
implementation.
