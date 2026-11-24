# Artifact lookup immediately after upload

PR: https://github.com/link-assistant/formal-ai/pull/1581

Published head `90b5159008da68c5c73534a5b5a4963b81deb04a` has two
observed failing workflows. The remaining native and CodeQL producer jobs are
still running at this checkpoint; no final-head or full-issue success is claimed.

## Original failures

Workflows run [44804037588](https://github.com/link-assistant/formal-ai/actions/runs/44804037588)
fails in its Windows artifact round trip. In the original complete run stream,
line 3160 records upload finalization at `19:38:48.1191302Z`, artifact ID
`14659919894`; lines 3158–3162 preserve the archive digest, matching name and
successful upload. Line 3231 records `Artifact not found for name:
artifact-roundtrip-windows-2025` at `19:38:49.1925243Z`. The same job uploaded
with `actions/upload-artifact@v7` and downloaded with pinned v8.0.2. Extraction
is skipped after the lookup failure. The source makes one client lookup, with
no action-level recovery. A backend visibility delay is an inference from these
observations, not an established backend cause. Actionlint, concurrency checks
and their existing canaries pass in this run.

Layered CI run [44804037844](https://github.com/link-assistant/formal-ai/actions/runs/44804037844)
has one failed assertion in JavaScript shard 1: original lines 8368–8398 identify
`fast-pull-request-production-profile.test.mjs` and the three omitted expected
timing-input paths. The production sparse checkout already contains those exact
inputs. Correct the expectation without broadening the checkout or its budget;
the existing physical checkout regression still verifies each consumed file.

## Repair and verification

The artifact adapter makes at most three actual pinned download attempts, with
two- and four-second pauses and unchanged caller deadlines. Each attempt keeps
all original selectors, producer authority, strict digest validation and fatal
deprecation handling. Before a retry, delete partial raw archives and require a
successful reset. Cancellation prevents another download. Only the first two
transport attempts may recover; the final transport failure and every extraction
failure remain fatal. Successful output still comes only from real extraction.

The original adapter fails seven of ten bounded controls; three existing safety
controls pass. The candidate passes all ten, then an added failed-reset control
also passes through the maintained CI test entrypoint. These tests execute the
actual composite shell steps and actual Python ZIP extraction; only the remote
transport and backoff clock are stubbed. They do not establish hosted transport
or backend recovery. The final maintained artifact suite passes 17 cases,
including all 11 retry controls; the profile suite passes 18; the shard and real
sparse-checkout suite passes 50. Seven touched JavaScript gates and whitespace
checks pass. No local Rust build runs.

Retain the initial nested Node-runner output failure, its isolated child-context
repair, and the mistaken regression-suite path separately from corrected runs.
The original hosted failure is the profile expectation's before-fix control.
The evidence manifest under `evidence/checks/artifact-lookup-retry/` binds every
retained investigation and check payload by bytes and SHA-256.

Upstream research distinguishes wildcard misuse in download-artifact #337 and
v3/v4 incompatibility in #250 from this same-job, compatible-version observation.
Both #250 screenshots were downloaded with authentication, verified as PNGs and
viewed: one shows the Installer lookup error, the other shows its artifact list.
The older #63 report concerns a different backend's download 404. None proves
the present backend cause or an upstream repair. The attempted toolkit version
ref does not exist (HTTP 404); no pinned client-source claim relies on it.

All remaining warnings, template comparisons, dependency publication, macro
semantics, original budgets/live delivery and independent reasoning criteria
remain open. Strict original identity stays 31/53 with 22 separately scoped
assertion reviews, and independently accepted reasoning stays 0/20.
