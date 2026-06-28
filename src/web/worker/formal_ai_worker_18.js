// Worker module 19 of 21. Loaded by ../formal_ai_worker.js.
function evaluateFramePolicy(headers, targetUrl, embedderOrigin) {
  const frameHeaders = normalizeFramePolicyHeaders(headers);
  const xFrameOptions = frameHeaders["x-frame-options"] || "";
  const csp = frameHeaders["content-security-policy"] || "";
  let target;
  try {
    target = new URL(targetUrl);
  } catch (_error) {
    return { status: "unknown", reason: "the target URL could not be parsed" };
  }

  const xFrameDirectives = xFrameOptions
    .split(",")
    .map((part) => part.trim().toLowerCase())
    .filter(Boolean);
  const sourceSets = frameAncestorsSourceSets(csp);
  const cspHasFrameAncestorsNone = sourceSets.some((sources) =>
    sources.includes("'none'"),
  );
  if (xFrameDirectives.includes("deny")) {
    return {
      status: "blocked",
      reason: cspHasFrameAncestorsNone
        ? "the page sends X-Frame-Options: DENY and CSP frame-ancestors 'none'"
        : "the page sends X-Frame-Options: DENY",
    };
  }
  if (xFrameDirectives.includes("sameorigin")) {
    let embedder;
    try {
      embedder = embedderOrigin ? new URL(embedderOrigin) : null;
    } catch (_error) {
      embedder = null;
    }
    if (!embedder || embedder.origin !== target.origin) {
      return {
        status: "blocked",
        reason: "the page sends X-Frame-Options: SAMEORIGIN",
      };
    }
  }

  if (sourceSets.length > 0) {
    let embedder;
    try {
      embedder = embedderOrigin ? new URL(embedderOrigin) : null;
    } catch (_error) {
      embedder = null;
    }
    if (!embedder) {
      return {
        status: "unknown",
        reason: "the current web app origin is unavailable",
      };
    }
    for (const sources of sourceSets) {
      if (sources.includes("'none'")) {
        return {
          status: "blocked",
          reason: "the page sends CSP frame-ancestors 'none'",
        };
      }
      if (
        sources.length > 0 &&
        !sources.some((source) => sourceExpressionMatches(source, target, embedder))
      ) {
        return {
          status: "blocked",
          reason:
            "the page's CSP frame-ancestors directive does not include this web app",
        };
      }
    }
  }

  return {
    status: "allowed",
    reason: "no blocking X-Frame-Options or CSP frame-ancestors policy was detected",
  };
}

async function detectFramePolicy(url) {
  const evidence = [`url_preview:frame_policy_check:${FRAME_POLICY_CHECK_ENDPOINT}`];
  if (typeof fetch !== "function") {
    return {
      status: "unknown",
      reason: "browser fetch is not available",
      evidence: evidence.concat("url_preview:frame_policy:unknown"),
    };
  }
  if (!isPublicHttpUrl(url)) {
    return {
      status: "unknown",
      reason: "only public HTTP(S) URLs are checked by the frame-policy service",
      evidence: evidence.concat("url_preview:frame_policy:unknown"),
    };
  }

  try {
    const response = await fetch(framePolicyCheckUrl(url), {
      method: "GET",
      mode: "cors",
      credentials: "omit",
    });
    evidence.push(`url_preview:frame_policy_status:${response.status}`);
    if (!response.ok) {
      return {
        status: "unknown",
        reason: `the frame-policy service returned HTTP ${response.status}`,
        evidence: evidence.concat("url_preview:frame_policy:unknown"),
      };
    }
    const data = await response.json();
    const headers = (data && (data.headers || (data.data && data.data.headers))) || null;
    if (!headers || typeof headers !== "object") {
      return {
        status: "unknown",
        reason: "the frame-policy service did not return response headers",
        evidence: evidence.concat("url_preview:frame_policy:unknown"),
      };
    }
    const verdict = evaluateFramePolicy(headers, url, currentEmbedderOrigin());
    return {
      ...verdict,
      evidence: evidence.concat(`url_preview:frame_policy:${verdict.status}`),
    };
  } catch (_error) {
    return {
      status: "unknown",
      reason: "the frame-policy service could not be reached from this browser",
      evidence: evidence.concat("url_preview:frame_policy:unknown"),
    };
  }
}

function directExternalLinkAnswer(url, framePolicy, leadingLine) {
  const lines = [leadingLine || `I suggest opening this in a new tab: [${url}](${url}).`, ""];
  if (framePolicy && framePolicy.status === "blocked") {
    lines.push(
      `I checked the page's frame policy, and it does not allow embedding here because ${framePolicy.reason}.`,
    );
  } else if (framePolicy && framePolicy.status === "unknown") {
    lines.push(
      `I could not verify that this page allows embedding here because ${framePolicy.reason}.`,
    );
  } else {
    lines.push("I could not verify that this page allows embedding here.");
  }
  lines.push(
    "Browser JavaScript also cannot read the page content directly unless the site allows CORS, so the direct external link is the reliable option.",
  );
  return lines.join("\n");
}

async function tryFetch(prompt) {
  const normalized = normalizePrompt(prompt);
  const url = extractHttpFetchUrl(prompt, normalized);
  if (!url) return null;

  const evidence = [`http_fetch:request:${url}`];

  if (typeof fetch !== "function") {
    return {
      intent: "http_fetch",
      content: `HTTP fetch is not available in this environment.\n\nURL: [${url}](${url})`,
      confidence: 0.5,
      evidence,
      iframeUrl: url,
    };
  }

  try {
    const response = await fetch(url, { method: "GET", mode: "cors" });
    const status = response.status;
    const contentType = response.headers.get("content-type") || "";
    let body = "";
    if (contentType.includes("text/") || contentType.includes("application/json")) {
      const text = await response.text();
      body = text.length > 2000 ? `${text.slice(0, 2000)}\n\n*(truncated — ${text.length} bytes total)*` : text;
    }
    evidence.push(`http_fetch:status:${status}`);
    const lines = [
      `Fetched \`${url}\` — status **${status}**.`,
      "",
    ];
    if (body) {
      lines.push("Response body:");
      lines.push("```");
      lines.push(body);
      lines.push("```");
    } else {
      lines.push(`Content-Type: \`${contentType || "unknown"}\` — binary or empty body, not shown.`);
      lines.push("");
      lines.push(`You can view this URL directly: [${url}](${url})`);
    }
    return {
      intent: "http_fetch",
      content: lines.join("\n"),
      confidence: 0.95,
      evidence,
      iframeUrl: null,
    };
  } catch (err) {
    // CORS block or network failure. Check target frame policy before choosing
    // between an iframe preview and a direct external link.
    const isCors =
      err instanceof TypeError &&
      (err.message.toLowerCase().includes("cors") ||
        err.message.toLowerCase().includes("network") ||
        err.message.toLowerCase().includes("failed to fetch"));
    evidence.push(`http_fetch:error:${isCors ? "cors" : "network"}`);
    const framePolicy = await detectFramePolicy(url);
    evidence.push(...framePolicy.evidence);
    const fetchFailureLine = `Could not fetch \`${url}\` directly${isCors ? " (CORS restriction)" : " (network error)"}.`;
    if (framePolicy.status !== "allowed") {
      evidence.push(`url_preview:external_link:${url}`);
      return {
        intent: "http_fetch",
        content: directExternalLinkAnswer(
          url,
          framePolicy,
          `${fetchFailureLine}\n\nI suggest opening this in a new tab: [${url}](${url}).`,
        ),
        confidence: 0.75,
        evidence,
        iframeUrl: null,
      };
    }
    evidence.push(`url_preview:iframe:${url}`);
    const lines = [
      fetchFailureLine,
      "",
      "I checked the page's frame policy and can show it in the embedded frame below.",
    ];
    return {
      intent: "http_fetch",
      content: lines.join("\n"),
      confidence: 0.8,
      evidence,
      iframeUrl: url,
    };
  }
}

async function tryUrlNavigate(prompt) {
  const normalized = normalizePrompt(prompt);
  const url = extractUrlNavigateUrl(prompt, normalized);
  if (!url) return null;

  const evidence = [`url_navigate:request:${url}`];
  const framePolicy = await detectFramePolicy(url);
  evidence.push(...framePolicy.evidence);
  if (framePolicy.status !== "allowed") {
    evidence.push(`url_preview:external_link:${url}`);
    return {
      intent: "url_navigate",
      content: directExternalLinkAnswer(url, framePolicy),
      confidence: 0.95,
      evidence,
      iframeUrl: null,
    };
  }

  evidence.push(`url_preview:iframe:${url}`);
  const lines = [
    "I checked the page's frame policy and can show it here.",
    "",
    `Direct link: [${url}](${url}).`,
  ];

  return {
    intent: "url_navigate",
    content: lines.join("\n"),
    confidence: 0.95,
    evidence,
    iframeUrl: url,
  };
}

// Reciprocal Rank Fusion constant — Cormack et al. 2009 use k = 60 and we
// match that so combined ranks stay comparable across the CLI, server, and
// browser surfaces (issue #133).
//
// The authoritative value lives in `web_search_core::WEB_SEARCH_RRF_K` and is
// fetched from the WASM worker once it boots; the JS constants below are
// pre-WASM fallbacks used during init() and on browsers where the worker
// could not instantiate. The Rust→WASM port is the source of truth (R194).
const WEB_SEARCH_RRF_K_FALLBACK = 60;
const WEB_SEARCH_CONCURRENCY_FALLBACK = 5;
const WEB_SEARCH_PROVIDER_LIMIT_FALLBACK = 10;

const WEB_SEARCH_TEXT_ENCODER = new TextEncoder();
const WEB_SEARCH_TEXT_DECODER = new TextDecoder();

function webSearchRrfK() {
  if (wasm && typeof wasm.web_search_rrf_k === "function") {
    return wasm.web_search_rrf_k() >>> 0;
  }
  return WEB_SEARCH_RRF_K_FALLBACK;
}

function webSearchConcurrency() {
  if (wasm && typeof wasm.web_search_concurrency_per_category === "function") {
    return wasm.web_search_concurrency_per_category() >>> 0;
  }
  return WEB_SEARCH_CONCURRENCY_FALLBACK;
}

function webSearchProviderLimit() {
  if (wasm && typeof wasm.web_search_provider_limit === "function") {
    return wasm.web_search_provider_limit() >>> 0;
  }
  return WEB_SEARCH_PROVIDER_LIMIT_FALLBACK;
}

function wasmWriteInput(text) {
  if (!wasm || typeof wasm.input_ptr !== "function") return -1;
  const bytes = WEB_SEARCH_TEXT_ENCODER.encode(text);
  const capacity =
    typeof wasm.input_capacity === "function" ? wasm.input_capacity() : bytes.length;
  if (bytes.length > capacity) return -1;
  const view = new Uint8Array(wasm.memory.buffer, wasm.input_ptr(), bytes.length);
  view.set(bytes);
  return bytes.length;
}

function wasmReadOutput(length) {
  if (!wasm || typeof wasm.output_ptr !== "function" || length <= 0) return "";
  const view = new Uint8Array(wasm.memory.buffer, wasm.output_ptr(), length);
  return WEB_SEARCH_TEXT_DECODER.decode(view);
}

// Engine-core bridges (R194 follow-up). Each function returns a value when
// the WASM core is available, or `null` so the caller can fall back to the
// pure-JS branch. Keeping a JS fallback covers offline mode and old browsers
// where `WebAssembly.instantiate` is unavailable, but the canonical answer
// always comes from Rust when the worker booted successfully.
function wasmNormalizePrompt(text) {
  if (!wasm || typeof wasm.engine_normalize_prompt !== "function") return null;
  const length = wasmWriteInput(String(text || ""));
  if (length < 0) return null;
  const written = wasm.engine_normalize_prompt(length) >>> 0;
  return wasmReadOutput(written);
}

function wasmDetectLanguage(text) {
  if (!wasm || typeof wasm.engine_detect_language !== "function") return null;
  const length = wasmWriteInput(String(text || ""));
  if (length < 0) return null;
  const written = wasm.engine_detect_language(length) >>> 0;
  const slug = wasmReadOutput(written);
  return slug || null;
}

// Returns `{ ok: true, value }` on success, `{ ok: false, error }` on parse
// or runtime failure (division by zero, overflow). `null` means the WASM core
// is unavailable — the caller should fall back to the JS parser.
function wasmEvaluateArithmetic(expression) {
  if (!wasm || typeof wasm.engine_evaluate_arithmetic !== "function") return null;
  const length = wasmWriteInput(String(expression || ""));
  if (length < 0) return null;
  const written = wasm.engine_evaluate_arithmetic(length) >>> 0;
  if (written === 0) return null;
  const text = wasmReadOutput(written);
  if (text.startsWith("ERR:")) {
    return { ok: false, error: text.slice(4) };
  }
  return { ok: true, value: text };
}

function wasmStableId(prefix, value) {
  if (!wasm || typeof wasm.engine_stable_id !== "function") return null;
  const payload = `${String(prefix || "")}\n${String(value || "")}`;
  const length = wasmWriteInput(payload);
  if (length < 0) return null;
  const written = wasm.engine_stable_id(length) >>> 0;
  return wasmReadOutput(written) || null;
}

function wasmSelectUnknownOpener(prompt, language) {
  if (!wasm || typeof wasm.engine_select_unknown_opener !== "function") return null;
  const payload = `${String(language || "")}\n${String(prompt || "")}`;
  const length = wasmWriteInput(payload);
  if (length < 0) return null;
  const written = wasm.engine_select_unknown_opener(length) >>> 0;
  return wasmReadOutput(written) || null;
}

function serializeIntentRouteForWasm(normalized, rawPrompt, route) {
  const lines = [String(normalized || ""), String(rawPrompt || "")];
  const append = (kind, value) => {
    const text = String(value || "");
    if (text && !/[\t\r\n]/.test(text)) lines.push(`${kind}\t${text}`);
  };
  for (const value of route.keywords || []) append("K", value);
  for (const value of route.phrases || []) append("P", value);
  for (const value of route.tokens || []) append("T", value);
  for (const combo of route.combos || []) {
    if (!Array.isArray(combo) || combo.length === 0) continue;
    const fields = combo
      .map((value) => String(value || ""))
      .filter((value) => value && !/[\t\r\n]/.test(value));
    if (fields.length > 0) lines.push(`C\t${fields.join("\t")}`);
  }
  return lines.join("\n");
}

function wasmMatchIntentRoute(normalized, rawPrompt, route) {
  if (!wasm || typeof wasm.engine_match_intent_route !== "function") return null;
  const length = wasmWriteInput(
    serializeIntentRouteForWasm(normalized, rawPrompt, route),
  );
  if (length < 0) return null;
  return (wasm.engine_match_intent_route(length) >>> 0) === 1;
}

// Delegates to `web_search_request_evidence` when the WASM core is loaded;
// otherwise returns null so the caller can fall back to the JS list. The
// Rust side owns the canonical evidence shape (issue #133 R194).
function wasmWebSearchRequestEvidence(query, language) {
  if (!wasm || typeof wasm.web_search_request_evidence !== "function") return null;
  const payload = `${String(query || "")}\n${String(language || "")}`;
  const length = wasmWriteInput(payload);
  if (length < 0) return null;
  const written = wasm.web_search_request_evidence(length) >>> 0;
  if (written === 0) return null;
  const text = wasmReadOutput(written);
  return text ? text.split("\n") : null;
}

// Delegates to `web_search_fuse`. Returns the fused entries array or null when
// WASM is unavailable / the payload exceeds the static INPUT buffer.
function wasmReciprocalRankFusion(perProviderResults) {
  if (!wasm || typeof wasm.web_search_fuse !== "function") return null;
  const rows = [];
  for (const provider of perProviderResults) {
    const id = String(provider.id || "");
    const list = Array.isArray(provider.results) ? provider.results : [];
    list.forEach((item, index) => {
      if (!item || !item.url) return;
      const rank = index + 1;
      const title = String(item.title || item.url).replace(/[\t\n]/g, " ");
      const excerpt = String(item.excerpt || "").replace(/[\t\n]/g, " ");
      const url = String(item.url).replace(/[\t\n]/g, " ");
      rows.push(`${id}\t${rank}\t${url}\t${title}\t${excerpt}`);
    });
  }
  if (rows.length === 0) return [];
  const length = wasmWriteInput(rows.join("\n"));
  if (length < 0) return null;
  const written = wasm.web_search_fuse(length) >>> 0;
  if (written === 0) return [];
  const text = wasmReadOutput(written);
  if (!text) return [];
  return parseFusedOutput(text);
}

// Parse the `serialize_rrf_output` format: one entry per line, fields
// separated by tabs, providers serialised as `id#rank` joined by `;`. The
// shape matches `web_search_core::serialize_rrf_output`.
function parseFusedOutput(text) {
  return text
    .split("\n")
    .filter((line) => line.length > 0)
    .map((line) => {
      const fields = line.split("\t");
      const url = fields[0] || "";
      const title = fields[1] || url;
      const excerpt = fields[2] || "";
      const score = Number.parseFloat(fields[3] || "0") || 0;
      const providerSpecs = (fields[4] || "")
        .split("+")
        .filter((part) => part.length > 0)
        .map((part) => {
          const hash = part.lastIndexOf("#");
          if (hash < 0) return { id: part, rank: 0 };
          return {
            id: part.slice(0, hash),
            rank: Number.parseInt(part.slice(hash + 1), 10) || 0,
          };
        });
      return { url, title, excerpt, score, providers: providerSpecs };
    });
}

// Session-scoped CORS disable list. When a provider fetch throws a CORS or
// network error we record the timestamp so the planner skips it for the rest
// of the session and records the decision in memory. Issue #180: we also
// pre-probe every provider once per session so the first user query does not
// pay for failed sockets — the result is cached in `WEB_SEARCH_AVAILABLE`
// alongside the disable list.
const WEB_SEARCH_DISABLED = new Map();
const WEB_SEARCH_AVAILABLE = new Map();
const WEB_SEARCH_DIAGNOSTICS = [];
let WEB_SEARCH_PROBE_PROMISE = null;

function webSearchDisable(providerId, reason) {
  if (!WEB_SEARCH_DISABLED.has(providerId)) {
    WEB_SEARCH_DISABLED.set(providerId, { reason, at: Date.now() });
  }
}

function webSearchIsDisabled(providerId) {
  return WEB_SEARCH_DISABLED.has(providerId);
}

function webSearchMarkAvailable(providerId, info) {
  WEB_SEARCH_AVAILABLE.set(providerId, Object.assign({ at: Date.now() }, info || {}));
  WEB_SEARCH_DISABLED.delete(providerId);
}

// Issue #180: record a single HTTP exchange so the diagnostics panel can
// surface the raw request/response/conversion trace. We keep a small ring
// buffer in RAM so very long sessions do not bloat memory.
const WEB_SEARCH_DIAG_LIMIT = 64;
function recordWebSearchDiagnostic(entry) {
  if (!entry || typeof entry !== "object") return;
  WEB_SEARCH_DIAGNOSTICS.push(entry);
  while (WEB_SEARCH_DIAGNOSTICS.length > WEB_SEARCH_DIAG_LIMIT) {
    WEB_SEARCH_DIAGNOSTICS.shift();
  }
}

function consumeWebSearchDiagnostics() {
  if (WEB_SEARCH_DIAGNOSTICS.length === 0) return [];
  const snapshot = WEB_SEARCH_DIAGNOSTICS.slice();
  WEB_SEARCH_DIAGNOSTICS.length = 0;
  return snapshot;
}

async function fetchProviderJson(providerId, url, options) {
  if (typeof fetch !== "function") {
    webSearchDisable(providerId, "no_fetch");
    recordWebSearchDiagnostic({
      providerId,
      url,
      method: (options && options.method) || "GET",
      requestHeaders: (options && options.headers) || null,
      ok: false,
      error: "fetch unavailable",
    });
    return { ok: false, error: "fetch unavailable", finalUrl: url };
  }
  const startedAt = Date.now();
  try {
    const response = await fetch(url, options || { mode: "cors" });
    const status = response ? response.status : 0;
    const statusText = response ? response.statusText : "";
    if (!response || !response.ok) {
      recordWebSearchDiagnostic({
        providerId,
        url,
        method: (options && options.method) || "GET",
        requestHeaders: (options && options.headers) || null,
        ok: false,
        status,
        statusText,
        elapsedMs: Date.now() - startedAt,
      });
      return { ok: false, status, statusText, finalUrl: url };
    }
    const text = await response.text();
    let data = null;
    try {
      data = text ? JSON.parse(text) : null;
    } catch (parseError) {
      const message = parseError instanceof Error ? parseError.message : String(parseError);
      recordWebSearchDiagnostic({
        providerId,
        url,
        method: (options && options.method) || "GET",
        requestHeaders: (options && options.headers) || null,
        ok: false,
        status,
        statusText,
        elapsedMs: Date.now() - startedAt,
        responseSnippet: text.slice(0, 1024),
        error: `parse_error: ${message}`,
      });
      return { ok: false, error: `parse_error: ${message}`, finalUrl: url };
    }
    webSearchMarkAvailable(providerId, { status });
    recordWebSearchDiagnostic({
      providerId,
      url,
      method: (options && options.method) || "GET",
      requestHeaders: (options && options.headers) || null,
      ok: true,
      status,
      statusText,
      elapsedMs: Date.now() - startedAt,
      responseSnippet: text.length > 4096 ? `${text.slice(0, 4096)}…` : text,
      responseBytes: text.length,
    });
    return { ok: true, status, data, finalUrl: url };
  } catch (error) {
    const message = error instanceof Error ? error.message : String(error);
    const isCors =
      message.toLowerCase().includes("cors") ||
      message.toLowerCase().includes("network") ||
      message.toLowerCase().includes("failed to fetch");
    webSearchDisable(providerId, isCors ? "cors" : "network");
    recordWebSearchDiagnostic({
      providerId,
      url,
      method: (options && options.method) || "GET",
      requestHeaders: (options && options.headers) || null,
      ok: false,
      elapsedMs: Date.now() - startedAt,
      error: message,
      cors: isCors,
    });
    return { ok: false, error: message, finalUrl: url, cors: isCors };
  }
}

// Issue #180: shared text-shaping helpers used by every web-search provider so
// the rendered bullet looks consistent regardless of which provider produced
// the entry. `extractDomain` returns the bare host (without `www.`),
// `extractQuoteAroundQuery` walks the response body and returns a short
// Google-style snippet that contains the original query word when possible,
// and `escapeRegExp` is the standard helper used by the snippet picker.
function escapeRegExp(value) {
  return String(value || "").replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
}

function extractDomain(url) {
  const raw = String(url || "").trim();
  if (!raw) return "";
  try {
    const u = new URL(raw);
    return u.hostname.replace(/^www\./i, "");
  } catch (_error) {
    const match = raw.match(/^[a-z][a-z0-9+.\-]*:\/\/([^\/?#]+)/i);
    if (match) return match[1].replace(/^www\./i, "");
    return "";
  }
}

function extractQuoteAroundQuery(text, query, maxChars) {
  const max = typeof maxChars === "number" && maxChars > 0 ? Math.floor(maxChars) : 200;
  const raw = String(text || "").replace(/\s+/g, " ").trim();
  if (!raw) return "";
  if (raw.length <= max) return raw;
  const q = String(query || "").trim();
  if (q) {
    const candidates = [q, ...q.split(/\s+/)].filter((value, index, array) =>
      value && array.indexOf(value) === index,
    );
    for (const candidate of candidates) {
      if (!candidate || candidate.length < 2) continue;
      const re = new RegExp(escapeRegExp(candidate), "i");
      const match = raw.match(re);
      if (match && typeof match.index === "number") {
        const half = Math.max(20, Math.floor((max - candidate.length) / 2));
        let start = Math.max(0, match.index - half);
        let end = Math.min(raw.length, start + max);
        if (start > 0) {
          const space = raw.lastIndexOf(" ", start);
          if (space > 0 && match.index - space <= half + 20) start = space + 1;
        }
        if (end < raw.length) {
          const space = raw.indexOf(" ", end);
          if (space > 0 && space - start <= max + 40) end = space;
        }
        let snippet = raw.slice(start, end).trim();
        if (start > 0) snippet = "… " + snippet;
        if (end < raw.length) snippet = snippet + " …";
        return snippet;
      }
    }
  }
  let cut = raw.slice(0, max);
  const lastPeriod = Math.max(
    cut.lastIndexOf(". "),
    cut.lastIndexOf("! "),
    cut.lastIndexOf("? "),
    cut.lastIndexOf("。"),
  );
  if (lastPeriod > max * 0.5) return cut.slice(0, lastPeriod + 1).trim();
  const lastSpace = cut.lastIndexOf(" ");
  if (lastSpace > max * 0.5) cut = cut.slice(0, lastSpace);
  return cut.trim() + " …";
}

const PROVIDER_DISPLAY_LABELS = {
  duckduckgo: "DuckDuckGo",
  "internet-archive": "Internet Archive",
  wikipedia: "Википедия",
  wikidata: "Викидата",
  wiktionary: "Викисловарь",
  wikinews: "Викиновости",
};

const PROVIDER_DISPLAY_LABELS_BY_LANG = {
  en: {
    duckduckgo: "DuckDuckGo",
    "internet-archive": "Internet Archive",
    wikipedia: "Wikipedia",
    wikidata: "Wikidata",
    wiktionary: "Wiktionary",
    wikinews: "Wikinews",
  },
  ru: {
    duckduckgo: "DuckDuckGo",
    "internet-archive": "Архив Интернета",
    wikipedia: "Википедия",
    wikidata: "Викидата",
    wiktionary: "Викисловарь",
    wikinews: "Викиновости",
  },
  zh: {
    duckduckgo: "DuckDuckGo",
    "internet-archive": "互联网档案馆",
    wikipedia: "维基百科",
    wikidata: "维基数据",
    wiktionary: "维基词典",
    wikinews: "维基新闻",
  },
  hi: {
    duckduckgo: "DuckDuckGo",
    "internet-archive": "इंटरनेट आर्काइव",
    wikipedia: "विकिपीडिया",
    wikidata: "विकिडेटा",
    wiktionary: "विक्षनरी",
    wikinews: "Wikinews",
  },
};

function providerDisplayLabel(providerId, language) {
  const code = String(language || "").toLowerCase().slice(0, 2);
  const table = PROVIDER_DISPLAY_LABELS_BY_LANG[code] || PROVIDER_DISPLAY_LABELS_BY_LANG.en;
  return table[providerId] || PROVIDER_DISPLAY_LABELS[providerId] || providerId;
}

async function searchDuckDuckGo(query, language, limit) {
  // DuckDuckGo Instant Answer — CORS-readable, no key. Returns the abstract
  // and a flat list of related-topic links. We treat the abstract link plus
  // the related topics as the ranked result list (issue #133).
  //
  // Issue #153: the previous signature was (query, limit) but the dispatcher
  // calls every provider as (query, language, providerLimit). That meant
  // `limit` was set to a 2-letter language code like "en", and
  // `results.slice(0, "en")` silently returned an empty array, so DuckDuckGo
  // contributed nothing to the fused ranking.
  const cap = typeof limit === "number" && Number.isFinite(limit) && limit > 0
    ? Math.floor(limit)
    : 5;
  const params = ["q=" + encodeURIComponent(query), "format=json", "no_redirect=1", "no_html=1"];
  if (language && /^[a-z]{2,3}$/i.test(language) && language !== "en") {
    // DuckDuckGo accepts a `kl` region hint (lowercase locale). We do not
    // require a region/country code so a bare language is treated as the
    // canonical locale for that language; failing that, DDG falls back to
    // English content gracefully.
    params.push("kl=" + encodeURIComponent(`${language}-${language}`));
  }
  const url = "https://api.duckduckgo.com/?" + params.join("&");
  const outcome = await fetchProviderJson("duckduckgo", url);
  if (!outcome.ok || !outcome.data) {
    return { ok: false, results: [], finalUrl: outcome.finalUrl, error: outcome.error };
  }
  const data = outcome.data;
  const results = [];
  if (data.AbstractURL && data.AbstractText) {
    results.push({
      title: data.Heading || query,
      url: data.AbstractURL,
      excerpt: stripHtml(data.AbstractText),
    });
  }
  const topics = Array.isArray(data.RelatedTopics) ? data.RelatedTopics : [];
  for (const topic of topics) {
    if (!topic) continue;
    if (topic.FirstURL && topic.Text) {
      results.push({
        title: topic.Text.split(" - ")[0] || topic.Text,
        url: topic.FirstURL,
        excerpt: stripHtml(topic.Text),
      });
    } else if (Array.isArray(topic.Topics)) {
      for (const inner of topic.Topics) {
        if (inner && inner.FirstURL && inner.Text) {
          results.push({
            title: inner.Text.split(" - ")[0] || inner.Text,
            url: inner.FirstURL,
            excerpt: stripHtml(inner.Text),
          });
        }
      }
    }
    if (results.length >= cap) break;
  }
  return { ok: true, results: results.slice(0, cap), finalUrl: outcome.finalUrl };
}

async function searchWikipediaWebProvider(query, language, limit) {
  // Reuse the existing helper but adapt the shape to {title, url, excerpt}.
  const result = await searchWikipediaPages(query, language, limit);
  if (!result || !Array.isArray(result.pages)) {
    return { ok: false, results: [], finalUrl: "", language: language || "en" };
  }
  // R194/issue-153: thread the Wikipedia page key through so cross-source
  // deduplication can match `Apple_(disambiguation)` against the Wikidata
  // sitelink `enwiki: Apple_(disambiguation)` even if the URLs disagree on
  // percent-encoding.
  const results = result.pages.slice(0, limit).map((page) => ({
    title: page.title,
    url: page.url,
    excerpt: page.excerpt,
    wikipediaKey: page.key || page.title || "",
    wikipediaLanguage: result.language,
    virtualId: `WP:${page.key || page.title || query}`,
    sourceKind: "wikipedia",
  }));
  return {
    ok: true,
    results,
    language: result.language,
    finalUrl: `https://${result.language}.wikipedia.org/w/rest.php/v1/search/page?q=${encodeURIComponent(query)}`,
  };
}

async function searchWikidataEntities(query, language, limit) {
  const lang = language && /^[a-z]{2,3}$/i.test(language) ? language : "en";
  // R194/issue-153: request `sitelinks/urls` so each entity carries its
  // Wikipedia URL inline. We use that to merge entries returned by the
  // Wikipedia provider with the same entity (otherwise the user sees the
  // same fact as two bullets — "Apple" via Wikidata Q89 and "Apple" via
  // enwiki).
  const url =
    "https://www.wikidata.org/w/api.php?action=wbsearchentities&search=" +
    encodeURIComponent(query) +
    "&language=" +
    encodeURIComponent(lang) +
    "&format=json&origin=*&props=sitelinks/urls&limit=" +
    encodeURIComponent(limit);
  const outcome = await fetchProviderJson("wikidata", url);
  if (!outcome.ok || !outcome.data || !Array.isArray(outcome.data.search)) {
    return { ok: false, results: [], finalUrl: outcome.finalUrl, error: outcome.error };
  }
  const results = outcome.data.search.slice(0, limit).map((entry) => {
    const sitelinks = entry.sitelinks && typeof entry.sitelinks === "object"
      ? entry.sitelinks
      : {};
    const wikipediaLang = sitelinks[`${lang}wiki`] ? lang : "en";
    const wikipediaEntry =
      sitelinks[`${wikipediaLang}wiki`] || sitelinks.enwiki || null;
    const wikipediaUrl = wikipediaEntry && wikipediaEntry.url
      ? wikipediaEntry.url
      : "";
    const wikipediaKey = wikipediaEntry && wikipediaEntry.title
      ? String(wikipediaEntry.title).replace(/\s+/g, "_")
      : "";
    return {
      title: entry.label || entry.id || query,
      url: entry.concepturi || `https://www.wikidata.org/wiki/${entry.id}`,
      excerpt: stripHtml(entry.description || ""),
      qid: entry.id || "",
      virtualId: entry.id || "",
      sourceKind: "wikidata",
      wikipediaUrl,
      wikipediaKey,
      wikipediaLanguage: wikipediaEntry ? wikipediaLang : "",
    };
  });
  return { ok: true, results, finalUrl: outcome.finalUrl };
}

async function searchInternetArchive(query, language, limit) {
  // Issue #153: archive.org publishes a CORS-enabled `advancedsearch.php`
  // endpoint that returns ranked results across the entire collection (web
  // captures, books, audio, software, ...). This complements the DuckDuckGo
  // Instant Answer (which mostly returns a single Wikipedia abstract) and
  // gives the agent another general-purpose web search fallback to draw on
  // when the structured providers (Wikidata/Wikipedia) miss the query.
  const cap = typeof limit === "number" && Number.isFinite(limit) && limit > 0
    ? Math.floor(limit)
    : 5;
  const params = [
    "q=" + encodeURIComponent(query),
    "fl%5B%5D=identifier",
    "fl%5B%5D=title",
    "fl%5B%5D=description",
    "fl%5B%5D=creator",
    "sort%5B%5D=" + encodeURIComponent("downloads desc"),
    "rows=" + encodeURIComponent(cap),
    "page=1",
    "output=json",
  ];
  const url = "https://archive.org/advancedsearch.php?" + params.join("&");
  const outcome = await fetchProviderJson("internet-archive", url);
  if (
    !outcome.ok ||
    !outcome.data ||
    !outcome.data.response ||
    !Array.isArray(outcome.data.response.docs)
  ) {
    return { ok: false, results: [], finalUrl: outcome.finalUrl, error: outcome.error };
  }
  const docs = outcome.data.response.docs;
  const results = docs.slice(0, cap).map((doc) => {
    const identifier = doc.identifier || "";
    const description = Array.isArray(doc.description)
      ? doc.description.join(" • ")
      : (doc.description || "");
    const creator = Array.isArray(doc.creator)
      ? doc.creator.join(", ")
      : (doc.creator || "");
    const excerpt = stripHtml(creator ? `${creator} — ${description}` : description);
    return {
      title: doc.title || identifier || query,
      url: identifier ? `https://archive.org/details/${identifier}` : `https://archive.org/search.php?query=${encodeURIComponent(query)}`,
      excerpt,
      virtualId: `IA:${identifier || query}`,
      sourceKind: "internet-archive",
    };
  });
  return { ok: true, results, finalUrl: outcome.finalUrl };
}

// Issue #180: Wiktionary opensearch is a CORS-readable provider that returns
// short dictionary definitions — exactly the kind of "fragment containing the
// original request" the rendering template needs. We reuse the same
// `fetchProviderJson` plumbing so the diagnostics panel records the raw call.
async function searchWiktionary(query, language, limit) {
  const cap = typeof limit === "number" && Number.isFinite(limit) && limit > 0
    ? Math.floor(limit)
    : 5;
  const lang = language && /^[a-z]{2,3}$/i.test(language) ? language : "en";
  const ordered = [lang, "en"].filter(
    (value, index, array) => value && array.indexOf(value) === index,
  );
  const collected = [];
  let lastFinalUrl = "";
  let lastError = "";
  for (const candidate of ordered) {
    const base = WIKTIONARY_SEARCH_HOSTS[candidate] || WIKTIONARY_SEARCH_HOSTS.en;
    const url = `${base}?action=opensearch&search=${encodeURIComponent(query)}&limit=${cap}&format=json&origin=*`;
    const outcome = await fetchProviderJson("wiktionary", url);
    lastFinalUrl = outcome.finalUrl || lastFinalUrl;
    if (!outcome.ok || !Array.isArray(outcome.data) || !Array.isArray(outcome.data[1])) {
      if (outcome.error) lastError = outcome.error;
      continue;
    }
    const titles = outcome.data[1];
    const descriptions = Array.isArray(outcome.data[2]) ? outcome.data[2] : [];
    const urls = Array.isArray(outcome.data[3]) ? outcome.data[3] : [];
    for (let index = 0; index < titles.length && collected.length < cap; index += 1) {
      const title = titles[index] || query;
      const description = stripHtml(
        descriptions[index] || wiktionaryFallbackDescription(title, candidate),
      );
      const itemUrl = urls[index] ||
        `https://${candidate}.wiktionary.org/wiki/${encodeURIComponent(title)}`;
      collected.push({
        title,
        url: itemUrl,
        excerpt: description,
        wiktionaryKey: String(title).replace(/\s+/g, "_"),
        wiktionaryLanguage: candidate,
        virtualId: `WT:${candidate}:${String(title).replace(/\s+/g, "_")}`,
        sourceKind: "wiktionary",
      });
    }
    if (collected.length > 0) break;
  }
  if (collected.length === 0) {
    return { ok: false, results: [], finalUrl: lastFinalUrl, error: lastError || "no_results" };
  }
  return { ok: true, results: collected.slice(0, cap), finalUrl: lastFinalUrl };
}

function wikinewsFallbackDescription(title, language) {
  if (language === "ru") {
    return `В Wikinews есть новостная статья «${title}».`;
  }
  if (language === "zh") {
    return `Wikinews 有“${title}”这篇新闻。`;
  }
  if (language === "hi") {
    return `Wikinews में "${title}" के लिए समाचार लेख है।`;
  }
  return `Wikinews has a news article titled "${title}".`;
}

// Issue #400: Wikinews exposes the same CORS-readable MediaWiki opensearch
// endpoint as Wiktionary, and is the source requested for latest-news prompts.
async function searchWikinews(query, language, limit) {
  const cap = typeof limit === "number" && Number.isFinite(limit) && limit > 0
    ? Math.floor(limit)
    : 5;
  const lang = language && /^[a-z]{2,3}$/i.test(language) ? language : "en";
  const ordered = [lang, "en"].filter(
    (value, index, array) => value && array.indexOf(value) === index,
  );
  const collected = [];
  let lastFinalUrl = "";
  let lastError = "";
  for (const candidate of ordered) {
    const base = WIKINEWS_SEARCH_HOSTS[candidate] || WIKINEWS_SEARCH_HOSTS.en;
    const url = `${base}?action=opensearch&search=${encodeURIComponent(query)}&limit=${cap}&format=json&origin=*`;
    const outcome = await fetchProviderJson("wikinews", url);
    lastFinalUrl = outcome.finalUrl || lastFinalUrl;
    if (!outcome.ok || !Array.isArray(outcome.data) || !Array.isArray(outcome.data[1])) {
      if (outcome.error) lastError = outcome.error;
      continue;
    }
    const titles = outcome.data[1];
    const descriptions = Array.isArray(outcome.data[2]) ? outcome.data[2] : [];
    const urls = Array.isArray(outcome.data[3]) ? outcome.data[3] : [];
    for (let index = 0; index < titles.length && collected.length < cap; index += 1) {
      const title = titles[index] || query;
      const description = stripHtml(
        descriptions[index] || wikinewsFallbackDescription(title, candidate),
      );
      const itemUrl = urls[index] ||
        `https://${candidate}.wikinews.org/wiki/${encodeURIComponent(title)}`;
      const key = String(title).replace(/\s+/g, "_");
      collected.push({
        title,
        url: itemUrl,
        excerpt: description,
        wikinewsKey: key,
        wikinewsLanguage: candidate,
        virtualId: `WN:${candidate}:${key}`,
        sourceKind: "wikinews",
      });
    }
    if (collected.length > 0) break;
  }
  if (collected.length === 0) {
    return { ok: false, results: [], finalUrl: lastFinalUrl, error: lastError || "no_results" };
  }
  return { ok: true, results: collected.slice(0, cap), finalUrl: lastFinalUrl };
}

// Issue #180: The priority order requested in the issue is
// DuckDuckGo → Internet Archive → Wikipedia → Wikidata → Wiktionary → Wikinews → rest.
// We also keep the corresponding light-weight probe URL so the per-session
// availability check at the top of `tryWebSearch` can pre-flight every
// provider once instead of failing the first user query.
const WEB_SEARCH_PROVIDERS = [
  {
    id: "duckduckgo",
    label: "DuckDuckGo Instant Answer",
    priority: 1,
    probeUrl: "https://api.duckduckgo.com/?q=ping&format=json&no_redirect=1&no_html=1",
    run: (query, language, limit) => searchDuckDuckGo(query, language, limit),
  },
  {
    id: "internet-archive",
    label: "Internet Archive (archive.org)",
    priority: 2,
    probeUrl:
      "https://archive.org/advancedsearch.php?q=ping&fl%5B%5D=identifier&rows=1&page=1&output=json",
    run: (query, language, limit) =>
      searchInternetArchive(query, language, limit),
  },
  {
    id: "wikipedia",
    label: "Wikipedia REST",
    priority: 3,
    probeUrl: "https://en.wikipedia.org/w/rest.php/v1/search/page?q=ping&limit=1",
    run: (query, language, limit) =>
      searchWikipediaWebProvider(query, language, limit),
  },
  {
    id: "wikidata",
    label: "Wikidata entities",
    priority: 4,
    probeUrl:
      "https://www.wikidata.org/w/api.php?action=wbsearchentities&search=ping&language=en&format=json&origin=*&limit=1",
    run: (query, language, limit) =>
      searchWikidataEntities(query, language, limit),
  },
  {
    id: "wiktionary",
    label: "Wiktionary opensearch",
    priority: 5,
    probeUrl:
      "https://en.wiktionary.org/w/api.php?action=opensearch&search=ping&limit=1&format=json&origin=*",
    run: (query, language, limit) =>
      searchWiktionary(query, language, limit),
  },
  {
    id: "wikinews",
    label: "Wikinews opensearch",
    priority: 6,
    probeUrl:
      "https://en.wikinews.org/w/api.php?action=opensearch&search=ping&limit=1&format=json&origin=*",
    run: (query, language, limit) =>
      searchWikinews(query, language, limit),
  },
];

const WEB_SEARCH_PROVIDER_PRIORITY = WEB_SEARCH_PROVIDERS.reduce((acc, provider, index) => {
  acc[provider.id] = typeof provider.priority === "number" ? provider.priority : index + 1;
  return acc;
}, Object.create(null));

// Issue #180: pre-probe every provider exactly once per browser session. The
// result lives in `WEB_SEARCH_AVAILABLE` / `WEB_SEARCH_DISABLED` for the rest
// of the worker's lifetime so subsequent queries skip CORS-blocked endpoints
// without re-burning a socket. We return a shared promise so concurrent
// callers cooperate on the same probe batch.
function ensureWebSearchProviderProbes() {
  if (WEB_SEARCH_PROBE_PROMISE) return WEB_SEARCH_PROBE_PROMISE;
  if (typeof fetch !== "function") {
    WEB_SEARCH_PROBE_PROMISE = Promise.resolve([]);
    return WEB_SEARCH_PROBE_PROMISE;
  }
  WEB_SEARCH_PROBE_PROMISE = (async () => {
    const tasks = WEB_SEARCH_PROVIDERS.map((provider) => async () => {
      if (!provider.probeUrl) return null;
      const startedAt = Date.now();
      try {
        const response = await fetch(provider.probeUrl, { mode: "cors" });
        const status = response ? response.status : 0;
        if (response && response.ok) {
          webSearchMarkAvailable(provider.id, { probedAt: startedAt, status });
          recordWebSearchDiagnostic({
            providerId: provider.id,
            url: provider.probeUrl,
            method: "GET",
            ok: true,
            status,
            elapsedMs: Date.now() - startedAt,
            phase: "probe",
          });
          return { providerId: provider.id, ok: true, status };
        }
        recordWebSearchDiagnostic({
          providerId: provider.id,
          url: provider.probeUrl,
          method: "GET",
          ok: false,
          status,
          elapsedMs: Date.now() - startedAt,
          phase: "probe",
        });
        return { providerId: provider.id, ok: false, status };
      } catch (error) {
        const message = error instanceof Error ? error.message : String(error);
        const isCors =
          message.toLowerCase().includes("cors") ||
          message.toLowerCase().includes("network") ||
          message.toLowerCase().includes("failed to fetch");
        webSearchDisable(provider.id, isCors ? "cors" : "network");
        recordWebSearchDiagnostic({
          providerId: provider.id,
          url: provider.probeUrl,
          method: "GET",
          ok: false,
          elapsedMs: Date.now() - startedAt,
          error: message,
          cors: isCors,
          phase: "probe",
        });
        return { providerId: provider.id, ok: false, error: message, cors: isCors };
      }
    });
    return runWithConcurrencyLimit(tasks, webSearchConcurrency());
  })();
  return WEB_SEARCH_PROBE_PROMISE;
}

async function runWithConcurrencyLimit(tasks, limit) {
  // Simple p-limit style runner so we never exceed the browser's per-origin
  // socket budget. Tasks are async functions returning a value; results are
  // returned in the original order.
  const cap = Math.max(1, Math.min(limit, tasks.length));
  const results = new Array(tasks.length);
  let cursor = 0;
  async function worker() {
    while (true) {
      const index = cursor;
      cursor += 1;
      if (index >= tasks.length) return;
      results[index] = await tasks[index]();
    }
  }
  await Promise.all(Array.from({ length: cap }, () => worker()));
  return results;
}
