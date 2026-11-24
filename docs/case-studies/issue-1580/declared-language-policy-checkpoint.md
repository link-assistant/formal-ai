# Declared answer languages and unresolved JavaScript requests

Issue: https://github.com/link-assistant/formal-ai/issues/1580
Pull request: https://github.com/link-assistant/formal-ai/pull/1581

The original unchanged native Spanish fixture distinguishes an adopted definition
frame from an unresolved operative question. Its complete how-question response
includes the unresolved focus, shared guidance and failure-report invitation.
After the localized opener correction, the production JavaScript worker still
returned the generic localized teaching response for that question.

## Cause and change

`data/seed/agent-info.lino` declares the answer languages as a reference list.
The native terminal reasoning path checks this declaration; the JavaScript helper
previously accepted only the detector's `unknown` language. Detection recognizes
Spanish, which is absent from the answer-language declaration, so it bypassed the
helper and lost the native fixture's complete unresolved-reasoning response.

The helper now reads the declaration with the maintained Links Notation list
parser. A language outside that declaration can reach the existing terminal
unresolved response after earlier handlers and source research decline. No
language identifier or test prompt is added to production routing. Whole-prompt
focus delimiters follow the native quote and terminal-punctuation handling, and
the trace keeps the actual detected language. The helper continues to distinguish
observed local matches from verified answers.

## Reproduction and verification

Run the focused file separately:

```sh
timeout 600 node --max-old-space-size=256 --test --test-concurrency=1 rust/tests/web/worker-unknown-language.test.mjs
```

The new full-response regression extracts both inputs and the complete expected
body from the unchanged native fixture, rather than replacing it with a shortened
expectation. A held-out control changes the declared metadata while keeping the
same language value. Before the production change, six cases passed and both new
cases failed. After the change, all eight cases pass, including Persian, Arabic,
empty-input, greeting, unverified-memory and localized opener controls.

The complete before/after logs and command outcomes are retained under
`experiments/issue-1580/evidence/checks/declared-language-policy/`. The before log
is 5,121 bytes, SHA-256 `13e633d926000ed1d06f292027540c720aa1936b36f7f0e3104a4c3e158b1db2`;
the after log is 884 bytes, SHA-256 `db6f0196c9a76dcd1c2f69ad98ceb7a1a9eb7d7963f0918db3191ecfebc7ae11`.
Both complete logs were read. The initial nine-gate publication run passed
eight gates and rejected the first 38-line draft against the unchanged 33-line
worker ceiling. Removing single-use bindings and retaining the prior compact
style restores 33 lines without changing the limit. The eight cases pass again
on this final source; its complete log is 885 bytes, SHA-256
`c60ff6f4c6b8c1702c2ec37684c016675f738aff55f8f3373eede40fa9ea28cd`.
All six affected source gates pass after that refactor. No `data/meta` input
changes, so the full inventory is not repeated. JavaScript/Rust projection
verification passes with 49 refused modules; it does not establish full parity.

## Published-head CI and limits

On edffcb18, all 23 workflows completed: 22 passed and Layered CI failed at
checkout in JavaScript suite group 2. Direct job 134552599041 has 171 lines,
all read. Lines 124–126 show the configured low-speed abort, bad pack header and
promisor fetch failure for the `.bun-version` blob. No application test ran in
that job. The aggregate run log, paginated job inventory and complete direct job
log are retained under `experiments/issue-1580/evidence/ci/opener-native-boundary-edffcb18/`.
One failed-job retry was requested after preserving the original attempt.
Checkout and both JavaScript suite groups pass on attempt 2. The underlying
transport cause remains unverified; no retry loop is added.

Attempt 2 instead fails in TypeScript projection job 134558393446. All 320
direct job-log lines were read. File-set parity passes at line 290; line 302
identifies the stale `formal_ai_worker_seed_responses_and_language.ts` mirror,
and the maintained translation check fails at line 304. Its complete raw log is
24,798 bytes, SHA-256
`0683ff1370cdcc53b75bee48cff6f5904c1bc68b61611c851a29f7432e1184fa`.
Both attempt inventories, aggregate logs and direct failed logs remain separate.

The maintained `scripts/translate-es.mjs --check` locally reproduces that drift
and the additional mirror drift from this new language-policy change. Its write
mode regenerates exactly those two TypeScript files. The final check passes all
717 maintained projections, and both changed mirrors equal their JavaScript
sources byte for byte. The complete before/write/after logs are retained under
`checks/declared-language-policy/type-projection/`. Hosted verification of the
next head is still required.

The preceding publication metadata assertion failed because it stripped only the
local body's terminal newline. Fresh verification confirms the exact GitHub body
and remote head; this was not a missing push or description update.

No native application build or test was run. The original adopted Spanish
definition route, the native full-response regression and the wider native,
template, upstream, benchmark and independent-reasoning requirements remain
unfinished. Supplied scripts, tests, patches and conclusions receive zero
independent authorship credit. Keep PR #1581 draft and the goal active.

## Final publication checks

All five final release, naming and size gates pass after the maintained mirror
regeneration. The 717-file TypeScript projection check and whitespace check pass
again; all complete summary logs are read and retained. Main 015a6831 is already
merged. The 03:50 UTC feedback refresh finds no changed issue comments, PR
conversation comments, inline comments or reviews, and no required sub-issues.
Every edffcb18 workflow is completed, both failed attempts are collected, and
the active and queued run lists are empty. The next head still needs CI.
