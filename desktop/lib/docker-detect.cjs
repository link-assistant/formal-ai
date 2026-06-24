"use strict";

// Desktop Docker detection (issue #541, R2): decide whether a usable Docker
// daemon is reachable. Two real-world failure modes made the desktop app report
// "Docker unavailable" even when Docker Desktop was installed AND running:
//
//   1. PATH gap on GUI launch. A macOS `.app` bundle (and many Linux desktop
//      launchers) started from Finder/Dock/GNOME does NOT inherit the user's
//      interactive shell PATH. The `docker` CLI lives in /usr/local/bin or
//      /opt/homebrew/bin, which are absent from the minimal GUI PATH, so a bare
//      `spawn("docker")` failed with ENOENT and we wrongly concluded Docker was
//      missing.
//
//   2. Probe cached forever. The previous implementation memoised the result on
//      the first call, so a user who started Docker Desktop *after* opening the
//      app never saw it become available without a full restart.
//
// This module resolves the `docker` binary across well-known install locations
// (plus PATH and a FORMAL_AI_DOCKER_BIN override) and re-probes on a short TTL so
// a daemon that comes up later is detected. Every side-effecting dependency
// (spawnSync, existsSync, clock) is injected so the whole contract is
// unit-testable without a real Docker daemon, matching the rest of desktop/lib.

const DEFAULT_OK_TTL_MS = 30000;
const DEFAULT_FAIL_TTL_MS = 3000;
const DEFAULT_PROBE_TIMEOUT_MS = 5000;

// Well-known install locations for the `docker` CLI, in priority order. Absolute
// paths are checked with existsSync first so a GUI-launched app finds Docker even
// with an empty PATH; a bare "docker" is appended last so a terminal launch (or
// any environment where PATH *is* inherited) still works.
function candidateDockerPaths(env = {}, platform = process.platform) {
  const candidates = [];
  const override = String(env.FORMAL_AI_DOCKER_BIN || "").trim();
  if (override) {
    candidates.push(override);
  }
  if (platform === "win32") {
    const programFiles = env.ProgramFiles || "C:\\Program Files";
    const programW6432 = env.ProgramW6432 || programFiles;
    candidates.push(
      `${programFiles}\\Docker\\Docker\\resources\\bin\\docker.exe`,
      `${programW6432}\\Docker\\Docker\\resources\\bin\\docker.exe`,
      "docker.exe",
    );
  } else {
    candidates.push(
      "/usr/local/bin/docker",
      "/opt/homebrew/bin/docker",
      "/usr/bin/docker",
      "/Applications/Docker.app/Contents/Resources/bin/docker",
      // NixOS / nix-darwin profile.
      "/run/current-system/sw/bin/docker",
      "docker",
    );
  }
  return candidates;
}

function isPathFallback(candidate) {
  return candidate === "docker" || candidate === "docker.exe";
}

function createDockerDetector(options = {}) {
  const env = options.env || {};
  const platform = options.platform || process.platform;
  const spawnSync = options.spawnSync;
  const existsSync = typeof options.existsSync === "function" ? options.existsSync : () => false;
  const now = typeof options.now === "function" ? options.now : () => Date.now();
  const log = typeof options.log === "function" ? options.log : () => {};
  const okTtl = Number.isFinite(options.okTtlMs) ? options.okTtlMs : DEFAULT_OK_TTL_MS;
  const failTtl = Number.isFinite(options.failTtlMs) ? options.failTtlMs : DEFAULT_FAIL_TTL_MS;
  const probeTimeout = Number.isFinite(options.probeTimeoutMs)
    ? options.probeTimeoutMs
    : DEFAULT_PROBE_TIMEOUT_MS;

  if (typeof spawnSync !== "function") {
    throw new Error("createDockerDetector requires a spawnSync(cmd, args, opts) function");
  }

  let binaryCache; // undefined = unresolved, string = resolved path or PATH fallback
  let probe = { available: false, checkedAt: 0, initialized: false };

  // Resolve (and cache) the docker binary. A cached *absolute* path is
  // re-validated with existsSync so an uninstall is noticed; a PATH fallback is
  // always kept.
  function resolveDockerBinary() {
    if (typeof binaryCache === "string") {
      if (isPathFallback(binaryCache)) {
        return binaryCache;
      }
      try {
        if (existsSync(binaryCache)) {
          return binaryCache;
        }
      } catch (_error) {
        /* fall through to re-resolution */
      }
      binaryCache = undefined;
    }
    for (const candidate of candidateDockerPaths(env, platform)) {
      if (isPathFallback(candidate)) {
        binaryCache = candidate;
        log("docker binary: using PATH lookup", candidate);
        return candidate;
      }
      let found = false;
      try {
        found = Boolean(existsSync(candidate));
      } catch (_error) {
        found = false;
      }
      if (found) {
        binaryCache = candidate;
        log("docker binary resolved at", candidate);
        return candidate;
      }
    }
    binaryCache = platform === "win32" ? "docker.exe" : "docker";
    return binaryCache;
  }

  // A single `docker version --format {{.Server.Version}}` call. It exits 0 with a
  // non-empty server version ONLY when the daemon is actually reachable, so it
  // distinguishes "CLI present but daemon down" from "fully usable".
  function probeDocker() {
    const bin = resolveDockerBinary();
    let available = false;
    let detail = "";
    try {
      const result = spawnSync(bin, ["version", "--format", "{{.Server.Version}}"], {
        stdio: ["ignore", "pipe", "pipe"],
        timeout: probeTimeout,
        encoding: "utf8",
      });
      if (result && result.error) {
        detail = result.error.message ? result.error.message : String(result.error);
      } else {
        const status = result && typeof result.status === "number" ? result.status : 1;
        const serverVersion = String((result && result.stdout) || "").trim();
        available = status === 0 && serverVersion.length > 0;
        detail = available ? `server ${serverVersion}` : `exit ${status}`;
      }
    } catch (error) {
      available = false;
      detail = error && error.message ? error.message : String(error);
    }
    log(`docker probe via ${bin}: ${available ? "available" : "unavailable"} (${detail})`);
    return available;
  }

  // TTL-cached availability. A positive result is cached for okTtl; a negative
  // result for the shorter failTtl so a daemon that comes up later is picked up
  // within a few seconds without restarting the app.
  function dockerIsAvailable() {
    const ts = now();
    const ttl = probe.available ? okTtl : failTtl;
    if (probe.initialized && ts - probe.checkedAt < ttl) {
      return probe.available;
    }
    const available = probeDocker();
    probe = { available, checkedAt: ts, initialized: true };
    return available;
  }

  // Force the next dockerIsAvailable()/resolveDockerBinary() to re-run from
  // scratch (used by tests and by an explicit "re-check" action).
  function invalidate() {
    probe = { available: false, checkedAt: 0, initialized: false };
    binaryCache = undefined;
  }

  return {
    resolveDockerBinary,
    dockerIsAvailable,
    invalidate,
    candidates: () => candidateDockerPaths(env, platform),
  };
}

module.exports = {
  createDockerDetector,
  candidateDockerPaths,
  DEFAULT_OK_TTL_MS,
  DEFAULT_FAIL_TTL_MS,
};
