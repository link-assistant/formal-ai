# Immutable BuildKit setup and actual mirror recovery

Pull request: https://github.com/link-assistant/formal-ai/pull/1581

The resilient setup wrapper called `moby/buildkit:buildx-stable-1` pinned, but
that tag can move. Four direct setup calls also allowed an implicit mutable
BuildKit default. Docker Hub and mirror.gcr.io independently returned the same
5,296-byte multi-platform index on 20 November 2026. Its SHA-256 equals both
registry response identities:

`baf6138eb8c37644800c7c913fba051e840ebb9ba13eaf92d92ef9a1712d0b4c`

The wrapper default and all four direct callers now select that immutable index.
It retains Linux amd64, arm, arm64, ppc64le, riscv64 and s390x. No image layer is
downloaded locally. Caller-selected images remain supported, direct callers keep
cleanup disabled, and the wrapper keeps cleanup enabled by default.

The earlier mirror pre-pull/retag approach also could not recover actual setup.
The official Docker action unconditionally pulls its selected image. A real
local Docker CLI rejects digest retagging with “refusing to create a tag with a
digest reference”; even a cache seed cannot bypass that official pull. The
pinned and freshly fetched Rust pipeline template already retries the actual
setup action with a mirror reference. This repair adopts that approach: execute
canonical setup, then retry actual setup on failure with the identical digest at
the mirror. Disabled mirrors and failure of both setups remain fatal. Validate
one-line references before writing action outputs; retain opt-in diagnostics.
The template's existing fallback is not reported as an unfixed upstream defect.

Ten local controls execute the real shell blocks and model only the remote
setup outcomes. They cover canonical success, actual-action fallback wiring,
both failures, disabled mirrors, explicit references, malformed input, output
write failure, hosted probe admission and a narrowly scoped outage injector.
The complete existing subprocess suite imports them. Seven focused JavaScript
files pass sequentially. Whole-payload inverses preserve every existing job and
container-workflow field except the four explicit image options; the workflow
addition contains only the new read-only probe and its source triggers. Both
shell blocks pass ShellCheck, and all five touched gates pass.

The hosted probe initializes real BuildKit daemons without building application
images. It verifies the selected RepoDigest, running daemon, image ID and builder
identity. Its second setup injects failure only for an exact canonical Docker
pull; mirror pulls and all other Docker commands still execute for real. A
12-minute job ceiling, individual step limits, sparse read-only checkout and an
always-retained artifact bound the trial. Actual hosted daemon startup and
mirror recovery remain pending on the next published commit; local adapter
passes do not grant that credit.

Retained failed attempts include fixture quoting mistakes, the superseded
cache-only draft, a harness injecting NODE_OPTIONS into a check that deliberately
refuses it, and an interrupted gate run at the Formal AI wrapper's 60-second
boundary. The corrected gate continuation completes under a detached supervisor
in 68.267 seconds, peak child RSS 814,504 KiB; no local Rust compilation occurs.
Complete original/compressed identities, registry responses, primary upstream
sources, helpers, traces and outputs are under
`experiments/issue-1580/evidence/checks/buildkit-bootstrap-identity/`.
Supplied commands and patches earn zero independent authorship or reasoning
credit. Every broader unfinished issue requirement remains open.
