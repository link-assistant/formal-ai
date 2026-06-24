import assert from "node:assert/strict";
import { test } from "node:test";
import { createRequire } from "node:module";

const require = createRequire(import.meta.url);
const { createDockerDetector, candidateDockerPaths } = require("../lib/docker-detect.cjs");

// A scripted spawnSync: `byBinary` maps a resolved binary path to the result the
// fake `docker version` call returns ({status, stdout} or {error}). Every call is
// recorded so tests can assert which binary was probed and how often.
function makeSpawnSync(byBinary = {}) {
  const calls = [];
  const spawnSync = (bin, args) => {
    calls.push({ bin, args });
    if (Object.prototype.hasOwnProperty.call(byBinary, bin)) {
      return byBinary[bin];
    }
    // Default: ENOENT, mirroring a binary that is not on PATH.
    const error = new Error(`spawn ${bin} ENOENT`);
    error.code = "ENOENT";
    return { error, status: null, stdout: "", stderr: "" };
  };
  return { spawnSync, calls };
}

// A controllable clock so TTL behaviour is deterministic.
function makeClock(start = 0) {
  let t = start;
  return { now: () => t, advance: (ms) => { t += ms; } };
}

const SERVER_OK = { status: 0, stdout: "27.1.1\n", stderr: "", error: null };
const DAEMON_DOWN = { status: 1, stdout: "", stderr: "Cannot connect to the Docker daemon", error: null };

test("candidateDockerPaths puts absolute install locations before the bare PATH fallback (unix)", () => {
  const paths = candidateDockerPaths({}, "darwin");
  assert.ok(paths.includes("/usr/local/bin/docker"));
  assert.ok(paths.includes("/opt/homebrew/bin/docker"));
  assert.ok(paths.includes("/Applications/Docker.app/Contents/Resources/bin/docker"));
  // The bare "docker" PATH lookup must be LAST so absolute paths win on GUI launch.
  assert.equal(paths[paths.length - 1], "docker");
  assert.ok(paths.indexOf("/usr/local/bin/docker") < paths.indexOf("docker"));
});

test("candidateDockerPaths uses Docker Desktop's Windows install path", () => {
  const paths = candidateDockerPaths({ ProgramFiles: "C:\\Program Files" }, "win32");
  assert.ok(
    paths.includes("C:\\Program Files\\Docker\\Docker\\resources\\bin\\docker.exe"),
    `expected the Windows Docker Desktop path, got ${JSON.stringify(paths)}`,
  );
  assert.equal(paths[paths.length - 1], "docker.exe");
});

test("FORMAL_AI_DOCKER_BIN override is probed first", () => {
  const paths = candidateDockerPaths({ FORMAL_AI_DOCKER_BIN: "/custom/docker" }, "linux");
  assert.equal(paths[0], "/custom/docker");
});

// THE issue-#541 regression: a macOS .app launched from Finder has an empty PATH,
// so a bare `spawn("docker")` returns ENOENT — but Docker IS installed at
// /usr/local/bin/docker and the daemon IS running. The detector must resolve the
// absolute path and report Docker as available.
test("detects an installed+running Docker even when PATH does not contain docker (GUI launch)", () => {
  const { spawnSync, calls } = makeSpawnSync({
    // Bare "docker" (PATH lookup) fails with ENOENT — the GUI-launch symptom.
    docker: { error: Object.assign(new Error("spawn docker ENOENT"), { code: "ENOENT" }) },
    // The real binary lives here and the daemon answers.
    "/usr/local/bin/docker": SERVER_OK,
  });
  const detector = createDockerDetector({
    spawnSync,
    platform: "darwin",
    existsSync: (p) => p === "/usr/local/bin/docker",
    now: () => 0,
  });
  assert.equal(detector.resolveDockerBinary(), "/usr/local/bin/docker");
  assert.equal(detector.dockerIsAvailable(), true);
  // It must have probed the resolved absolute path, never the bare "docker".
  assert.ok(calls.some((c) => c.bin === "/usr/local/bin/docker"));
  assert.ok(!calls.some((c) => c.bin === "docker"));
});

test("reports unavailable when the CLI exists but the daemon is down", () => {
  const { spawnSync } = makeSpawnSync({ "/usr/local/bin/docker": DAEMON_DOWN });
  const detector = createDockerDetector({
    spawnSync,
    platform: "linux",
    existsSync: (p) => p === "/usr/local/bin/docker",
    now: () => 0,
  });
  assert.equal(detector.dockerIsAvailable(), false);
});

// The old bug: the probe was cached forever, so starting Docker after the app
// opened never took effect. With the TTL, a failed probe is re-run after failTtl
// and a daemon that came up is detected without restarting the app.
test("re-probes after the failure TTL so a later-started daemon becomes available", () => {
  const clock = makeClock(1000);
  let daemonUp = false;
  const spawnSync = (bin) => {
    assert.equal(bin, "/usr/local/bin/docker");
    return daemonUp ? SERVER_OK : DAEMON_DOWN;
  };
  const detector = createDockerDetector({
    spawnSync,
    platform: "linux",
    existsSync: (p) => p === "/usr/local/bin/docker",
    now: clock.now,
    failTtlMs: 3000,
    okTtlMs: 30000,
  });
  assert.equal(detector.dockerIsAvailable(), false);
  // Within the failure TTL: cached, still false (no re-probe even though daemon up).
  daemonUp = true;
  clock.advance(1000);
  assert.equal(detector.dockerIsAvailable(), false);
  // After the failure TTL elapses, it re-probes and now sees the daemon.
  clock.advance(3000);
  assert.equal(detector.dockerIsAvailable(), true);
});

test("caches a positive result within the ok TTL (does not spawn on every call)", () => {
  const clock = makeClock(0);
  let probes = 0;
  const spawnSync = () => {
    probes += 1;
    return SERVER_OK;
  };
  const detector = createDockerDetector({
    spawnSync,
    platform: "linux",
    existsSync: () => true,
    now: clock.now,
    okTtlMs: 30000,
  });
  assert.equal(detector.dockerIsAvailable(), true);
  clock.advance(10000);
  assert.equal(detector.dockerIsAvailable(), true);
  assert.equal(probes, 1, "a cached positive result must not re-spawn docker within the TTL");
});

test("falls back to the bare PATH lookup when no absolute install location exists", () => {
  const { spawnSync } = makeSpawnSync({ docker: SERVER_OK });
  const detector = createDockerDetector({
    spawnSync,
    platform: "linux",
    existsSync: () => false,
    now: () => 0,
  });
  // No absolute path exists, so it must fall back to "docker" (works on a
  // terminal launch where PATH is inherited).
  assert.equal(detector.resolveDockerBinary(), "docker");
  assert.equal(detector.dockerIsAvailable(), true);
});

test("invalidate() forces a fresh binary resolution and probe", () => {
  let exists = false;
  let probes = 0;
  const spawnSync = (bin) => {
    probes += 1;
    return bin === "/usr/local/bin/docker" ? SERVER_OK : { status: 1, stdout: "", error: null };
  };
  const detector = createDockerDetector({
    spawnSync,
    platform: "linux",
    existsSync: (p) => exists && p === "/usr/local/bin/docker",
    now: () => 0,
  });
  // First resolution: nothing installed yet -> PATH fallback, daemon down.
  assert.equal(detector.resolveDockerBinary(), "docker");
  // Install Docker, then invalidate to force re-resolution.
  exists = true;
  detector.invalidate();
  assert.equal(detector.resolveDockerBinary(), "/usr/local/bin/docker");
  assert.equal(detector.dockerIsAvailable(), true);
  assert.ok(probes >= 1);
});
