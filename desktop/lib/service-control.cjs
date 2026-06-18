"use strict";

// Desktop service-control: start, stop, and report the prepared Docker
// containers for the Telegram bot and the OpenAI-compatible agentic server with
// a single action.
//
// Issue #438 (follow-up): the desktop app must start/stop both the Telegram bot
// container and the OpenAI-compatible server container with one click, and the
// same flow must be easy to drive on a server. This module holds the lifecycle
// logic (which `docker` arguments each service needs, how running state is read,
// how stale containers are reaped) behind an injected `runDocker` runner so the
// whole contract is unit-testable without a live Docker daemon. `main.cjs` wires
// it to a real `docker` child process and exposes it over IPC; servers reuse the
// exact same argument vectors via `docker compose` / `docker run`.

const DEFAULT_IMAGE = "ghcr.io/link-assistant/formal-ai:latest";
const DEFAULT_SERVER_PORT = 8080;
// Each service gets its OWN inner-Docker volume: two DinD daemons cannot share a
// single /var/lib/docker, so the bot and the server must not collide if both run.
const TELEGRAM_VOLUME = "formal-ai-telegram-docker:/var/lib/docker";
const SERVER_VOLUME = "formal-ai-server-docker:/var/lib/docker";

// Resolve the image once so a locally built image or an optional Docker Hub
// mirror can be substituted with the same `FORMAL_AI_DOCKER_IMAGE` override the
// root `compose.yaml` already honors.
function resolveImage(env = {}) {
  const raw = String(env.FORMAL_AI_DOCKER_IMAGE || "").trim();
  return raw || DEFAULT_IMAGE;
}

function resolveServerPort(env = {}) {
  const parsed = Number.parseInt(String(env.FORMAL_AI_SERVER_PORT || ""), 10);
  return Number.isInteger(parsed) && parsed > 0 ? parsed : DEFAULT_SERVER_PORT;
}

// The two managed services. Both run from the single prepared image: the
// Telegram bot keeps the image's default `formal-ai telegram --mode polling`
// command, and the server overrides the command with `formal-ai serve` so the
// OpenAI-compatible API (and its in-container Docker sandbox for agentic mode)
// listens on a published loopback port.
function serviceDefinitions(env = {}) {
  const image = resolveImage(env);
  const serverPort = resolveServerPort(env);
  return {
    telegram: {
      key: "telegram",
      label: "Telegram bot",
      container: "formal-ai-telegram",
      image,
      requiresEnv: ["TELEGRAM_BOT_TOKEN"],
      buildRunArgs(options = {}) {
        const token = String(
          options.token || (options.env && options.env.TELEGRAM_BOT_TOKEN) || "",
        ).trim();
        return [
          "run",
          "-d",
          "--name",
          this.container,
          "--restart",
          "unless-stopped",
          "--privileged",
          "-e",
          `TELEGRAM_BOT_TOKEN=${token}`,
          "-v",
          TELEGRAM_VOLUME,
          this.image,
        ];
      },
    },
    server: {
      key: "server",
      label: "OpenAI-compatible server",
      container: "formal-ai-server",
      image,
      port: serverPort,
      requiresEnv: [],
      buildRunArgs() {
        return [
          "run",
          "-d",
          "--name",
          this.container,
          "--restart",
          "unless-stopped",
          "--privileged",
          "-p",
          `127.0.0.1:${this.port}:${this.port}`,
          "-v",
          SERVER_VOLUME,
          this.image,
          "formal-ai",
          "serve",
          "--host",
          "0.0.0.0",
          "--port",
          String(this.port),
        ];
      },
    },
  };
}

function serviceKeys() {
  return Object.keys(serviceDefinitions());
}

// Read what a service exposes for the UI/server status panels without touching
// Docker: stable label, container name, image, and (for the server) the
// published loopback URL.
function describeService(service) {
  const summary = {
    key: service.key,
    label: service.label,
    container: service.container,
    image: service.image,
  };
  if (typeof service.port === "number") {
    summary.port = service.port;
    summary.url = `http://127.0.0.1:${service.port}`;
  }
  return summary;
}

function normalizeResult(result) {
  if (!result || typeof result !== "object") {
    return { code: 1, stdout: "", stderr: "" };
  }
  return {
    code: typeof result.code === "number" ? result.code : result.code ? 1 : 0,
    stdout: String(result.stdout || ""),
    stderr: String(result.stderr || ""),
  };
}

function createServiceControl(options = {}) {
  const env = options.env || {};
  const services = serviceDefinitions(env);
  const runDocker = options.runDocker;
  const dockerAvailable =
    typeof options.dockerAvailable === "function" ? options.dockerAvailable : () => true;

  if (typeof runDocker !== "function") {
    throw new Error("createServiceControl requires a runDocker(args) function");
  }

  function lookup(key) {
    const service = services[key];
    if (!service) {
      throw new Error(`unknown service: ${String(key)}`);
    }
    return service;
  }

  async function run(args) {
    return normalizeResult(await runDocker(args));
  }

  // Resolve running/stopped/absent from a single `docker inspect`. Absent
  // containers exit non-zero, which we map to "absent" rather than an error so
  // the UI can show a clean "stopped" state before the first start.
  async function status(key) {
    const service = lookup(key);
    const base = describeService(service);
    if (!dockerAvailable()) {
      return { ...base, state: "docker-unavailable", running: false };
    }
    const result = await run([
      "inspect",
      "-f",
      "{{.State.Running}}",
      service.container,
    ]);
    if (result.code !== 0) {
      return { ...base, state: "absent", running: false };
    }
    const running = result.stdout.trim() === "true";
    return { ...base, state: running ? "running" : "stopped", running };
  }

  async function statusAll() {
    const entries = await Promise.all(serviceKeys().map((key) => status(key)));
    return { dockerAvailable: Boolean(dockerAvailable()), services: entries };
  }

  async function start(key, startOptions = {}) {
    const service = lookup(key);
    const base = describeService(service);
    if (!dockerAvailable()) {
      return {
        ok: false,
        ...base,
        state: "docker-unavailable",
        running: false,
        reason: "Docker is not available on this machine",
      };
    }

    // Required configuration (the Telegram token) must be present before we ask
    // Docker to start anything, so the failure is a clear message instead of a
    // crash-looping container.
    const missing = (service.requiresEnv || []).filter((name) => {
      const provided =
        (startOptions.env && startOptions.env[name]) ||
        (name === "TELEGRAM_BOT_TOKEN" ? startOptions.token : "") ||
        env[name];
      return !String(provided || "").trim();
    });
    if (missing.length > 0) {
      return {
        ok: false,
        ...base,
        state: "missing-config",
        running: false,
        reason: `missing required configuration: ${missing.join(", ")}`,
        missing,
      };
    }

    const current = await status(key);
    if (current.running) {
      return { ok: true, ...base, state: "running", running: true, alreadyRunning: true };
    }

    // Reap any stale stopped/created container with the same name so a fresh
    // `docker run --name` does not collide. Best-effort: ignore the result.
    await run(["rm", "-f", service.container]);

    const runResult = await run(service.buildRunArgs({ ...startOptions, env }));
    if (runResult.code !== 0) {
      return {
        ok: false,
        ...base,
        state: "error",
        running: false,
        reason: (runResult.stderr || runResult.stdout || "docker run failed").trim(),
      };
    }
    return {
      ok: true,
      ...base,
      state: "running",
      running: true,
      containerId: runResult.stdout.trim(),
    };
  }

  async function stop(key) {
    const service = lookup(key);
    const base = describeService(service);
    if (!dockerAvailable()) {
      return {
        ok: false,
        ...base,
        state: "docker-unavailable",
        running: false,
        reason: "Docker is not available on this machine",
      };
    }
    const stopResult = await run(["stop", service.container]);
    // Remove the container so a later start is clean; ignore removal failures
    // (e.g. the container was already gone).
    await run(["rm", "-f", service.container]);
    if (stopResult.code !== 0) {
      const reason = (stopResult.stderr || "").toLowerCase();
      // `docker stop` on an absent container is a no-op success for our purposes.
      if (reason.includes("no such container")) {
        return { ok: true, ...base, state: "stopped", running: false, alreadyStopped: true };
      }
      return {
        ok: false,
        ...base,
        state: "error",
        running: false,
        reason: (stopResult.stderr || stopResult.stdout || "docker stop failed").trim(),
      };
    }
    return { ok: true, ...base, state: "stopped", running: false };
  }

  return {
    services,
    describe: () => serviceKeys().map((key) => describeService(services[key])),
    status,
    statusAll,
    start,
    stop,
  };
}

module.exports = {
  DEFAULT_IMAGE,
  DEFAULT_SERVER_PORT,
  resolveImage,
  resolveServerPort,
  serviceDefinitions,
  serviceKeys,
  describeService,
  createServiceControl,
};
