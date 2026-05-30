import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const scriptDir = path.dirname(fileURLToPath(import.meta.url));
const desktopDir = path.resolve(scriptDir, "..");
const repoRoot = path.resolve(desktopDir, "..");
const sourceWeb = path.join(repoRoot, "src", "web");
const sourceSeed = path.join(repoRoot, "data", "seed");
const outputWeb = path.join(desktopDir, "dist-web");
const outputSeed = path.join(outputWeb, "seed");
const outputBin = path.join(desktopDir, "bin");

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
  fs.cpSync(from, to, { recursive: true });
}

copyDirectory(sourceWeb, outputWeb);
copyDirectory(sourceSeed, outputSeed);

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
