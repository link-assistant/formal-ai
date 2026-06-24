import assert from "node:assert/strict";
import { test } from "node:test";
import { createRequire } from "node:module";

const require = createRequire(import.meta.url);
const {
  REPO,
  LATEST_RELEASE_API,
  VSIX_ASSET_PATTERN,
  CLI_CANDIDATES,
  resolveCliCandidates,
  selectVsixAsset,
  createVsCodeInstaller,
} = require("../lib/vscode-install.cjs");

// A scripted command runner: `byKey` maps a "<cmd> <args…>" prefix to the
// {code,stdout,stderr} the fake CLI call returns; unmatched calls fail with
// ENOENT, mirroring a CLI that is not on PATH. Every call is recorded so tests
// assert the exact argument vectors and detection order.
function makeRunner(byKey = {}) {
  const calls = [];
  const runCommand = async (cmd, args) => {
    calls.push({ cmd, args });
    const key = [cmd, ...args].join(" ");
    for (const prefix of Object.keys(byKey)) {
      if (key === prefix || key.startsWith(`${prefix} `)) {
        return byKey[prefix];
      }
    }
    const error = new Error(`spawn ${cmd} ENOENT`);
    error.code = "ENOENT";
    throw error;
  };
  return { runCommand, calls };
}

function releaseWith(assetNames, tag = "v0.215.0") {
  return {
    tag_name: tag,
    assets: assetNames.map((name) => ({
      name,
      browser_download_url: `https://github.com/${REPO}/releases/download/${tag}/${name}`,
    })),
  };
}

// A harness that wires a scripted runner, a canned release JSON, and a recording
// downloader into the installer under test.
function makeInstaller({ runnerResponses = {}, release, downloadImpl, env = {} } = {}) {
  const { runCommand, calls } = makeRunner(runnerResponses);
  const downloads = [];
  const fetched = [];
  const installer = createVsCodeInstaller({
    env,
    runCommand,
    fetchJson: async (url) => {
      fetched.push(url);
      if (release instanceof Error) {
        throw release;
      }
      return release;
    },
    downloadFile: async (url, dest) => {
      downloads.push({ url, dest });
      if (typeof downloadImpl === "function") {
        return downloadImpl(url, dest);
      }
      return dest;
    },
    joinPath: (dir, file) => `${dir}/${file}`,
    tmpDir: () => "/tmp/formal-ai",
    log: () => {},
  });
  return { installer, calls, downloads, fetched };
}

const CODE_OK = { code: 0, stdout: "1.96.0\nabcdef\nx64\n", stderr: "" };

test("VSIX_ASSET_PATTERN matches the packaged extension filename only", () => {
  assert.ok(VSIX_ASSET_PATTERN.test("formal-ai-vscode-0.215.0.vsix"));
  assert.ok(VSIX_ASSET_PATTERN.test("formal-ai-vscode-1.2.3-rc.1.vsix"));
  assert.ok(!VSIX_ASSET_PATTERN.test("formal-ai-vscode-0.215.0.vsix.sha256"));
  assert.ok(!VSIX_ASSET_PATTERN.test("formal-ai-desktop-0.215.0.AppImage"));
});

test("LATEST_RELEASE_API targets the canonical repo", () => {
  assert.equal(REPO, "link-assistant/formal-ai");
  assert.equal(LATEST_RELEASE_API, "https://api.github.com/repos/link-assistant/formal-ai/releases/latest");
});

test("resolveCliCandidates honors the override first, then the known editors", () => {
  assert.deepEqual(resolveCliCandidates({}), CLI_CANDIDATES);
  const withOverride = resolveCliCandidates({ FORMAL_AI_VSCODE_BIN: "/opt/code/bin/code" });
  assert.equal(withOverride[0], "/opt/code/bin/code");
  // Known editors still follow, and the override is not duplicated.
  assert.ok(withOverride.includes("code-insiders"));
  assert.equal(withOverride.filter((c) => c === "/opt/code/bin/code").length, 1);
});

test("selectVsixAsset picks the .vsix and ignores checksum/other assets", () => {
  const asset = selectVsixAsset(
    releaseWith(["SHA256SUMS.txt", "formal-ai-desktop-0.215.0.AppImage", "formal-ai-vscode-0.215.0.vsix"]),
  );
  assert.ok(asset);
  assert.equal(asset.name, "formal-ai-vscode-0.215.0.vsix");
  assert.equal(asset.tag, "v0.215.0");
  assert.match(asset.url, /formal-ai-vscode-0\.215\.0\.vsix$/);
});

test("selectVsixAsset returns null when no extension is published", () => {
  assert.equal(selectVsixAsset(releaseWith(["SHA256SUMS.txt"])), null);
  assert.equal(selectVsixAsset(null), null);
  assert.equal(selectVsixAsset({ assets: "nope" }), null);
});

test("the factory requires its side-effecting dependencies", () => {
  assert.throws(() => createVsCodeInstaller({}), /runCommand/);
  assert.throws(() => createVsCodeInstaller({ runCommand: () => {} }), /fetchJson/);
  assert.throws(
    () => createVsCodeInstaller({ runCommand: () => {}, fetchJson: () => {} }),
    /downloadFile/,
  );
});

test("detectCli returns the first editor that answers --version", async () => {
  const { installer, calls } = makeInstaller({
    runnerResponses: { "code-insiders --version": CODE_OK },
  });
  const detected = await installer.detectCli();
  assert.deepEqual(detected, { cli: "code-insiders", version: "1.96.0" });
  // `code` was probed first (ENOENT), then `code-insiders` answered.
  assert.deepEqual(calls.slice(0, 2).map((c) => c.cmd), ["code", "code-insiders"]);
});

test("detectCli returns null when no VS Code CLI is present", async () => {
  const { installer } = makeInstaller({ runnerResponses: {} });
  assert.equal(await installer.detectCli(), null);
});

test("install: detect → resolve → download → install-extension happy path", async () => {
  const { installer, calls, downloads, fetched } = makeInstaller({
    runnerResponses: {
      "code --version": CODE_OK,
      "code --install-extension": { code: 0, stdout: "Extension installed.", stderr: "" },
    },
    release: releaseWith(["formal-ai-vscode-0.215.0.vsix"]),
  });
  const result = await installer.install();
  assert.equal(result.ok, true);
  assert.equal(result.state, "installed");
  assert.equal(result.cli, "code");
  assert.equal(result.cliVersion, "1.96.0");
  assert.equal(result.asset, "formal-ai-vscode-0.215.0.vsix");
  assert.equal(result.vsix, "/tmp/formal-ai/formal-ai-vscode-0.215.0.vsix");

  assert.deepEqual(fetched, [LATEST_RELEASE_API]);
  assert.equal(downloads.length, 1);
  assert.equal(downloads[0].dest, "/tmp/formal-ai/formal-ai-vscode-0.215.0.vsix");

  // The install command runs the resolved CLI against the downloaded vsix with
  // --force so a re-install upgrades in place.
  const installCall = calls.find((c) => c.args[0] === "--install-extension");
  assert.deepEqual(installCall.args, [
    "--install-extension",
    "/tmp/formal-ai/formal-ai-vscode-0.215.0.vsix",
    "--force",
  ]);
});

test("install: no VS Code CLI yields an actionable no-vscode-cli state", async () => {
  const { installer, downloads } = makeInstaller({
    runnerResponses: {},
    release: releaseWith(["formal-ai-vscode-0.215.0.vsix"]),
  });
  const result = await installer.install();
  assert.equal(result.ok, false);
  assert.equal(result.state, "no-vscode-cli");
  assert.match(result.reason, /VS Code/);
  // Without a CLI we never hit the network or download anything.
  assert.equal(downloads.length, 0);
});

test("install: a release without a .vsix yields no-release-asset", async () => {
  const { installer } = makeInstaller({
    runnerResponses: { "code --version": CODE_OK },
    release: releaseWith(["SHA256SUMS.txt"]),
  });
  const result = await installer.install();
  assert.equal(result.ok, false);
  assert.equal(result.state, "no-release-asset");
  assert.equal(result.cli, "code");
});

test("install: a failed release lookup is reported, not thrown", async () => {
  const { installer } = makeInstaller({
    runnerResponses: { "code --version": CODE_OK },
    release: new Error("network down"),
  });
  const result = await installer.install();
  assert.equal(result.ok, false);
  assert.equal(result.state, "release-lookup-failed");
  assert.match(result.reason, /network down/);
});

test("install: a download error surfaces as download-failed", async () => {
  const { installer } = makeInstaller({
    runnerResponses: { "code --version": CODE_OK },
    release: releaseWith(["formal-ai-vscode-0.215.0.vsix"]),
    downloadImpl: () => {
      throw new Error("HTTP 503");
    },
  });
  const result = await installer.install();
  assert.equal(result.ok, false);
  assert.equal(result.state, "download-failed");
  assert.match(result.reason, /503/);
});

test("install: a non-zero code --install-extension is reported as install-failed", async () => {
  const { installer } = makeInstaller({
    runnerResponses: {
      "code --version": CODE_OK,
      "code --install-extension": { code: 1, stdout: "", stderr: "Unable to install extension" },
    },
    release: releaseWith(["formal-ai-vscode-0.215.0.vsix"]),
  });
  const result = await installer.install();
  assert.equal(result.ok, false);
  assert.equal(result.state, "install-failed");
  assert.match(result.reason, /Unable to install/);
});

test("install: a bundled vsixPath skips the network entirely", async () => {
  const { installer, downloads, fetched, calls } = makeInstaller({
    runnerResponses: {
      "code --version": CODE_OK,
      "code --install-extension": { code: 0, stdout: "Extension installed.", stderr: "" },
    },
    release: releaseWith(["formal-ai-vscode-0.215.0.vsix"]),
  });
  const result = await installer.install({ vsixPath: "/bundled/formal-ai-vscode.vsix" });
  assert.equal(result.ok, true);
  assert.equal(result.vsix, "/bundled/formal-ai-vscode.vsix");
  assert.equal(fetched.length, 0);
  assert.equal(downloads.length, 0);
  const installCall = calls.find((c) => c.args[0] === "--install-extension");
  assert.equal(installCall.args[1], "/bundled/formal-ai-vscode.vsix");
});
