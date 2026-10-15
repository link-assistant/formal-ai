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
    const configuredDeadline = process.env.FORMAL_AI_JOB_DEADLINE_EPOCH;
    let deadlineMilliseconds = null;
    if (configuredDeadline !== undefined) {
      if (!/^[0-9]+$/u.test(configuredDeadline)) throw new Error("Invalid desktop native startup job deadline");
      deadlineMilliseconds = Number(configuredDeadline) * 1000;
      if (!Number.isSafeInteger(deadlineMilliseconds)) throw new Error("Unsupported desktop native startup job deadline");
    }
    const startupBudget = () => {
      if (deadlineMilliseconds === null) return 10_000;
      const remaining = deadlineMilliseconds - Date.now();
      if (!Number.isSafeInteger(remaining) || remaining <= 0 || remaining > 2147483647) {
        throw new Error("Desktop native startup job deadline expired or unsupported");
      }
      // Match the existing selected-artifact --version smoke cap while staying
      // inside the same immutable packaging deadline; standalone calls keep 10s.
      return Math.min(30_000, remaining);
    };
    const timeoutMs = startupBudget();
    const started = Date.now();
    const result = spawnSync(destination, ["--version"], { encoding: "utf8", timeout: timeoutMs, maxBuffer: 1_048_576 });
    const elapsedMs = Date.now() - started;
    version = String(result.stdout || "").trim();
    if (result.error || result.signal || result.status !== 0 || version !== "formal-ai " + expectedVersion) {
      throw new Error("Desktop native binary startup/version verification failed: " +
        JSON.stringify({
          timeoutMs, elapsedMs, status: result.status ?? null, signal: result.signal ?? null,
          error: result.error ? { code: result.error.code ?? null, message: String(result.error.message).slice(0, 4096) } : null,
          stdout: String(result.stdout ?? "").slice(0, 4096),
          stdoutCharacters: String(result.stdout ?? "").length,
          stdoutTruncated: String(result.stdout ?? "").length > 4096,
          stderr: String(result.stderr ?? "").slice(0, 4096),
          stderrCharacters: String(result.stderr ?? "").length,
          stderrTruncated: String(result.stderr ?? "").length > 4096,
        }));
    }
    if (digest(destination) !== sha256 || digest(source) !== sha256) throw new Error("Desktop native binary changed during startup");
    startupBudget();
  }
  fs.rmSync(path.join(outputBin, "README.txt"), { force: true });
  return { source, destination, sha256, version };
}
