#!/usr/bin/env node
// Issue #1017 — verify `desktop/scripts/prefetch-builder-toolsets.mjs` against
// the real electron-builder packages and the real CDN.
//
// The prefetch script hard-codes the toolset filenames and SHA-256 digests that
// app-builder-lib and dmg-builder consult before they touch the network. Those
// constants only work while they match the installed packages, so this script
// checks them for every platform/arch pair, then performs one real download to
// prove the archive lands exactly where `downloadAndExtract` looks for it and
// that a second run is a cache hit with no request at all.
//
// Usage (installs the two packages into a scratch directory by default):
//
//   node experiments/verify-builder-toolset-prefetch.mjs
//   node experiments/verify-builder-toolset-prefetch.mjs --base-dir desktop
//
// Recorded result on 2026-08-17 with app-builder-lib@26.15.7 and
// dmg-builder@26.15.7: all 8 pairs `ok`, 7zip-linux-x64.tar.gz downloaded
// (1 307 612 bytes) to <cache>/7zip@1.0.0/7zip-linux-x64.tar.gz, second run
// `cached`.

import { createRequire } from "node:module";
import { execFileSync } from "node:child_process";
import fs from "node:fs";
import fsp from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";

const here = path.dirname(fileURLToPath(import.meta.url));
const repoRoot = path.resolve(here, "..");
const script = path.join(
  repoRoot,
  "desktop",
  "scripts",
  "prefetch-builder-toolsets.mjs",
);

const argv = process.argv.slice(2);
const baseDirFlag = argv.indexOf("--base-dir");
const versions = { "app-builder-lib": "26.15.7", "dmg-builder": "26.15.7" };

function installScratch() {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "eb-toolset-parity-"));
  fs.writeFileSync(
    path.join(root, "package.json"),
    `${JSON.stringify({ name: "eb-toolset-parity", private: true }, null, 2)}\n`,
  );
  const specs = Object.entries(versions).map(([name, v]) => `${name}@${v}`);
  console.log(`installing ${specs.join(" ")} into ${root}`);
  execFileSync(
    "npm",
    ["install", "--no-save", "--no-audit", "--no-fund", "--ignore-scripts", ...specs],
    { cwd: root, stdio: "inherit" },
  );
  return root;
}

const baseDir =
  baseDirFlag === -1
    ? installScratch()
    : path.resolve(repoRoot, argv[baseDirFlag + 1]);

const toolsets = await import(script);
const require = createRequire(path.join(baseDir, "package.json"));

let drifted = 0;
for (const platform of ["darwin", "linux", "win32"]) {
  for (const arch of ["x64", "arm64"]) {
    for (const toolset of toolsets.requiredToolsets(platform, arch)) {
      const source = fs.readFileSync(require.resolve(toolset.module), "utf8");
      const problems = toolsets.checkToolsetConstants(toolset, source);
      drifted += problems.length > 0 ? 1 : 0;
      console.log(
        `${platform}/${arch} ${toolset.filename}: ${
          problems.length > 0 ? `DRIFT: ${problems.join("; ")}` : "ok"
        }`,
      );
    }
  }
}

const cacheDir = fs.mkdtempSync(path.join(os.tmpdir(), "eb-toolset-cache-"));
const env = { ...process.env, ELECTRON_BUILDER_CACHE: cacheDir };
const first = await toolsets.prefetchToolsets({ env, baseDir });
const second = await toolsets.prefetchToolsets({ env, baseDir });
console.log(`first run:  ${first.map((r) => `${r.filename}=${r.status}`).join(" ")}`);
console.log(`second run: ${second.map((r) => `${r.filename}=${r.status}`).join(" ")}`);

const downloaded = first.filter((r) => r.status === "downloaded");
for (const result of downloaded) {
  const { size } = await fsp.stat(result.path);
  console.log(`${result.path} (${size} bytes)`);
}
await fsp.rm(cacheDir, { recursive: true, force: true });

const cachedSecond = second.every((r) => r.status === "cached");
if (drifted > 0 || downloaded.length === 0 || !cachedSecond) {
  console.error(
    `FAILED: drifted=${drifted} downloaded=${downloaded.length} secondRunAllCached=${cachedSecond}`,
  );
  process.exit(1);
}
console.log("OK: constants match the installed packages and the cache seeds correctly");
