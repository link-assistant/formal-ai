# Bounded actions and observable failure

Authority: [issue #1328](https://github.com/link-assistant/formal-ai/issues/1328) and the maintainer's requirement to bound repeated actions throughout the codebase.

Every retry must have a finite attempt ceiling. External operations also need a finite per-attempt deadline or cancellation boundary. A failed deterministic precondition (invalid schema, missing local Git ref, permission denial) must be repaired locally or reported; repeating it unchanged is not progress. Count attempts from the transcript when the client owns execution, including unknown tools, missing receipts, changed error wording, namespace-qualified tools, and reused call IDs. Restarting the server must not reset this budget when the client resends the transcript. A client that discards its history cannot supply that evidence. Continuation cues preserve the standing objective; a genuinely new request opens its existing evidence window.

The planner stops after three declarations of one canonical tool/argument pair in its current evidence window, regardless of result classification. This is a conservative total ceiling within that window: a complex task that intentionally needs the same call again must open a fresh explicit request after inspecting progress. The whole objective also has a finite ceiling of 128 tool declarations, including cycles of distinct calls. Distinct arguments get distinct per-action budgets; changing error wording or JSON key order does not. Earlier successful-repeat and identical-failure stops remain active and can stop sooner. All planner routes, including symbolic command reroutes, pass the protocol boundary guard.

A narrative bug report must not authorize quoted reproduction commands or bind unrelated output/version text as a file target. Command-output capture requires an owned imperative and a locally bound output clause; displaying that captured file never proves delivery of unrelated issue requirements.

Responses ingestion must preserve namespace-qualified tool identity. Outgoing semantic arguments must be projected onto the client's advertised schema (`query` to `input` for search), and an unbound required argument must produce a terminal planning failure, never an empty tool call. Schema projection applies recursively to owned object properties and array items, preserving explicit invalid values and array order. Required descriptive metadata may use the actual user request, after declared defaults, identity and enum guards; missing actions, identities, numbers or arrays are never invented. The boundary records the selected tool, bound arguments, state, and retry ceiling at debug level through the existing redacting sink. Oversized diagnostic fields use explicit post-redaction size/digest/limited-text summaries within the existing diagnostic reader budget. Full CST artifacts, dialog payload custody, control records and original decode limits remain unchanged. Unsupported schema references are refused, rather than presented as validated.

Push targets use an explicitly bound valid branch or a current-branch observation in the client workspace. An unbound `origin` refspec failure selects local Git recovery before web research. Explicit branch bindings and other remotes are retained for inspection, never silently replaced by the current branch or `origin`. Detached HEAD fails without pushing. Failed or empty search results consume the search attempt and cannot reopen the same repair rung indefinitely.

Exact prompt normalization is memoized without truncating or changing the request. Native caching has one process-wide memory ceiling, shared by connection threads; JavaScript caching belongs to its installed source host. Both bound retained entries and text. The address-only production CPU profile does not establish that normalization alone accounts for the reported 130 seconds; source replay and hosted native timing must distinguish measured improvements from that inference.

Fresh-request cost must be measured separately from exhausted-history stopping.
Retain cold and repeated observations at the actual maintained request entrypoint,
with the original requested action and independent argument/quality checks. A
normalization microbenchmark or an exhausted transcript that bypasses planning
does not establish fresh-request latency. Preserve complete available input;
identify missing wire/tool context explicitly. Public synthetic stress inputs
remain separately labelled and cannot replace an unavailable original replay.
Native execution stays hosted, with an outer supervised deadline that also stops
a stalled synchronous planner. Report measured wall/CPU scope and unavailable
measurements honestly; an observation deadline alone is not a performance pass.
Immutable grammar compilation may be cached for its declared installation/host
lifetime. Do not cache prompt-dependent ownership decisions or bypass quote,
path, domain, full-match, ambiguity or host-reset checks.

Diagnostic persistence is best effort but its failure is observable: probe the configured sink at startup, report its real directory and IO cause once, disable a failed file sink instead of retrying every diagnostic, retain stderr, and expose the state through `/health`. Memory storage errors must retain their original IO category and cause instead of being mislabeled as incompatible schema.

## Retry audit

- Planner and all protocol wire routes: transcript-derived ceiling; no recognized error shape is required.
- Repair search/fetch/apply ladder: existing rung/source ceilings remain; failed or empty search now stops, and refspec repair reads local branch state.
- Native driver, recursive execution, orchestration, rewriting, prerequisite recovery, and completion authoring: existing finite turn/depth/step/attempt ceilings remain. Their original controls must continue to pass.
- Telegram polling: three consecutive transport/parse failures stop with the original error; a valid batch resets that failure budget. Healthy daemon polling remains long lived and cancellable.
- Background dreaming: three failed idle runs stop the worker with an explicit error; successful work resets the failure budget. Healthy service operation continues.
- Provenance streaming: three consecutive interrupted reads stop with the original IO error; actual read progress resets the budget.
- Structured diagnostics: one failing sink attempt disables that sink for the configured process; other sinks continue. Reconfigure/restart after repairing the directory.
- Artifact, apt, Git fetch, macOS packaging, builder-toolset and marketplace publication retries already use finite attempts; their existing tests remain mandatory. Builder-toolset options reject non-finite or excessive counts and durations before IO: at most eight attempts, 60 seconds of configured backoff, and 15 minutes per transfer timer.
- Pages deployment verification already has an outer deadline; each curl poll now also has connect, transfer and retry deadlines.
- HTTP, WebSocket, WebRTC-signaling and logging-proxy listeners use one shared dispatcher: acceptance and connection-thread dispatch each have their own three-consecutive-failure ceiling. A successful accept resets only acceptance failures; successful dispatch resets only dispatch failures. Exhaustion returns the actual IO error to the supervisor. Healthy serving remains long lived.
- Parser/tokenizer/source walkers advance a finite cursor or ascend a finite path, and healthy event loops serve new operations. These are not retries of one failed action and must retain their progress/cancellation boundaries. Runner-capacity sampling intentionally observes new samples until its supervising CI process ends.

Native compilation and product execution run only in CI. Local checks use maintained JavaScript sources and disposable data fixtures. Supplied integration controllers receive no autonomous-authorship credit. For this immediate release, follow the owner's [manual-fix authorization](../case-studies/issue-1328/critical-delivery-scope.md); direct JavaScript execution is allowed without routing each change through Formal AI.


## Cross-repository CI evidence

Collect recent failed Actions jobs from Formal AI and Hive Mind with
`scripts/collect-ci-failures.mjs`; retain bounded start/tail observations,
source/run/job identities, redacted diagnostics, and deduplicated fingerprints.
Unavailable, truncated, timed-out and cancelled observations remain explicit.
A recovered warning cannot become a declared cause of terminal failure.
See [the inspected failures and collection contract](../case-studies/issue-1328/ci-failure-collection.md).

Dispatched evidence checks must use a validated nonreserved base-ref variable.
Native projection dispatches require API-authenticated current same-repository
PR head, base and merge witnesses, matching physical checkout and clean tracked
inputs before and after execution. An environment selector alone grants no authority.
Artifact finalization recovery has three real upload attempts under an enclosing
step deadline. It retains original immutable names, bytes, scope and receipt IDs;
it cannot invent a successful upload, overwrite artifacts, rerun workflows, or
skip an original acceptance check.

Original Hive delivery keeps separate same-source image preparation, unchanged client solving, and independent compiler/output verification. Each job respects the existing 30-minute CI ceiling; the solver has a 21-minute step bound, retaining the existing 70% step-to-job headroom policy. Transfers bind archive and evidence bytes to the current source, binary and workflow run. Staged evidence grants no success or authorship credit; the original harness and final independent oracle remain mandatory.


The Node log-file reader must reject oversized regular files from descriptor
metadata before allocating or reading payload bytes. Every bounded read must make
valid finite byte progress and detect concurrent growth or replacement. Preserve
the public log CLI's stable-symlink behavior and the private provenance reader's
no-symlink policy; one shared implementation enforces their declared options.
A failed partial source write must preserve the original source and retain its
failure evidence. Source transformation writes stage a bounded complete candidate,
recheck the observed preimage and identity, and independently verify the physical
result. This is an optimistic filesystem concurrency check, not an atomic CAS
primitive against an uncooperative concurrent writer.

External command acceptance must include startup, input completion, child status,
output EOF and finite cleanup; a timer that stops at parent exit is insufficient.
Captured stdout and stderr have finite byte limits, preserve binary content and
never become complete success after overflow, timeout or incomplete cleanup.
Preparation and post-processing subprocesses need their own explicit finite
boundaries when they run outside the main execution observer. A multi-stage
preparation uses one remaining budget rather than resetting a deadline per stage.
Keep the original primary failure and bounded partial evidence when cleanup also
fails.

A foreground client may use operator-owned terminal signal cancellation only
when stdin and stdout are actual terminals. Unattended and piped client calls
must use the finite command supervisor. These policies must remain distinct from
healthy long-lived services and cannot authorize an unchanged failed retry.

The hosted web suite must reject known direct blocking process bypasses and
stale canonical command-capture projections. Structural checking must state its
local-provenance and macro/dynamic-flow limits; it is not proof of every external
I/O path. Real process controls must cover blocked input, inherited pipes, binary
capture, overflow, unchanged nonzero statuses, failed spawn and finite cleanup.
