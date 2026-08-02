"use strict";

// Read-only native browsing for Electron and the VS Code Node host. The
// upstream packages are ESM, so they are loaded lazily: opening the app does not
// start Chromium, and tests can inject a browser without downloading one.

const SEARCH_ENGINES = Object.freeze(["google", "bing", "duckduckgo"]);

function searchMarkdown(query, results) {
  const lines = [`Web results for **${query}**`, ""];
  for (const [index, result] of results.entries()) {
    const title = String(result.title || result.url || "Result");
    const url = String(result.url || "");
    lines.push(`${index + 1}. **[${title}](${url})**`);
    if (result.snippet) {
      lines.push(`   > ${String(result.snippet).replace(/\s+/g, " ").trim()}`);
    }
    const sources = Array.isArray(result.sources)
      ? result.sources
      : result.source
        ? [result.source]
        : [];
    if (sources.length > 0) {
      lines.push(`   _${sources.join(", ")}_`);
    }
    lines.push("");
  }
  while (lines.at(-1) === "") lines.pop();
  return lines.join("\n");
}

function createWebTools(options = {}) {
  const browserEngine = String(options.browserEngine || "playwright");
  let browserPromise = null;

  async function defaultCreateBrowser() {
    const capture = await import("@link-assistant/web-capture/src/browser.js");
    const browserOptions = { ...(options.browserOptions || {}) };
    if (options.browserExecutablePath) {
      browserOptions.executablePath = String(options.browserExecutablePath);
    }
    return capture.createBrowser(browserEngine, browserOptions);
  }

  const createBrowser = options.createBrowser || defaultCreateBrowser;

  async function browser() {
    if (!browserPromise) {
      browserPromise = Promise.resolve(createBrowser());
    }
    return browserPromise;
  }

  function browserCommander() {
    return {
      async runAction(action) {
        const instance = await browser();
        const page = await instance.newPage();
        const controller = new AbortController();
        try {
          return await action(page, controller.signal);
        } finally {
          await page.close();
        }
      },
    };
  }

  async function search(input = {}) {
    const query = String(input.query || input.prompt || "").trim();
    if (!query) throw new Error("web_search requires a query");
    const limit = Math.max(1, Math.min(20, Number(input.limit) || 10));
    const webSearch = options.webSearchModule || (await import("@link-assistant/web-search"));
    const providers = SEARCH_ENGINES.map((engine) =>
      webSearch.createBrowserProvider({ engine, browserCommander: browserCommander() }),
    );
    const settled = await Promise.allSettled(
      providers.map((provider) => provider.search(query, { limit })),
    );
    const byProvider = {};
    SEARCH_ENGINES.forEach((engine, index) => {
      byProvider[engine] = settled[index].status === "fulfilled" ? settled[index].value : [];
    });
    const results = webSearch
      .mergeResults(byProvider, { strategy: "rrf", removeDuplicates: true })
      .slice(0, limit);
    return {
      query,
      providers: [...SEARCH_ENGINES],
      strategy: "rrf",
      engine: browserEngine,
      results,
      body: searchMarkdown(query, results),
    };
  }

  async function fetchRendered(input = {}) {
    const url = String(input.url || "").trim();
    if (!/^https?:\/\//i.test(url)) throw new Error("web_fetch requires an http(s) url");
    const instance = await browser();
    const page = await instance.newPage();
    try {
      await page.goto(url, { waitUntil: "networkidle0", timeout: 30000 });
      const html = String((await page.content()) || "");
      const visibleText = await page.evaluate(() => document.body?.innerText || "");
      return {
        url,
        engine: browserEngine,
        body: String(visibleText || html),
        html,
      };
    } finally {
      await page.close();
    }
  }

  async function close() {
    if (!browserPromise) return;
    const instance = await browserPromise;
    browserPromise = null;
    await instance.close();
  }

  return { search, fetch: fetchRendered, close };
}

module.exports = { SEARCH_ENGINES, createWebTools, searchMarkdown };
