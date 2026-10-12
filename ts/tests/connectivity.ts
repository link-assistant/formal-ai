const SERVICES = [
  {
    id: "duckduckgo-web",
    name: "DuckDuckGo",
    category: "search",
    pageUrl: "https://duckduckgo.com/?q=formal-ai",
    apiUrl: "https://api.duckduckgo.com/?q=formal-ai&format=json&no_redirect=1&no_html=1",
    apiLabel: "instant answer",
    note: "Default search engine — CORS-readable Instant Answer JSON",
  },
  {
    id: "google-web",
    name: "Google Search",
    category: "search",
    pageUrl: "https://www.google.com/search?q=formal-ai",
    apiUrl: "https://suggestqueries.google.com/complete/search?client=firefox&q=formal-ai",
    apiLabel: "suggest",
    note: "Search page plus suggestion endpoint",
  },
  {
    id: "bing-web",
    name: "Bing Search",
    category: "search",
    pageUrl: "https://www.bing.com/search?q=formal-ai",
    apiUrl: "https://api.bing.com/osjson.aspx?query=formal-ai",
    apiLabel: "suggest",
    note: "Search page plus suggestion endpoint",
  },
  {
    id: "brave-web",
    name: "Brave Search",
    category: "search",
    pageUrl: "https://search.brave.com/search?q=formal-ai",
    apiUrl: "",
    apiLabel: "none",
    note: "No public unauthenticated browser API",
  },
  {
    id: "yahoo-web",
    name: "Yahoo Search",
    category: "search",
    pageUrl: "https://search.yahoo.com/search?p=formal-ai",
    apiUrl: "",
    apiLabel: "none",
    note: "Search page only",
  },
  {
    id: "yandex-web",
    name: "Yandex Search",
    category: "search",
    pageUrl: "https://yandex.com/search/?text=formal-ai",
    apiUrl: "",
    apiLabel: "none",
    note: "Search page only — XML API requires a subscription",
  },
  {
    id: "ecosia-web",
    name: "Ecosia",
    category: "search",
    pageUrl: "https://www.ecosia.org/search?q=formal-ai",
    apiUrl: "",
    apiLabel: "none",
    note: "Search page only",
  },
  {
    id: "mojeek-web",
    name: "Mojeek",
    category: "search",
    pageUrl: "https://www.mojeek.com/search?q=formal-ai",
    apiUrl: "",
    apiLabel: "none",
    note: "Search page only — JSON API requires a key",
  },
  {
    id: "startpage-web",
    name: "Startpage",
    category: "search",
    pageUrl: "https://www.startpage.com/do/search?query=formal-ai",
    apiUrl: "",
    apiLabel: "none",
    note: "Search page only",
  },
  {
    id: "wikipedia-api",
    name: "Wikipedia",
    category: "knowledge",
    pageUrl: "https://en.wikipedia.org/wiki/Formal_language",
    apiUrl: "https://en.wikipedia.org/w/rest.php/v1/search/page?q=formal-ai&limit=3",
    apiLabel: "REST search",
    note: "CORS-readable REST API",
  },
  {
    id: "wikidata-api",
    name: "Wikidata",
    category: "knowledge",
    pageUrl: "https://www.wikidata.org/wiki/Wikidata:Main_Page",
    apiUrl:
      "https://www.wikidata.org/w/api.php?action=wbsearchentities&search=formal-ai&language=en&format=json&origin=*",
    apiLabel: "entity search",
    note: "MediaWiki API with origin=*",
  },
  {
    id: "wiktionary-api",
    name: "Wiktionary",
    category: "knowledge",
    pageUrl: "https://en.wiktionary.org/wiki/formal",
    apiUrl:
      "https://en.wiktionary.org/w/api.php?action=opensearch&search=formal-ai&limit=3&format=json&origin=*",
    apiLabel: "opensearch",
    note: "MediaWiki opensearch with origin=*",
  },
  {
    id: "wikinews-api",
    name: "Wikinews",
    category: "knowledge",
    pageUrl: "https://www.wikinews.org/",
    apiUrl:
      "https://en.wikinews.org/w/api.php?action=opensearch&search=formal-ai&limit=3&format=json&origin=*",
    apiLabel: "opensearch",
    note: "MediaWiki opensearch with origin=*",
  },
  {
    id: "cambridge-dictionary",
    name: "Cambridge Dictionary",
    category: "knowledge",
    pageUrl: "https://dictionary.cambridge.org/us/dictionary/english/digress",
    apiUrl: "",
    apiLabel: "none",
    note: "Dictionary page only — use the proxy mode for HTML capture",
  },
  {
    id: "merriam-webster-dictionary",
    name: "Merriam-Webster",
    category: "knowledge",
    pageUrl: "https://www.merriam-webster.com/dictionary/digress",
    apiUrl: "",
    apiLabel: "none",
    note: "Dictionary page only — use the proxy mode for HTML capture",
  },
  {
    id: "dictionary-com",
    name: "Dictionary.com",
    category: "knowledge",
    pageUrl: "https://www.dictionary.com/browse/digress",
    apiUrl: "",
    apiLabel: "none",
    note: "Dictionary page only — use the proxy mode for HTML capture",
  },
  {
    id: "collins-dictionary",
    name: "Collins English Dictionary",
    category: "knowledge",
    pageUrl: "https://www.collinsdictionary.com/dictionary/english/digress",
    apiUrl: "",
    apiLabel: "none",
    note: "Dictionary page only — use the proxy mode for HTML capture",
  },
  {
    id: "dbpedia-api",
    name: "DBpedia Lookup",
    category: "knowledge",
    pageUrl: "https://lookup.dbpedia.org/",
    apiUrl: "https://lookup.dbpedia.org/api/search?query=formal-ai&format=json",
    apiLabel: "search",
    note: "Public DBpedia lookup",
  },
  {
    id: "openlibrary-api",
    name: "Open Library",
    category: "knowledge",
    pageUrl: "https://openlibrary.org/search?q=formal-ai",
    apiUrl: "https://openlibrary.org/search.json?q=formal-ai&limit=3",
    apiLabel: "search.json",
    note: "Public book search API",
  },
  {
    id: "openalex-api",
    name: "OpenAlex",
    category: "knowledge",
    pageUrl: "https://openalex.org/works?filter=title.search:formal-ai",
    apiUrl: "https://api.openalex.org/works?search=formal-ai&per-page=3",
    apiLabel: "works",
    note: "Public works API",
  },
  {
    id: "crossref-api",
    name: "Crossref",
    category: "knowledge",
    pageUrl: "https://search.crossref.org/?q=formal-ai",
    apiUrl: "https://api.crossref.org/works?query=formal-ai&rows=3",
    apiLabel: "works",
    note: "Public DOI metadata API",
  },
  {
    id: "semantic-scholar-api",
    name: "Semantic Scholar",
    category: "knowledge",
    pageUrl: "https://www.semanticscholar.org/search?q=formal-ai",
    apiUrl:
      "https://api.semanticscholar.org/graph/v1/paper/search?query=formal-ai&limit=3&fields=title,url",
    apiLabel: "paper search",
    note: "Public graph search endpoint",
  },
  {
    id: "arxiv-api",
    name: "arXiv",
    category: "papers",
    pageUrl: "https://arxiv.org/search/?query=formal-ai&start=0",
    apiUrl: "https://export.arxiv.org/api/query?search_query=all:formal-ai&max_results=3",
    apiLabel: "atom export",
    note: "Public Atom XML export — sends Access-Control-Allow-Origin: *",
  },
  {
    id: "europepmc-api",
    name: "Europe PMC",
    category: "papers",
    pageUrl: "https://europepmc.org/search?query=formal-ai",
    apiUrl:
      "https://www.ebi.ac.uk/europepmc/webservices/rest/search?query=formal-ai&format=json&resultType=lite&pageSize=3",
    apiLabel: "rest search",
    note: "Public biomedical paper search",
  },
  {
    id: "doaj-api",
    name: "DOAJ",
    category: "papers",
    pageUrl: "https://doaj.org/search/articles?source=%7B%22query%22%3A%7B%22query_string%22%3A%7B%22query%22%3A%22formal-ai%22%7D%7D%7D",
    apiUrl: "https://doaj.org/api/search/articles/formal-ai?pageSize=3",
    apiLabel: "articles",
    note: "Directory of Open Access Journals",
  },
  {
    id: "github-code",
    name: "GitHub",
    category: "code",
    pageUrl: "https://github.com/search?q=formal-ai&type=repositories",
    apiUrl: "https://api.github.com/search/repositories?q=formal-ai&per_page=3",
    apiLabel: "repositories",
    note: "Public repo search — 10 req/min unauthenticated",
  },
  {
    id: "gitlab-code",
    name: "GitLab",
    category: "code",
    pageUrl: "https://gitlab.com/search?search=formal-ai&scope=projects",
    apiUrl: "https://gitlab.com/api/v4/search?scope=projects&search=formal-ai",
    apiLabel: "projects",
    note: "Public project search",
  },
  {
    id: "codeberg-code",
    name: "Codeberg",
    category: "code",
    pageUrl: "https://codeberg.org/explore/repos?q=formal-ai",
    apiUrl: "https://codeberg.org/api/v1/repos/search?q=formal-ai&limit=3",
    apiLabel: "repos search",
    note: "Forgejo public repo search",
  },
  {
    id: "gitee-code",
    name: "Gitee",
    category: "code",
    pageUrl: "https://search.gitee.com/?q=formal-ai&type=repository",
    apiUrl: "https://gitee.com/api/v5/search/repositories?q=formal-ai&per_page=3",
    apiLabel: "repositories",
    note: "Public repo search (China)",
  },
  {
    id: "bitbucket-code",
    name: "Bitbucket Cloud",
    category: "code",
    pageUrl: "https://bitbucket.org/repo/all?name=formal-ai",
    apiUrl: 'https://api.bitbucket.org/2.0/repositories?q=name~"formal-ai"&pagelen=3',
    apiLabel: "repositories",
    note: "Public repo search with strict q syntax",
  },
  {
    id: "gitflic-code",
    name: "GitFlic",
    category: "code",
    pageUrl: "https://gitflic.ru/project?search=formal-ai",
    apiUrl: "",
    apiLabel: "none",
    note: "HTML-only search (Russia)",
  },
];

// Maximum concurrent fetches per category — modern browsers cap per-origin
// sockets near six, so five keeps a slot free for the rest of the page.
const CATEGORY_CONCURRENCY = 5;

const state = {
  mode: "direct",
  results: {},
  // CORS auto-disable: when a direct browser fetch throws a CORS/network error
  // for a given service+kind we record the timestamp so subsequent runs in the
  // same session skip the call and the UI marks it as disabled. The user can
  // re-enable the row at any time.
  disabled: {},
};

const elements = {
  matrix: document.getElementById("service-matrix"),
  proxySettings: document.querySelector("[data-testid='proxy-settings']"),
  proxyBase: document.getElementById("proxy-base"),
  proxyEndpoint: document.getElementById("proxy-endpoint"),
  summaryTotal: document.querySelector("[data-testid='summary-total']"),
  summaryOk: document.querySelector("[data-testid='summary-ok']"),
  summaryBlocked: document.querySelector("[data-testid='summary-blocked']"),
  summaryIdle: document.querySelector("[data-testid='summary-idle']"),
  overlay: document.querySelector("[data-testid='frame-overlay']"),
  overlayTitle: document.getElementById("frame-overlay-title"),
  overlayFrame: document.querySelector("[data-testid='frame-overlay'] iframe"),
};

function textEl(tagName, className, text) {
  const element = document.createElement(tagName);
  if (className) {
    element.className = className;
  }
  element.textContent = text;
  return element;
}

function button(label, testId, action, serviceId) {
  const element = document.createElement("button");
  element.type = "button";
  element.textContent = label;
  element.dataset.testid = testId;
  element.dataset.action = action;
  if (serviceId) {
    element.dataset.serviceId = serviceId;
  }
  return element;
}

function renderServices() {
  const fragment = document.createDocumentFragment();
  for (const service of SERVICES) {
    const row = document.createElement("article");
    row.className = "service-row";
    row.dataset.serviceId = service.id;
    row.dataset.serviceRow = "true";
    row.dataset.testid = `service-${service.id}`;

    const serviceCell = document.createElement("div");
    serviceCell.className = "service-title";
    const badge = textEl("span", `service-badge ${service.category}`, service.category);
    const title = textEl("strong", "", service.name);
    const note = textEl("span", "meta-line", service.note);
    const pageTarget = textEl("span", "target-line", service.pageUrl);
    serviceCell.append(badge, title, note, pageTarget);

    const pageCell = document.createElement("div");
    pageCell.className = "check-cell";
    pageCell.append(
      statusPill("Idle", "page-status"),
      button("Fetch page", "run-page-fetch", "run-page", service.id),
      finalUrl("page-final-url"),
    );

    const apiCell = document.createElement("div");
    apiCell.className = "check-cell";
    const apiStatus = statusPill(service.apiUrl ? "Idle" : "No API", "api-status");
    const apiButton = button("Fetch API", "run-api-fetch", "run-api", service.id);
    apiButton.disabled = !service.apiUrl;
    apiCell.append(apiStatus, apiButton, textEl("span", "meta-line", service.apiLabel));
    apiCell.append(finalUrl("api-final-url"));

    const frameCell = document.createElement("div");
    frameCell.className = "frame-cell";
    frameCell.append(
      statusPill("Idle", "frame-status"),
      button("Frame", "toggle-frame", "toggle-frame", service.id),
    );

    const resultCell = document.createElement("div");
    resultCell.className = "result-cell";
    const proxyUrl = textEl("span", "final-url", "");
    proxyUrl.dataset.testid = "proxy-final-url";
    const preview = document.createElement("pre");
    preview.className = "result-preview";
    preview.dataset.testid = "result-preview";
    preview.textContent = "Idle";
    resultCell.append(proxyUrl, preview);

    const framePanel = document.createElement("div");
    framePanel.className = "frame-panel";
    framePanel.dataset.testid = "frame-panel";
    framePanel.hidden = true;

    const frameToolbar = document.createElement("div");
    frameToolbar.className = "frame-toolbar";
    frameToolbar.append(
      textEl("span", "target-line", service.pageUrl),
      button("Expand", "expand-frame", "expand-frame", service.id),
    );

    const iframe = document.createElement("iframe");
    iframe.className = "inline-frame";
    iframe.title = `${service.name} iframe diagnostics`;
    iframe.loading = "lazy";
    iframe.sandbox =
      "allow-forms allow-popups allow-popups-to-escape-sandbox allow-same-origin allow-scripts";
    iframe.addEventListener("load", () => {
      if (iframe.getAttribute("src")) {
        updateStatus(service.id, "frame", "Load event", "ok");
      }
    });
    framePanel.append(frameToolbar, iframe);

    row.append(serviceCell, pageCell, apiCell, frameCell, resultCell, framePanel);
    fragment.append(row);
  }

  elements.matrix.append(fragment);
  updateSummary();
}

function statusPill(text, testId) {
  const element = textEl("span", "status-pill", text);
  element.dataset.testid = testId;
  return element;
}

function finalUrl(testId) {
  const element = textEl("span", "final-url", "");
  element.dataset.testid = testId;
  return element;
}

function serviceById(serviceId) {
  return SERVICES.find((service) => service.id === serviceId);
}

function rowFor(serviceId) {
  return document.querySelector(`[data-service-id="${serviceId}"]`);
}

function updateStatus(serviceId, kind, label, tone) {
  const row = rowFor(serviceId);
  if (!row) {
    return;
  }
  const pill = row.querySelector(`[data-testid="${kind}-status"]`);
  if (!pill) {
    return;
  }
  pill.textContent = label;
  pill.className = "status-pill";
  if (tone) {
    pill.classList.add(tone);
  }
}

function updatePreview(serviceId, text) {
  const row = rowFor(serviceId);
  const preview = row && row.querySelector("[data-testid='result-preview']");
  if (preview) {
    preview.textContent = text || "Empty response";
  }
}

function updateFinalUrl(serviceId, kind, finalUrlValue) {
  const row = rowFor(serviceId);
  if (!row) {
    return;
  }
  const finalUrlElement = row.querySelector(`[data-testid="${kind}-final-url"]`);
  if (finalUrlElement) {
    finalUrlElement.textContent = finalUrlValue;
  }
  const proxyUrlElement = row.querySelector("[data-testid='proxy-final-url']");
  if (proxyUrlElement) {
    proxyUrlElement.textContent = state.mode === "proxy" ? finalUrlValue : "";
  }
}

function setRowRunning(serviceId, running) {
  const row = rowFor(serviceId);
  if (row) {
    row.classList.toggle("is-running", running);
  }
}

function buildFetchUrl(originalUrl) {
  if (state.mode === "direct") {
    return originalUrl;
  }
  const base = (elements.proxyBase.value || "http://localhost:3000").replace(/\/+$/, "");
  const endpoint = elements.proxyEndpoint.value.startsWith("/")
    ? elements.proxyEndpoint.value
    : `/${elements.proxyEndpoint.value}`;
  return `${base}${endpoint}?url=${encodeURIComponent(originalUrl)}`;
}

function resultKey(serviceId, kind) {
  return `${serviceId}:${kind}`;
}

function statusText(response) {
  const reason = response.statusText ? ` ${response.statusText}` : "";
  return `${response.status}${reason}`.trim();
}

function outcomeTone(outcome) {
  if (outcome.ok) {
    return "ok";
  }
  return outcome.blocked ? "blocked" : "error";
}

function outcomeLabel(outcome) {
  if (outcome.ok) {
    return statusText(outcome);
  }
  if (outcome.status) {
    return statusText(outcome);
  }
  return outcome.blocked ? "Blocked/failed" : "Error";
}

function disableKey(serviceId, kind) {
  return `${serviceId}:${kind}`;
}

function isDisabled(serviceId, kind) {
  return Boolean(state.disabled[disableKey(serviceId, kind)]);
}

function markDisabled(serviceId, kind, reason) {
  state.disabled[disableKey(serviceId, kind)] = {
    reason,
    at: new Date().toISOString(),
  };
}

async function runFetch(serviceId, kind) {
  const service = serviceById(serviceId);
  if (!service) {
    return;
  }
  const originalUrl = kind === "api" ? service.apiUrl : service.pageUrl;
  if (!originalUrl) {
    updateStatus(serviceId, kind, "No API", "blocked");
    return;
  }
  if (isDisabled(serviceId, kind)) {
    updateStatus(serviceId, kind, "Blocked (CORS, disabled)", "blocked");
    updatePreview(
      serviceId,
      `Skipped — disabled for this session after a prior CORS/network failure.`,
    );
    return;
  }

  const finalUrlValue = buildFetchUrl(originalUrl);
  updateStatus(serviceId, kind, "Running", "");
  updateFinalUrl(serviceId, kind, finalUrlValue);
  updatePreview(serviceId, `Fetching ${finalUrlValue}`);
  setRowRunning(serviceId, true);

  const startedAt = performance.now();
  try {
    const response = await fetch(finalUrlValue, {
      method: "GET",
      mode: "cors",
      cache: "no-store",
    });
    const body = await response.text();
    const outcome = {
      serviceId,
      kind,
      mode: state.mode,
      originalUrl,
      finalUrl: finalUrlValue,
      ok: response.ok,
      status: response.status,
      statusText: response.statusText,
      contentType: response.headers.get("content-type") || "",
      elapsedMs: Math.round(performance.now() - startedAt),
      completedAt: new Date().toISOString(),
      preview: truncate(body, 1600),
    };
    state.results[resultKey(serviceId, kind)] = outcome;
    updateStatus(serviceId, kind, outcomeLabel(outcome), outcomeTone(outcome));
    updatePreview(serviceId, outcome.preview);
  } catch (error) {
    const message = error instanceof Error ? error.message : String(error);
    const lower = message.toLowerCase();
    const isCors =
      state.mode === "direct" &&
      (lower.includes("cors") ||
        lower.includes("network") ||
        lower.includes("failed to fetch"));
    const directPrefix =
      state.mode === "direct"
        ? "Direct browser fetch failed"
        : "web-capture proxy fetch failed";
    const outcome = {
      serviceId,
      kind,
      mode: state.mode,
      originalUrl,
      finalUrl: finalUrlValue,
      ok: false,
      blocked: true,
      status: 0,
      statusText: "",
      error: message,
      elapsedMs: Math.round(performance.now() - startedAt),
      completedAt: new Date().toISOString(),
      preview: `${directPrefix}: ${message}`,
      disabledForSession: isCors,
    };
    state.results[resultKey(serviceId, kind)] = outcome;
    if (isCors) {
      markDisabled(serviceId, kind, "cors");
    }
    updateStatus(
      serviceId,
      kind,
      isCors ? "Blocked (CORS, disabled)" : outcomeLabel(outcome),
      "blocked",
    );
    updatePreview(serviceId, outcome.preview);
  } finally {
    setRowRunning(serviceId, false);
    updateSummary();
  }
}

function truncate(text, maxLength) {
  if (text.length <= maxLength) {
    return text;
  }
  return `${text.slice(0, maxLength)}\n... truncated at ${maxLength} characters`;
}

function toggleFrame(serviceId) {
  const service = serviceById(serviceId);
  const row = rowFor(serviceId);
  if (!service || !row) {
    return;
  }
  const panel = row.querySelector("[data-testid='frame-panel']");
  const iframe = panel && panel.querySelector("iframe");
  const trigger = row.querySelector("[data-testid='toggle-frame']");
  if (!panel || !iframe || !trigger) {
    return;
  }

  const shouldOpen = panel.hidden;
  panel.hidden = !shouldOpen;
  trigger.setAttribute("aria-expanded", String(shouldOpen));
  if (shouldOpen) {
    iframe.src = service.pageUrl;
    updateStatus(serviceId, "frame", "Requested", "");
  }
}

function expandFrame(serviceId) {
  const service = serviceById(serviceId);
  if (!service || !elements.overlay || !elements.overlayFrame || !elements.overlayTitle) {
    return;
  }
  elements.overlayTitle.textContent = service.name;
  elements.overlayFrame.src = service.pageUrl;
  elements.overlay.hidden = false;
  document.body.style.overflow = "hidden";
  document.getElementById("close-frame-overlay")?.focus();
}

function closeFrameOverlay() {
  if (!elements.overlay || !elements.overlayFrame) {
    return;
  }
  elements.overlay.hidden = true;
  elements.overlayFrame.removeAttribute("src");
  document.body.style.overflow = "";
}

function setMode(mode) {
  state.mode = mode;
  document.querySelectorAll("[data-mode]").forEach((element) => {
    const active = element.getAttribute("data-mode") === mode;
    element.classList.toggle("is-active", active);
    element.setAttribute("aria-pressed", String(active));
  });
  elements.proxySettings?.classList.toggle("is-visible", mode === "proxy");
}

async function runWithLimit(jobs, limit) {
  const cap = Math.max(1, Math.min(limit, jobs.length));
  let cursor = 0;
  async function worker() {
    while (true) {
      const index = cursor;
      cursor += 1;
      if (index >= jobs.length) return;
      await jobs[index]();
    }
  }
  await Promise.all(Array.from({ length: cap }, () => worker()));
}

async function runAll(kind) {
  const services = SERVICES.filter((service) =>
    kind === "api" ? service.apiUrl : service.pageUrl,
  );
  const grouped = new Map();
  for (const service of services) {
    const category = service.category || "other";
    if (!grouped.has(category)) grouped.set(category, []);
    grouped.get(category).push(service);
  }
  // Each category runs its own concurrency-capped pool so that, e.g., the
  // five search engines fire in parallel without overwhelming the per-origin
  // socket budget. Categories themselves run in parallel because no two
  // share an origin (issue #133).
  await Promise.all(
    Array.from(grouped.values()).map((bucket) => {
      const jobs = bucket.map((service) => () => runFetch(service.id, kind));
      return runWithLimit(jobs, CATEGORY_CONCURRENCY);
    }),
  );
}

function updateSummary() {
  const results = Object.values(state.results);
  const ok = results.filter((result) => result.ok).length;
  const blocked = results.filter((result) => !result.ok).length;
  const possibleChecks =
    SERVICES.length + SERVICES.filter((service) => Boolean(service.apiUrl)).length;
  if (elements.summaryTotal) {
    elements.summaryTotal.textContent = String(results.length);
  }
  if (elements.summaryOk) {
    elements.summaryOk.textContent = String(ok);
  }
  if (elements.summaryBlocked) {
    elements.summaryBlocked.textContent = String(blocked);
  }
  if (elements.summaryIdle) {
    elements.summaryIdle.textContent = String(Math.max(possibleChecks - results.length, 0));
  }
}

function exportLog() {
  const payload = {
    exportedAt: new Date().toISOString(),
    mode: state.mode,
    proxy: {
      base: elements.proxyBase?.value || "",
      endpoint: elements.proxyEndpoint?.value || "",
    },
    services: SERVICES,
    results: Object.values(state.results),
    disabled: Object.assign({}, state.disabled),
    concurrency: {
      perCategory: CATEGORY_CONCURRENCY,
      categories: Array.from(new Set(SERVICES.map((service) => service.category))),
    },
    userAgent: navigator.userAgent,
    assetVersion: window.FORMAL_AI_ASSET_VERSION || "",
  };
  const blob = new Blob([JSON.stringify(payload, null, 2)], {
    type: "application/json",
  });
  const url = URL.createObjectURL(blob);
  const link = document.createElement("a");
  link.href = url;
  link.download = `formal-ai-connectivity-${new Date().toISOString().replace(/[:.]/g, "-")}.json`;
  document.body.append(link);
  link.click();
  link.remove();
  URL.revokeObjectURL(url);
}

function bindEvents() {
  document.addEventListener("click", (event) => {
    const target = event.target instanceof Element ? event.target : null;
    const modeButton = target?.closest("[data-mode]");
    if (modeButton) {
      setMode(modeButton.getAttribute("data-mode") || "direct");
      return;
    }

    const actionButton = target?.closest("[data-action]");
    if (!actionButton) {
      return;
    }
    const serviceId = actionButton.getAttribute("data-service-id") || "";
    const action = actionButton.getAttribute("data-action");
    if (action === "run-page") {
      void runFetch(serviceId, "page");
    } else if (action === "run-api") {
      void runFetch(serviceId, "api");
    } else if (action === "toggle-frame") {
      toggleFrame(serviceId);
    } else if (action === "expand-frame") {
      expandFrame(serviceId);
    }
  });

  document.getElementById("run-pages")?.addEventListener("click", () => {
    void runAll("page");
  });
  document.getElementById("run-apis")?.addEventListener("click", () => {
    void runAll("api");
  });
  document.getElementById("export-log")?.addEventListener("click", exportLog);
  document.getElementById("close-frame-overlay")?.addEventListener("click", closeFrameOverlay);
  document.addEventListener("keydown", (event) => {
    if (event.key === "Escape" && !elements.overlay?.hidden) {
      closeFrameOverlay();
    }
  });
}

renderServices();
bindEvents();
