import assert from "node:assert/strict";
import { test } from "node:test";
import { createRequire } from "node:module";

const require = createRequire(import.meta.url);
const {
  DEFAULT_IMAGE,
  resolveImage,
  resolveServerPort,
  serviceDefinitions,
  serviceKeys,
  describeService,
  createServiceControl,
} = require("../lib/service-control.cjs");

// A scripted `docker` runner: `responses` maps an argv prefix (joined with " ")
// to the {code,stdout,stderr} it should return; unmatched calls succeed empty.
// Every call is recorded in `calls` so tests assert the exact argument vectors.
function makeRunner(responses = {}) {
  const calls = [];
  const runDocker = async (args) => {
    calls.push(args);
    const key = args.join(" ");
    for (const prefix of Object.keys(responses)) {
      if (key === prefix || key.startsWith(`${prefix} `)) {
        return responses[prefix];
      }
    }
    return { code: 0, stdout: "", stderr: "" };
  };
  return { runDocker, calls };
}

test("resolveImage falls back to the GHCR default and honors the override", () => {
  assert.equal(resolveImage({}), DEFAULT_IMAGE);
  assert.equal(resolveImage({ FORMAL_AI_DOCKER_IMAGE: "  " }), DEFAULT_IMAGE);
  assert.equal(resolveImage({ FORMAL_AI_DOCKER_IMAGE: "local/img:1" }), "local/img:1");
});

test("resolveServerPort defaults to 8080 and accepts a valid override", () => {
  assert.equal(resolveServerPort({}), 8080);
  assert.equal(resolveServerPort({ FORMAL_AI_SERVER_PORT: "9000" }), 9000);
  assert.equal(resolveServerPort({ FORMAL_AI_SERVER_PORT: "nope" }), 8080);
  assert.equal(resolveServerPort({ FORMAL_AI_SERVER_PORT: "0" }), 8080);
});

test("serviceKeys exposes exactly the telegram and server services", () => {
  assert.deepEqual(serviceKeys(), ["telegram", "server"]);
});

test("telegram run args keep the default command and inject the token", () => {
  const services = serviceDefinitions();
  const args = services.telegram.buildRunArgs({ token: "secret-token" });
  assert.deepEqual(args, [
    "run",
    "-d",
    "--name",
    "formal-ai-telegram",
    "--restart",
    "unless-stopped",
    "--privileged",
    "-e",
    "TELEGRAM_BOT_TOKEN=secret-token",
    "-v",
    "formal-ai-telegram-docker:/var/lib/docker",
    DEFAULT_IMAGE,
  ]);
  // The default image command (formal-ai telegram --mode polling) is NOT
  // overridden, so the image's ENTRYPOINT/CMD runs the bot.
  assert.ok(!args.includes("serve"));
});

test("server run args publish a loopback port and override the command to serve", () => {
  const services = serviceDefinitions({ FORMAL_AI_SERVER_PORT: "8080" });
  const args = services.server.buildRunArgs();
  assert.ok(args.includes("-p"));
  assert.ok(args.includes("127.0.0.1:8080:8080"));
  // Each DinD service gets its own inner-Docker volume so two daemons never
  // contend for one /var/lib/docker.
  assert.ok(args.includes("formal-ai-server-docker:/var/lib/docker"));
  assert.ok(!args.includes("formal-ai-telegram-docker:/var/lib/docker"));
  const tail = args.slice(args.indexOf(DEFAULT_IMAGE) + 1);
  assert.deepEqual(tail, ["formal-ai", "serve", "--host", "0.0.0.0", "--port", "8080"]);
});

test("describeService advertises a loopback URL for the server only", () => {
  const services = serviceDefinitions();
  assert.equal(describeService(services.telegram).url, undefined);
  assert.equal(describeService(services.server).url, "http://127.0.0.1:8080");
});

test("statusAll reports running, stopped, and absent containers", async () => {
  const { runDocker } = makeRunner({
    "inspect -f {{.State.Running}} formal-ai-telegram": { code: 0, stdout: "true\n", stderr: "" },
    "inspect -f {{.State.Running}} formal-ai-server": { code: 1, stdout: "", stderr: "no such object" },
  });
  const control = createServiceControl({ runDocker });
  const status = await control.statusAll();
  assert.equal(status.dockerAvailable, true);
  const telegram = status.services.find((s) => s.key === "telegram");
  const server = status.services.find((s) => s.key === "server");
  assert.equal(telegram.state, "running");
  assert.equal(telegram.running, true);
  assert.equal(server.state, "absent");
  assert.equal(server.running, false);
});

test("status reports docker-unavailable without shelling out", async () => {
  const { runDocker, calls } = makeRunner();
  const control = createServiceControl({ runDocker, dockerAvailable: () => false });
  const status = await control.status("telegram");
  assert.equal(status.state, "docker-unavailable");
  assert.equal(status.running, false);
  assert.equal(calls.length, 0);
});

test("start refuses the telegram bot when the token is missing", async () => {
  const { runDocker, calls } = makeRunner();
  const control = createServiceControl({ runDocker, env: {} });
  const result = await control.start("telegram", {});
  assert.equal(result.ok, false);
  assert.equal(result.state, "missing-config");
  assert.deepEqual(result.missing, ["TELEGRAM_BOT_TOKEN"]);
  // No docker command should run for a misconfigured start.
  assert.equal(calls.length, 0);
});

test("start reaps a stale container then runs the telegram bot with the token", async () => {
  const { runDocker, calls } = makeRunner({
    "inspect -f {{.State.Running}} formal-ai-telegram": { code: 1, stdout: "", stderr: "absent" },
    "run -d --name formal-ai-telegram": { code: 0, stdout: "container-id-123\n", stderr: "" },
  });
  const control = createServiceControl({ runDocker });
  const result = await control.start("telegram", { token: "tok" });
  assert.equal(result.ok, true);
  assert.equal(result.state, "running");
  assert.equal(result.containerId, "container-id-123");
  // status -> rm -f (reap) -> run
  assert.deepEqual(calls[1], ["rm", "-f", "formal-ai-telegram"]);
  assert.ok(calls[2].includes("TELEGRAM_BOT_TOKEN=tok"));
});

test("start is idempotent when the container is already running", async () => {
  const { runDocker, calls } = makeRunner({
    "inspect -f {{.State.Running}} formal-ai-server": { code: 0, stdout: "true\n", stderr: "" },
  });
  const control = createServiceControl({ runDocker });
  const result = await control.start("server", {});
  assert.equal(result.ok, true);
  assert.equal(result.alreadyRunning, true);
  // Only the status probe should run; no rm/run.
  assert.ok(calls.every((c) => c[0] === "inspect"));
});

test("start surfaces a docker run failure as an error with the reason", async () => {
  const { runDocker } = makeRunner({
    "inspect -f {{.State.Running}} formal-ai-server": { code: 1, stdout: "", stderr: "absent" },
    "run -d --name formal-ai-server": { code: 1, stdout: "", stderr: "port already allocated" },
  });
  const control = createServiceControl({ runDocker });
  const result = await control.start("server", {});
  assert.equal(result.ok, false);
  assert.equal(result.state, "error");
  assert.match(result.reason, /port already allocated/);
});

test("stop stops and removes the container", async () => {
  const { runDocker, calls } = makeRunner({
    "stop formal-ai-telegram": { code: 0, stdout: "formal-ai-telegram\n", stderr: "" },
  });
  const control = createServiceControl({ runDocker });
  const result = await control.stop("telegram");
  assert.equal(result.ok, true);
  assert.equal(result.state, "stopped");
  assert.deepEqual(calls[0], ["stop", "formal-ai-telegram"]);
  assert.deepEqual(calls[1], ["rm", "-f", "formal-ai-telegram"]);
});

test("stop treats an already-absent container as success", async () => {
  const { runDocker } = makeRunner({
    "stop formal-ai-server": { code: 1, stdout: "", stderr: "Error: No such container: formal-ai-server" },
  });
  const control = createServiceControl({ runDocker });
  const result = await control.stop("server");
  assert.equal(result.ok, true);
  assert.equal(result.state, "stopped");
  assert.equal(result.alreadyStopped, true);
});

test("createServiceControl rejects a missing runDocker dependency", () => {
  assert.throws(() => createServiceControl({}), /requires a runDocker/);
});

test("unknown service keys raise a clear error", async () => {
  const { runDocker } = makeRunner();
  const control = createServiceControl({ runDocker });
  await assert.rejects(() => control.status("nope"), /unknown service: nope/);
});
