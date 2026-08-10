# Issue #982 — Safe persisted-memory upgrades

Issue: <https://github.com/link-assistant/formal-ai/issues/982>
Pull request: <https://github.com/link-assistant/formal-ai/pull/985>

## Collected evidence

`raw-data/github/` was produced by the repository's own
`formal-ai github-logs collect` command. Its manifest covers the issue, all
issue comments, recent issues/PRs/runs, and repository metadata. The pull
request has a separate complete provenance bundle under
`docs/case-studies/pull-request-985/raw-data/github/`.

The real external `@link-assistant/agent` CLI was driven twice against the
candidate `formal-ai serve` endpoint with two different natural-language
requests. The full client/server logs, exact leaf output, and deterministic
session JSON are preserved in:

| Leaf | External Agent session | Evidence |
| --- | --- | --- |
| Preflight/migration contract | `ses_01b6d03deffePqZ8rVcHzlop7p` | `self-hosting/contract/` |
| Byte-exact rollback contract | `ses_01b6cb1e4ffew2XslZgBkNR7EB` | `self-hosting/rollback/` |

Each run completed four real OpenAI-compatible chat turns: persist plan, write
the requested file, verify its exact bytes, and finish. The integration suite
regenerates the session JSON and checks both committed leaf outputs
byte-for-byte. These are two independently authored leaves in the ten-leaf
implementation decomposition, meeting the repository's 20% self-authored floor
without attributing manually written Rust or documentation to Formal AI.

The reviewed smallest-leaf decomposition is:

| # | Leaf | Author |
| --- | --- | --- |
| 1 | Detect and report persisted schema compatibility | Human |
| 2 | Expose preflight through the CLI | Human |
| 3 | Implement the locked backup/stage/rename transaction | Human |
| 4 | Retain unknown event fields during ordinary writes | Human |
| 5 | Publish compatibility in server health | Human |
| 6 | Cover released/target/incompatible fixtures | Human |
| 7 | Prove interruption, retry, and writer-lock behavior | Human |
| 8 | Exercise old/candidate containers on one named volume | Human |
| 9 | Author the concise preflight/migration contract leaf | Formal AI + Agent CLI |
| 10 | Author the concise byte-exact rollback contract leaf | Formal AI + Agent CLI |

The measured share is therefore `2 / 10 = 20%`.

## Root cause

Released Formal AI persisted an unversioned `demo_memory` document and every
writer simply reserialized whatever the current parser understood. That had
three coupled hazards:

1. There was no persisted version contract, so an operator could not decide
   whether a new binary could read a volume before starting it.
2. Parsing ignored unrecognized event fields and reserialization dropped them,
   so even a seemingly harmless ordinary write could lose forward metadata.
3. There was no distinct migration transaction: no shared-lock boundary,
   verified rollback copy, staged validation, atomic commit receipt, or
   interruption/retry protocol.

The correct boundary is therefore not "make startup migrate." Startup must stay
boring. Inspection is pure, normal writers retain the detected representation,
and only a named operator command may cross a schema boundary.

## Design and safety invariants

Schema 1 is the released unversioned format. Schema 2 adds exactly one root
line, `schema_version "2"`; old readers ignore it. Preflight reads bytes only
and reports binary version, path presence, detected/minimum/maximum/target
schema, compatibility, required migration/id/state, rollback support, event
count, source SHA-256, and structured refusal details.

Explicit migration performs this transaction while holding the same sibling
lock as ordinary writers:

1. Read the source and fail closed on malformed, too-old, or too-new schemas.
2. Resolve distinct backup/receipt paths and write a byte-exact backup.
3. Read the backup back and verify both equality and digest.
4. Transform only the root marker, then validate target schema and event count.
5. Write a create-new same-directory stage with source permissions and `fsync`.
6. Atomically rename the stage over memory and sync the parent directory.
7. Atomically write a JSON receipt containing before/after digests and rollback.

An interruption before rename deletes the stage and leaves the original bytes
untouched; the verified backup remains reusable. A retry with schema 1 reuses a
matching backup, while a retry after commit sees schema 2 and returns
`changed: false`. A live writer or incompatible schema returns nonzero JSON and
never touches the memory file.

## Requirement and test map

| Concern | Regression evidence |
| --- | --- |
| Pure preflight and machine-readable fields | `upgrade_status_detects_released_schema_without_mutating_memory` and `upgrade_status_for_missing_path_creates_nothing` |
| All supported schema fixtures, including the released zero-byte initial state | `fixtures_cover_every_readable_schema`, `released_zero_byte_store_is_readable_and_upgradeable`, and `tests/fixtures/memory/` |
| Atomic/lossless migration, receipt, retry, rollback | `migration_is_atomic_lossless_receipted_idempotent_and_rollback_safe` |
| Cancellation before commit | `interrupted_migration_keeps_original_byte_identical_and_retryable` |
| Shared writer coordination | `live_writer_lock_causes_machine_readable_refusal_without_modification` |
| Future/corrupt incompatible data | `future_schema_is_refused_nonzero_and_never_modified` and `malformed_memory_is_refused_nonzero_and_never_modified` |
| Startup/ordinary writes do not upgrade old data, while new stores use the target schema | `ordinary_server_write_preserves_released_schema_and_unknown_metadata` and `first_write_to_a_new_store_uses_the_target_schema` |
| Health contract | `health_exposes_schema_compatibility_without_triggering_migration` |
| Real old/candidate volume lifecycle | `experiments/issue_982_memory_upgrade/run_container_upgrade.sh` in the Docker CI job |
| Agent-authored output/session replay | `formal_ai_agent_authored_leaves_and_sessions_replay_byte_for_byte` |

## Operator reproduction

```bash
formal-ai memory upgrade-status --path memory.lino --format json
formal-ai memory migrate --path memory.lino \
  --backup memory.schema-1.backup \
  --receipt memory-upgrade-receipt.json \
  --format json
formal-ai memory query --path memory.lino --prompt 'recall migration canary'
cp memory.schema-1.backup memory.lino
```

Detailed rollout, refusal, retry, and rollback guidance is published in
`docs/configuration/memory.md`.
