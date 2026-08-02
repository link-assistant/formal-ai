# Requirement-by-requirement solution plans

## Evidence integrity

Use the existing benchmark manifests as authoritative command/policy data. Run
each command once per proposal batch, parse explicit benchmark counts before
Cargo's one-test summary, hash status/stdout/stderr, and clone the immutable
evidence onto every decision. Never deserialize executable evidence from a
proposal. Command failure becomes blocking evidence; malformed success is a
protocol error.

## General learning and materialization

Keep `PromotionProposal` generic over source, summary, seed path, and Links
Notation payload. Coalesce edits by target, then submit the complete desired
file as a literal task to the existing E36 generalized agentic planner/driver.
Extract the non-plan `write_file` call and compare target/content exactly before
writing. This reuses the existing capability path instead of creating a
promotion-only file writer. Preserve the resulting deterministic session id.

## Review branch and outer gates

Require a clean Git worktree, create the content-addressed promotion branch, and
stop after local materialization. Provide commit/draft-PR commands but do not
push, open, merge, or claim future GitHub CI success. This preserves the issue's
explicit human-review boundary and makes remote mutation separately authorized.

## Regression strategy

Start with red tests for the observed terminal-quote corruption, injected
runner/counts, coding pass-rate regression, failed commands, malformed output,
unsafe paths, exact Agent bytes, local branch creation, dry run, confirmation,
and bundle durability. Use deterministic command doubles in CLI integration;
retain a separate real canonical replay and external Agent CLI example as
real-world evidence.
