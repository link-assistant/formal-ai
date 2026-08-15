import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import test from "node:test";
import { createRequire } from "node:module";
import { fileURLToPath } from "node:url";

import { bundleWebTools } from "./bundle-web-tools.mjs";

const scriptDir = path.dirname(fileURLToPath(import.meta.url));
const vscodeDir = path.resolve(scriptDir, "..");
const repoRoot = path.resolve(vscodeDir, "..");
const require = createRequire(import.meta.url);

// Issue #1014: exercise the package's real dependency graph. Source-only smoke
// checks missed Playwright's optional chromium-bidi imports and let VSIX
// packaging fail only after the dependency update reached CI.
test("desktop web tools can be bundled for the VSIX", async () => {
  const outputDir = fs.mkdtempSync(path.join(os.tmpdir(), "formal-ai-vsix-bundle-"));
  const outfile = path.join(outputDir, "web-tools.cjs");
  try {
    await assert.doesNotReject(() =>
      bundleWebTools({
        entryPoint: path.join(repoRoot, "desktop", "lib", "web-tools.cjs"),
        outfile,
        nodeModulesDir: path.join(vscodeDir, "node_modules"),
      }),
    );
    assert.ok(fs.statSync(outfile).size > 0, "bundle must not be empty");
    assert.doesNotThrow(() => require(outfile), "bundle must load in Node");
    const bundle = fs.readFileSync(outfile, "utf8");
    assert.match(bundle, /import\("playwright"\)/);
    assert.doesNotMatch(bundle, /chromium-bidi\/lib\/cjs/);
  } finally {
    fs.rmSync(outputDir, { recursive: true, force: true });
  }
});
