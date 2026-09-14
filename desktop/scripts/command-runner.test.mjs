import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import test from "node:test";
import { createRequire } from "node:module";

const require = createRequire(import.meta.url);
const { createCommandRunner } = require("../lib/command-runner.cjs");
const repoRoot = path.resolve(import.meta.dirname, "../..");

test("production manifests and focused fallback limitations stay mapped", () => {
  const desktopManifest = JSON.parse(fs.readFileSync(path.join(repoRoot, "desktop/package.json")));
  const vscodeManifest = JSON.parse(fs.readFileSync(path.join(repoRoot, "vscode/package.json")));
  const cargoManifest = fs.readFileSync(path.join(repoRoot, "Cargo.toml"), "utf8");
  const caseStudy = fs.readFileSync(
    path.join(repoRoot, "docs/case-studies/issue-990/README.md"),
    "utf8",
  );

  assert.equal(desktopManifest.dependencies["command-stream"], "0.19.0");
  assert.equal(vscodeManifest.dependencies["command-stream"], undefined);
  assert.match(cargoManifest, /^command-stream = "=0\.16\.0"$/m);
  // Issue #189 is that a CommonJS host could not reach this library at all.
  // The guard used to grep the CommonJS entry for its `terminal-capture.mjs`
  // re-export, but 0.19.0 turned that entry into a thin `require(esm)` wrapper
  // and the re-export moved into the ESM graph behind it -- so the text was
  // gone while the capability was not. Asserting on the loaded exports asks the
  // question the grep was standing in for, and keeps asking it through
  // whatever the package does to its file layout next.
  const componentExports = require("command-stream");
  for (const name of ["captureTerminal", "openTerminal", "unrollTerminalFrames"]) {
    assert.equal(
      typeof componentExports[name],
      "function",
      `command-stream must reach ${name} from CommonJS`,
    );
  }
  for (const upstreamIssue of [189, 190, 191, 192]) {
    assert.match(caseStudy, new RegExp(`command-stream/issues/${upstreamIssue}`));
  }
});

test("production command adapter streams stdout/stderr and preserves a non-zero exit", async () => {
  const chunks = [];
  const runner = createCommandRunner();
  const result = await runner.run(
    process.execPath,
    ["-e", "process.stdout.write('out'); process.stderr.write('err'); process.exit(7)"],
    {
      onStdout: (chunk) => chunks.push(["stdout", String(chunk)]),
      onStderr: (chunk) => chunks.push(["stderr", String(chunk)]),
    },
  );

  assert.equal(result.code, 7);
  assert.equal(result.stdout, "out");
  assert.equal(result.stderr, "err");
  assert.deepEqual(chunks, [["stdout", "out"], ["stderr", "err"]]);
  assert.equal(result.component, "command-stream");
});

test("production command adapter cancels the real child process", async () => {
  const controller = new AbortController();
  const runner = createCommandRunner();
  const completion = runner.run(
    process.execPath,
    ["-e", "process.stdout.write('ready\\n'); setInterval(() => {}, 1000)"],
    {
      signal: controller.signal,
      onStdout: (chunk) => {
        if (String(chunk).includes("ready")) controller.abort();
      },
    },
  );

  const result = await completion;
  assert.equal(result.code, 143);
  assert.match(result.stdout, /ready/);
  assert.equal(result.cancelled, true);
});

test("the same production adapter executes host and Docker-selected commands", async (context) => {
  const tempDir = fs.mkdtempSync(path.join(os.tmpdir(), "formal-ai-command-runner-"));
  context.after(() => fs.rmSync(tempDir, { recursive: true, force: true }));
  const dockerShim = path.join(tempDir, "docker-shim.cjs");
  fs.writeFileSync(
    dockerShim,
    "process.stdout.write(JSON.stringify(process.argv.slice(2)));\n",
  );
  const runner = createCommandRunner();

  const host = await runner.runTool({
    isolation: "host",
    command: `${JSON.stringify(process.execPath)} -e "process.stdout.write('host')"`,
  });
  const docker = await runner.runTool({
    isolation: "docker",
    command: "printf sandbox",
    dockerBinary: process.execPath,
    image: "example/image:1",
    dockerPrefixArgs: [dockerShim],
  });

  assert.equal(host.stdout, "host");
  assert.deepEqual(JSON.parse(docker.stdout), [
    "run", "--rm", "example/image:1", "sh", "-c", "printf sandbox",
  ]);
});
