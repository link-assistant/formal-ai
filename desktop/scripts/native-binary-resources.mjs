import fs from "node:fs";
import path from "node:path";
import { createHash } from "node:crypto";
import { spawnSync } from "node:child_process";

/** Copy a native engine and prove its bytes; release mode also proves startup. */
export function bundleNativeBinary({
  repoRoot, outputBin, platform = process.platform, configuredBinary = "",
  required = false, expectedVersion = "",
}) {
  const name = platform === "win32" ? "formal-ai.exe" : "formal-ai";
  const candidates = configuredBinary ? [configuredBinary] : [
    path.join(repoRoot, "rust", "target", "release", name),
    path.join(repoRoot, "target", "release", name),
    ...required ? [] : [
      path.join(repoRoot, "rust", "target", "debug", name),
      path.join(repoRoot, "target", "debug", name),
    ],
  ];
  const source = candidates.find(candidate => fs.existsSync(candidate) && fs.statSync(candidate).isFile());
  if (!source) {
    if (required || configuredBinary) throw new Error("Required desktop native binary is missing: " + candidates.join(", "));
    return null;
  }
  fs.mkdirSync(outputBin, { recursive: true });
  const destination = path.join(outputBin, name);
  fs.copyFileSync(source, destination);
  if (platform !== "win32") fs.chmodSync(destination, 0o755);
  const digest = file => createHash("sha256").update(fs.readFileSync(file)).digest("hex");
  const sha256 = digest(destination);
  if (sha256 !== digest(source)) throw new Error("Desktop native binary changed while copying");
  let version = null;
  if (required) {
    if (!expectedVersion) throw new Error("Required desktop native binary has no expected package version");
    const result = spawnSync(destination, ["--version"], { encoding: "utf8", timeout: 10_000, maxBuffer: 1_048_576 });
    version = String(result.stdout || "").trim();
    if (result.error || result.signal || result.status !== 0 || version !== "formal-ai " + expectedVersion) {
      throw new Error("Desktop native binary startup/version verification failed: " +
        String(result.error?.message || result.stderr || version || result.signal || result.status));
    }
  }
  fs.rmSync(path.join(outputBin, "README.txt"), { force: true });
  return { source, destination, sha256, version };
}
