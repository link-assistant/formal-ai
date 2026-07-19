import http from "node:http";
import { createRequire } from "node:module";

const require = createRequire(import.meta.url);
const { createWebTools } = require("../desktop/lib/web-tools.cjs");

const server = http.createServer((_request, response) => {
  response.setHeader("content-type", "text/html; charset=utf-8");
  response.end(`<!doctype html><html><body><main id="result">loading</main>
    <script>document.querySelector('#result').textContent = 'rendered-javascript-proof';</script>
  </body></html>`);
});

await new Promise((resolve) => server.listen(0, "127.0.0.1", resolve));
const address = server.address();
const tools = createWebTools();
try {
  const result = await tools.fetch({ url: `http://127.0.0.1:${address.port}/app` });
  if (!result.body.includes("rendered-javascript-proof")) {
    throw new Error("headless browser did not extract the rendered page content");
  }
  console.log(JSON.stringify({ ok: true, engine: result.engine, rendered: true }));
} finally {
  await tools.close();
  await new Promise((resolve) => server.close(resolve));
}
