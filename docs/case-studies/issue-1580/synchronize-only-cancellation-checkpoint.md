# Preserve independently requested workflow observations

Current Hive Mind guidance reserves cancellation for pull_request synchronize
checks. The previous 31 workflow sources contain 75 cancellable workflow/job
scopes that can discard independently requested manual, ready-for-review or
push observations. The retained pre-fix control fails the actual event matrix.

Each of those scopes now cancels only a synchronized pull request. Other events
gain the run ID in their existing group. Original workflow/job/matrix identities
remain intact, and shared writers retain their non-cancelling queues. The
companion concurrency checker enforces both requirements in the existing
workflow linter lane. Negative controls reject broad PR cancellation, literal
true, missing suffixes and suffixes that still conflate independent events.

The actual 75 source scopes pass 1,200 event/group assertions across eight event
forms. All affected JavaScript observers and Rust formatting pass. Native
workflow observer execution remains pending hosted CI. The initial retry with
an inherited NODE_OPTIONS value was correctly refused by the compiler observer;
the clean retry passes and both observations remain preserved.

The canonical original release workflow receives only matching policy edits.
The official deployed-mode generator verifies the real caller, all 52 original
operations and their transfer bindings. Serialized release-staged.yml is byte
identical. A rejected wrong-mode attempt remains preserved and earns no credit.
The complete maintained observation suite passes 186 cases; full local gate
results will be recorded separately when that worker finishes.

Supplied implementation earns zero independent authorship/reasoning credit. The full issue remains unfinished.

PR: https://github.com/link-assistant/formal-ai/pull/1581
