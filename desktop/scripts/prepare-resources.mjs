import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { spawnSync } from "node:child_process";

const scriptDir = path.dirname(fileURLToPath(import.meta.url));
const desktopDir = path.resolve(scriptDir, "..");
const repoRoot = path.resolve(desktopDir, "..");
const sourceWeb = path.join(repoRoot, "src", "web");
const sourceSeed = path.join(repoRoot, "data", "seed");
const outputWeb = path.join(desktopDir, "dist-web");
const outputSeed = path.join(outputWeb, "seed");
const outputBin = path.join(desktopDir, "bin");
const outputBrowser = path.join(desktopDir, "browser-runtime");

// Keep the desktop wrapper version in lockstep with the Rust crate so the
// release assets (and the /download page links) always match Cargo.toml, the
// single source of truth for the formal-ai version.
function syncDesktopVersion() {
  const cargoTomlPath = path.join(repoRoot, "Cargo.toml");
  const desktopPackagePath = path.join(desktopDir, "package.json");
  const cargoToml = fs.readFileSync(cargoTomlPath, "utf8");
  const packageSection = cargoToml.split(/^\[/m).find((s) => s.startsWith("package]"));
  const versionMatch = packageSection && packageSection.match(/^\s*version\s*=\s*"([^"]+)"/m);
  if (!versionMatch) {
    console.warn("Could not read [package] version from Cargo.toml; leaving desktop version unchanged.");
    return;
  }
  const cargoVersion = versionMatch[1];
  const desktopPackage = JSON.parse(fs.readFileSync(desktopPackagePath, "utf8"));
  if (desktopPackage.version === cargoVersion) {
    console.log(`Desktop version already in sync with Cargo.toml: ${cargoVersion}`);
    return;
  }
  const previous = desktopPackage.version;
  desktopPackage.version = cargoVersion;
  fs.writeFileSync(desktopPackagePath, `${JSON.stringify(desktopPackage, null, 2)}\n`);
  console.log(`Synced desktop version ${previous} -> ${cargoVersion} (from Cargo.toml)`);
}

syncDesktopVersion();

function copyDirectory(from, to) {
  fs.rmSync(to, { recursive: true, force: true });
  fs.mkdirSync(path.dirname(to), { recursive: true });
  // Issue #808: without `verbatimSymlinks` Node resolves every symlink target
  // to an absolute path (see the `verbatimSymlinks` default in `fs.cpSync`), so
  // the Chrome for Testing framework aliases inside browser-runtime end up
  // pointing at ~/.cache/ms-playwright/... After packaging, `codesign --verify
  // --deep` rejects the bundle with "invalid destination for symbolic link in
  // bundle". Copying the links verbatim keeps them relative and inside the app.
  fs.cpSync(from, to, { recursive: true, verbatimSymlinks: true });
}

copyDirectory(sourceWeb, outputWeb);
copyDirectory(sourceSeed, outputSeed);

// Playwright stores downloaded browsers outside node_modules, where
// electron-builder cannot see them. Install Chromium for the current target,
// copy the complete browser directory into extraResources, and record the
// executable relative path so every packaged desktop build is self-contained.
const playwrightCli = path.join(desktopDir, "node_modules", "playwright", "cli.js");
if (fs.existsSync(playwrightCli)) {
  const install = spawnSync(process.execPath, [playwrightCli, "install", "chromium"], {
    cwd: desktopDir,
    stdio: "inherit",
  });
  if (install.status !== 0) {
    throw new Error("Could not install the Chromium runtime required by desktop web capture");
  }
  const { chromium } = await import("playwright");
  const executable = chromium.executablePath();
  if (!fs.existsSync(executable)) {
    throw new Error(`Playwright reported a missing Chromium executable: ${executable}`);
  }
  const browserSource = path.dirname(path.dirname(executable));
  copyDirectory(browserSource, outputBrowser);
  const relativeExecutable = path.relative(browserSource, executable);
  fs.writeFileSync(path.join(outputBrowser, "executable-path.txt"), `${relativeExecutable}\n`);
  console.log(`Prepared desktop browser runtime: ${outputBrowser}`);
}

fs.mkdirSync(outputBin, { recursive: true });
const binaryName = process.platform === "win32" ? "formal-ai.exe" : "formal-ai";
const configuredBinary = process.env.FORMAL_AI_DESKTOP_BINARY || "";
const releaseBinary = path.join(repoRoot, "target", "release", binaryName);
const debugBinary = path.join(repoRoot, "target", "debug", binaryName);
const binarySource = [configuredBinary, releaseBinary, debugBinary].find(
  (candidate) => candidate && fs.existsSync(candidate),
);

if (binarySource) {
  const binaryDestination = path.join(outputBin, binaryName);
  fs.copyFileSync(binarySource, binaryDestination);
  if (process.platform !== "win32") {
    fs.chmodSync(binaryDestination, 0o755);
  }
  console.log(`Prepared desktop binary: ${binaryDestination}`);
} else {
  fs.writeFileSync(
    path.join(outputBin, "README.txt"),
    "Run `cargo build --release` or set FORMAL_AI_DESKTOP_BINARY before packaging to bundle formal-ai.\n",
  );
  console.warn("No formal-ai binary found; packaged app will fall back to formal-ai on PATH.");
}

console.log(`Prepared desktop web resources: ${outputWeb}`);
