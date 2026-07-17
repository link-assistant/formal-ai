import assert from "node:assert/strict";
import { test } from "node:test";
import { createRequire } from "node:module";

const require = createRequire(import.meta.url);
const { createWebTools } = require("../lib/web-tools.cjs");

function fakeBrowser() {
  const visited = [];
  let closed = false;
  return {
    visited,
    get closed() {
      return closed;
    },
    async newPage() {
      let currentUrl = "";
      return {
        async goto(url) {
          currentUrl = url;
          visited.push(url);
        },
        async evaluate(action) {
          if (String(action).includes("innerText")) return "rendered by javascript";
          const engine = currentUrl.includes("google")
            ? "google"
            : currentUrl.includes("bing")
              ? "bing"
              : "duckduckgo";
          return [
            {
              title: `${engine} shared`,
              url: "https://example.com/shared",
              snippet: `${engine} shared result`,
            },
            {
              title: `${engine} only`,
              url: `https://example.com/${engine}`,
              snippet: `${engine} result`,
            },
          ];
        },
        async content() {
          return "<html><body><main>rendered by javascript</main></body></html>";
        },
        async close() {},
      };
    },
    async close() {
      closed = true;
    },
  };
}

test("search visits Google, Bing, and DuckDuckGo in a headless browser and fuses with RRF", async () => {
  const browser = fakeBrowser();
  const tools = createWebTools({
    createBrowser: async () => browser,
    waitAfterNavigationMs: 0,
  });

  const result = await tools.search({ query: "formal ai", limit: 5 });

  assert.equal(browser.visited.length, 3);
  assert.ok(browser.visited.some((url) => url.startsWith("https://www.google.com/search")));
  assert.ok(browser.visited.some((url) => url.startsWith("https://www.bing.com/search")));
  assert.ok(browser.visited.some((url) => url.startsWith("https://duckduckgo.com/")));
  assert.deepEqual(result.providers, ["google", "bing", "duckduckgo"]);
  assert.equal(result.strategy, "rrf");
  assert.equal(result.results[0].url, "https://example.com/shared");
  assert.deepEqual(
    new Set(result.results[0].sources),
    new Set(["browser-google", "browser-bing", "browser-duckduckgo"]),
  );
  assert.match(result.body, /google shared/i);
});

test("web fetch extracts browser-rendered content from a JavaScript-heavy page", async () => {
  const browser = fakeBrowser();
  const tools = createWebTools({ createBrowser: async () => browser });

  const result = await tools.fetch({ url: "https://example.com/javascript-app" });

  assert.deepEqual(browser.visited, ["https://example.com/javascript-app"]);
  assert.match(result.body, /rendered by javascript/);
  assert.match(result.html, /<main>rendered by javascript<\/main>/);
  assert.equal(result.engine, "playwright");
});

test("closing web tools releases the shared browser", async () => {
  const browser = fakeBrowser();
  const tools = createWebTools({ createBrowser: async () => browser });
  await tools.fetch({ url: "https://example.com" });
  await tools.close();
  assert.equal(browser.closed, true);
});
