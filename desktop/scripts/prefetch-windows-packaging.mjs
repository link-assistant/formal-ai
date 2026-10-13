import fs from "node:fs";
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { execFileSync } from "node:child_process";
import path from "node:path";
import { createRequire } from "node:module";
import { fileURLToPath, pathToFileURL } from "node:url";
import { prefetchToolsets } from "./prefetch-builder-toolsets.mjs";
export async function prefetchWindowsPackaging({version, config = {}, cacheDir, downloadElectronArtifactZip, getMakeNsisPath, getNsisPluginsPath, prefetchTools}) {
  if (typeof version !== "string" || !/^\d+\.\d+\.\d+(?:-[0-9A-Za-z.-]+)?$/.test(version)) throw new Error("Missing locked Electron version");
  const nsis = config.toolsets?.nsis;
  const options = config.nsis ?? {};
  // Start all checksum-qualified upstream downloads before waiting for any one.
  return Promise.all([
    downloadElectronArtifactZip({version, platformName: "win32", arch: "arm64", artifactName: "electron", cacheDir, electronDownload: config.electronDownload}),
    getMakeNsisPath(nsis, options.customNsisBinary),
    getNsisPluginsPath(nsis, options.customNsisResources),
    prefetchTools(),
  ]);
}
export function validateSelectedDesktop(selectedDesktop, source, protocolDirectory, expectedProtocolCommit) {
  const selectedRoot = path.dirname(selectedDesktop);
  const git = args => execFileSync("git", args, {cwd: selectedRoot, encoding: "utf8", maxBuffer: 1024 * 1024}).trim();
  assert.equal(git(["rev-parse", "HEAD"]), source.source_commit);
  assert.equal(git(["rev-parse", "HEAD^{tree}"]), source.source_tree);
  const protocol = JSON.parse(fs.readFileSync(path.join(protocolDirectory, "packaging-protocol-selection.json"), "utf8"));
  assert.equal(protocol.commit, expectedProtocolCommit);
  assert.deepEqual(protocol.files.map(file => file.name).sort(), ["install-node-dependencies.sh", "prefetch-builder-toolsets.mjs", "prefetch-windows-packaging.mjs"]);
  for (const file of protocol.files) {
    const bytes = fs.readFileSync(path.join(protocolDirectory, file.name));
    assert.equal(bytes.length, file.bytes);
    assert.equal(createHash("sha256").update(bytes).digest("hex"), file.sha256);
  }
  for (const name of ["package.json", "package-lock.json"]) {
    const actual = fs.readFileSync(path.join(selectedDesktop, name));
    const committed = execFileSync("git", ["show", `HEAD:desktop/${name}`], {cwd: selectedRoot, maxBuffer: 1024 * 1024});
    assert.deepEqual(actual, committed, "selected Desktop input differs from committed source");
  }
}
async function main() {
  const selectedDesktop = path.resolve(process.argv[2]);
  const source = JSON.parse(fs.readFileSync(process.argv[3], "utf8"));
  validateSelectedDesktop(selectedDesktop, source, path.dirname(fileURLToPath(import.meta.url)), process.env.FORMAL_AI_PACKAGING_PROTOCOL_COMMIT);
  if (process.argv[4] === "--validate-only") return;
  const require = createRequire(path.join(selectedDesktop, "package.json"));
  const lock = JSON.parse(fs.readFileSync(path.join(selectedDesktop, "package-lock.json"), "utf8"));
  const installed = require("electron/package.json").version;
  const version = lock.packages?.["node_modules/electron"]?.version;
  if (installed !== version) throw new Error("Installed Electron differs from locked version");
  const config = JSON.parse(fs.readFileSync(path.join(selectedDesktop, "package.json"), "utf8")).build;
  const {downloadElectronArtifactZip} = require("app-builder-lib/out/util/electronGet.js");
  const {getMakeNsisPath, getNsisPluginsPath} = require("app-builder-lib/out/toolsets/windows.js");
  await prefetchWindowsPackaging({version, config, cacheDir: process.env.ELECTRON_CACHE, downloadElectronArtifactZip, getMakeNsisPath, getNsisPluginsPath, prefetchTools: () => prefetchToolsets({baseDir: selectedDesktop})});
}
if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) await main();
