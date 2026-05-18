import { mkdir, writeFile } from "node:fs/promises";
import { createRequire } from "node:module";
import { dirname, resolve } from "node:path";

const require = createRequire(import.meta.url);
const { chromium } = require("../tests/e2e/node_modules/@playwright/test");

const outputPath = resolve(
  process.argv[2] ||
    "docs/case-studies/issue-107/raw-data/browser-provider-probe.json",
);

const requestedOriginUrl =
  process.env.FORMAL_AI_TESTS_URL ||
  "https://link-assistant.github.io/formal-ai/tests";
const fallbackOriginUrl =
  process.env.FORMAL_AI_FALLBACK_URL || "https://link-assistant.github.io/formal-ai/";

const providers = [
  {
    name: "google_home",
    url: "https://www.google.com/",
  },
  {
    name: "google_search",
    url: "https://www.google.com/search?q=formal-ai",
  },
  {
    name: "bing_search",
    url: "https://www.bing.com/search?q=formal-ai",
  },
  {
    name: "duckduckgo_html_search",
    url: "https://duckduckgo.com/html/?q=formal-ai",
  },
  {
    name: "brave_search",
    url: "https://search.brave.com/search?q=formal-ai",
  },
  {
    name: "wikipedia_rest_search",
    url: "https://en.wikipedia.org/w/rest.php/v1/search/page?q=formal%20verification&limit=1",
  },
  {
    name: "wikipedia_summary",
    url: "https://en.wikipedia.org/api/rest_v1/page/summary/Formal_verification",
  },
];

async function fetchProbe(page, provider) {
  return page.evaluate(async ({ name, url }) => {
    const startedAt = performance.now();
    const result = {
      name,
      url,
      ok: false,
      status: null,
      responseType: null,
      contentType: null,
      readable: false,
      elapsedMs: null,
      sample: "",
      errorName: "",
      errorMessage: "",
    };
    try {
      const response = await fetch(url, {
        mode: "cors",
        headers: { accept: "text/html,application/json,text/plain,*/*" },
      });
      result.ok = response.ok;
      result.status = response.status;
      result.responseType = response.type;
      result.contentType = response.headers.get("content-type") || "";
      const readable =
        result.contentType.includes("text/") ||
        result.contentType.includes("json") ||
        result.contentType.includes("xml");
      if (readable) {
        const text = await response.text();
        result.readable = true;
        result.sample = text.replace(/\s+/g, " ").slice(0, 240);
      }
    } catch (error) {
      result.errorName = error?.name || "Error";
      result.errorMessage = String(error?.message || error);
    } finally {
      result.elapsedMs = Math.round(performance.now() - startedAt);
    }
    return result;
  }, provider);
}

async function iframeProbe(page, provider) {
  return page.evaluate(async ({ name, url }) => {
    const startedAt = performance.now();
    return new Promise((resolveProbe) => {
      const iframe = document.createElement("iframe");
      const result = {
        name,
        url,
        loadEvent: false,
        errorEvent: false,
        elapsedMs: null,
      };
      const finish = () => {
        result.elapsedMs = Math.round(performance.now() - startedAt);
        iframe.remove();
        resolveProbe(result);
      };
      const timer = setTimeout(finish, 5000);
      iframe.onload = () => {
        clearTimeout(timer);
        result.loadEvent = true;
        finish();
      };
      iframe.onerror = () => {
        clearTimeout(timer);
        result.errorEvent = true;
        finish();
      };
      iframe.setAttribute("sandbox", "allow-scripts allow-same-origin allow-forms");
      iframe.src = url;
      iframe.style.width = "1px";
      iframe.style.height = "1px";
      document.body.appendChild(iframe);
    });
  }, provider);
}

const browser = await chromium.launch();
const page = await browser.newPage();
const requestedOriginResponse = await page.goto(requestedOriginUrl, {
  waitUntil: "domcontentloaded",
  timeout: 30000,
});
let activeOriginUrl = requestedOriginUrl;
let activeOriginResponse = requestedOriginResponse;
if (!requestedOriginResponse || !requestedOriginResponse.ok()) {
  activeOriginUrl = fallbackOriginUrl;
  activeOriginResponse = await page.goto(fallbackOriginUrl, {
    waitUntil: "domcontentloaded",
    timeout: 30000,
  });
}

const output = {
  capturedAt: new Date().toISOString(),
  requestedOriginUrl,
  fallbackOriginUrl,
  activeOriginUrl,
  requestedOriginStatus: requestedOriginResponse?.status() || null,
  activeOriginStatus: activeOriginResponse?.status() || null,
  actualUrl: page.url(),
  userAgent: await page.evaluate(() => navigator.userAgent),
  fetch: [],
  iframe: [],
};

for (const provider of providers) {
  output.fetch.push(await fetchProbe(page, provider));
  output.iframe.push(await iframeProbe(page, provider));
}

await browser.close();
await mkdir(dirname(outputPath), { recursive: true });
await writeFile(outputPath, `${JSON.stringify(output, null, 2)}\n`, "utf8");
console.log(outputPath);
