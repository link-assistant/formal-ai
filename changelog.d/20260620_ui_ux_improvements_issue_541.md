---
bump: minor
---

### Fixed
- Desktop app no longer reports "Docker unavailable" when Docker Desktop is installed and running. The `docker` binary is now resolved across well-known install locations (`/usr/local/bin`, `/opt/homebrew/bin`, `/Applications/Docker.app/...`, Windows `Program Files`, NixOS), fixing the GUI-launch PATH gap, and availability is re-probed on a short TTL so a daemon started after the app opened is detected without a restart (issue #541).
- Collapsed reasoning preview now shows the current step in full (at least one whole paragraph) instead of clipping it to a single ellipsised line, so the thinking trace is actually readable while collapsed (issue #541).

### Added
- Minimum message animation time setting (Settings → "Minimum thinking animation"). Reasoning steps now reveal one-by-one and the answer body fades in only after the trace has played out, so the deterministic engine's instant answers still feel considered. Defaults to 2 seconds; set it to 0 for immediate display. Honours `prefers-reduced-motion` (issue #541).
- `FORMAL_AI_DESKTOP_DEBUG` environment variable enables verbose desktop diagnostics (Docker binary resolution and probe results) to help diagnose environment-specific problems.
- `FORMAL_AI_DOCKER_BIN` environment variable overrides the resolved `docker` binary path.
