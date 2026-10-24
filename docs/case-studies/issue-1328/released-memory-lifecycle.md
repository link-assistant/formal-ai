# Released-memory lifecycle verifier

The original three-client run [40139076853](https://github.com/link-assistant/formal-ai/actions/runs/40139076853)
stopped during preparation of the published `v0.355.8` runtime, before any client
task or generated issue/PR. The `hive-released-lifecycle-evidence` artifact
records `FallbackDidNotRecordAnswer`; cleanup completed. Both supported-schema
migration passes preserved the original events and backup. Native PID1 exited
successfully with a held HTTP connection in approximately 129 ms on each pass.

The verifier incorrectly required an assistant `content` field. Both
`rust/src/memory_sync.rs` and `js/server/memory-store.mjs` intentionally record
the completed answer as an assistant **task**, with exact `inputs` and `outputs`
and an `evidence` link to its user message. The protocol recorder preserves the
wire answer, including whitespace. A synthetic test had invented an assistant
message with `content`, masking the mismatch.

The regression now calls the production JavaScript store writer. Before the
verifier correction, this real record fails acceptance while a substituted
assistant message incorrectly passes. After the correction, acceptance requires
the matching user message and linked assistant solve task, exact prompt and
wire answer, unchanged bytes on restart, original incompatible-store preservation,
healthy fallback, native PID1 identity, timely successful shutdown and cleanup.
Missing writes, incorrect inputs/outputs, missing evidence and substituted
message events all fail. A whitespace/Unicode answer verifies exact preservation.
These local controls emulate Docker/HTTP observations; actual released native
execution remains exclusively in hosted CI.

The first follow-up CI also exposed release-preparation debt: adding the fourth
recent release made the repeated `target_percentage_basis_points "389"` field
exceed the existing notation ceiling. The ledger now states its identical fields
once in `release-defaults`. Rust measurement readers, the release writer and both
status-renderer twins resolve these explicit defaults; row fields override them.
Legacy files without defaults retain their original epoch interpretation.
JavaScript and Rust archive rotation retain the defaults beside their complete
records, preventing inheritance from leaking across file boundaries. Automated
expansion reproduced every prior release field/value, and no measurement,
attribution evidence, target policy or notation ceiling was changed. Hosted
native tests cover inheritance, explicit overrides, legacy scope and future
release writing; local JavaScript controls cover rotation and status rendering.
The governed source witness is refreshed through its explicit review command:
every existing supported program, assertion and unsupported record is unchanged.
Its only additions are the two new filesystem/Git regressions, honestly marked
unsupported by the browser specification reader and still required in native CI.
The reader source itself and all coverage/notation ceilings remain unchanged.

Receipts retain the failing pass's phase, health, shutdown observations, memory
digest, event count and a bounded event sample. Text fields are represented by
their UTF-8 size and SHA-256, preserving useful field-shape diagnostics without
publishing answer text. A failed assertion cannot grant completion credit.

## Release and operational evidence

[Release v0.355.8](https://github.com/link-assistant/formal-ai/releases/tag/v0.355.8)
is published from `55e2f7ea83ad127c6f77c820bee3fdb996498943`, the prepared child
of PR 1406's merge `fd721a2277c58932fa49c65652db06b29a8993f4`.
The [main pipeline](https://github.com/link-assistant/formal-ai/actions/runs/40118461144),
[completed desktop producer](https://github.com/link-assistant/formal-ai/actions/runs/40129790411),
[public-download verification](https://github.com/link-assistant/formal-ai/actions/runs/40137698867)
and [VSIX successor](https://github.com/link-assistant/formal-ai/actions/runs/40137692520)
passed. Anonymous metadata observations confirmed both architectures of full
and slim GHCR images, version/default aliases, Docker Hub publication, native
receipts, macOS ad-hoc signing and browser release evidence. An unconfigured npm
registry publisher was explicitly skipped; release browser assets were verified.
This publication evidence does not imply original three-client task success.

Two bounded infrastructure recoveries were necessary: retrying a failed Intel
macOS artifact upload, then refreshing the retained VSIX producer and dependent
collector so their signed invocation matched the final desktop attempt. Exact
attestation checks were preserved. Automating this mixed-attempt recovery is
follow-up work, not delivered by the memory-verifier fix.

Published admission requires a new workflow execution, never a failed-job rerun
of the effectful workflow. A prior deferred admission rerun was refused before
effects by the existing first-attempt guard. After verifying that deferred
admissions contained no owner/client claims, a read-only source-qualified push
workflow completion created fresh execution 40139076853. Automating a safe
wakeup after late dispatch-only publishers remains follow-up work; do not weaken
source qualification, duplicate successful tasks or advance `main` during an
active publication/effect owner to obtain a wakeup.

No extra credentials are required. PR 1404 implements the default GitHub Actions
token fallback, following Hive Mind's staged-workflow transport. Preserve the
original task bodies, independent output oracle, two-pass/no-op acceptance and
trusted terminal-owner cleanup. Issue 1364 remains open until those real hosted
tasks pass using the released image.
