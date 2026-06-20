---
bump: minor
---

### Fixed
- Desktop app no longer reports "Docker unavailable" when Docker Desktop is installed and running. The `docker` binary is now resolved across well-known install locations (`/usr/local/bin`, `/opt/homebrew/bin`, `/Applications/Docker.app/...`, Windows `Program Files`, NixOS), fixing the GUI-launch PATH gap, and availability is re-probed on a short TTL so a daemon started after the app opened is detected without a restart (issue #541).

### Added
- `FORMAL_AI_DESKTOP_DEBUG` environment variable enables verbose desktop diagnostics (Docker binary resolution and probe results) to help diagnose environment-specific problems.
- `FORMAL_AI_DOCKER_BIN` environment variable overrides the resolved `docker` binary path.
