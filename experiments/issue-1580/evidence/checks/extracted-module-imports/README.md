# Correct imports after the response and chat-message extractions

The existing hosted native lane reproduced the failure at published head
`d48b09883ac91cd1d2319bea2afbaf0df2df8f86`, run
[44843761970](https://github.com/link-assistant/formal-ai/actions/runs/44843761970).
Its original log is retained in `failed-run-full-log.stdout.gz`.

The seven remaining `seed.rs` call sites still need `parser::split_pipe_list`
(first error at original log line 42913). The response child already imports
its own copy of that name. Moving the chat-message type also leaves
`ThinkingStep` unused in `protocol.rs` (line 42990); the child imports it.
Restore the first parent's private import and remove the second parent's stale
private import. Public exports, function bodies and child imports stay intact.

The earlier exact block inverses and token comparisons proved preservation of
the moved text. They did not prove that the parents' remaining imports were
correct. The original hosted compiler failure is the before observation;
passing compilation after repair still requires a new hosted run. No native
compilation was attempted locally. Rustfmt, projection currency and whitespace
checks passed. Supplied edits and scripts receive zero independent authorship
or reasoning credit. This does not complete issue #1580.
