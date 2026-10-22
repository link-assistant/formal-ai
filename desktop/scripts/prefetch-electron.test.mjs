import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs/promises";
import { createRequire } from "node:module";
import os from "node:os";
import path from "node:path";
import test from "node:test";
import { boundedElectronDownloader, prefetchElectron } from "./prefetch-electron.mjs";

const url = "https://github.com/electron/electron/releases/download/v44.7.0/SHASUMS256.txt";
const httpError = status => Object.assign(new Error(`Response code ${status} (fixture)`), {
  response: {statusCode: status, url: `${url}?token=personal-secret`},
});

function harness(download, overrides = {}) {
  const logs = [];
  let clock = 1000;
  const downloader = boundedElectronDownloader({
    downloader: {download}, deadline: 20_000, now: () => clock,
    sleep: async milliseconds => { clock += milliseconds; },
    logger: {log: line => logs.push(JSON.parse(line)), warn: line => logs.push(JSON.parse(line))},
    stat: async () => ({size: 20}), readFile: async () => Buffer.from("checksum fixture"),
    ...overrides,
  });
  return {downloader, logs};
}

for (const status of [500, 502, 503, 504]) {
  test(`individual checksum GET recovers from ${status} without downloading the archive again`, async () => {
    let calls = 0;
    const {downloader, logs} = harness(async (actualUrl, target, options) => {
      assert.equal(actualUrl, url);
      assert.equal(options.retry.limit, 0);
      assert(options.timeout.request > 0 && options.timeout.request <= 19_000);
      if (++calls === 1) throw httpError(status);
    });
    await downloader.download(url, "fixture");
    assert.equal(calls, 2);
    assert.deepEqual(logs.map(row => [row.phase, row.attempt, row.status]), [
      ["checksum", 1, status], ["checksum", 2, "complete"],
    ]);
  });
}

test("persistent 500 stops at three physical requests across cache fallback and outer callers", async () => {
  let calls = 0;
  const {downloader} = harness(async () => { calls++; throw httpError(500); });
  for (let caller = 0; caller < 5; caller++) {
    await assert.rejects(downloader.download(url, "fixture"), error => {
      assert.match(error.message, /three-attempt/);
      assert.equal(error.response, undefined);
      assert.equal(error.code, undefined);
      assert.equal(error.cause, undefined);
      return true;
    });
  }
  assert.equal(calls, 3);
});

for (const status of [400, 401, 403, 404, 429, 501]) {
  test(`permanent or unsupported HTTP ${status} fails on its first request`, async () => {
    let calls = 0;
    const {downloader} = harness(async () => { calls++; throw httpError(status); });
    await assert.rejects(downloader.download(url, "fixture"));
    assert.equal(calls, 1);
  });
}

test("checksum, TLS and arbitrary failures are not retried", async () => {
  for (const code of ["CHECKSUM_MISMATCH", "CERT_HAS_EXPIRED", undefined]) {
    let calls = 0;
    const {downloader} = harness(async () => { calls++; throw Object.assign(new Error("fixture"), {code}); });
    await assert.rejects(downloader.download(url, "fixture"));
    assert.equal(calls, 1);
  }
});

test("transient connection reset has the same finite physical-request ceiling", async () => {
  let calls = 0;
  const {downloader} = harness(async () => {
    calls++;
    throw Object.assign(new Error("read ECONNRESET"), {code: "ECONNRESET"});
  });
  await assert.rejects(downloader.download(url, "fixture"));
  assert.equal(calls, 3);
});

test("remaining job deadline prevents another sleep or request", async () => {
  let calls = 0;
  const {downloader} = harness(async (actualUrl, target, options) => {
    calls++;
    assert.equal(options.timeout.request, 500);
    throw httpError(503);
  }, {deadline: 1500, sleep: async () => assert.fail("no time for backoff")});
  await assert.rejects(downloader.download(url, "fixture"));
  assert.equal(calls, 1);
  assert.throws(() => harness(async () => {}, {deadline: 999}), /expired/);
});

test("diagnostics omit userinfo, query, fragments, error body and unknown mirror paths", async () => {
  const {downloader, logs} = harness(async () => { throw httpError(500); });
  await assert.rejects(downloader.download("https://user:password@mirror.example/private-secret/file?token=secret#secret", "fixture"));
  const output = JSON.stringify(logs);
  assert(!/user|password|private-secret|personal-secret|token=|#secret/.test(output));
  assert.equal(logs[0].origin, "https://mirror.example");
  assert.equal(logs[0].artifact, "[redacted-path]");
  assert.equal(logs[0].method, "GET");
});

test("oversized checksum responses fail without retry or seeding", async () => {
  let calls = 0;
  const {downloader} = harness(async () => { calls++; }, {stat: async () => ({size: 1024 * 1024 + 1})});
  await assert.rejects(downloader.download(url, "fixture"));
  assert.equal(calls, 1);
  assert.equal(downloader.checksum, undefined);
});

async function seedFixture(t, corrupt = false) {
  const root = await fs.mkdtemp(path.join(os.tmpdir(), "formal-ai-electron-prefetch-"));
  t.after(() => fs.rm(root, {recursive: true, force: true}));
  const archive = path.join(root, "archive.fixture");
  await fs.writeFile(archive, "tiny fake archive; no product download");
  const artifact = "electron-v44.7.0-linux-arm64.zip";
  const checksum = createHash("sha256").update(await fs.readFile(archive)).digest("hex");
  const manifest = Buffer.from(`${corrupt ? "0".repeat(64) : checksum} *${artifact}\n`);
  return {
    root, archive, manifest,
    input: {
      config: {version: "44.7.0", platform: "linux", arch: "arm64"},
      get: {downloadArtifact: async config => {
        assert(!config.unsafelyDisableChecksums);
        assert(config.downloader);
        return archive;
      }},
      downloader: {checksum: manifest}, cacheRoot: path.join(root, "cache"),
      parseChecksumFile: text => Object.fromEntries(text.trim().split("\n").map(line => {
        const [digest, name] = line.split(" *");
        return [name, digest];
      })),
      logger: {log() {}},
    },
  };
}

test("versioned seed preserves exact upstream content only after authenticating the archive", async t => {
  const {input, archive, manifest} = await seedFixture(t);
  assert.equal(await prefetchElectron(input), archive);
  const files = await fs.readdir(input.cacheRoot);
  assert.deepEqual(files, ["SHASUMS256.txt-44.7.0"]);
  assert.deepEqual(await fs.readFile(path.join(input.cacheRoot, files[0])), manifest);
});

test("mismatched archive or disabled checksum verification can never create a seed", async t => {
  const {input} = await seedFixture(t, true);
  await assert.rejects(prefetchElectron(input), /checksum mismatch/);
  await assert.rejects(fs.stat(input.cacheRoot), {code: "ENOENT"});
  input.config.unsafelyDisableChecksums = true;
  await assert.rejects(prefetchElectron(input), /must remain enabled/);
});

test("exclusive seed staging never deletes a preexisting unowned file", async t => {
  const {input} = await seedFixture(t);
  await fs.mkdir(input.cacheRoot);
  const temporary = path.join(input.cacheRoot, `SHASUMS256.txt-44.7.0.${process.pid}.tmp`);
  await fs.writeFile(temporary, "unowned sentinel");
  await assert.rejects(prefetchElectron(input), {code: "EEXIST"});
  assert.equal(await fs.readFile(temporary, "utf8"), "unowned sentinel");
  await assert.rejects(fs.stat(path.join(input.cacheRoot, "SHASUMS256.txt-44.7.0")), {code: "ENOENT"});
});

test("successful rename relinquishes the staging path before another writer reuses it", async t => {
  const {input, manifest} = await seedFixture(t);
  const rename = fs.rename;
  t.mock.method(fs, "rename", async (source, destination) => {
    await rename(source, destination);
    await fs.writeFile(source, "later unowned sentinel");
  });
  await prefetchElectron(input);
  const destination = path.join(input.cacheRoot, "SHASUMS256.txt-44.7.0");
  assert.deepEqual(await fs.readFile(destination), manifest);
  assert.equal(await fs.readFile(`${destination}.${process.pid}.tmp`, "utf8"), "later unowned sentinel");
});

const require = createRequire(import.meta.url);
let installedBuilder;
try { installedBuilder = require.resolve("app-builder-lib/package.json"); } catch { /* CI packaging installs it. */ }
test("installed nested downloader validates recovered checksum and builder reuses the authenticated seed", {
  skip: !installedBuilder && "desktop dependencies are only installed in hosted packaging",
}, async t => {
  const {input, root, manifest} = await seedFixture(t);
  const builderRequire = createRequire(installedBuilder);
  const get = builderRequire("@electron/get");
  const upstream = builderRequire("./out/util/electronGet.js");
  let archiveCalls = 0;
  let checksumCalls = 0;
  const downloader = boundedElectronDownloader({
    deadline: Date.now() + 30_000, sleep: async () => {}, logger: {log() {}, warn() {}},
    downloader: {download: async (actualUrl, target) => {
      if (actualUrl.endsWith("SHASUMS256.txt")) {
        if (++checksumCalls === 1) throw httpError(500);
        await fs.writeFile(target, manifest);
      } else {
        archiveCalls++;
        await fs.copyFile(path.join(root, "archive.fixture"), target);
      }
    }},
  });
  const options = {
    version: "44.7.0", platformName: "linux", arch: "arm64", artifactName: "electron", cacheDir: input.cacheRoot,
  };
  const config = await upstream.buildElectronArtifactConfig(options);
  await prefetchElectron({...input, config, get, downloader, parseChecksumFile: upstream.parseChecksumFile});
  assert.equal(archiveCalls, 1);
  assert.equal(checksumCalls, 2);
  const seeded = await upstream.buildElectronArtifactConfig(options);
  assert(seeded.checksums["electron-v44.7.0-linux-arm64.zip"]);
  await get.downloadArtifact({...seeded, downloader: {download: async () => assert.fail("seeded build must be offline")}});
  assert.equal(archiveCalls, 1);
});
