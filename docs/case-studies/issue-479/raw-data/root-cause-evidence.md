# Root-cause evidence — issue #479

Captured 2026-06-14 from the live `link-assistant/formal-ai` repository.

## The smoking gun (Desktop Release run 27505853178)

The most recent automatic Desktop Release run before the fix:

| Field | Value | Meaning |
| --- | --- | --- |
| `databaseId` | `27505853178` | the run that should have built v0.201.0 assets |
| `event` | `workflow_run` | triggered by "CI/CD Pipeline" completion |
| `conclusion` | `success` | the workflow "succeeded"… because every real job was skipped |
| **display** `headSha` (`gh run list`) | `56ccb77e…` | the child **release** commit `chore: release v0.201.0` |
| **event payload** `workflow_run.head_sha` (read by resolve) | `0abd3f45…` | the **parent** — the commit CI/CD Pipeline actually ran on |
| resolve job | `success` | emitted `should_build=false` |
| build job | `skipped` | **no assets ever built** |
| finalize job | `skipped` | no SHA256SUMS.txt / provenance |

Verbatim resolve-step log (`desktop-release-27505853178.log`):

```
env:
  EVENT: workflow_run
  WORKFLOW_RUN_HEAD_SHA: 0abd3f45b61a68ed2b819189d7655c3a7cd8aa07
No release tag points at workflow_run head SHA 0abd3f45b61a68ed2b819189d7655c3a7cd8aa07; skipping desktop build.
```

## Why the two SHAs differ (the reconciliation)

`gh run list` **displays** the SHA the *triggered* workflow checks out (HEAD of `main`
at trigger time = the child release commit `56ccb77e`). But the resolve step does not
read that; it reads the **event payload** `github.event.workflow_run.head_sha`, which is
the head SHA of the *triggering* CI/CD Pipeline run — the **parent** commit `0abd3f45`.

```
tag v0.201.0  ->  commit 56ccb77e  ("chore: release v0.201.0")
                          | first parent
                          v
                  commit 0abd3f45  ("Merge pull request #472 …")  <- workflow_run.head_sha
```

The old inline resolve logic required *a tag whose commit EQUALS
`workflow_run.head_sha`*. The tag lives on the **child** `56ccb77e`; the head SHA is the
**parent** `0abd3f45`. The exact-SHA match therefore **never** succeeded for an
auto-release → `should_build=false` → build skipped → zero desktop assets →
`/download` shows "Not available in latest release" for every platform.

## Corroborating evidence — every release is asset-less

`releases-asset-evidence.json`: every release from **v0.187.0 through v0.201.0** has
`desktop_assets: 0` and `total_assets: 0`. The desktop build never uploaded a single
asset since this code path went live — consistent with a systematic resolve failure,
not a one-off flake.

## The fix

`scripts/desktop-release-resolve.sh` resolves the **latest published release** for
`workflow_run` events (Tier 2), keeping the exact-SHA match only as a defensive Tier 1.
A diagnostic confirms `latest_release.parent == workflow_run.head_sha` (the
"auto-release child" relationship) and logs it, but the build proceeds regardless so the
page self-heals. An idempotency guard skips only when the resolved release already
carries `formal-ai-desktop-*` assets, so re-runs are safe and the existing backlog
heals on the first pipeline completion after the fix lands.
