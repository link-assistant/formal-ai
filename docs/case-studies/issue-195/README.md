# Issue 195 Case Study

## Scope

Issue: <https://github.com/link-assistant/formal-ai/issues/195>

Pull request: <https://github.com/link-assistant/formal-ai/pull/220>

Branch: `issue-195-bed514694604`

Issue 195 asks the project to make the Docker story concrete for the
Telegram bot: the supported image must be Docker-in-Docker, it must be based
on Link Foundation Box, coding tasks must run through `start-command` with
Docker isolation so logs are tracked, and the root README must describe how to
start the bot in that container.

## Collected Data

Raw artifacts are preserved under `raw-data/`:

- `issue-195.json` and `issue-195-comments.json`: issue body and comments.
- `pr-220.json`, `pr-220-review-comments.json`,
  `pr-220-conversation-comments.json`, and `pr-220-reviews.json`: PR state and
  review surfaces.
- `ci-runs-branch.json`: recent branch workflow runs at collection time.
- `box-repo.json`, `box-releases.txt`, `box-readme.md`, and
  `box-dind-dockerfile`: Link Foundation Box facts and the upstream DinD
  Dockerfile.
- `start-repo.json` and `start-readme.md`: Link Foundation Start /
  `start-command` facts.
- `hive-mind-Dockerfile.dind`, `hive-mind-Dockerfile`, and
  `hive-mind-verify-docker-image.sh`: related `link-assistant/hive-mind`
  implementation patterns.
- `local-tool-availability.txt`: local availability check for Docker and
  `start-command`.
- `repro-before-docker-runtime.txt`: failing regression test output from the
  pre-fix Docker/runtime state.
- `after-docker-runtime.txt`, `after-docs-issue-195.txt`,
  `after-cli-environments.txt`, `cargo-test-all-features.txt`,
  `cargo-clippy.txt`, `cargo-fmt-check.txt`, and `check-file-size.txt`: local
  verification logs after the fix.

Local `logs/*.log` files are generated during development and ignored by git;
the tracked copies above preserve the evidence for the PR.

## Online Facts

The upstream Box repository describes `konard/box-dind` as the full Box image
with Docker Engine added. The current Box release list showed `v2.1.1` as the
latest release at collection time, so the Dockerfile pins
`konard/box-dind:2.1.1` instead of floating on `latest`.

Box's DinD Dockerfile keeps `/usr/local/bin/dind-entrypoint.sh` as the
entrypoint and leaves the final image as root so the entrypoint can start
`dockerd` before handing the requested command to the `box` user. Its README
documents `docker run --privileged konard/box-dind` as the default DinD
invocation, `--runtime=sysbox-runc` as the safer option where available, and
warns not to bind-mount `/var/run/docker.sock`.

The Start repository documents the executable as `$` from the `start-command`
package. The Docker isolation form is:

```bash
$ --isolated docker -- echo "hello from docker"
```

It records command output and metadata under `/tmp/start-command/logs/`, and
supports `--auto-remove-docker-container` for disposable nested containers.
The issue text used `--isolation docker`; the current upstream README and the
installed CLI snapshot use `--isolated docker`.

The related Hive Mind DinD image already follows the same upstream pattern:
`FROM konard/box-dind:2.1.1`, `DIND_STORAGE_DRIVER="vfs"`, installs
`start-command` with Bun, and keeps the DinD entrypoint as the final
entrypoint.

## Root Causes

1. The previous root `Dockerfile` built a Rust binary in `rust:1.82-slim` and
   copied it into `debian:bookworm-slim`. That image had no Docker daemon, no
   Box runtime, no `start-command`, and no DinD entrypoint.
2. The previous default command was `formal-ai serve --host 0.0.0.0 --port
   8080`, so the container started the HTTP server rather than the Telegram
   polling bot requested by the issue.
3. `data/seed/environments.lino` still described `docker_microservice` as an
   HTTP-server container, so `formal-ai environments` contradicted the desired
   Docker runtime.
4. The README only documented `docker run --rm -p 8080:8080 formal-ai`, which
   neither starts Telegram nor grants the privileges needed by DinD.

## Fixes

- Replaced the final Docker stage with `konard/box-dind:2.1.1`.
- Installed `start-command` via Bun and exposed the `$ --isolated docker
  --auto-remove-docker-container --` runner contract through
  `FORMAL_AI_START_RUNNER`.
- Kept `/usr/local/bin/dind-entrypoint.sh` as the image entrypoint, set
  `DIND_STORAGE_DRIVER="vfs"`, and made Telegram polling the default command.
- Added `scripts/verify-docker-runtime.sh` and copied it into the image as
  `verify-formal-ai-dind`.
- Updated `data/seed/environments.lino`, README, `REQUIREMENTS.md`, and
  `ARCHITECTURE.md` to describe the Docker-in-Docker Telegram runtime.
- Added regression tests in `tests/unit/docker_runtime.rs` and traceability
  checks in `tests/unit/docs_requirements.rs`.

## Verification Plan

- `cargo test --test unit docker_runtime -- --nocapture`
- `cargo test --test unit issue_195 -- --nocapture`
- `cargo test --test integration cli_environments_command_lists_every_supported_surface -- --nocapture`
- `cargo fmt --all -- --check`
- `cargo clippy --all-targets --all-features`
- `cargo test --all-features --verbose`
- `rust-script scripts/check-file-size.rs`

Local Docker is not installed in the prepared workspace, so the actual image
build and `verify-formal-ai-dind` runtime check must run in CI or a Docker
host. The README commands are the intended manual verification path.
