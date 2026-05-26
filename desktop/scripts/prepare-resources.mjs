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
