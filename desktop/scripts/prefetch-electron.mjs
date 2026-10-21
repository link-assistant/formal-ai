import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { createReadStream } from "node:fs";
import fs from "node:fs/promises";
import { createRequire } from "node:module";
import path from "node:path";
import { setTimeout as delay } from "node:timers/promises";
import { fileURLToPath, pathToFileURL } from "node:url";

const transientStatuses = new Set([500, 502, 503, 504]);
const transientCodes = new Set(["ECONNRESET", "ETIMEDOUT", "EAI_AGAIN", "EPIPE"]);

function safeRequest(url) {
  const parsed = new URL(url);
  assert.equal(parsed.protocol, "https:", "Electron downloads must use HTTPS");
  const basename = path.posix.basename(parsed.pathname);
  const phase = basename === "SHASUMS256.txt" ? "checksum" : "archive";
  // Paths on custom mirrors can contain credentials too. Only disclose known
  // public artifact basenames; never query strings, userinfo, headers or bodies.
  const artifact = /^(?:SHASUMS256\.txt|electron-v[\w.+-]+-(?:linux|darwin|win32)-(?:x64|arm64|ia32)\.zip)$/.test(basename)
    ? basename : "[redacted-path]";
  return {method: "GET", origin: parsed.origin, artifact, phase};
}

export function boundedElectronDownloader({
  downloader, deadline, now = Date.now, sleep = delay,
  logger = console, readFile = fs.readFile, stat = fs.stat,
}) {
  assert(Number.isSafeInteger(deadline) && deadline > now(), "Electron download deadline has expired");
  const attempts = new Map();
  let terminal;
  let checksum;
  return {
    get checksum() { return checksum; },
    async download(url, target, options = {}) {
      if (terminal) throw terminal;
      const request = safeRequest(url);
      const key = createHash("sha256").update(url).digest("hex");
      // The same object is also used for @electron/get's cache-validation
      // fallback. That fallback must not reset an exhausted request's limit.
      assert(attempts.has(key) || attempts.size < 8, "Unexpected Electron download fanout");
      while ((attempts.get(key) ?? 0) < 3) {
        const remaining = deadline - now();
        if (remaining <= 0) break;
        const attempt = (attempts.get(key) ?? 0) + 1;
        attempts.set(key, attempt);
        const started = now();
        try {
          await downloader.download(url, target, {
            ...options,
            // This layer owns retries. Neither got nor the outer builder may
            // multiply them; a terminal failure below has no retryable fields.
            retry: {limit: 0},
            timeout: {...options.timeout, request: Math.min(600_000, remaining)},
          });
          if (request.phase === "checksum") {
            assert((await stat(target)).size <= 1024 * 1024, "Oversized Electron checksum manifest");
            checksum = await readFile(target);
          }
          logger.log(JSON.stringify({event: "electron-prefetch", ...request, attempt, status: "complete"}));
          return;
        } catch (error) {
          const status = error?.response?.statusCode;
          const code = transientCodes.has(error?.code) ? error.code : undefined;
          logger.warn(JSON.stringify({
            event: "electron-prefetch", ...request, attempt,
            status: Number.isInteger(status) ? status : "failed", code,
            elapsedMs: Math.max(0, now() - started),
          }));
          if (!(transientStatuses.has(status) || code) || attempt === 3) break;
          const wait = attempt * 1000;
          if (now() + wait >= deadline) break;
          await sleep(wait);
        }
      }
      // Do not attach the raw HTTPError: its URL/options may hold credentials,
      // and builder's outer retry would otherwise restart the exhausted action.
      terminal = new Error(`Electron ${request.phase} GET failed within the three-attempt download budget`);
      throw terminal;
    },
  };
}

export async function prefetchElectron({
  config, get, downloader, parseChecksumFile, cacheRoot, logger = console,
}) {
  assert(!config.unsafelyDisableChecksums, "Electron checksum verification must remain enabled");
  const artifact = `electron-v${config.version}-${config.platform}-${config.arch}.zip`;
  const archive = await get.downloadArtifact({...config, downloader});
  // Seed only the upstream manifest that authenticated these exact bytes.
  // electron-builder's supported versioned seed then avoids another checksum
  // network request during packaging, while retaining its checksum validation.
  if (downloader.checksum) {
    const expected = parseChecksumFile(downloader.checksum.toString("utf8"))[artifact];
    assert.match(expected ?? "", /^[a-f0-9]{64}$/, "Missing exact Electron artifact checksum");
    const hash = createHash("sha256");
    for await (const chunk of createReadStream(archive)) hash.update(chunk);
    assert.equal(hash.digest("hex"), expected, "Electron archive checksum mismatch");
    await fs.mkdir(cacheRoot, {recursive: true});
    const destination = path.join(cacheRoot, `SHASUMS256.txt-${config.version}`);
    const temporary = `${destination}.${process.pid}.tmp`;
    const handle = await fs.open(temporary, "wx");
    try {
      try {
        await handle.writeFile(downloader.checksum);
      } finally {
        await handle.close();
      }
      await fs.rename(temporary, destination);
    } finally {
      await fs.rm(temporary, {force: true});
    }
  }
  logger.log(`Verified locked Electron ${config.version} for ${config.platform}/${config.arch}`);
  return archive;
}

async function main() {
  const desktop = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
  const require = createRequire(path.join(desktop, "package.json"));
  const lock = JSON.parse(await fs.readFile(path.join(desktop, "package-lock.json"), "utf8"));
  const version = lock.packages["node_modules/electron"].version;
  assert.equal(require("electron/package.json").version, version, "Installed Electron differs from lock");
  const builderRequire = createRequire(require.resolve("app-builder-lib/package.json"));
  const getRequire = createRequire(builderRequire.resolve("@electron/get/package.json"));
  assert.equal(getRequire("./package.json").version,
    lock.packages["node_modules/app-builder-lib/node_modules/@electron/get"].version);
  const {GotDownloader} = getRequire("./dist/cjs/GotDownloader.js");
  const upstream = require("app-builder-lib/out/util/electronGet.js");
  const build = require("./package.json").build;
  const config = await upstream.buildElectronArtifactConfig({
    version, platformName: process.platform, arch: process.arch, artifactName: "electron",
    cacheDir: process.env.ELECTRON_CACHE, electronDownload: build.electronDownload,
  });
  config.downloadOptions = {
    ...config.downloadOptions,
    agent: config.downloadOptions?.agent ?? builderRequire("builder-util").buildGotProxyAgent(),
    quiet: true,
  };
  // @electron/get v3 requires both controls to suppress its delayed progress
  // timer. Our per-request records already describe download progress/failures.
  process.env.ELECTRON_GET_NO_PROGRESS = "1";
  const rawDeadline = process.env.FORMAL_AI_JOB_DEADLINE_EPOCH;
  assert.match(rawDeadline ?? "", /^[0-9]+$/, "Missing job deadline for Electron download");
  const deadline = Number(rawDeadline) * 1000;
  const downloader = boundedElectronDownloader({downloader: new GotDownloader(), deadline});
  await prefetchElectron({
    config, get: builderRequire("@electron/get"), downloader,
    parseChecksumFile: upstream.parseChecksumFile,
    cacheRoot: config.cacheRoot ?? upstream.defaultElectronGetCacheRoot(),
  });
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) await main();
