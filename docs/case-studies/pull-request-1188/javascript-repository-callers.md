# JavaScript repository protocol callers

These Node callers consume data/meta/repository-workspace-protocol.lino, the same nine stages as the native SWE-bench, solve/coding-ladder and authoring callers. Their implementation stays at the Node boundary while the shared stage loops stay host-agnostic.

## Structural callers

Create a task JSON file with the exact base and requirement:

```json
{
  "requirement": "Edit the TEST_NAMES list to add \"beta\"",
  "clone": { "origin": "/absolute/local/remote", "base_commit": "FORTY_CHARACTER_COMMIT" },
  "tests": { "line": "npm test", "names": ["list membership"] }
}
```

```sh
node js/server/repository-workspace-command-line.mjs solve --workspace /tmp/isolated-case --task task.json
node js/server/repository-workspace-command-line.mjs swe-bench --workspace /tmp/isolated-case --task task.json
node js/server/repository-workspace-command-line.mjs coding-ladder --workspace /tmp/isolated-case --task task.json
```

A missing workspace is cloned from the declared local origin and checked out at the exact base. An existing workspace must already match that base. Moving branch names are refused before creating a clone. Programmatic runRepositoryCase callers may provide an injected process/network host; remote cloning requires an explicit host grant. No checkout of this shared repository was copied during verification.

The output carries the diff, linked observations, stopped/open items and one status per protocol stage; repository-protocol.lino is also written in the isolated checkout. Structural commit stays refused.

## Authoring

Authoring JSON declares repository, an outside scratch workspace, task, message, canonical pull_request URL, produces, optional into/contains/seed/context, and evidence directory. Only the declared seed/context file set is copied. commit defaults to false; true requests the gated four-trailer commit.

```sh
node js/server/repository-workspace-command-line.mjs authoring --task authoring-task.json
```

The default adapter starts the JavaScript Formal AI server on loopback and invokes the installed Agent CLI with model formalai/formal-ai, title generation and summarization disabled, and a bounded deadline. Artifact and destination paths are confined; symlinks and parent escapes are refused. Server memory stays outside the evidence directory and server/state cleanup runs after success or refusal.

## Observed live fixture (T1225, 2026-10-09)

Node Formal AI server plus actual installed Agent CLI 0.26.0 completed the task `Create file artifact.txt containing exactly: authored by Formal AI`. Every applicable stage through diff was observed; commit was refused because it was not requested. The sandbox initially rejected loopback binding in T1224; the authorized local-port retry passed.

Artifact bytes: `authored by Formal AI`. Session/model evidence:

```text
formal-ai session ses_ee2aae711ffeAMMXr9E6x6VMcV
formal-ai model formal-ai/javascript
```

Protocol trace:

```text
repository_protocol_trace
  caller "authoring"
  editor "agent_session"
  stage clone "observed"
  stage locate "observed"
  stage read "observed"
  stage serve "observed"
  stage edit "observed"
  stage session "observed"
  stage verify "observed"
  stage diff "observed"
  stage commit "refused"
```

Framed Agent CLI stream is captured in javascript-repository-authoring-stream.jsonl. Nine focused Node adapter replay cases also passed, including a real four-trailer Git commit in a tiny fixture, unchanged seed/missing session/contract refusals, exact-base tiny local cloning, timeouts and path confinement. Full Rust compilation remains CI-owned.
