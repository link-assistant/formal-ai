# Issue 918 Case Study

Issue [#918](https://github.com/link-assistant/formal-ai/issues/918) asked for
an explicit minimal compiled core, a decision for every specialized handler,
and problem-solving metadata rich enough to move domain behavior into links.
The result is an enforceable boundary and an honest, data-backed burn-down
baseline rather than a claim that the remaining migration is complete.

## 1. Collected Data

`raw-data/github/` preserves issue #918 and parent #914, the prepared PR #986,
all issue and PR feedback channels, recent repository activity, and PR #877—the
most recent handler-migration precedent. `raw-data/online-research.md` records
the primary semantic-data references used for the metadata shape. There were no
screenshots or image attachments in the issue or its comments.

## 2. Requirements

The complete mapping is in `requirements.md` and root `REQUIREMENTS.md` as
R918-1 through R918-6. It covers the boundary, exhaustive decision ledger,
metadata schema, coding regression floor, per-record gaps, and reproducible
self-hosting evidence.

## 3. Reproduction And Root Cause

The issue estimated 40 handler files. Recursive enumeration found 46 recursive handler sources
and 19,731 total lines: the existing #699 gate counted only
top-level `.rs` files and its method ledger did not constitute a source-level
decision for nested modules. Consequently a new nested handler could evade the
architectural census.

The seed had 3,484 direct concept records rooted at `meanings`. The 37 coding-path concepts
all had roles but lacked the other four required fields; the 3,447 other concepts
had at least one missing field with no structured
record of that debt. Prose-only progress could therefore drift from the data.

## 4. Implemented Design

`docs/design/minimal-core-boundary.md` admits only the meta algorithm, link
store, generic interpreters, and host surfaces. The recursive source ledger
classifies every handler as migrate, promote, or delete and records a concrete
reason. Three link-store/memory-program interpreter files pass promotion; 43
files remain migration candidates. Both per-file and aggregate line counts are
ceilings, so reductions must lower the committed baseline.

`data/meta/seed-metadata-schema.lino` defines role, precondition, effect, unit,
and example. FrameNet supplies the role-and-example shape; Wikidata supplies
typed statement-value and provenance precedent. All 37 coding-path concepts
now satisfy the contract. Sixteen stable shards represent every other missing
field as data, making review and gradual enrichment deterministic.

## 5. Verification

`tests/unit/issue_918.rs` independently checks the boundary, recursive ledger,
coding metadata, and exact gap data. `scripts/check-minimal-core-boundary.rs`
and `scripts/audit-seed-metadata.rs` each include self-tests and production
checks wired into CI. The gap auditor scans every direct seed concept, so a new
record cannot bypass it by living outside a hand-picked catalog.

`experiments/issue_918_agent_cli.sh` boots the real OpenAI-compatible server,
drives the installed Agent CLI, and compares one independently reviewed
minimal-core invariant leaf byte-for-byte. Its raw stream, server log, task,
session identifier, worktree status, and exact output live under
`agent-cli-evidence/`.
