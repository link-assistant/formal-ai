import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import test from "node:test";
import { fileURLToPath } from "node:url";

const scriptPath = path.join(
  path.dirname(fileURLToPath(import.meta.url)),
  "prepare-resources.mjs",
);

// Issue #808: `fs.cpSync` rewrites symlink targets to absolute paths unless
// `verbatimSymlinks` is set. Absolute links inside the packaged .app make
// `codesign --verify --deep` fail with "invalid destination for symbolic link
// in bundle", which broke every macOS build.
test("browser runtime copies keep symbolic links verbatim", () => {
  const source = fs.readFileSync(scriptPath, "utf8");
  assert.match(source, /fs\.cpSync\([^)]*verbatimSymlinks:\s*true/s);
});

test("verbatimSymlinks preserves relative framework aliases", () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "formal-ai-resources-"));
  const from = path.join(root, "from");
  const to = path.join(root, "to");
  fs.mkdirSync(path.join(from, "Versions", "A"), { recursive: true });
  fs.symlinkSync("A", path.join(from, "Versions", "Current"));

  fs.cpSync(from, to, { recursive: true, verbatimSymlinks: true });

  assert.equal(fs.readlinkSync(path.join(to, "Versions", "Current")), "A");
  fs.rmSync(root, { recursive: true, force: true });
});
