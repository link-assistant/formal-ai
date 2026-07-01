// Worker module 20 of 21. Loaded by ../formal_ai_worker.js.
function reciprocalRankFusion(perProviderResults, k) {
  // R194: the Rust/WASM core owns the fusion logic so the offline trace and
  // the browser worker agree to the last byte. We try WASM first and only
  // fall back to the JS implementation when the worker booted in
  // `js fallback` mode (e.g. WASM disabled in the browser).
  const wasmFused = wasmReciprocalRankFusion(perProviderResults);
  if (wasmFused !== null) {
    return wasmFused;
  }
  // Cormack, Clarke, Buettcher 2009: score(d) = Σ 1 / (k + rank_i(d)).
  const fused = new Map();
  for (const provider of perProviderResults) {
    const list = Array.isArray(provider.results) ? provider.results : [];
    list.forEach((item, index) => {
      if (!item || !item.url) return;
      const rank = index + 1;
      const score = 1 / (k + rank);
      const existing = fused.get(item.url);
      if (existing) {
        existing.score += score;
        existing.providers.push({ id: provider.id, rank });
        if (!existing.title && item.title) existing.title = item.title;
        if (!existing.excerpt && item.excerpt) existing.excerpt = item.excerpt;
      } else {
        fused.set(item.url, {
          url: item.url,
          title: item.title || item.url,
          excerpt: item.excerpt || "",
          score,
          providers: [{ id: provider.id, rank }],
        });
      }
    });
  }
  return Array.from(fused.values()).sort((a, b) => {
    if (b.score !== a.score) return b.score - a.score;
    return b.providers.length - a.providers.length;
  });
}

// Issue #153/#180: identify "the same entity" returned by different providers
// so the fused list shows one bullet with the other URLs collapsed under
// "Other sources:". A single result can carry several canonical identifiers
// (Wikidata Q-id, Wikipedia page key, Wiktionary headword) — dedupe walks all
// of them and merges into the first existing group it finds. Returning a
// list makes the Wikipedia↔Wikidata merge robust against percent-encoding
// differences in the two providers' URLs.
function canonicalEntityKeys(meta) {
  if (!meta) return [];
  const keys = [];
  if (meta.qid && /^Q\d+$/.test(meta.qid)) keys.push(`Q:${meta.qid}`);
  if (meta.wikipediaKey) {
    const lang = meta.wikipediaLanguage || "en";
    keys.push(`WP:${lang}:${meta.wikipediaKey}`);
  }
  if (meta.wiktionaryKey) {
    const lang = meta.wiktionaryLanguage || "en";
    keys.push(`WT:${lang}:${meta.wiktionaryKey}`);
  }
  if (meta.wikinewsKey) {
    const lang = meta.wikinewsLanguage || "en";
    keys.push(`WN:${lang}:${meta.wikinewsKey}`);
  }
  return keys;
}

// Backwards-compatible shim: prefer the primary key but keep the historical
// single-key signature for callers that still rely on it.
function canonicalEntityKey(meta) {
  const keys = canonicalEntityKeys(meta);
  return keys.length > 0 ? keys[0] : null;
}

function buildItemMetadataIndex(perProvider) {
  // The richer the meta the better — an entry that carries a Wikidata `qid`
  // is preferred over a Wikipedia-only entry for the same URL, because the
  // Q-id is what cross-provider dedupe groups by. Without this preference,
  // the Wikipedia URL would be indexed by the Wikipedia provider's meta
  // (`WP:en:Apple`) and a separate Wikidata entry for the same fact (`Q:Q89`)
  // would never collapse into one bullet.
  const byUrl = new Map();
  const rank = (item) => (item && item.qid ? 2 : 1);
  function record(url, item) {
    if (!url || !item) return;
    const existing = byUrl.get(url);
    if (!existing || rank(item) > rank(existing)) {
      byUrl.set(url, item);
    }
  }
  for (const provider of perProvider) {
    if (!provider || !Array.isArray(provider.results)) continue;
    for (const item of provider.results) {
      if (!item || !item.url) continue;
      record(item.url, item);
      // Wikidata results carry the Wikipedia URL of the same entity inline;
      // index that too so the Wikipedia provider's entry is recognised as
      // a duplicate of the Wikidata one.
      if (item.wikipediaUrl) record(item.wikipediaUrl, item);
    }
  }
  return byUrl;
}

function dedupeFusedEntries(fused, metaByUrl, evidence) {
  const groupsByKey = new Map();
  const allGroups = [];
  const standalone = [];

  function alreadyHasProvider(target, candidate) {
    return target.providers.some(
      (existing) => existing.id === candidate.id && existing.rank === candidate.rank,
    );
  }

  fused.forEach((entry, index) => {
    const meta = metaByUrl.get(entry.url) || null;
    const keys = canonicalEntityKeys(meta);
    const enriched = Object.assign({}, entry, {
      qid: (meta && meta.qid) || "",
      wikipediaKey: (meta && meta.wikipediaKey) || "",
      wikipediaLanguage: (meta && meta.wikipediaLanguage) || "",
      wiktionaryKey: (meta && meta.wiktionaryKey) || "",
      wiktionaryLanguage: (meta && meta.wiktionaryLanguage) || "",
      wikinewsKey: (meta && meta.wikinewsKey) || "",
      wikinewsLanguage: (meta && meta.wikinewsLanguage) || "",
      sourceKind: (meta && meta.sourceKind) || "",
      virtualId:
        (meta && meta.virtualId) ||
        (meta && meta.qid) ||
        (meta && meta.wikipediaKey ? `WP:${meta.wikipediaKey}` : ""),
      alternateUrls: [],
      keys: keys.slice(),
      originalRank: index,
    });

    if (keys.length === 0) {
      standalone.push(enriched);
      return;
    }
    let head = null;
    for (const key of keys) {
      if (groupsByKey.has(key)) {
        head = groupsByKey.get(key);
        break;
      }
    }
    if (!head) {
      allGroups.push(enriched);
      for (const key of keys) {
        if (!groupsByKey.has(key)) groupsByKey.set(key, enriched);
      }
      return;
    }
    // Found an existing group — absorb this entry into it.
    head.score += enriched.score;
    head.alternateUrls.push({
      url: enriched.url,
      title: enriched.title,
      providers: enriched.providers,
      sourceKind: enriched.sourceKind,
    });
    for (const p of enriched.providers) {
      if (!alreadyHasProvider(head, p)) head.providers.push(p);
    }
    // Register the absorbed entry's keys against the head group too so a third
    // provider matching either canonical id still merges in.
    for (const key of keys) {
      if (!groupsByKey.has(key)) groupsByKey.set(key, head);
    }
    // Prefer the richest virtualId once we know more identifiers.
    if (!head.virtualId && enriched.virtualId) head.virtualId = enriched.virtualId;
    if (!head.qid && enriched.qid) head.qid = enriched.qid;
    if (!head.wikipediaKey && enriched.wikipediaKey) {
      head.wikipediaKey = enriched.wikipediaKey;
      head.wikipediaLanguage = enriched.wikipediaLanguage;
    }
    if (!head.wiktionaryKey && enriched.wiktionaryKey) {
      head.wiktionaryKey = enriched.wiktionaryKey;
      head.wiktionaryLanguage = enriched.wiktionaryLanguage;
    }
    if (!head.wikinewsKey && enriched.wikinewsKey) {
      head.wikinewsKey = enriched.wikinewsKey;
      head.wikinewsLanguage = enriched.wikinewsLanguage;
    }
    if (Array.isArray(evidence)) {
      evidence.push(`web_search:dedupe:${keys[0]}:absorbed:${enriched.url}`);
    }
  });
  const merged = [...allGroups, ...standalone];
  merged.sort((a, b) => {
    if (b.score !== a.score) return b.score - a.score;
    if (b.providers.length !== a.providers.length) {
      return b.providers.length - a.providers.length;
    }
    // Issue #180: stable order by provider priority so DDG-led entries beat
    // Wikidata-only entries on perfect ties.
    const ap = providerPriorityScore(a.providers);
    const bp = providerPriorityScore(b.providers);
    if (ap !== bp) return ap - bp;
    return a.originalRank - b.originalRank;
  });
  return merged;
}

function providerPriorityScore(providers) {
  if (!Array.isArray(providers) || providers.length === 0) return 999;
  let best = 999;
  for (const p of providers) {
    const score = WEB_SEARCH_PROVIDER_PRIORITY[p && p.id] || 999;
    if (score < best) best = score;
  }
  return best;
}

// Issue #153: localized templates for the web search response. Keep these in
// sync with the visible UI strings in `src/web/i18n-catalog.lino`. The worker
// runs in a separate context that cannot import lino-i18n at runtime, so we
// inline the small subset that is actually rendered to chat. `en` is always
// the fallback when the catalogue for the active language is missing.
const WEB_SEARCH_TEXTS = {
  en: {
    header: (query, top, k) =>
      `Search results for \`${query}\` — top ${top} after reciprocal rank fusion (k = ${k}).`,
    otherSources: "Other sources",
    via: "via",
    readMore: "Read more",
    noResults: (query, providers) =>
      `No CORS-enabled web search results were returned for \`${query}\`.\n\nProviders tried: ${providers}.`,
    allDisabled: (providers) =>
      `All CORS-readable search providers are disabled for this session. Tried: ${providers}.`,
  },
  ru: {
    header: (query, top, k) =>
      `Результаты поиска для \`${query}\` — топ ${top} после реципрокного объединения рангов (k = ${k}).`,
    otherSources: "Другие источники",
    via: "через",
    readMore: "Подробнее",
    noResults: (query, providers) =>
      `Не получены результаты веб-поиска с поддержкой CORS для \`${query}\`.\n\nПопробованы провайдеры: ${providers}.`,
    allDisabled: (providers) =>
      `Все CORS-совместимые поисковые провайдеры отключены в этой сессии. Пробовали: ${providers}.`,
  },
  zh: {
    header: (query, top, k) =>
      `搜索 \`${query}\` 的结果 — 经互惠等级融合后的前 ${top} 项（k = ${k}）。`,
    otherSources: "其他来源",
    via: "来自",
    readMore: "阅读更多",
    noResults: (query, providers) =>
      `未获取到 \`${query}\` 的可用 CORS 搜索结果。\n\n已尝试的提供方：${providers}。`,
    allDisabled: (providers) =>
      `本会话中所有支持 CORS 的搜索提供方都已禁用。已尝试：${providers}。`,
  },
  hi: {
    header: (query, top, k) =>
      `\`${query}\` के लिए खोज परिणाम — रेसिप्रोकल रैंक फ़्यूज़न के बाद शीर्ष ${top} (k = ${k})।`,
    otherSources: "अन्य स्रोत",
    via: "के माध्यम से",
    readMore: "और पढ़ें",
    noResults: (query, providers) =>
      `\`${query}\` के लिए CORS-समर्थित कोई खोज परिणाम नहीं मिले।\n\nप्रयास किए गए प्रदाता: ${providers}.`,
    allDisabled: (providers) =>
      `इस सत्र के लिए सभी CORS-समर्थित खोज प्रदाता अक्षम हैं। प्रयास किया: ${providers}.`,
  },
};

function webSearchTexts(language) {
  const code = String(language || "").toLowerCase().slice(0, 2);
  return WEB_SEARCH_TEXTS[code] || WEB_SEARCH_TEXTS.en;
}

function normalizeBareTermResultLabel(value) {
  let text = cleanBareTermSearchFocus(value);
  if (!text) return "";
  if (typeof text.normalize === "function") {
    text = text.normalize("NFKC");
  }
  text = text
    .replace(/\s*\([^)]*\)\s*$/gu, "")
    .replace(/\s*（[^）]*）\s*$/gu, "")
    .replace(/\s+[-–—]\s+(?:wikipedia|wiktionary)\s*$/iu, "")
    .trim();
  return normalizePrompt(text).replace(/\s+/gu, " ").trim();
}

function unresolvedBareTermResultLabels(entry) {
  if (!entry) return [];
  const labels = [entry.title, entry.wikipediaKey, entry.wiktionaryKey];
  if (Array.isArray(entry.alternateUrls)) {
    for (const alternate of entry.alternateUrls) {
      labels.push(alternate && alternate.title);
    }
  }
  return labels.filter(Boolean);
}

function hasGroundedUnresolvedBareTermResult(query, results) {
  const normalizedQuery = normalizeBareTermResultLabel(query);
  if (!normalizedQuery || !Array.isArray(results)) return false;
  return results.some((entry) =>
    unresolvedBareTermResultLabels(entry).some(
      (label) => normalizeBareTermResultLabel(label) === normalizedQuery,
    ),
  );
}

async function tryWebSearch(prompt, language) {
  const normalized = normalizePrompt(prompt);
  const request = extractWebSearchRequest(prompt, normalized);
  if (!request || !request.query) return null;
  return runWebSearchQuery(request.query, language, request.kind);
}

async function runWebSearchQuery(query, language, queryKind) {
  query = String(query || "").trim();
  if (!query) return null;
  const rrfK = webSearchRrfK();
  const concurrency = webSearchConcurrency();
  const providerLimit = webSearchProviderLimit();
  const texts = webSearchTexts(language);

  // Issue #180: pre-probe every provider once per browser session so the
  // first user query does not waste sockets on CORS-blocked endpoints. The
  // probe results live in `WEB_SEARCH_AVAILABLE`/`WEB_SEARCH_DISABLED` for
  // the rest of the worker lifetime.
  await ensureWebSearchProviderProbes();

  // R194: the Rust core (`web_search_core::build_request_evidence`) is the
  // source of truth for the `web_search:*` evidence prefix. We prepend its
  // output and fall back to the inline list when the WASM worker booted in
  // `js fallback` mode.
  const evidence = [];
  const wasmEvidence = wasmWebSearchRequestEvidence(query, language || "");
  if (Array.isArray(wasmEvidence) && wasmEvidence.length > 0) {
    for (const line of wasmEvidence) {
      if (line) evidence.push(line);
    }
  } else {
    evidence.push(`web_search:request:${query}`);
    if (language) {
      evidence.push(`web_search:language:${language}`);
    }
    for (const provider of WEB_SEARCH_PROVIDERS) {
      evidence.push(`web_search:provider:${provider.id}`);
    }
    evidence.push(`web_search:combined:rrf:k=${rrfK}`);
  }
  if (queryKind) {
    evidence.push(`web_search:query_kind:${queryKind}`);
  }

  // Issue #180: providers are tried in declared priority order so the rendered
  // list matches the user's requested DDG → IA → WP → WD → Wiktionary
  // sequence whenever scores tie. Session-disabled providers are skipped on
  // top of the WASM-derived prefix and annotated for the diagnostics panel.
  const ordered = WEB_SEARCH_PROVIDERS.slice().sort((a, b) => {
    const pa = typeof a.priority === "number" ? a.priority : 999;
    const pb = typeof b.priority === "number" ? b.priority : 999;
    return pa - pb;
  });
  const active = ordered.filter((provider) => !webSearchIsDisabled(provider.id));
  for (const provider of ordered) {
    if (webSearchIsDisabled(provider.id)) {
      evidence.push(`web_search:disabled:${provider.id}`);
    } else if (WEB_SEARCH_AVAILABLE.has(provider.id)) {
      evidence.push(`web_search:available:${provider.id}`);
    }
  }

  if (active.length === 0) {
    if (queryKind === "unresolved_bare_term") return null;
    return {
      intent: "web_search",
      content: texts.allDisabled(WEB_SEARCH_PROVIDERS.map((p) => p.id).join(", ")),
      confidence: 0.3,
      evidence,
      diagnostics: { providers: [], httpExchanges: consumeWebSearchDiagnostics() },
    };
  }

  const tasks = active.map((provider) => async () => {
    const startedAt = Date.now();
    const outcome = await provider.run(query, language, providerLimit);
    return Object.assign({ id: provider.id, label: provider.label, elapsedMs: Date.now() - startedAt }, outcome);
  });
  const perProvider = await runWithConcurrencyLimit(tasks, concurrency);

  for (const provider of perProvider) {
    if (!provider.ok) {
      evidence.push(`web_search:provider:${provider.id}:error:${provider.error || "no_results"}`);
      continue;
    }
    evidence.push(`web_search:provider:${provider.id}:count:${provider.results.length}`);
    if (provider.language) {
      evidence.push(`web_search:provider:${provider.id}:language:${provider.language}`);
    }
    provider.results.forEach((item, index) => {
      evidence.push(`web_search:rank:${provider.id}:${index + 1}:${item.url}`);
    });
  }

  const fused = reciprocalRankFusion(perProvider, rrfK);
  const metaByUrl = buildItemMetadataIndex(perProvider);
  const deduped = dedupeFusedEntries(fused, metaByUrl, evidence);
  const top = deduped.slice(0, providerLimit);
  top.forEach((entry, index) => {
    evidence.push(`web_search:fused:${index + 1}:${entry.providers.map((p) => p.id).join("+")}:${entry.url}`);
    if (entry.virtualId) {
      evidence.push(`web_search:formal:${index + 1}:${entry.virtualId}`);
    }
  });

  const diagnostics = {
    query,
    language: language || "",
    providers: perProvider.map((p) => ({
      id: p.id,
      label: p.label,
      ok: !!p.ok,
      count: Array.isArray(p.results) ? p.results.length : 0,
      elapsedMs: p.elapsedMs || 0,
      finalUrl: p.finalUrl || "",
      error: p.error || "",
    })),
    httpExchanges: consumeWebSearchDiagnostics(),
    fused: top.map((entry, index) => ({
      rank: index + 1,
      url: entry.url,
      title: entry.title,
      score: entry.score,
      providers: entry.providers,
      alternateUrls: entry.alternateUrls,
      virtualId: entry.virtualId || "",
      keys: entry.keys || [],
    })),
  };

  if (
    queryKind === "unresolved_bare_term" &&
    !hasGroundedUnresolvedBareTermResult(query, top)
  ) {
    return null;
  }

  if (top.length === 0) {
    return {
      intent: "web_search",
      content: texts.noResults(query, active.map((p) => p.label).join(", ")),
      confidence: 0.35,
      evidence,
      diagnostics,
    };
  }

  // Issue #180: every fused result is rendered Google-style — a single line
  // with title + bare domain, an indented quote (a fragment containing the
  // original query when possible, truncated near ~220 chars), a "Read more"
  // link, and finally a faint "Другие источники:" line listing alternates
  // (provider label + url) without per-source excerpts.
  const lines = [texts.header(query, top.length, rrfK), ""];
  top.forEach((entry, index) => {
    const domain = extractDomain(entry.url);
    const titlePiece = `**[${entry.title || entry.url}](${entry.url})**`;
    const domainPiece = domain ? `  \`${domain}\`` : "";
    const idTag = entry.virtualId ? `  \`${entry.virtualId}\`` : "";
    lines.push(`${index + 1}. ${titlePiece}${domainPiece}${idTag}`);
    const quote = extractQuoteAroundQuery(entry.excerpt, query, 220);
    if (quote) {
      lines.push(`   > ${quote}`);
    }
    const sourceTags = entry.providers
      .map((p) => `${p.id}#${p.rank}`)
      .join(", ");
    lines.push(`   [${texts.readMore}](${entry.url}) — _${texts.via} ${sourceTags}_`);
    if (Array.isArray(entry.alternateUrls) && entry.alternateUrls.length > 0) {
      const others = entry.alternateUrls
        .map((alt) => {
          const labelProvider = pickPrimaryProviderId(alt.providers, alt.sourceKind);
          const label = providerDisplayLabel(labelProvider, language);
          return `[${label}](${alt.url})`;
        })
        .filter(Boolean);
      if (others.length > 0) {
        lines.push(`   _${texts.otherSources}: ${others.join(", ")}_`);
      }
    }
    lines.push("");
  });
  while (lines.length > 0 && lines[lines.length - 1] === "") lines.pop();

  // Resolve the formalization tuple now that we know the top-ranked entity.
  // Prefer a real Wikidata Q-id; fall back to the WP virtual id, then to the
  // bare normalised query. We scan the whole `top` slice instead of just
  // `top[0]` so that a DuckDuckGo result without an id at rank 1 still lets
  // us fold a Wikidata Q-id from rank 2+ into the resolved tuple.
  let formalizedObject = "";
  for (const entry of top) {
    if (entry && entry.virtualId) {
      formalizedObject = entry.virtualId;
      if (/^Q\d+$/.test(entry.virtualId)) break;
    }
  }

  return {
    intent: "web_search",
    content: lines.join("\n"),
    confidence: 0.85,
    evidence,
    formalizedObject,
    query,
    diagnostics,
  };
}

const PROMOTED_PROJECT_ORGS = ["link-assistant", "link-foundation", "linksplatform"];

function projectPromotionEnabled(preferences) {
  const value = preferences && preferences.associativeProjectPromotion;
  if (value === undefined || value === null || value === "") return true;
  if (value === true) return true;
  if (value === false) return false;
  const normalized = String(value).trim().toLowerCase();
  if (["0", "false", "no", "off", "disabled"].includes(normalized)) return false;
  if (["1", "true", "yes", "on", "enabled"].includes(normalized)) return true;
  return true;
}

function normalizeProjectTerm(value) {
  let term = normalizePrompt(value)
    .replace(/[-_]+/g, " ")
    .replace(/\s+/g, " ")
    .trim();
  for (const prefix of ["the ", "a ", "an "]) {
    if (term.startsWith(prefix)) {
      term = term.slice(prefix.length).trim();
      break;
    }
  }
  return term;
}

function projectRepoSlug(project) {
  return `${project.org}/${project.name}`;
}

function localizedProject(project, language) {
  if (!project || !Array.isArray(project.localized)) return null;
  return (
    project.localized.find((loc) => loc && loc.language === language) ||
    project.localized.find((loc) => loc && loc.language === "en") ||
    null
  );
}

function projectDisplayName(project, language) {
  const localized = localizedProject(project, language);
  return (localized && localized.displayName) || project.displayName || project.name || "";
}

function projectStatementsFor(project, language) {
  const localized = localizedProject(project, language);
  if (
    localized &&
    Array.isArray(localized.statements) &&
    localized.statements.length > 0
  ) {
    return localized.statements;
  }
  return Array.isArray(project && project.statements) ? project.statements : [];
}

function describeProjectRecord(project, language) {
  const statements = projectStatementsFor(project, language)
    .filter((statement) => {
      const kind = statement && statement.kind;
      return statement && statement.text && kind !== "install" && kind !== "example";
    })
    .slice()
    .sort((a, b) => Number(b.weight || 0) - Number(a.weight || 0))
    .slice(0, 3)
    .map((statement) => String(statement.text).trim())
    .filter(Boolean);
  if (statements.length > 0) return statements.join(" ");
  return project.description || projectDisplayName(project, language);
}

function projectMatchesAlias(project, normalizedTerm) {
  if (!project || !normalizedTerm) return false;
  const aliases = Array.isArray(project.aliases) ? project.aliases : [];
  return (
    normalizeProjectTerm(project.displayName) === normalizedTerm ||
    normalizeProjectTerm(project.name) === normalizedTerm ||
    normalizeProjectTerm(projectRepoSlug(project)) === normalizedTerm ||
    aliases.some((alias) => normalizeProjectTerm(alias) === normalizedTerm)
  );
}

function projectByAlias(term) {
  const normalizedTerm = normalizeProjectTerm(term);
  if (!normalizedTerm) return null;
  return PROJECTS.find((project) => projectMatchesAlias(project, normalizedTerm)) || null;
}

function isPromotedProject(project) {
  return PROMOTED_PROJECT_ORGS.some(
    (org) => String(project && project.org).toLowerCase() === org,
  );
}

function promotedProjectByRepo(owner, name) {
  const ownerLower = String(owner || "").toLowerCase();
  const nameLower = String(name || "").toLowerCase();
  return (
    PROJECTS.find(
      (project) =>
        isPromotedProject(project) &&
        String(project.org || "").toLowerCase() === ownerLower &&
        String(project.name || "").toLowerCase() === nameLower,
    ) || null
  );
}

function cleanRepositorySegment(segment) {
  const trimmed = String(segment || "").trim().replace(/\.git$/i, "");
  if (!trimmed || !/^[A-Za-z0-9._-]+$/.test(trimmed)) return "";
  return trimmed;
}

function isTextUrlExtractionPrompt(prompt) {
  const normalized = normalizePrompt(prompt).trimStart().toLowerCase();
  return (
    (normalized.startsWith("extract url ") ||
      normalized.startsWith("extract urls ")) &&
    normalized.includes(" from ")
  );
}

function repositoryFromUrl(url) {
  let parsed;
  try {
    parsed = new URL(url);
  } catch (_error) {
    return null;
  }
  const host = parsed.hostname.toLowerCase().replace(/^www\./, "");
  const platform =
    host === "github.com"
      ? { slug: "github", label: "GitHub", host: "github.com" }
      : host === "gitlab.com"
        ? { slug: "gitlab", label: "GitLab", host: "gitlab.com" }
        : host === "bitbucket.org"
          ? { slug: "bitbucket", label: "Bitbucket", host: "bitbucket.org" }
          : null;
  if (!platform) return null;
  const segments = parsed.pathname.split("/").filter(Boolean);
  const owner = cleanRepositorySegment(segments[0]);
  const name = cleanRepositorySegment(segments[1]);
  if (!owner || !name) return null;
  return {
    platform,
    owner,
    name,
    url: `https://${platform.host}/${owner}/${name}`,
  };
}

function repositoryFromSlug(term) {
  const parts = String(term || "").trim().split("/");
  if (parts.length !== 2) return null;
  const owner = cleanRepositorySegment(parts[0]);
  const name = cleanRepositorySegment(parts[1]);
  if (!owner || !name) return null;
  if (/^\d+$/.test(owner) && /^\d+$/.test(name)) return null;
  return {
    platform: { slug: "github", label: "GitHub", host: "github.com" },
    owner,
    name,
    url: `https://github.com/${owner}/${name}`,
  };
}

function repositoryFromPrompt(prompt) {
  const urlCandidate = firstUrlCandidate(prompt);
  if (urlCandidate) {
    const repo = repositoryFromUrl(urlCandidate.url);
    if (repo) return repo;
  }
  const query = extractConceptQuery(prompt);
  if (!query) return null;
  const term = String(query.termOriginal || query.term || "").trim();
  if (!term) return null;
  if (term.includes("://") || looksLikeHostname(term)) {
    const url = normalizeUrlCandidate(term);
    return url ? repositoryFromUrl(url) : null;
  }
  if (term.includes("/") && !/\s/.test(term)) {
    return repositoryFromSlug(term);
  }
  return null;
}

function repositorySlug(repo) {
  return `${repo.owner}/${repo.name}`;
}

const GITHUB_README_KNOWN_TOOLS = [
  "http_fetch",
  "url_navigate",
  "web_search",
  "wikipedia_lookup",
  "calculator",
  "eval_js",
  "read_local_file",
  "append_memory",
  "export_memory",
  "import_memory",
  "conversation_recall",
  "concept_lookup",
  "write_program",
  "intent_routing",
  "fact_lookup",
  "summarize_conversation",
  "brainstorm",
  "coreference",
  "roleplay",
];

// Issue #497: GitHub repository traffic visibility is a platform-policy answer,
// not a live repository-info fetch. These role names mirror the Rust constants
// in src/seed/roles/tooling.rs; all natural-language surfaces live in
// data/seed/meanings-web-search.lino and are loaded through MEANINGS_LINO.
const ROLE_GITHUB_REPOSITORY_PLATFORM = "github_repository_platform";
const ROLE_REPOSITORY_REFERENCE = "repository_reference";
const ROLE_GITHUB_REPOSITORY_TRAFFIC_SIGNAL = "github_repository_traffic_signal";
const ROLE_GITHUB_REPOSITORY_TRAFFIC_QUESTION =
  "github_repository_traffic_question";
const GITHUB_TRAFFIC_UI_DOC =
  "https://docs.github.com/en/repositories/viewing-activity-and-data-for-your-repository/viewing-traffic-to-a-repository";
const GITHUB_TRAFFIC_API_DOC = "https://docs.github.com/en/rest/metrics/traffic";
const DEFAULT_GITHUB_REPOSITORY = "link-assistant/formal-ai";

function githubRepositoryTrafficRepository() {
  const repository = String((AGENT_INFO && AGENT_INFO.repository) || "").trim();
  return repository || DEFAULT_GITHUB_REPOSITORY;
}

function githubRepositoryTrafficBody(language, repository) {
  return answerFor("github_repository_traffic", language)
    .replace("{repository}", repository)
    .replace("{traffic_ui_docs}", GITHUB_TRAFFIC_UI_DOC)
    .replace("{traffic_api_docs}", GITHUB_TRAFFIC_API_DOC);
}

function isGithubRepositoryTrafficQuestion(normalized, language) {
  const languages = language === "en" ? ["en"] : [language, "en"];
  return (
    mentionsRoleInLanguagesRaw(
      ROLE_GITHUB_REPOSITORY_PLATFORM,
      normalized,
      languages,
    ) &&
    mentionsRoleInLanguagesRaw(ROLE_REPOSITORY_REFERENCE, normalized, languages) &&
    mentionsRoleInLanguagesRaw(
      ROLE_GITHUB_REPOSITORY_TRAFFIC_SIGNAL,
      normalized,
      languages,
    ) &&
    mentionsRoleInLanguagesRaw(
      ROLE_GITHUB_REPOSITORY_TRAFFIC_QUESTION,
      normalized,
      languages,
    )
  );
}

function tryGithubRepositoryTraffic(normalized, language) {
  if (!isGithubRepositoryTrafficQuestion(normalized, language)) return null;
  const repository = githubRepositoryTrafficRepository();
  return {
    intent: "github_repository_traffic",
    content: githubRepositoryTrafficBody(language, repository),
    confidence: 0.92,
    evidence: [
      "github_repository_traffic:platform:github",
      `github_repository_traffic:repository:${repository}`,
      "github_repository_traffic:access:push_or_write_access_required",
      "github_repository_traffic:window:last_14_days",
      "github_repository_traffic:aggregate:views_uniques_clones_referrers_paths",
      "github_repository_traffic:privacy:no_individual_identity",
      `source:${GITHUB_TRAFFIC_UI_DOC}`,
      `source:${GITHUB_TRAFFIC_API_DOC}`,
      `language:${language}`,
    ],
  };
}

function githubRepositoryInfoRequest(prompt, normalized) {
  const repo = repositoryFromPrompt(prompt);
  if (!repo || !repo.platform || repo.platform.slug !== "github") return null;
  const markers = [
    "extract information",
    "main programming language",
    "programming language",
    "number of stars",
    "star count",
    "stars",
    "last commit",
    "readme",
  ];
  if (!containsAny(normalized, markers)) return null;
  return repo;
}

function githubApiRepositoryUrl(repo, suffix) {
  const owner = encodeURIComponent(repo.owner);
  const name = encodeURIComponent(repo.name);
  return `https://api.github.com/repos/${owner}/${name}${suffix || ""}`;
}

async function fetchGithubJson(url, label, diagnostics) {
  if (typeof fetch !== "function") {
    throw new Error("fetch unavailable");
  }
  const startedAt = Date.now();
  const headers = { Accept: "application/vnd.github+json" };
  const response = await fetch(url, { headers });
  const text = await response.text();
  const entry = {
    providerId: "github",
    url,
    method: "GET",
    requestHeaders: headers,
    ok: Boolean(response && response.ok),
    status: response ? response.status : 0,
    statusText: response ? response.statusText : "",
    elapsedMs: Date.now() - startedAt,
    responseSnippet: text.slice(0, 1024),
    unified: `github_api ${label}`,
  };
  if (Array.isArray(diagnostics)) diagnostics.push(entry);
  if (!response || !response.ok) {
    throw new Error(`HTTP ${entry.status} ${entry.statusText}`.trim());
  }
  try {
    return text ? JSON.parse(text) : null;
  } catch (error) {
    const message = error instanceof Error ? error.message : String(error);
    throw new Error(`JSON parse failed: ${message}`);
  }
}

function decodeGithubReadmeContent(readme) {
  if (!readme || typeof readme !== "object") return "";
  if (typeof readme.content === "string" && readme.encoding === "base64") {
    const encoded = readme.content.replace(/\s+/g, "");
    if (typeof atob !== "function") return "";
    try {
      const binary = atob(encoded);
      const bytes = new Uint8Array(binary.length);
      for (let index = 0; index < binary.length; index += 1) {
        bytes[index] = binary.charCodeAt(index);
      }
      if (typeof TextDecoder === "function") {
        return new TextDecoder("utf-8").decode(bytes);
      }
      let decoded = "";
      for (let index = 0; index < binary.length; index += 1) {
        decoded += String.fromCharCode(bytes[index]);
      }
      return decoded;
    } catch (_error) {
      return "";
    }
  }
  return "";
}

function pushUniqueToolName(out, seen, name) {
  const cleaned = String(name || "")
    .trim()
    .replace(/^[@#]+/u, "")
    .replace(/[.,:;!?]+$/u, "");
  if (!cleaned || cleaned.length > 48) return;
  if (!/^[a-z][a-z0-9_-]*$/i.test(cleaned)) return;
  const key = cleaned.toLowerCase();
  if (seen.has(key)) return;
  seen.add(key);
  out.push(cleaned);
}

function pushUniqueToolDisplayName(out, seen, name) {
  const cleaned = String(name || "")
    .replace(/`+/gu, "")
    .replace(/\*\*/gu, "")
    .trim()
    .replace(/[.,:;!?]+$/u, "")
    .replace(/\s+/gu, " ");
  if (!cleaned || cleaned.length > 80 || !/[A-Za-z]/u.test(cleaned)) return;
  if (/\b(?:agentic|available|supported|search|execution|utility)\s+tools?\b/i.test(cleaned)) {
    return;
  }
  const key = cleaned.toLowerCase();
  if (seen.has(key)) return;
  seen.add(key);
  out.push(cleaned);
}

function extractReadmeTools(markdown) {
  const text = String(markdown || "");
  const lower = text.toLowerCase();
  const out = [];
  const seen = new Set();
  for (const tool of GITHUB_README_KNOWN_TOOLS) {
    if (lower.includes(tool.toLowerCase())) {
      pushUniqueToolName(out, seen, tool);
    }
  }
  const lines = text.split(/\r?\n/);
  let inToolsSection = false;
  let toolsSectionLevel = 0;
  for (const line of lines) {
    const heading = /^(\s{0,3})(#{1,6})\s+(.+?)\s*#*\s*$/u.exec(line);
    if (heading) {
      const level = heading[2].length;
      const headingText = heading[3].trim();
      if (/\btools?\b/i.test(headingText)) {
        inToolsSection = true;
        toolsSectionLevel = level;
      } else if (inToolsSection && level > toolsSectionLevel) {
        pushUniqueToolDisplayName(out, seen, headingText);
      } else {
        inToolsSection = false;
        toolsSectionLevel = 0;
      }
      continue;
    }
    if (!inToolsSection) continue;
    const bullet = /^\s*[-*+]\s+(?:`([^`]+)`|([A-Za-z][A-Za-z0-9_-]*))/u.exec(line);
    if (bullet) {
      pushUniqueToolName(out, seen, bullet[1] || bullet[2]);
    }
    const codeMatches = line.matchAll(/`([A-Za-z][A-Za-z0-9_-]*)`/gu);
    for (const match of codeMatches) {
      pushUniqueToolName(out, seen, match[1]);
    }
  }
  return out;
}

function githubCommitDate(commits) {
  if (!Array.isArray(commits) || commits.length === 0) return null;
  const commit = commits[0] && commits[0].commit ? commits[0].commit : null;
  return (
    (commit && commit.committer && commit.committer.date) ||
    (commit && commit.author && commit.author.date) ||
    null
  );
}

async function tryGithubRepositoryInfo(repo, language, preferences = {}) {
  const diagnostics = [];
  const errors = [];
  let repoData = null;
  let commits = null;
  let readme = null;

  // Issue #444: honor the GitHub opt-out. When disabled we skip every live
  // api.github.com fetch and report that the trusted service was turned off in
  // settings instead of contacting it.
  const githubEnabled = externalServiceEnabled(preferences, "externalServiceGithub");
  if (githubEnabled) {
    try {
      repoData = await fetchGithubJson(githubApiRepositoryUrl(repo), "repository", diagnostics);
    } catch (error) {
      errors.push(`repository: ${error instanceof Error ? error.message : String(error)}`);
    }
    try {
      commits = await fetchGithubJson(
        githubApiRepositoryUrl(repo, "/commits?per_page=1"),
        "commits",
        diagnostics,
      );
    } catch (error) {
      errors.push(`commits: ${error instanceof Error ? error.message : String(error)}`);
    }
    try {
      readme = await fetchGithubJson(githubApiRepositoryUrl(repo, "/readme"), "readme", diagnostics);
    } catch (error) {
      errors.push(`readme: ${error instanceof Error ? error.message : String(error)}`);
    }
  }

  const slug = repositorySlug(repo);
  const readmeMarkdown = decodeGithubReadmeContent(readme);
  const payload = {
    repository: (repoData && repoData.full_name) || slug,
    mainProgrammingLanguage: (repoData && repoData.language) || null,
    stars:
      repoData && Number.isFinite(Number(repoData.stargazers_count))
        ? Number(repoData.stargazers_count)
        : null,
    lastCommitDate: githubCommitDate(commits) || null,
    readmeTools: extractReadmeTools(readmeMarkdown),
  };
  if (errors.length > 0) {
    payload.errors = errors;
  }
  if (!githubEnabled) {
    payload.note = "GitHub is disabled in settings; no live data was fetched.";
  }
  const content = `\`\`\`json\n${JSON.stringify(payload, null, 2)}\n\`\`\``;
  return {
    intent: "github_repo_info",
    content,
    confidence: !githubEnabled ? 0.4 : errors.length === 0 ? 0.92 : 0.55,
    evidence: [
      `github_repo_info:repository:${slug}`,
      ...(githubEnabled
        ? [`source:${repo.url}`, "source:https://api.github.com"]
        : ["github_repo_info:service_disabled:github"]),
      `language:${language}`,
      ...(errors.length > 0 ? errors.map((error) => `github_repo_info:error:${error}`) : []),
    ],
    diagnostics: {
      providers: [],
      httpExchanges: diagnostics,
    },
  };
}

function genericProjectLookupAnswer(prompt, language, repo, promotionEnabled) {
  const evidence = [];
  if (!promotionEnabled) evidence.push("project_lookup:promotion:disabled");
  if (repo) {
    const slug = repositorySlug(repo);
    evidence.push(`project_lookup:repository:${repo.platform.slug}:${slug}`);
    evidence.push(`source:${repo.url}`);
    let content;
    if (language === "ru") {
      content = `Это запрос о репозитории [${slug}](${repo.url}) на ${repo.platform.label}.\n\nОбычный путь project_lookup ищет и резюмирует README или описание проекта на GitHub, GitLab и Bitbucket без особого правила для отдельного названия. Если репозиторий находится в продвигаемых организациях и продвижение включено, он будет показан первым.`;
    } else if (language === "hi") {
      content = `यह ${repo.platform.label} पर रिपॉजिटरी [${slug}](${repo.url}) के लिए lookup है.\n\nGeneric project_lookup path GitHub, GitLab, और Bitbucket पर README या project descriptions summarize कर सकता है, किसी single name के लिए special case के बिना. अगर repository promoted organizations में है और promotion enabled है, तो वह पहले दिखाई जाएगी.`;
    } else if (language === "zh") {
      content = `这是 ${repo.platform.label} 上的仓库查询： [${slug}](${repo.url})。\n\n通用 project_lookup 路径可以汇总 GitHub、GitLab 和 Bitbucket 上的 README 或项目描述，不需要为单个名称写特殊规则。如果该仓库属于 promoted organizations 且 promotion 已开启，它会优先显示。`;
    } else {
      content = `This is a repository lookup for [${slug}](${repo.url}) on ${repo.platform.label}.\n\nThe generic project_lookup path can summarize README or project descriptions from GitHub, GitLab, and Bitbucket without a special case for any single name. If the repository belongs to a promoted organization and promotion is enabled, that repository is listed first.`;
    }
    return { intent: "project_lookup", content, confidence: 0.82, evidence };
  }
  evidence.push("project_lookup:repository_hosts:GitHub,GitLab,Bitbucket");
  let content;
  if (language === "ru") {
    content = "Это обычный запрос project_lookup о проекте или репозитории.\n\nЯ не выделяю специальный репозиторий, потому что продвижение ассоциативных репозиториев отключено. Дальше следует искать и резюмировать подходящие проекты на GitHub, GitLab и Bitbucket и похожих хостингах.";
  } else if (language === "hi") {
    content = "यह project या repository के लिए generic project_lookup request है.\n\nमैं किसी specific repository को प्राथमिकता नहीं दे रहा हूँ क्योंकि associative repository promotion disabled है. अगला step GitHub, GitLab, Bitbucket और similar hosts पर matching projects खोजना और summarize करना है.";
  } else if (language === "zh") {
    content = "这是一个关于项目或仓库的通用 project_lookup 请求。\n\n我不会优先选择某个特定仓库，因为 associative repository promotion 已关闭。下一步是在 GitHub、GitLab、Bitbucket 和类似托管平台中搜索并汇总匹配项目。";
  } else {
    content = "This is a generic project_lookup request for a project or repository.\n\nI am not privileging a specific repository because associative repository promotion is disabled. The next step is to search and summarize matching projects across GitHub, GitLab, Bitbucket, and similar hosts.";
  }
  return { intent: "project_lookup", content, confidence: 0.72, evidence };
}

async function renderPromotedProjectLookup(prompt, language, project) {
  const displayName = projectDisplayName(project, language);
  const repo = projectRepoSlug(project);
  const url = project.url || `https://github.com/${repo}`;
  const description = describeProjectRecord(project, language);
  const orgs = PROMOTED_PROJECT_ORGS.join(", ");
  let preferredLine;
  if (language === "ru") {
    preferredLine = `В контексте репозиториев ${orgs} под \`${displayName}\` я прежде всего имею в виду [${repo}](${url}) — ${description}`;
  } else if (language === "hi") {
    preferredLine = `${orgs} repository context में \`${displayName}\` से मेरा पहला मतलब [${repo}](${url}) है — ${description}`;
  } else if (language === "zh") {
    preferredLine = `在 ${orgs} 仓库上下文中，\`${displayName}\` 首先指 [${repo}](${url}) — ${description}`;
  } else {
    preferredLine = `In the ${orgs} repository context, \`${displayName}\` should first mean [${repo}](${url}) — ${description}`;
  }

  const search = await runWebSearchQuery(displayName, language);
  const evidence = [
    `project:promoted:${repo}`,
    `source:${url}`,
    "summarization:mode:short",
    `summarization:language:${language}`,
  ];
  if (search && Array.isArray(search.evidence)) {
    evidence.push(...search.evidence);
  } else {
    evidence.push("web_search:no_results");
  }

  const lines = [preferredLine];
  if (search && search.content) {
    lines.push("");
    lines.push(
      language === "ru"
        ? "Другие найденные в интернете репозитории и сущности:"
        : language === "hi"
          ? "Internet पर मिले दूसरे repositories और entities:"
          : language === "zh"
            ? "互联网上找到的其他仓库和实体："
            : "Other repositories and entities found online:",
    );
    lines.push("");
    lines.push(search.content);
  } else {
    lines.push("");
    lines.push(
      language === "ru"
        ? "Интернет-поиск по другим совпадениям не вернул результатов через доступные CORS-провайдеры."
        : language === "hi"
          ? "दूसरे matches के लिए web search उपलब्ध CORS providers से results नहीं लौटा."
          : language === "zh"
            ? "通过可用的 CORS providers 搜索其他匹配项没有返回结果。"
            : "Web search for other matches returned no results through the available CORS providers.",
    );
  }

  return {
    intent: "project_lookup",
    content: lines.join("\n"),
    confidence: 0.9,
    evidence,
  };
}

async function tryProjectLookup(prompt, language, preferences) {
  return tryProjectLookupForPrompt(prompt, prompt, language, preferences);
}

async function tryProjectLookupForPrompt(prompt, lookupPrompt, language, preferences) {
  if (isTextUrlExtractionPrompt(lookupPrompt)) return null;
  if (githubRepositoryInfoRequest(lookupPrompt, String(lookupPrompt || "").toLowerCase())) {
    return null;
  }
  const promotionEnabled = projectPromotionEnabled(preferences);
  const repo = repositoryFromPrompt(lookupPrompt);
  if (repo) {
    const promoted = promotionEnabled
      ? promotedProjectByRepo(repo.owner, repo.name)
      : null;
    if (promoted) {
      return renderPromotedProjectLookup(prompt, language, promoted);
    }
    return genericProjectLookupAnswer(prompt, language, repo, promotionEnabled);
  }

  const query = extractConceptQuery(lookupPrompt);
  if (!query) return null;
  const project = projectByAlias(query.termOriginal || query.term);
  if (!project) return null;
  if (promotionEnabled && isPromotedProject(project)) {
    return renderPromotedProjectLookup(prompt, language, project);
  }
  return genericProjectLookupAnswer(prompt, language, null, promotionEnabled);
}

function isLanguageReanswerFollowup(normalized) {
  const text = String(normalized || "").trim().toLowerCase();
  if (!text) return false;
  if (
    text.includes("do not understand") ||
    text.includes("don't understand") ||
    text.includes("dont understand") ||
    text.includes("can't understand") ||
    text.includes("cant understand") ||
    text.includes("cannot understand") ||
    text.includes("не понимаю") ||
    text.includes("не понял") ||
    text.includes("не поняла") ||
    text.includes("समझ नहीं") ||
    text.includes("नहीं आती") ||
    text.includes("不懂") ||
    text.includes("看不懂") ||
    text.includes("听不懂")
  ) {
    return true;
  }
  const wordCount = text.split(/\s+/u).filter(Boolean).length;
  return (
    wordCount <= 4 &&
    (text.includes("in english") ||
      text.includes("in russian") ||
      text.includes("in hindi") ||
      text.includes("in chinese") ||
      text.includes("на английском") ||
      text.includes("по-английски") ||
      text.includes("на русском") ||
      text.includes("по-русски") ||
      text.includes("по русски") ||
      text.includes("на хинди") ||
      text.includes("на китайском") ||
      text.includes("अंग्रेजी में") ||
      text.includes("अंग्रेज़ी में") ||
      text.includes("रूसी में") ||
      text.includes("हिंदी में") ||
      text.includes("हिन्दी में") ||
      text.includes("चीनी में") ||
      text.includes("用英文") ||
      text.includes("用英语") ||
      text.includes("用英語") ||
      text.includes("用俄语") ||
      text.includes("用俄語") ||
      text.includes("用印地语") ||
      text.includes("用印地文") ||
      text.includes("用中文") ||
      text.includes("用汉语") ||
      text.includes("用漢語"))
  );
}

async function tryResponseLanguageFollowup(prompt, normalized, history, preferences) {
  const targetLanguage = detectResponseLanguage(normalized);
  if (!targetLanguage) return null;
  if (!isLanguageReanswerFollowup(normalized)) return null;
  const previousUser = lastHistoryTurn(history, "user");
  if (!String(previousUser || "").trim()) return null;

  const answer = await tryProjectLookupForPrompt(
    prompt,
    previousUser,
    targetLanguage,
    preferences,
  );
  if (!answer) return null;
  return {
    ...answer,
    evidence: [
      `response_language_followup:target:${targetLanguage}`,
      `language_to:${targetLanguage}`,
      "response_language_followup:handler:project_lookup",
      ...(Array.isArray(answer.evidence) ? answer.evidence : []),
    ],
  };
}

function pickPrimaryProviderId(providers, sourceKind) {
  if (sourceKind === "wikidata") return "wikidata";
  if (sourceKind === "wikipedia") return "wikipedia";
  if (sourceKind === "wiktionary") return "wiktionary";
  if (sourceKind === "internet-archive") return "internet-archive";
  if (Array.isArray(providers) && providers.length > 0) {
    const sorted = providers.slice().sort(
      (a, b) => (WEB_SEARCH_PROVIDER_PRIORITY[a.id] || 999) - (WEB_SEARCH_PROVIDER_PRIORITY[b.id] || 999),
    );
    return sorted[0].id;
  }
  return "";
}

function cleanContextValue(value) {
  return String(value || "").replace(/\s+/g, " ").trim();
}

function evidenceFromUserContext(userContext) {
  if (!userContext || typeof userContext !== "object") return [];
  const evidence = [];
  const fields = [
    ["uiLanguage", "ui_language"],
    ["browserLanguage", "browser_language"],
    ["colorScheme", "color_scheme"],
    ["timeZone", "time_zone"],
    ["locationInference", "location_inference"],
  ];
  for (const [key, label] of fields) {
    const value = cleanContextValue(userContext[key]);
    if (value) evidence.push(`user_context:${label}:${value}`);
  }
  return evidence;
}

function attachUserContext(answer, userContext) {
  if (!answer || typeof answer !== "object") return answer;
  const evidence = evidenceFromUserContext(userContext);
  if (evidence.length === 0) return answer;
  const steps = Array.isArray(answer.steps) ? answer.steps.slice() : [];
  const detail = evidence
    .map((item) => item.replace(/^user_context:/, ""))
    .join(", ");
  steps.push({ step: "user_context", detail });
  return Object.assign({}, answer, {
    evidence: [
      ...(Array.isArray(answer.evidence) ? answer.evidence : []),
      ...evidence,
    ],
    steps: withThinkingLevels(steps),
  });
}

// Issue #153: every prompt should be formalized as a Subject-Verb-Object tuple
// regardless of source language. We emit a deterministic, offline formalization
// here (so the trace is stable even when no APIs are reachable) and, when a
// downstream handler resolves the object to a Wikidata/Wikipedia/Wiktionary
// item, we emit a second `formalize_resolved` step with the real ids. Ids use
// canonical prefixes: `Q<n>` / `P<n>` for Wikidata, `WP:<title>` for
// Wikipedia-only items, `WT:<word>` for Wiktionary-only items, `OP:<verb>` for
// the symbolic operation, and `@USER` for the implicit user subject.
const FORMALIZATION_VERBS = [
  // English
  { verb: "what are the steps to", op: "OP:procedure" },
  { verb: "show me how to", op: "OP:procedure" },
  { verb: "tell me how to", op: "OP:procedure" },
  { verb: "how should i", op: "OP:procedure" },
  { verb: "how could i", op: "OP:procedure" },
  { verb: "how would i", op: "OP:procedure" },
  { verb: "how can i", op: "OP:procedure" },
  { verb: "how do i", op: "OP:procedure" },
  { verb: "how to", op: "OP:procedure" },
  { verb: "search", op: "OP:search" },
  { verb: "find", op: "OP:search" },
  { verb: "lookup", op: "OP:lookup" },
  { verb: "look up", op: "OP:lookup" },
  { verb: "define", op: "OP:define" },
  { verb: "what is", op: "OP:define" },
  { verb: "who is", op: "OP:identify" },
  { verb: "explain", op: "OP:define" },
  { verb: "compute", op: "OP:compute" },
  { verb: "calculate", op: "OP:compute" },
  { verb: "hello", op: "OP:greet" },
  { verb: "hi", op: "OP:greet" },
  { verb: "goodbye", op: "OP:farewell" },
  { verb: "bye", op: "OP:farewell" },
  // Russian
  { verb: "найди", op: "OP:search" },
  { verb: "поищи", op: "OP:search" },
  { verb: "поиск", op: "OP:search" },
  { verb: "что такое", op: "OP:define" },
  { verb: "кто такой", op: "OP:identify" },
  { verb: "объясни", op: "OP:define" },
  { verb: "посчитай", op: "OP:compute" },
  { verb: "вычисли", op: "OP:compute" },
  { verb: "привет", op: "OP:greet" },
  { verb: "здравствуй", op: "OP:greet" },
  { verb: "пока", op: "OP:farewell" },
  { verb: "до свидания", op: "OP:farewell" },
  // Hindi
  { verb: "खोज", op: "OP:search" },
  { verb: "ढूंढ", op: "OP:search" },
  { verb: "क्या है", op: "OP:define" },
  { verb: "कौन है", op: "OP:identify" },
  { verb: "नमस्ते", op: "OP:greet" },
  { verb: "अलविदा", op: "OP:farewell" },
  // Chinese
  { verb: "搜索", op: "OP:search" },
  { verb: "查找", op: "OP:search" },
  { verb: "什么是", op: "OP:define" },
  { verb: "是谁", op: "OP:identify" },
  { verb: "你好", op: "OP:greet" },
  { verb: "再见", op: "OP:farewell" },
];

function exactFormalizationMatch(prompt, normalized) {
  const haystack = String(normalized || "").toLowerCase();
  const raw = String(prompt || "");
  const rawLower = String(prompt || "").toLowerCase();
  for (const { verb, op } of FORMALIZATION_VERBS) {
    if (haystack.startsWith(verb + " ") || haystack === verb) {
      return {
        op,
        verb,
        objectText: haystack === verb ? "" : normalized.slice(verb.length),
        interpretations: [],
      };
    }
    if (rawLower.startsWith(verb + " ") || rawLower === verb) {
      return {
        op,
        verb,
        objectText: rawLower === verb ? "" : raw.slice(verb.length),
        interpretations: [],
      };
    }
    if (haystack.includes(" " + verb + " ")) {
      return { op, verb, objectText: null, interpretations: [] };
    }
  }
  return null;
}

function fuzzyFormalizationMatch(prompt) {
  const matches = FORMALIZATION_VERBS
    .map((entry) => {
      const match = fuzzyPrefixMatch(prompt, entry.verb);
      return match ? Object.assign({ entry }, match) : null;
    })
    .filter(Boolean)
    .sort((left, right) =>
      left.typoCount - right.typoCount || right.end - left.end,
    );
  const best = matches[0];
  if (!best) return null;
  const peers = matches.filter(
    (match) => match.typoCount === best.typoCount && match.end === best.end,
  );
  if (peers.length > 1) {
    return {
      ambiguous: true,
      suggestions: peers.map((match) => match.entry.verb),
      interpretations: [],
    };
  }
  return {
    op: best.entry.op,
    verb: best.entry.verb,
    objectText: String(prompt || "").slice(best.end),
    interpretations: [best.interpretation],
  };
}

function detectFormalizationMatch(prompt, normalized) {
  return exactFormalizationMatch(prompt, normalized) || fuzzyFormalizationMatch(prompt);
}
