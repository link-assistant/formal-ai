# Issue #1084 — multi-arch container images

## Problem

`ghcr.io/link-assistant/formal-ai:latest` publishes **one** architecture,
`linux/amd64` (the second manifest entry is the build attestation). Cause:
five `docker/build-push-action@v7` steps in `release.yml` (lines 151, 715,
760, 920, 963) never pass `platforms:`, and every one of those jobs runs on
`ubuntu-latest`. Apple Silicon, Graviton and arm CI runners get
qemu-user-static emulation at best — violating principle 13 of
hive-mind's `CI-CD-BEST-PRACTICES.md`, "Container Images: Native Runners
per Architecture".

## What unblocked the fix natively

- base `konard/box-dind:2.1.1` already publishes `amd64` + `arm64`
  (Docker Hub manifest list verified in the issue),
- `ubuntu-24.04-arm` runners already in use (`desktop-release.yml:195`),
- no `setup-qemu-action` anywhere — a native matrix introduces no emulation.

## Delivered

`.github/workflows/container-images.yml`:

- **Native matrix**: one build job per architecture (`ubuntu-latest` for
  amd64, `ubuntu-24.04-arm` for arm64), each `docker/build-push-action` step
  passing `platforms: linux/<arch>` — the runner's own architecture, so
  nothing emulates.
- **Manifest merge by digest**: per-arch images are built `load:`-ed,
  runtime-verified (slim: `verify-formal-ai-slim`, no `--privileged`; full:
  the existing `verify-formal-ai-dind`), saved as artifacts, and the merge
  job turns them into one multi-arch tag with
  `docker buildx imagetools create --amend` — the buildx-prescribed shape
  for native matrices.
- **Both variants**: `:slim` (#1153) and the full self-contained image.
- **Size report + budget** in the merge job (`sizes` output → release notes,
  2 GB hard fail on slim), satisfying #1153's "size is reported in CI".
- `workflow_dispatch` with `push` input (default false: build + verify +
  merge dry-run only) and `workflow_call` so `release.yml` can adopt the same
  jobs instead of carrying parallel logic.

## Residual (GUARD-tracked, in docs/integration-manifest.md)

The five `build-push-action` steps inside `release.yml` itself still build
single-arch; they migrate to call these jobs (`workflow_call`) in the same
edit that lands the #1187 credential resolver there — both edits touch the
same lines' neighborhood and the file is under the integration GUARD until
the #1168/#1169 commits land. The full image's arm64 build is excluded from
the matrix until its first green run (its compile stage has only been
exercised on amd64); the exclusion is a matrix `exclude:`, so enabling it is
a one-line change after that run.
