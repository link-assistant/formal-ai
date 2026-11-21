# Execution-budget annotation line boundaries

A child can finish a write without a newline. Previously the wrapper appended
its completion message, warning, or error directly to those bytes. GitHub then
did not see the annotation delimiter at the start of a line. NUL-terminated
stdout also joined the completion message.

Run `node --test --test-concurrency=1
experiments/issue-1580/budget-annotation-boundaries.test.mjs` to reproduce three
finite cases. The completion child writes a NUL; the warning child writes partial
stderr; the deadline child disables the warning threshold so an earlier warning
cannot accidentally provide the error's missing boundary. All three fail against
the saved pre-change wrapper and pass after the repair.

Each annotation now uses one printf call with a leading newline. Completion
messages also start a fresh line. This keeps streaming output and the existing
whole-process-group lifetime, grace, termination-acknowledgement and exit-status
behavior. It does not introduce a relay process or output spool files.

The maintained template-observation adapter passes 116 cases, including existing
clock and child-process lifetime controls. The final strengthened three-case
test passes separately. Shell syntax, readable-code, file-size and abbreviation
gates pass. A gate initially rejected an abbreviated registry-manifest evidence
filename; only that stored filename and its manifest reference changed. The
initial failure and final passing output are retained.

The upstream JavaScript template independently implements fresh-line handling;
its relay architecture is not needed for this local repair. Hosted CI for the
new commit and all wider issue #1580 acceptance criteria remain unfinished.
