import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import http from "node:http";
import os from "node:os";
import path from "node:path";
import test from "node:test";
import { fileURLToPath } from "node:url";

import {
  archivePath,
  binariesMirrorUrl,
  cacheDirectory,
  checkToolsetConstants,
  dmgbuildFilename,
  downloadWithRetry,
  prefetchToolsets,
  requiredToolsets,
  sevenZipFilename,
  toolsetUrl,
} from "./prefetch-builder-toolsets.mjs";

const scriptDir = path.dirname(fileURLToPath(import.meta.url));
const desktopDir = path.resolve(scriptDir, "..");

function scratch(label) {
  return fs.mkdtempSync(path.join(os.tmpdir(), `formal-ai-prefetch-${label}-`));
}

function sha256(buffer) {
  return createHash("sha256").update(buffer).digest("hex");
}

// A stand-in for the installed package, holding the two constants
// `checkToolsetConstants` re-derives.
function fakeInstall(root, toolsets) {
  fs.writeFileSync(path.join(root, "package.json"), JSON.stringify({ name: "fake-desktop" }));
  for (const toolset of toolsets) {
    const file = path.join(root, "node_modules", toolset.module);
    fs.mkdirSync(path.dirname(file), { recursive: true });
    fs.writeFileSync(
      file,
      `const checksums = { "${toolset.filename}": "${toolset.sha256}" };\n` +
        `downloadBuilderToolset({ releaseName: "${toolset.releaseName}", checksums });\n`,
    );
  }
}

async function serve(handler) {
  const server = http.createServer(handler);
  await new Promise((resolve) => server.listen(0, "127.0.0.1", resolve));
  const { port } = server.address();
  return {
    origin: `http://127.0.0.1:${port}/`,
    async close() {
      await new Promise((resolve) => server.close(resolve));
    },
  };
}

test("host toolsets are selected the way electron-builder selects them", () => {
  assert.equal(sevenZipFilename("darwin", "x64"), "7zip-darwin-x86_64.tar.gz");
  assert.equal(sevenZipFilename("darwin", "arm64"), "7zip-darwin-arm64.tar.gz");
  assert.equal(sevenZipFilename("linux", "x64"), "7zip-linux-x64.tar.gz");
  assert.equal(sevenZipFilename("linux", "arm64"), "7zip-linux-arm64.tar.gz");
  assert.equal(sevenZipFilename("win32", "arm64"), "7zip-win-arm64.tar.gz");
  assert.equal(sevenZipFilename("aix", "x64"), null);

  assert.equal(dmgbuildFilename("x64"), "dmgbuild-bundle-x86_64-75c8a6c.tar.gz");
  assert.equal(dmgbuildFilename("arm64"), "dmgbuild-bundle-arm64-75c8a6c.tar.gz");
});

// The x64 macOS runner is where the ten-minute stall was observed, so its two
// toolsets are the ones that must be seeded.
test("macOS x64 requires both the 7za and dmgbuild archives", () => {
  const toolsets = requiredToolsets("darwin", "x64");
  assert.deepEqual(
    toolsets.map((toolset) => toolset.filename),
    ["7zip-darwin-x86_64.tar.gz", "dmgbuild-bundle-x86_64-75c8a6c.tar.gz"],
  );
  for (const toolset of toolsets) {
    assert.match(toolset.sha256, /^[0-9a-f]{64}$/);
  }

  // dmgbuild is macOS-only; 7za is downloaded on every packaged platform, which
  // is why the prefetch step is not gated on `--mac`.
  for (const platform of ["linux", "win32"]) {
    assert.deepEqual(
      requiredToolsets(platform, "x64").map((toolset) => toolset.id),
      ["7zip"],
    );
  }
});

// app-builder-lib/out/util/electronGet.js `getCacheDirectory`.
test("cache directory matches the one electron-builder reads", () => {
  assert.equal(
    cacheDirectory({}, "darwin", "/Users/runner"),
    path.join("/Users/runner", "Library", "Caches", "electron-builder"),
  );
  assert.equal(
    cacheDirectory({}, "linux", "/home/runner"),
    path.join("/home/runner", ".cache", "electron-builder"),
  );
  assert.equal(
    cacheDirectory({ XDG_CACHE_HOME: "/var/cache" }, "linux", "/home/runner"),
    path.join("/var/cache", "electron-builder"),
  );
  assert.equal(
    cacheDirectory({ ELECTRON_BUILDER_CACHE: "/mnt/eb-cache" }, "darwin", "/Users/runner"),
    "/mnt/eb-cache",
  );
  // A relative override has no path root, so upstream ignores it; so must we,
  // or the archive would be seeded where nothing looks for it.
  assert.equal(
    cacheDirectory({ ELECTRON_BUILDER_CACHE: "relative" }, "darwin", "/Users/runner"),
    path.join("/Users/runner", "Library", "Caches", "electron-builder"),
  );
});

test("archive path is <cacheDir>/<releaseName>/<filename>", () => {
  const [sevenZip, dmgbuild] = requiredToolsets("darwin", "x64");
  assert.equal(
    archivePath(dmgbuild, "/cache"),
    path.join("/cache", "dmg-builder@1.2.5", "dmgbuild-bundle-x86_64-75c8a6c.tar.gz"),
  );
  assert.equal(
    toolsetUrl(sevenZip, {}),
    "https://github.com/electron-userland/electron-builder-binaries/releases/download/" +
      "7zip@1.0.0/7zip-darwin-x86_64.tar.gz",
  );
  assert.equal(
    binariesMirrorUrl({ ELECTRON_BUILDER_BINARIES_MIRROR: "https://mirror.example/eb" }),
    "https://mirror.example/eb/",
  );
  // Plain http is rejected unless electron-builder's own opt-in is set.
  assert.equal(
    binariesMirrorUrl({ ELECTRON_BUILDER_BINARIES_MIRROR: "http://mirror.example/eb" }),
    "https://github.com/electron-userland/electron-builder-binaries/releases/download/",
  );
});

test("an upstream version bump is reported instead of silently seeding a dead file", () => {
  const [sevenZip] = requiredToolsets("linux", "x64");
  const current =
    `const checksums = { "${sevenZip.filename}": "${sevenZip.sha256}" };\n` +
    `releaseName: \`${sevenZip.releaseName}\`,\n`;
  assert.deepEqual(checkToolsetConstants(sevenZip, current), []);

  const bumped = current.replace("7zip@1.0.0", "7zip@1.1.0");
  assert.deepEqual(checkToolsetConstants(sevenZip, bumped), ['release "7zip@1.0.0" is no longer requested']);

  const rehashed = current.replace(sevenZip.sha256, "0".repeat(64));
  assert.deepEqual(checkToolsetConstants(sevenZip, rehashed), [
    `checksum entry for "${sevenZip.filename}" does not match`,
  ]);
});

// When dependencies are installed, the constants above must still describe the
// package that will actually run. This is the check that catches an
// electron-builder upgrade in a pull request rather than in a release.
test("constants agree with the installed electron-builder, when it is installed", { skip: !fs.existsSync(path.join(desktopDir, "node_modules", "app-builder-lib")) }, () => {
  for (const toolset of requiredToolsets(process.platform, process.arch)) {
    const installed = path.join(desktopDir, "node_modules", toolset.module);
    if (!fs.existsSync(installed)) {
      continue;
    }
    assert.deepEqual(
      checkToolsetConstants(toolset, fs.readFileSync(installed, "utf8")),
      [],
      `${toolset.module} no longer matches the prefetch constants`,
    );
  }
});

test("a completed prefetch lands where downloadAndExtract looks for it", async () => {
  const root = scratch("download");
  const cacheDir = path.join(root, "cache");
  const payload = Buffer.from("7za payload");
  const toolsets = requiredToolsets("linux", "x64").map((toolset) => ({
    ...toolset,
    sha256: sha256(payload),
  }));
  fakeInstall(root, toolsets);

  let requests = 0;
  const server = await serve((request, response) => {
    requests += 1;
    response.writeHead(200, { "content-length": payload.length });
    response.end(payload);
  });

  try {
    const results = await prefetchToolsets({
      env: { ELECTRON_BUILDER_BINARIES_MIRROR: `https://mirror.invalid/` },
      platform: "linux",
      arch: "x64",
      baseDir: root,
      cacheDir,
      logger: { log() {}, warn() {} },
      toolsets,
      fetchImpl: (url, init) => fetch(new URL(new URL(url).pathname.slice(1), server.origin), init),
      downloadOptions: { attempts: 2, retryDelayMs: 0, stallTimeoutMs: 5000, totalTimeoutMs: 20000 },
    });

    assert.deepEqual(
      results.map((result) => result.status),
      ["downloaded"],
    );
    const target = path.join(cacheDir, "7zip@1.0.0", "7zip-linux-x64.tar.gz");
    assert.equal(fs.readFileSync(target).toString(), payload.toString());
    assert.equal(requests, 1);

    // A second run must be free: the archive is already there and checksums.
    const again = await prefetchToolsets({
      env: {},
      platform: "linux",
      arch: "x64",
      baseDir: root,
      cacheDir,
      logger: { log() {}, warn() {} },
      toolsets,
      fetchImpl: () => assert.fail("a cached archive must not be re-downloaded"),
    });
    assert.deepEqual(
      again.map((result) => result.status),
      ["cached"],
    );
    assert.equal(requests, 1);
  } finally {
    await server.close();
    fs.rmSync(root, { recursive: true, force: true });
  }
});

// The CI failure this script exists for: a connection that is accepted and then
// produces no bytes. electron-builder waits the full ten-minute request timeout
// and records the error even after its own retry succeeds; here the stalled
// attempt is abandoned after `stallTimeoutMs` and the retry completes.
test("a stalled connection is abandoned and retried instead of held open", async () => {
  const root = scratch("stall");
  const payload = Buffer.from("dmgbuild payload");
  const digest = sha256(payload);
  const target = path.join(root, "archive.tar.gz");

  let attempts = 0;
  const stalled = [];
  const server = await serve((request, response) => {
    attempts += 1;
    if (attempts === 1) {
      response.writeHead(200);
      response.write("partial");
      stalled.push(response);
      return; // never ends: the observed failure mode
    }
    response.writeHead(200, { "content-length": payload.length });
    response.end(payload);
  });

  try {
    const started = Date.now();
    const result = await downloadWithRetry(server.origin, target, {
      attempts: 3,
      retryDelayMs: 0,
      stallTimeoutMs: 250,
      totalTimeoutMs: 20000,
      expectedSha256: digest,
    });
    const elapsed = Date.now() - started;

    assert.equal(result, digest);
    assert.equal(attempts, 2);
    assert.equal(fs.readFileSync(target).toString(), payload.toString());
    assert.ok(elapsed < 10000, `the stalled attempt must not be waited out (took ${elapsed} ms)`);
  } finally {
    for (const response of stalled) {
      response.destroy();
    }
    await server.close();
    fs.rmSync(root, { recursive: true, force: true });
  }
});

test("a corrupted download is rejected rather than cached", async () => {
  const root = scratch("checksum");
  const target = path.join(root, "archive.tar.gz");
  const server = await serve((request, response) => response.end("not the expected bytes"));

  try {
    await assert.rejects(
      downloadWithRetry(server.origin, target, {
        attempts: 2,
        retryDelayMs: 0,
        stallTimeoutMs: 5000,
        totalTimeoutMs: 20000,
        expectedSha256: sha256(Buffer.from("the expected bytes")),
      }),
      /checksum mismatch/,
    );
    assert.equal(fs.existsSync(target), false, "a mismatched archive must not be left behind");
  } finally {
    await server.close();
    fs.rmSync(root, { recursive: true, force: true });
  }
});

// A prefetch is an optimisation. If it cannot run, packaging must proceed
// exactly as it did before this script existed.
test("prefetch failures warn and never fail packaging", async () => {
  const root = scratch("failure");
  const payload = Buffer.from("7za payload");
  const toolsets = requiredToolsets("linux", "x64").map((toolset) => ({
    ...toolset,
    sha256: sha256(payload),
  }));
  fakeInstall(root, toolsets);
  const warnings = [];
  const server = await serve((request, response) => {
    response.writeHead(404);
    response.end("missing");
  });

  try {
    const results = await prefetchToolsets({
      env: {},
      platform: "linux",
      arch: "x64",
      baseDir: root,
      cacheDir: path.join(root, "cache"),
      logger: { log() {}, warn: (message) => warnings.push(message) },
      toolsets,
      fetchImpl: (url, init) => fetch(server.origin, init),
      downloadOptions: { attempts: 1, retryDelayMs: 0, stallTimeoutMs: 5000, totalTimeoutMs: 20000 },
    });

    assert.deepEqual(
      results.map((result) => result.status),
      ["failed"],
    );
    assert.match(warnings.join("\n"), /Toolset prefetch failed/);
    assert.equal(fs.existsSync(path.join(root, "cache", "7zip@1.0.0")), true);
    assert.deepEqual(fs.readdirSync(path.join(root, "cache", "7zip@1.0.0")), []);
  } finally {
    await server.close();
    fs.rmSync(root, { recursive: true, force: true });
  }

  // A missing dependency tree is a skip, not a crash: the script runs before
  // packaging on every platform, including ones where it has nothing to do.
  const bare = scratch("bare");
  fs.writeFileSync(path.join(bare, "package.json"), "{}");
  const skipped = await prefetchToolsets({
    env: {},
    platform: "linux",
    arch: "x64",
    baseDir: bare,
    cacheDir: path.join(bare, "cache"),
    logger: { log() {}, warn() {} },
    fetchImpl: () => assert.fail("an unresolvable toolset must not be downloaded"),
  });
  assert.deepEqual(
    skipped.map((result) => result.status),
    ["skipped"],
  );
  fs.rmSync(bare, { recursive: true, force: true });
});
