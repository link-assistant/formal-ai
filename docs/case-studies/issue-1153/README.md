# Issue #1153 — slim runtime image

## Problem

`ghcr.io/link-assistant/formal-ai:latest` (0.352.1, `sha256:e23c2e4c…`) is
**24.3 GB on disk** (6.2 GB compressed, 59 layers), because it extends
`konard/box-dind:2.1.1`. Hive Mind pulls it onto every host as its
`--model formal-ai` sidecar and runs exactly one command:

```
docker run … --env DIND_SKIP_DAEMON=1 --volume hive-mind-formal-ai-memory:/home/box/.formal-ai \
  ghcr.io/link-assistant/formal-ai:latest formal-ai serve --agent-mode --host 0.0.0.0 --port 8080
```

The inner Docker daemon is explicitly skipped, no language toolchain is
invoked, and the Dockerfile's own `FORMAL_AI_START_ISOLATION=docker` runs
tasks in separate containers — so the outer image's 23.5 GB of toolchains
serve no runtime path of this workload.

## Delivered

| Artifact | What it is |
|---|---|
| `Dockerfile.slim` | `ubuntu:24.04` runtime (same glibc 2.39 as the pipeline's `ubuntu-latest` build) + `ca-certificates`, `curl`, `git`, `nodejs`, `bun` and the three global CLIs (`start-command`, `@link-assistant/agent`, `agent-commander`) + the `formal-ai` binary. Same `BINARY_SOURCE=compile\|prebuilt` contract as the main Dockerfile. No Docker engine, no toolchains, **no `VOLUME` declaration** (point 2: the DinD variant's volume line creates an orphaned anonymous volume on every un-mounted `docker run` — four accumulated on one Hive Mind host), built-in `HEALTHCHECK` on `/health`, non-root `box` user, `CMD serve --agent-mode`. |
| `scripts/verify-docker-slim-runtime.sh` | Installed as `/usr/local/bin/verify-formal-ai-slim`: `--version`, then `serve --agent-mode` on a scratch port, then `/health` must answer within 30 s — with no `--privileged` and no inner daemon (acceptance criterion 1). |
| `docker/compose.slim.yml` | The sidecar as one command, memory volume mounted, for local runs and for Hive Mind hosts to mirror. |
| `.github/workflows/container-images.yml` | Publishes `:slim` (issue #1084 gives it `linux/amd64` + `linux/arm64` via native runners, no emulation) and reports the size, failing above a 2 GB budget (acceptance criterion 2). |

## Size accounting (layer budget)

Formal AI itself measured ~0.8 GB of the 24.3 GB image (binary 208 MB + the
global CLIs 507 MB + nodejs 59 MB); the slim image carries exactly that class
of content on `ubuntu:24.04` (~78 MB) plus `curl`/`git`/`ca-certificates`, so
the expected size is **~1 GB**. The CI job enforces the budget so a regression
(toolchain accidentally re-added, CLI balloons) fails the build instead of
silently re-inflating the sidecar.

Per-variant sizes are printed by the workflow's size-report step and belong
in the release notes (point 5; see the workflow's `sizes` job output).

## What stays open, honestly

- **Base-image layer bloat (point 3)** — the 1.89 GB `setfacl -R` layer and
  the toolchain layers belong to `konard/box`, not this repository; fixing
  them there shrinks every image built on `box`. Filed upstream, out of this
  PR's reach.
- **`DIND_STORAGE_DRIVER=vfs` (point 4)** — a property of the full variant's
  environment; the slim image has no inner daemon at all, so the vfs cost no
  longer applies to the sidecar workload. The full variant keeps vfs until
  `fuse-overlayfs` is validated on the Hive Mind hosts.
- **Hive Mind switching its default** (acceptance criterion 3) — the switch
  is `HIVE_MIND_FORMAL_AI_IMAGE=ghcr.io/link-assistant/formal-ai:slim`
  (link-assistant/hive-mind#2305); it lands in Hive Mind's own repository
  once this tag is published by the workflow above.
