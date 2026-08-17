// Seed electron-builder's own toolset archive cache before packaging.
//
// Issue #1017 / run 31998612713: `Build macos-x64` produced a complete DMG,
// ZIP and both blockmaps and then still exited 1 with
// `⨯ Timeout awaiting 'request' for 600000ms  failedTask=build`. The stalled
// request was electron-builder fetching `dmgbuild-bundle-x86_64-75c8a6c.tar.gz`
// from the `electron-builder-binaries` release; its internal retry recovered two
// seconds later and the build finished, but the timed-out request had already
// been recorded by an AsyncTaskManager, whose `awaitTasks()` rethrew it at the
// end of an otherwise successful build.
//
// app-builder-lib keeps a predictable archive cache next to the extract
// directory and consults it *before* touching `@electron/get`
// (`app-builder-lib/out/util/electronGet.js`, `downloadBuilderToolset`):
//
//     const archiveCachePath = path.join(
//       getCacheDirectory({ allowEnvVarOverride: true }), releaseName, filenameWithExt)
//
//     // Predictable archive cache: <cacheDir>/<releaseName>/<filename>, next to
//     // the extract dir. downloadAndExtract checks here before touching
//     // @electron/get and persists the archive here after every successful
//     // download, so subsequent builds never need a network round-trip.
//
// A hit is checksum-validated against the same `checksums` map the download
// path uses, so seeding this file with the exact bytes upstream expects removes
// the network round-trip that stalled — without patching electron-builder.
//
// The fetch here is deliberately impatient where electron-builder is patient:
// a stalled connection is aborted after `stallTimeoutMs` of silence and retried,
// instead of holding one socket open for the full ten-minute request timeout.
//
// Failures are reported as warnings and never fail the build: a missed prefetch
// only restores the previous behaviour, where electron-builder downloads the
// toolset itself. Set FORMAL_AI_PREFETCH_STRICT=1 to make them fatal (used by
// the tests), and FORMAL_AI_PREFETCH_VERBOSE=1 for per-attempt tracing. Both
// default to off.

import { createHash } from "node:crypto";
import fs from "node:fs";
import fsp from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import { createRequire } from "node:module";
import { fileURLToPath, pathToFileURL } from "node:url";

const scriptDir = path.dirname(fileURLToPath(import.meta.url));
const desktopDir = path.resolve(scriptDir, "..");

// Mirrors app-builder-lib/out/toolsets/7zip.js `checksums`.
const SEVEN_ZIP_CHECKSUMS = {
  "7zip-linux-ia32.tar.gz": "24a5d5bfe81506d0bfe21a812588119ae3deb757e8ba084b2339d8e899543686",
  "7zip-darwin-arm64.tar.gz": "496a341abe210aae1a25bc202ee97f6de6c76a3dc80f91d96616be05502d72c1",
  "7zip-darwin-x86_64.tar.gz": "496a341abe210aae1a25bc202ee97f6de6c76a3dc80f91d96616be05502d72c1",
  "7zip-linux-arm64.tar.gz": "5aff5034206b78f8261249ceb922b5c7e04c9bdb733784d8f5b6df9732cf1f79",
  "7zip-win-arm64.tar.gz": "ac3f38f96ce7498096a123bb0862dd6db863a7353c9e9e1c15f73c183adf6620",
  "7zip-win-ia32.tar.gz": "ac3f38f96ce7498096a123bb0862dd6db863a7353c9e9e1c15f73c183adf6620",
  "7zip-win-x64.tar.gz": "be071f15bd6da2f78fe81c6ddef2009b0c4d8a51f36b780cb806c7e6df95e1b3",
  "7zip-linux-x64.tar.gz": "d151bb44b2a9d9bfc52813ce4cac042916394a0ab8a56bd5d221a5dc9ef8d5d7",
};

// Mirrors dmg-builder/out/dmgUtil.js `getDmgVendorPath`.
const DMGBUILD_CHECKSUMS = {
  "dmgbuild-bundle-arm64-75c8a6c.tar.gz":
    "793404d0c96687e27d5ee40a668d498c92e36a64d6c2906df511031adb33cbeb",
  "dmgbuild-bundle-x86_64-75c8a6c.tar.gz":
    "1664972f9cc2d6e8fce3b63e42cd30078aff602669c5856939c4519921200433",
};

// Upstream's app-builder-lib/out/toolsets/7zip.js `getFilename()`, verbatim in
// behaviour: 7za is a *host* tool, so it is selected by the host platform and
// architecture, never by the electron-builder target.
export function sevenZipFilename(platform, arch) {
  if (platform === "darwin") {
    return arch === "arm64" ? "7zip-darwin-arm64.tar.gz" : "7zip-darwin-x86_64.tar.gz";
  }
  if (platform === "linux") {
    if (arch === "arm64") {
      return "7zip-linux-arm64.tar.gz";
    }
    if (arch === "ia32") {
      return "7zip-linux-ia32.tar.gz";
    }
    return "7zip-linux-x64.tar.gz";
  }
  if (platform === "win32") {
    if (arch === "arm64") {
      return "7zip-win-arm64.tar.gz";
    }
    if (arch === "ia32") {
      return "7zip-win-ia32.tar.gz";
    }
    return "7zip-win-x64.tar.gz";
  }
  return null;
}

// dmgbuild also runs on the host: `getDmgVendorPath` reads `process.arch`.
export function dmgbuildFilename(arch) {
  return `dmgbuild-bundle-${arch === "arm64" ? "arm64" : "x86_64"}-75c8a6c.tar.gz`;
}

// The toolsets electron-builder downloads for a packaging run on this host.
// `module` and `constants` are what `checkToolsetConstants` re-derives from the
// installed package, so a version bump upstream is reported instead of silently
// seeding a file nothing will ever read.
export function requiredToolsets(platform = process.platform, arch = process.arch) {
  const toolsets = [];
  const sevenZip = sevenZipFilename(platform, arch);
  if (sevenZip != null) {
    toolsets.push({
      id: "7zip",
      module: "app-builder-lib/out/toolsets/7zip.js",
      releaseName: "7zip@1.0.0",
      filename: sevenZip,
      sha256: SEVEN_ZIP_CHECKSUMS[sevenZip],
    });
  }
  if (platform === "darwin") {
    const dmgbuild = dmgbuildFilename(arch);
    toolsets.push({
      id: "dmgbuild",
      module: "dmg-builder/out/dmgUtil.js",
      releaseName: "dmg-builder@1.2.5",
      filename: dmgbuild,
      sha256: DMGBUILD_CHECKSUMS[dmgbuild],
    });
  }
  return toolsets;
}

// app-builder-lib/out/util/electronGet.js `getCacheDirectory`, replicated for
// the `allowEnvVarOverride: true` call that `downloadBuilderToolset` makes.
export function cacheDirectory(env = process.env, platform = os.platform(), homeDir = os.homedir()) {
  const override = env.ELECTRON_BUILDER_CACHE?.trim();
  if (override && path.parse(override).root) {
    return override;
  }
  const appName = "electron-builder";
  if (platform === "darwin") {
    return path.join(homeDir, "Library", "Caches", appName);
  }
  if (platform === "win32") {
    const localAppData = env.LOCALAPPDATA?.trim();
    const username = env.USERNAME?.trim()?.toLowerCase();
    const isSystemUser =
      localAppData?.toLowerCase()?.includes("\\windows\\system32\\") || username === "system";
    if (!localAppData || isSystemUser) {
      return path.join(os.tmpdir(), `${appName}-cache`);
    }
    return path.join(localAppData, appName, "Cache");
  }
  const xdgCache = env.XDG_CACHE_HOME;
  return xdgCache && path.parse(xdgCache).root
    ? path.join(xdgCache, appName)
    : path.join(homeDir, ".cache", appName);
}

export function archivePath(toolset, cacheDir) {
  return path.join(cacheDir, toolset.releaseName, toolset.filename);
}

// app-builder-lib/out/util/electronGet.js `getBinariesMirrorUrl`, same variable
// order, so a mirrored CI environment seeds from the same host it would have
// downloaded from.
export function binariesMirrorUrl(env = process.env) {
  const allowHttp = env.ELECTRON_BUILDER_BINARIES_ALLOW_HTTP === "true";
  for (const name of [
    "NPM_CONFIG_ELECTRON_BUILDER_BINARIES_MIRROR",
    "npm_config_electron_builder_binaries_mirror",
    "npm_package_config_electron_builder_binaries_mirror",
    "ELECTRON_BUILDER_BINARIES_MIRROR",
  ]) {
    const value = env[name]?.trim();
    if (!value) {
      continue;
    }
    let parsed;
    try {
      parsed = new URL(value);
    } catch {
      continue;
    }
    if (parsed.protocol !== "https:" && !(allowHttp && parsed.protocol === "http:")) {
      continue;
    }
    return value.endsWith("/") ? value : `${value}/`;
  }
  return "https://github.com/electron-userland/electron-builder-binaries/releases/download/";
}

export function toolsetUrl(toolset, env = process.env) {
  return `${binariesMirrorUrl(env)}${toolset.releaseName}/${toolset.filename}`;
}

// Confirm the installed package still asks for exactly this release, filename
// and checksum. Substring checks, not a parser: the point is to notice a
// version bump, and any shape this fails to recognise degrades to a warning
// plus the pre-existing behaviour.
export function checkToolsetConstants(toolset, moduleSource) {
  const problems = [];
  if (!moduleSource.includes(toolset.releaseName)) {
    problems.push(`release "${toolset.releaseName}" is no longer requested`);
  }
  const entry = new RegExp(
    `"${toolset.filename.replace(/[.*+?^${}()|[\]\\]/g, "\\$&")}"\\s*:\\s*"${toolset.sha256}"`,
  );
  if (!entry.test(moduleSource)) {
    problems.push(`checksum entry for "${toolset.filename}" does not match`);
  }
  return problems;
}

export async function sha256File(file) {
  const hash = createHash("sha256");
  const handle = await fsp.open(file, "r");
  try {
    const stream = handle.createReadStream();
    for await (const chunk of stream) {
      hash.update(chunk);
    }
  } finally {
    await handle.close();
  }
  return hash.digest("hex");
}

// One attempt, with two independent deadlines: `stallTimeoutMs` re-armed on
// every chunk (the failure mode observed in CI was a connection that produced
// no bytes at all), and `totalTimeoutMs` for the whole transfer.
export async function downloadOnce(
  url,
  destination,
  { stallTimeoutMs, totalTimeoutMs, fetchImpl = fetch } = {},
) {
  const controller = new AbortController();
  let stallTimer = null;
  const armStall = () => {
    if (stallTimer != null) {
      clearTimeout(stallTimer);
    }
    stallTimer = setTimeout(
      () => controller.abort(new Error(`no data received for ${stallTimeoutMs} ms`)),
      stallTimeoutMs,
    );
  };
  const totalTimer = setTimeout(
    () => controller.abort(new Error(`transfer exceeded ${totalTimeoutMs} ms`)),
    totalTimeoutMs,
  );
  try {
    armStall();
    const response = await fetchImpl(url, { signal: controller.signal, redirect: "follow" });
    if (!response.ok) {
      throw new Error(`HTTP ${response.status} ${response.statusText} for ${url}`);
    }
    const hash = createHash("sha256");
    const handle = await fsp.open(destination, "w");
    try {
      for await (const chunk of response.body) {
        armStall();
        hash.update(chunk);
        await handle.write(chunk);
      }
    } finally {
      await handle.close();
    }
    return hash.digest("hex");
  } finally {
    if (stallTimer != null) {
      clearTimeout(stallTimer);
    }
    clearTimeout(totalTimer);
  }
}

const delay = (ms) => new Promise((resolve) => setTimeout(resolve, ms));

export async function downloadWithRetry(url, destination, options = {}) {
  const {
    attempts = 4,
    retryDelayMs = 2000,
    stallTimeoutMs = 30000,
    totalTimeoutMs = 300000,
    fetchImpl = fetch,
    expectedSha256,
    log = () => {},
  } = options;
  let lastError = null;
  for (let attempt = 1; attempt <= attempts; attempt++) {
    try {
      const digest = await downloadOnce(url, destination, {
        stallTimeoutMs,
        totalTimeoutMs,
        fetchImpl,
      });
      if (expectedSha256 && digest !== expectedSha256) {
        throw new Error(`checksum mismatch: expected ${expectedSha256}, got ${digest}`);
      }
      return digest;
    } catch (error) {
      lastError = error;
      log(`attempt ${attempt}/${attempts} failed: ${error.message ?? error}`);
      await fsp.rm(destination, { force: true });
      if (attempt < attempts) {
        await delay(retryDelayMs * attempt);
      }
    }
  }
  throw lastError ?? new Error(`failed to download ${url}`);
}

function resolveModuleSource(toolset, baseDir) {
  const require = createRequire(path.join(baseDir, "package.json"));
  const resolved = require.resolve(toolset.module);
  return { path: resolved, source: fs.readFileSync(resolved, "utf8") };
}

export async function prefetchToolsets(options = {}) {
  const {
    env = process.env,
    platform = process.platform,
    arch = process.arch,
    baseDir = desktopDir,
    cacheDir = cacheDirectory(env, os.platform()),
    fetchImpl = fetch,
    downloadOptions = {},
    logger = console,
    toolsets = requiredToolsets(platform, arch),
  } = options;
  const verbose = env.FORMAL_AI_PREFETCH_VERBOSE === "1";
  const results = [];

  for (const toolset of toolsets) {
    const target = archivePath(toolset, cacheDir);
    const result = { id: toolset.id, filename: toolset.filename, path: target };
    if (!toolset.sha256) {
      result.status = "skipped";
      result.reason = `no known checksum for ${toolset.filename}`;
      results.push(result);
      logger.warn(`::warning title=Toolset prefetch skipped::${result.reason}`);
      continue;
    }

    let installed;
    try {
      installed = resolveModuleSource(toolset, baseDir);
    } catch (error) {
      result.status = "skipped";
      result.reason = `cannot resolve ${toolset.module}: ${error.message ?? error}`;
      results.push(result);
      logger.warn(`::warning title=Toolset prefetch skipped::${result.reason}`);
      continue;
    }

    const problems = checkToolsetConstants(toolset, installed.source);
    if (problems.length > 0) {
      result.status = "stale";
      result.reason = `${toolset.module} changed upstream (${problems.join("; ")})`;
      results.push(result);
      logger.warn(
        `::warning title=Toolset prefetch out of date::${result.reason}. Update ` +
          "desktop/scripts/prefetch-builder-toolsets.mjs from the installed package.",
      );
      continue;
    }

    if (fs.existsSync(target) && (await sha256File(target)) === toolset.sha256) {
      result.status = "cached";
      results.push(result);
      logger.log(`Toolset already cached: ${target}`);
      continue;
    }

    const url = toolsetUrl(toolset, env);
    const temporary = `${target}.${process.pid}.part`;
    try {
      await fsp.mkdir(path.dirname(target), { recursive: true });
      await downloadWithRetry(url, temporary, {
        ...downloadOptions,
        fetchImpl,
        expectedSha256: toolset.sha256,
        log: verbose ? (message) => logger.log(`[prefetch] ${toolset.filename}: ${message}`) : undefined,
      });
      await fsp.rename(temporary, target);
      result.status = "downloaded";
      results.push(result);
      logger.log(`Prefetched ${toolset.filename} -> ${target}`);
    } catch (error) {
      await fsp.rm(temporary, { force: true });
      result.status = "failed";
      result.reason = `${url}: ${error.message ?? error}`;
      results.push(result);
      // Never fail the build: electron-builder downloads the toolset itself if
      // the cache is cold, which is exactly the pre-existing behaviour.
      logger.warn(
        `::warning title=Toolset prefetch failed::${result.reason}. ` +
          "electron-builder will download it during packaging.",
      );
    }
    if (verbose) {
      logger.log(`[prefetch] ${toolset.id}: ${JSON.stringify(result)}`);
    }
  }
  return results;
}

async function main() {
  const results = await prefetchToolsets();
  const failed = results.filter((r) => r.status === "failed" || r.status === "stale");
  if (failed.length > 0 && process.env.FORMAL_AI_PREFETCH_STRICT === "1") {
    console.error(`Toolset prefetch failed: ${failed.map((r) => r.reason).join("; ")}`);
    process.exitCode = 1;
  }
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  await main();
}
