import {
  getDefaultProviderIds,
  mergeResults,
} from "@link-assistant/web-search";

const PACKAGE = "@link-assistant/web-search";
const VERSION = "0.10.3";
const WEB_CAPTURE_BASE_URL = "http://localhost:3000";
const DEFAULT_TIMEOUT_MS = 2000;
const defaultProviderIds = Object.freeze(getDefaultProviderIds());

function urlKey(url) {
  try {
    const parsed = new URL(url);
    return `${parsed.hostname}${parsed.pathname}`.replace(/\/+$/, "").toLowerCase();
  } catch (_error) {
    return String(url || "").toLowerCase();
  }
}

function orderProviders(providerIds) {
  const order = new Map(defaultProviderIds.map((id, index) => [id, index]));
  return providerIds.slice().sort((left, right) =>
    (order.get(left.id || left) ?? 999) - (order.get(right.id || right) ?? 999));
}

function fuseResults(perProviderResults, k, evidence) {
  const grouped = Object.create(null);
  for (const provider of perProviderResults) {
    grouped[provider.id] = (Array.isArray(provider.results) ? provider.results : [])
      .filter((item) => item && item.url)
      .map((item, index) => ({
        title: item.title || item.url,
        url: item.url,
        snippet: item.excerpt || "",
        source: provider.id,
        rank: index + 1,
      }));
  }
  try {
    const merged = mergeResults(grouped, { strategy: "rrf", rrfK: k, removeDuplicates: true });
    if (!Array.isArray(merged)) return null;
    evidence.push(`web_search:component:${PACKAGE}@${VERSION}:defaultProviders`);
    evidence.push(`web_search:component:${PACKAGE}@${VERSION}:mergeResults`);
    return merged.map((result) => {
      const key = urlKey(result.url);
      const providers = [];
      let metadata = null;
      for (const provider of perProviderResults) {
        const list = Array.isArray(provider.results) ? provider.results : [];
        list.forEach((item, index) => {
          if (!item || urlKey(item.url) !== key) return;
          providers.push({ id: provider.id, rank: index + 1 });
          if (!metadata) metadata = item;
        });
      }
      return {
        url: result.url,
        title: result.title || result.url,
        excerpt: result.snippet || "",
        score: result.score || 0,
        providers,
        sourceTier: (metadata && metadata.sourceTier) || "",
        sourceLanguage: (metadata && metadata.sourceLanguage) || "",
      };
    });
  } catch (error) {
    const kind = error instanceof Error ? error.name : "unknown";
    evidence.push(`web_search:component_error:${PACKAGE}@${VERSION}:${kind}`);
    return null;
  }
}

async function fetchWithDeadline(url, options, timeoutMs) {
  if (typeof AbortController !== "function") return fetch(url, options);
  const controller = new AbortController();
  let timedOut = false;
  const timer = setTimeout(() => { timedOut = true; controller.abort(); }, timeoutMs);
  try {
    return await fetch(url, { ...options, signal: controller.signal });
  } catch (error) {
    if (timedOut) throw new Error(`timeout after ${timeoutMs}ms`);
    throw error;
  } finally {
    clearTimeout(timer);
  }
}

function failureKind(error) {
  const message = error instanceof Error ? error.message.toLowerCase() : String(error).toLowerCase();
  if ((error && error.name === "AbortError") || message.includes("timeout")) return "timeout";
  if (message.includes("cors") || message.includes("network") || message.includes("failed to fetch")) return "network";
  return "transport";
}

async function fetchUrl(targetUrl, evidence, timeoutMs = DEFAULT_TIMEOUT_MS) {
  const componentUrl = `${WEB_CAPTURE_BASE_URL}/fetch?url=${encodeURIComponent(targetUrl)}`;
  try {
    // /fetch preserves the target status and bytes, including target 5xx.
    const response = await fetchWithDeadline(componentUrl, {
      method: "GET", mode: "cors", credentials: "omit",
    }, timeoutMs);
    evidence.push("http_fetch:component:web-capture:http-get-fetch");
    evidence.push(`http_fetch:component_request:${componentUrl}`);
    return response;
  } catch (error) {
    evidence.push(`http_fetch:component_error:${failureKind(error)}`);
    return fetchWithDeadline(targetUrl, { method: "GET", mode: "cors" }, timeoutMs);
  }
}

// Browser workers cannot consume the page's window globals. This IIFE exposes
// the browser-safe published package calls and the bounded HTTP adapter.
self.FormalAIWebSearchComponent = Object.freeze({
  package: PACKAGE,
  version: VERSION,
  defaultProviderIds,
  orderProviders,
  fuseResults,
  fetchUrl,
});
