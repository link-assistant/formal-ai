# Released-client progress and placeholder cleanup

[PR 1410](https://github.com/link-assistant/formal-ai/pull/1410) is merged.
Its source `ffd74428f328c174f85ad11c0d0fa24f63ce4ee8` produced v0.355.10.
The main producer [40203624738](https://github.com/link-assistant/formal-ai/actions/runs/40203624738)
and every applicable downstream publisher and public-download verifier passed.
Anonymous manifest checks independently verified the published full/slim Docker
images on both architectures and their current default aliases. Public data
receipts also verified the native archives, both macOS ad hoc signatures,
browser/hosted-worker distribution, crate and GitHub extension package. Optional
npm/extension marketplaces remain explicitly unconfigured, with no publication
credit. No local Rust compilation or product download/execution was used.

The fresh released-client owner
[40222223625](https://github.com/link-assistant/formal-ai/actions/runs/40222223625)
failed. The shipped native binary, three actual client/MCP preflights and four
memory/shutdown lifecycle cycles passed. Actual task acceptance did not:

- Agent job `121262646192` reached its 540-second first-pass deadline while
  still making progress. Its nine completed model requests wrote the source,
  verifier and staged workflow, then observed the missing local Scala compiler.
  Individual HTTP responses took 48–51 seconds.
- Codex job `121262646267` reached the same deadline after committing/pushing
  the Rust task and obtaining an exact-head staged-readiness receipt. That
  receipt explicitly grants no hosted-CI credit. Model responses took about
  29 seconds, and the PR was still a draft.
- Claude job `121262646170` completed without a timeout. The original harness
  observed a ready Kotlin PR, correct output, successful workflow execution
  with `act`, and a final title/body. It rejected the remaining `.gitkeep`
  placeholder in the diff.

The follow-up uses the shared meta-language inventory/hygiene programs to
remove only a tracked root bootstrap placeholder once every recipe source file
exists, is nonempty and decodes as UTF-8. An empty file qualifies; a nonempty
file must match Hive's complete generated metadata template, a valid timestamp,
the exact requested issue URL and the client's observed Git branch. The actual
failure file has 174 bytes, Git blob `7a9c542bcb875ce677c30723bf1bb50601f727e2`;
the initial empty-file-only draft was insufficient and is corrected before merge.
Inventory grants scope only after complete regular-file reads and retains the
full observed placeholder text. Hygiene repeats the source/template checks,
requires that exact preimage, checks the fresh branch and rechecks file identity
immediately before removal. Unrecognized nonempty files, untracked placeholders,
nested placeholders, symlinks, unrelated source and documentation never gain
deletion authority. Changed placeholder content or missing replacement source
refuses cleanup. Rust and JavaScript use the same source programs; TypeScript is
generated from JavaScript.

Native tool classification previously reloaded the immutable capability
registry for each candidate capability. The follow-up caches that registry
and reads it once per tool classification, matching JavaScript's existing
cached-data behavior. WASM does not retain native allocations across allocator
epochs. This caches installed vocabulary, never task answers or tool outcomes.
Captured public requests now exercise the JavaScript and native protocol paths
under a finite pure-planning latency budget. Native request phase diagnostics
identify grounding, contract projection and planning time when request tracing
is enabled. Native performance and full delivery remain to be demonstrated by
CI and a fresh released first-attempt owner; local JavaScript success grants
neither native performance nor real-client acceptance credit.

The public Codex export removed part of a tool schema as account metadata
because the shell schema contains a `login` option. The regression retains
that loss and requires refusal instead of reconstructing unavailable evidence.
Improve schema-aware export separately, retaining personal-data redaction.
Agent's npm-based MCP availability probe also printed an unsuccessful `npx`
lookup before accepting the global package; this warning is recorded separately
and is not established as the cause of its coding timeout.

The failed owner must receive a terminal authenticated cleanup outcome.
GitHub limits `workflow_run` chaining to three levels
([official documentation](https://docs.github.com/en/actions/reference/workflows-and-actions/events-that-trigger-workflows#workflow_run));
cleanup following the release → download verification → original-delivery
chain therefore also depends on the bounded scheduled discovery fallback.
That workflow is active. Its last scheduled run completed discovery without
effects while this owner was still running. A single fresh discovery of that
proven effect-free preparation was requested after the owner became terminal;
the immutable-claim checks still prohibit retrying any uncertain cleanup claim.
That discovery selected the now-terminal owner, but cleanup retained all three
fixtures: the PR list endpoint omits the `merged` boolean and the old guard
required it there. Outcome artifact `12612169513` records zero closes and three
retentions under immutable claim `12612284304`. The follow-up reads the fresh
PR detail and binds its number, branch and head to the list observation before
requiring explicit `merged: false` and `merged_at: null`. Missing detail fields,
a changed head and actual merges refuse before any write. The old claim stays
quarantined and receives no success credit or automatic retry.

Issue 1364 stays open. Preserve every failed receipt. Do not rerun the effectful
original workflow. Merge only after exact-head CI passes and the preceding
owner/cleanup is terminal, then verify the new publication and all three fresh
first-pass deliveries, no-op second passes, independent output checks and
trusted cleanup. Broader deferred requirements retain their existing status.

Before the first push, 104 targeted JavaScript regressions passed: 27 source
inventory/restart/captured-request tests, 19 source/recipe parity tests and 58
cleanup/custody tests. Maintained generators verified all 660 TypeScript copies,
468 embedded data members and the native specification registry. The changed
JavaScript/TypeScript scope introduced zero duplicate groups against its real
base; existing repository duplication debt is not claimed fixed. Rust source
formatting passed. Native compilation, native latency and new released-client
acceptance are reserved for CI.
