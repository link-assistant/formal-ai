import assert from "node:assert/strict";
import fs from "node:fs";
import http from "node:http";
import path from "node:path";
import { createRequire } from "node:module";
import { fileURLToPath } from "node:url";

const experimentDir = path.dirname(fileURLToPath(import.meta.url));
const repoRoot = path.resolve(experimentDir, "../..");
const vscodeDir = process.env.FORMAL_AI_VSIX_ROOT || path.join(repoRoot, "vscode");
const runtimeDir = path.join(vscodeDir, "browser-runtime");
const executableRelative = fs.readFileSync(
  path.join(runtimeDir, "executable-path.txt"),
  "utf8",
).trim();
const browserExecutablePath = path.join(runtimeDir, executableRelative);
const require = createRequire(import.meta.url);
const { createWebTools } = require(path.join(vscodeDir, "src/lib/vendor/web-tools.cjs"));

const server = http.createServer((_request, response) => {
  response.writeHead(200, { "content-type": "text/html" });
  response.end("<!doctype html><title>Issue 1014</title><main>VSIX browser smoke passed</main>");
});
await new Promise((resolve) => server.listen(0, "127.0.0.1", resolve));

const address = server.address();
const browserEngine = process.argv[2] || "playwright";
const tools = createWebTools({ browserExecutablePath, browserEngine });
try {
  const result = await tools.fetch({ url: `http://127.0.0.1:${address.port}/` });
  assert.match(result.body, /VSIX browser smoke passed/);
  assert.equal(result.engine, browserEngine);
  console.log(`Packaged VSIX ${browserEngine} browser capture passed`);
} finally {
  await tools.close();
  await new Promise((resolve, reject) =>
    server.close((error) => (error ? reject(error) : resolve())),
  );
}
