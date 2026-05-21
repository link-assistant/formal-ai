# Issue 195 Online Research

## Link Foundation Box

- Repository: <https://github.com/link-foundation/box>
- Box README snapshot: `box-readme.md`
- DinD Dockerfile snapshot: `box-dind-dockerfile`
- Release list snapshot: `box-releases.txt`

Relevant facts:

- `konard/box-dind` is the full Box image plus Docker Engine.
- The DinD variant preserves `/usr/local/bin/dind-entrypoint.sh` as the
  entrypoint and leaves the final image as root so it can start `dockerd`.
- Default runtime requires `docker run --privileged`; `--runtime=sysbox-runc`
  is documented as the safer option where available.
- The README warns against bind-mounting `/var/run/docker.sock` because that
  breaks the per-container Docker daemon isolation model.
- The latest release observed during collection was `v2.1.1`, so
  `konard/box-dind:2.1.1` is the pinned base image.

## Link Foundation Start

- Repository: <https://github.com/link-foundation/start>
- README snapshot: `start-readme.md`

Relevant facts:

- The package is named `start-command`, and the installed command is `$`.
- Docker process isolation is invoked with `--isolated docker`.
- The issue text says `--isolation docker`; the current upstream README uses
  `--isolated docker`, so the implementation follows the current flag.
- Command output and execution metadata are saved under
  `/tmp/start-command/logs/`.
- `--auto-remove-docker-container` removes disposable nested containers after
  execution.

## Related Link Assistant Work

- Hive Mind DinD Dockerfile snapshot: `hive-mind-Dockerfile.dind`
- Hive Mind standard Dockerfile snapshot: `hive-mind-Dockerfile`
- Hive Mind image verification script snapshot:
  `hive-mind-verify-docker-image.sh`

Relevant facts:

- Hive Mind already uses `FROM konard/box-dind:2.1.1` for its DinD image.
- It sets `DIND_STORAGE_DRIVER="vfs"` for compatibility with nested Docker on
  common overlay-backed hosts.
- It installs `start-command` with Bun and keeps the DinD entrypoint as the
  final image entrypoint.
