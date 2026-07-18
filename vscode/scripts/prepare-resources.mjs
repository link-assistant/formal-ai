import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { spawnSync } from "node:child_process";

// Package-time resource preparation for the VS Code extension (`vsce package`).
//
// Issue #353: the extension renders the committed `src/web/` chat UI inside a
// Webview and reuses the desktop shell's tool-router / memory-sync clients. In a
// dev checkout those assets are resolved across the repo (src/web, data/seed,
// desktop/lib); a packaged `.vsix` is self-contained, so this script copies them
// into the extension directory:
//
//   ../../src/web              -> vscode/dist-web
//   ../../data/seed            -> vscode/dist-web/seed
//   ../../desktop/lib/*.cjs    -> vscode/src/lib/vendor
//
// `chat-view.cjs` (resourceRootCandidates) prefers `dist-web/` and falls back to
// the dev layout; `extension.node.cjs` (requireReused) prefers `src/lib/vendor`
// and falls back to `<repo>/desktop/lib`. Unlike the Electron shell there is no
// binary to bundle — the Node host launches `formal-ai serve` from PATH / cargo,
// and the web host never starts a process at all.

const scriptDir = path.dirname(fileURLToPath(import.meta.url));
const vscodeDir = path.resolve(scriptDir, "..");
const repoRoot = path.resolve(vscodeDir, "..");
const sourceWeb = path.join(repoRoot, "src", "web");
const sourceSeed = path.join(repoRoot, "data", "seed");
const sourceLib = path.join(repoRoot, "desktop", "lib");
const sourceLicense = path.join(repoRoot, "LICENSE");
const outputLicense = path.join(vscodeDir, "LICENSE");
const outputWeb = path.join(vscodeDir, "dist-web");
const outputSeed = path.join(outputWeb, "seed");
const outputVendor = path.join(vscodeDir, "src", "lib", "vendor");
const outputBrowser = path.join(vscodeDir, "browser-runtime");

// Reused desktop modules copied verbatim into the package so the Node host can
// `require` them without reaching outside the extension.
const VENDOR_MODULES = ["tool-router.cjs", "memory-sync.cjs", "web-tools.cjs"];

// Keep the extension version in lockstep with the Rust crate so the Marketplace
// listing always matches Cargo.toml, the single source of truth for the
// formal-ai version (mirrors desktop/scripts/prepare-resources.mjs).
function syncExtensionVersion() {
  const cargoTomlPath = path.join(repoRoot, "Cargo.toml");
  const vscodePackagePath = path.join(vscodeDir, "package.json");
  const cargoToml = fs.readFileSync(cargoTomlPath, "utf8");
  const packageSection = cargoToml.split(/^\[/m).find((s) => s.startsWith("package]"));
  const versionMatch = packageSection && packageSection.match(/^\s*version\s*=\s*"([^"]+)"/m);
  if (!versionMatch) {
    console.warn("Could not read [package] version from Cargo.toml; leaving extension version unchanged.");
    return;
  }
  const cargoVersion = versionMatch[1];
  const vscodePackage = JSON.parse(fs.readFileSync(vscodePackagePath, "utf8"));
  if (vscodePackage.version === cargoVersion) {
    console.log(`Extension version already in sync with Cargo.toml: ${cargoVersion}`);
    return;
  }
  const previous = vscodePackage.version;
  vscodePackage.version = cargoVersion;
  fs.writeFileSync(vscodePackagePath, `${JSON.stringify(vscodePackage, null, 2)}\n`);
  console.log(`Synced extension version ${previous} -> ${cargoVersion} (from Cargo.toml)`);
}

function copyDirectory(from, to) {
  fs.rmSync(to, { recursive: true, force: true });
  fs.mkdirSync(path.dirname(to), { recursive: true });
  fs.cpSync(from, to, { recursive: true });
}

syncExtensionVersion();
fs.copyFileSync(sourceLicense, outputLicense);

copyDirectory(sourceWeb, outputWeb);
copyDirectory(sourceSeed, outputSeed);

// VS Code users must not need a separately installed Chrome. Bundle the same
// Playwright Chromium runtime as the Electron distribution and leave a
// relative executable manifest for the extension host.
const playwrightCli = path.join(vscodeDir, "node_modules", "playwright", "cli.js");
if (fs.existsSync(playwrightCli)) {
  const install = spawnSync(process.execPath, [playwrightCli, "install", "chromium"], {
    cwd: vscodeDir,
    stdio: "inherit",
  });
  if (install.status !== 0) {
    throw new Error("Could not install the Chromium runtime required by VS Code web capture");
  }
  const { chromium } = await import("playwright");
  const executable = chromium.executablePath();
  const browserSource = path.dirname(path.dirname(executable));
  copyDirectory(browserSource, outputBrowser);
  fs.writeFileSync(
    path.join(outputBrowser, "executable-path.txt"),
    `${path.relative(browserSource, executable)}\n`,
  );
}

fs.mkdirSync(outputVendor, { recursive: true });
for (const moduleName of VENDOR_MODULES) {
  fs.copyFileSync(path.join(sourceLib, moduleName), path.join(outputVendor, moduleName));
}

console.log(`Prepared VS Code web resources: ${outputWeb}`);
console.log(`Vendored desktop modules: ${VENDOR_MODULES.join(", ")} -> ${outputVendor}`);
