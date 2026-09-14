#!/usr/bin/env node
// Issue #1017 / D15 — reproduce, without a network and without electron-builder,
// the two shipped behaviours that turned a complete macOS build into a red job.
//
// Part 1 (`builder-util`): a rejection recorded by an `AsyncTaskManager` is
//   terminal. `errors` is append-only, so a task that succeeds afterwards cannot
//   retract it -- which is why run 95255998673 failed *after* both
//   `artifactBuildCompleted` events had fired.
//
// Part 2 (`got`): `timeout: { request: N }` is a *total* deadline. A connection
//   that is accepted and then never written to burns all of N before anything
//   notices, whereas a `socket` sub-timeout detects the same dead connection in
//   a fraction of it. This is the measurement behind the upstream suggestion in
//   `dev/log/issues/1017/pulls/1018/upstream-reports/electron-builder-async-task-manager-stale-error.md`.
//
// Usage (installs the packages into a scratch directory; no arguments):
//
//   node experiments/issue-1017-electron-builder-stale-error/run.mjs
//
// Recorded result on 2026-08-17 with builder-util@26.15.3 and got@11.8.6:
//   part 1: rejected: Timeout awaiting 'request' for 600000ms   (the success is lost)
//   part 2: request-only deadline fired after 3024 ms ("awaiting 'request'"); the same
//           dead socket with a `socket` sub-timeout fired after 1009 ms ("awaiting 'socket'")

import { createRequire } from "node:module";
import { execFileSync } from "node:child_process";
import fs from "node:fs";
import net from "node:net";
import os from "node:os";
import path from "node:path";

// The versions electron-builder 26.15.7 resolves to in `desktop/package-lock.json`.
const specs = ["builder-util@26.15.3", "builder-util-runtime@9.7.0", "got@11.8.6"];
const root = fs.mkdtempSync(path.join(os.tmpdir(), "issue-1017-eb-repro-"));
fs.writeFileSync(
  path.join(root, "package.json"),
  `${JSON.stringify({ name: "issue-1017-eb-repro", private: true }, null, 2)}\n`,
);
console.log(`installing ${specs.join(" ")} into ${root}`);
execFileSync(
  "npm",
  ["install", "--no-save", "--no-audit", "--no-fund", "--ignore-scripts", ...specs],
  { cwd: root, stdio: "inherit" },
);
const require = createRequire(path.join(root, "package.json"));

console.log("\n== part 1: a recorded rejection outlives the success that followed it ==");
const { AsyncTaskManager } = require("builder-util");
const { CancellationToken } = require("builder-util-runtime");

const manager = new AsyncTaskManager(new CancellationToken());
manager.addTask(Promise.reject(new Error("Timeout awaiting 'request' for 600000ms")));
manager.addTask(Promise.resolve("artifact written"));

let part1Rejected = false;
try {
  const resolved = await manager.awaitTasks();
  console.log("resolved:", resolved);
} catch (error) {
  part1Rejected = true;
  console.log("rejected:", error.message);
  console.log(
    "  -> the second task resolved, and `errors` has no way to record that; " +
      "awaitTasks() throws errors[0] regardless",
  );
}

console.log("\n== part 2: a total deadline cannot see a dead connection early ==");
const got = require("got");
// Accepts the connection, then writes nothing -- the stalled socket of run 95255998673.
const server = net.createServer(() => {});
await new Promise((resolve) => server.listen(0, "127.0.0.1", resolve));
const url = `http://127.0.0.1:${server.address().port}/dmgbuild-bundle.tar.gz`;

async function timeToFailure(timeout) {
  const started = process.hrtime.bigint();
  try {
    await got(url, { timeout, retry: 0 });
    return { ms: null, code: "unexpected success" };
  } catch (error) {
    return {
      ms: Number((process.hrtime.bigint() - started) / 1_000_000n),
      code: error.code,
      message: error.message,
    };
  }
}

const requestOnly = await timeToFailure({ request: 3000 });
const withSocket = await timeToFailure({ request: 3000, socket: 1000 });
server.close();

console.log(
  `request-only deadline: ${requestOnly.ms} ms (${requestOnly.code}) -- ${requestOnly.message}`,
);
console.log(
  `with socket sub-timeout: ${withSocket.ms} ms (${withSocket.code}) -- ${withSocket.message}`,
);
console.log(
  "  -> the same dead socket is detected in a third of the time; at electron-builder's\n" +
    "     production numbers that is 600 s of a job's clock versus a few seconds",
);

fs.rmSync(root, { recursive: true, force: true });

const part2Faster = withSocket.ms !== null && requestOnly.ms !== null && withSocket.ms < requestOnly.ms;
if (!part1Rejected || !part2Faster) {
  console.error(
    `FAILED: part1Rejected=${part1Rejected} part2Faster=${part2Faster} -- upstream behaviour changed`,
  );
  process.exit(1);
}
console.log("\nOK: both behaviours reproduce as reported upstream");
