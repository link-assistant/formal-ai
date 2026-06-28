// Worker module 11 of 21. Loaded by ../formal_ai_worker.js.
function relationConfig(relation) {
  return FACT_RELATIONS.find((entry) => entry.relation === relation) || null;
}

// Markers that flag the user wants a fresh (uncached) result. Detected in all
// four supported languages plus a couple of common English phrasings.
const FORCE_FRESH_MARKERS = [
  "fresh",
  "no cache",
  "no-cache",
  "without cache",
  "skip cache",
  "ignore cache",
  "refresh",
  "не из кэша",
  "не из кеша",
  "без кэша",
  "без кеша",
  "обнови",
  "свежий ответ",
  "свежие данные",
  "ताज़ा",
  "ताज़े",
  "बिना कैश",
  "नया जवाब",
  "刷新",
  "新鲜",
  "不要缓存",
  "不用缓存",
];

function shouldForceFresh(normalized, prompt) {
  const lowerPrompt = String(prompt || "").toLowerCase();
  return FORCE_FRESH_MARKERS.some(
    (marker) => normalized.includes(marker) || lowerPrompt.includes(marker),
  );
}

// Multilingual relation patterns. Each entry has a list of triggers that, when
// present in the normalized prompt, identify the relation. Subject extraction
// uses the `extract` regexes which capture the subject term verbatim from the
// original (un-normalized) prompt — that preserves Cyrillic/Devanagari/CJK
// scripts that the normalizer otherwise strips.
const FACT_QUESTION_PATTERNS = [
  {
    relation: "capital",
    // English
    extract: [
      /\bcapital\s+(?:city\s+)?of\s+(?:the\s+)?([^?.!,;:]+?)(?:[?.!,;:]|$)/i,
      /\b([^?.!,;:]+?)['’]s\s+capital\b/i,
      /\bwhich\s+city\s+is\s+(?:the\s+)?capital\s+of\s+([^?.!,;:]+?)(?:[?.!,;:]|$)/i,
      /\bwhich\s+city\s+is\s+([^?.!,;:]+?)['’]s\s+capital\b/i,
      // Russian: "столица России", "какова столица России",
      // "столицей какой страны является Москва" — only the first form is
      // resolved; the inverted form falls through to other handlers.
      /столица\s+([^?.!,;:]+?)(?:[?.!,;:]|$)/i,
      /какова\s+столица\s+([^?.!,;:]+?)(?:[?.!,;:]|$)/i,
      /какая\s+столица\s+([^?.!,;:]+?)(?:[?.!,;:]|$)/i,
      // Hindi: "X की राजधानी क्या है"
      /([^?.!,;:]+?)\s+की\s+राजधानी(?:\s+क्या\s+है)?(?:[?.!,;:]|$)/i,
      // Chinese: "X的首都" / "X的首都是什么"
      /([^?。.!!,,;:、]+?)的首都(?:是什么|是哪里|是哪个城市)?(?:[?。.!!,,;:、]|$)/i,
    ],
  },
  {
    relation: "population",
    extract: [
      /\bpopulation\s+of\s+(?:the\s+)?([^?.!,;:]+?)(?:[?.!,;:]|$)/i,
      /\bhow\s+many\s+people\s+(?:live|are\s+there)\s+in\s+([^?.!,;:]+?)(?:[?.!,;:]|$)/i,
      /\b([^?.!,;:]+?)['’]s\s+population\b/i,
      /население\s+([^?.!,;:]+?)(?:[?.!,;:]|$)/i,
      /какое\s+население\s+([^?.!,;:]+?)(?:[?.!,;:]|$)/i,
      /([^?.!,;:]+?)\s+की\s+(?:जनसंख्या|आबादी)(?:[?.!,;:]|$)/i,
      /([^?。.!!,,;:、]+?)的人口(?:是多少|有多少)?(?:[?。.!!,,;:、]|$)/i,
    ],
  },
  {
    relation: "currency",
    extract: [
      /\bcurrency\s+of\s+(?:the\s+)?([^?.!,;:]+?)(?:[?.!,;:]|$)/i,
      /\b([^?.!,;:]+?)['’]s\s+currency\b/i,
      /валюта\s+([^?.!,;:]+?)(?:[?.!,;:]|$)/i,
      /какая\s+валюта\s+в\s+([^?.!,;:]+?)(?:[?.!,;:]|$)/i,
      /([^?.!,;:]+?)\s+की\s+मुद्रा(?:[?.!,;:]|$)/i,
      /([^?。.!!,,;:、]+?)的(?:货币|貨幣)(?:是什么|是哪种)?(?:[?。.!!,,;:、]|$)/i,
    ],
  },
  {
    relation: "official_language",
    extract: [
      /\bofficial\s+language\s+of\s+(?:the\s+)?([^?.!,;:]+?)(?:[?.!,;:]|$)/i,
      /\bwhat\s+language\s+(?:do\s+they\s+speak|is\s+spoken)\s+in\s+([^?.!,;:]+?)(?:[?.!,;:]|$)/i,
      /государственный\s+язык\s+([^?.!,;:]+?)(?:[?.!,;:]|$)/i,
      /официальный\s+язык\s+([^?.!,;:]+?)(?:[?.!,;:]|$)/i,
      /([^?.!,;:]+?)\s+की\s+(?:राजभाषा|आधिकारिक\s+भाषा)(?:[?.!,;:]|$)/i,
      /([^?。.!!,,;:、]+?)的(?:官方语言|官方語言)(?:[?。.!!,,;:、]|$)/i,
    ],
  },
  {
    relation: "continent",
    extract: [
      /\bcontinent\s+(?:is\s+)?([^?.!,;:]+?)\s+(?:on|in)\b/i,
      /\bwhich\s+continent\s+is\s+([^?.!,;:]+?)\s+(?:on|in)\b/i,
      /на\s+каком\s+континенте\s+(?:находится|расположена|расположен)\s+([^?.!,;:]+?)(?:[?.!,;:]|$)/i,
      /([^?.!,;:]+?)\s+किस\s+महाद्वीप\s+में\s+है(?:[?.!,;:]|$)/i,
      /([^?。.!!,,;:、]+?)在哪个(?:大洲|洲)(?:[?。.!!,,;:、]|$)/i,
    ],
  },
  {
    relation: "author_of_book",
    extract: [
      /\bwho\s+wrote\s+([^?.!,;:]+?)(?:[?.!,;:]|$)/i,
      /\bwho\s+is\s+(?:the\s+)?author\s+of\s+([^?.!,;:]+?)(?:[?.!,;:]|$)/i,
      /\b(?:author|writer)\s+of\s+([^?.!,;:]+?)(?:[?.!,;:]|$)/i,
      /\bwho\s+was\s+([^?.!,;:]+?)\s+written\s+by\b/i,
      /кто\s+написал\s+([^?.!,;:]+?)(?:[?.!,;:]|$)/i,
      /кто\s+автор\s+([^?.!,;:]+?)(?:[?.!,;:]|$)/i,
      /автор\s+([^?.!,;:]+?)(?:[?.!,;:]|$)/i,
      /([^?.!,;:]+?)\s+के\s+लेखक\s+कौन(?:\s+हैं|\s+है)?(?:[?.!,;:]|$)/i,
      /([^?.!,;:]+?)\s+किसने\s+लिख(?:ा|ी)(?:[?.!,;:]|$)/i,
      /किसने\s+लिख(?:ा|ी)\s+([^?.!,;:]+?)(?:[?.!,;:]|$)/i,
      /([^?。.!!,,;:、]+?)的作者(?:是谁|是誰)?(?:[?。.!!,,;:、]|$)/i,
      /([^?。.!!,,;:、]+?)(?:是谁|是誰)写的(?:[?。.!!,,;:、]|$)/i,
      /(?:谁|誰)写(?:了)?([^?。.!!,,;:、]+?)(?:[?。.!!,,;:、]|$)/i,
    ],
  },
];

// Words/phrases that should be stripped from a captured subject before we
// hand it off to Wikidata. These are not part of the entity name — they leak
// from question prefixes the regex didn't consume (e.g. "the country called
// France" → "France"). Order matters: longer prefixes first.
const SUBJECT_TRIM_PREFIXES = [
  "the country called ",
  "the country ",
  "country ",
  "the city of ",
  "the city ",
  "city of ",
  "country called ",
  "republic of ",
  "kingdom of ",
  "is ",
  "in ",
  "of the ",
  "of ",
  "страна ",
  "страны ",
  "стране ",
  "страну ",
];

function trimSubjectTerm(raw) {
  let value = String(raw || "")
    .replace(/[«»"'`“”„‟‹›]+/g, "")
    .replace(/\s+/g, " ")
    .trim();
  let changed = true;
  while (changed) {
    changed = false;
    const lower = value.toLowerCase();
    for (const prefix of SUBJECT_TRIM_PREFIXES) {
      if (lower.startsWith(prefix)) {
        value = value.slice(prefix.length).trim();
        changed = true;
        break;
      }
    }
  }
  return value;
}

function parseFactQuestion(prompt, normalized) {
  const text = String(prompt || "");
  if (!text.trim()) return null;
  for (const pattern of FACT_QUESTION_PATTERNS) {
    for (const regex of pattern.extract) {
      const match = regex.exec(text);
      if (!match) continue;
      const subjectTerm = trimSubjectTerm(match[1]);
      if (!subjectTerm) continue;
      // Reject single-letter or pure-punctuation captures so we don't fire
      // on noise like "x." or "?".
      if (subjectTerm.length < 2 && !/[Ѐ-鿿]/.test(subjectTerm)) {
        continue;
      }
      return {
        relation: pattern.relation,
        subjectTerm,
        language: detectLanguage(prompt),
        forceFresh: shouldForceFresh(normalized, prompt),
      };
    }
  }
  return null;
}

// In-memory cache. Keyed by `relation:subject_normalized:language`. The TTL
// matches the user-requested 1 week. Pre-warmed from FACTS at init() so the
// offline test matrix sees the same starting cache the Rust solver does.
const FACT_QUERY_CACHE = new Map();
const FACT_QUERY_TTL_MS = 7 * 24 * 60 * 60 * 1000;

function factCacheKey(relation, subjectTerm, language) {
  return [
    String(relation || "").toLowerCase(),
    String(subjectTerm || "")
      .toLowerCase()
      .replace(/\s+/g, " ")
      .trim(),
    String(language || "en").toLowerCase(),
  ].join(":");
}

function factCacheGet(relation, subjectTerm, language) {
  const key = factCacheKey(relation, subjectTerm, language);
  const entry = FACT_QUERY_CACHE.get(key);
  if (!entry) return null;
  if (
    entry.expiresAt &&
    typeof entry.expiresAt === "number" &&
    entry.expiresAt < Date.now()
  ) {
    FACT_QUERY_CACHE.delete(key);
    return null;
  }
  return entry;
}

function factCachePut(relation, subjectTerm, language, value) {
  const key = factCacheKey(relation, subjectTerm, language);
  const ttl = typeof value.ttlMs === "number" ? value.ttlMs : FACT_QUERY_TTL_MS;
  const entry = Object.assign({}, value, {
    expiresAt: Date.now() + ttl,
  });
  FACT_QUERY_CACHE.set(key, entry);
  return entry;
}

// Pre-warm the runtime cache from the seed `facts.lino`. Each seed record can
// optionally declare `relation`, `subjectQid`, `valueQid`, plus per-language
// `subjectLabel`/`valueLabel`/`valueText` overrides — those are the structured
// cache anchors. The legacy fields (`summary`, `subjectAliases`,
// `questionKeywords`) remain in place for the `tryFactLookup` substring path.
function warmFactCacheFromSeed() {
  if (!Array.isArray(FACTS)) return;
  const languages = ["en", "ru", "hi", "zh"];
  for (const record of FACTS) {
    if (!record || !record.relation || !record.subjectAliases) continue;
    const localizedMap = new Map();
    if (Array.isArray(record.localized)) {
      for (const loc of record.localized) {
        if (loc && loc.language) localizedMap.set(loc.language, loc);
      }
    }
    for (const lang of languages) {
      const loc = localizedMap.get(lang) || localizedMap.get("en") || {};
      const summary =
        (loc && loc.summary) || record.summary || "";
      const source = (loc && loc.source) || record.source || "";
      const sourceKind =
        (loc && loc.sourceKind) || record.sourceKind || "wikipedia";
      const valueLabel = (loc && loc.valueLabel) || record.valueLabel || "";
      const subjectLabel =
        (loc && loc.subjectLabel) || record.subjectLabel || "";
      // The aliases for the subject language drive cache key lookup. For each
      // alias (already lowercased by seed_loader.js), pre-seed a cache entry.
      const aliases = Array.isArray(record.subjectAliases)
        ? record.subjectAliases
        : [];
      for (const alias of aliases) {
        if (!alias) continue;
        factCachePut(record.relation, alias, lang, {
          relation: record.relation,
          subjectTerm: alias,
          subjectLabel: subjectLabel || alias,
          subjectQid: record.subjectQid || "",
          valueLabel,
          valueQid: record.valueQid || "",
          summary,
          source,
          sourceKind,
          language: lang,
          fromSeed: true,
          ttlMs: FACT_QUERY_TTL_MS,
        });
      }
    }
  }
}

async function wikidataSearchEntity(term, language) {
  if (typeof fetch !== "function") return null;
  // Wikidata supports per-language search; English fallback ensures broad
  // recall even for non-Latin scripts.
  const ordered = [language, "en"].filter(
    (value, index, array) => value && array.indexOf(value) === index,
  );
  for (const lang of ordered) {
    const url = `${WIKIDATA_API}?action=wbsearchentities&format=json&origin=*&type=item&limit=5&language=${encodeURIComponent(
      lang,
    )}&search=${encodeURIComponent(term)}`;
    try {
      const response = await fetch(url, {
        headers: {
          accept: "application/json",
          "api-user-agent":
            "formal-ai-demo (https://github.com/link-assistant/formal-ai)",
        },
      });
      if (!response || !response.ok) continue;
      const data = await response.json();
      if (data && Array.isArray(data.search) && data.search.length > 0) {
        const hit = data.search[0];
        return {
          qid: hit.id,
          label: hit.label || term,
          description: hit.description || "",
          language: lang,
        };
      }
    } catch (_error) {
      // Try the next language.
    }
  }
  return null;
}

function wikidataConceptUrl(hit) {
  const id = hit && hit.id ? String(hit.id) : "";
  if (id) return `https://www.wikidata.org/wiki/${encodeURIComponent(id)}`;
  const conceptUri = hit && hit.concepturi ? String(hit.concepturi) : "";
  const qid = conceptUri.match(/Q\d+/);
  if (qid) return `https://www.wikidata.org/wiki/${qid[0]}`;
  return "https://www.wikidata.org/wiki/Wikidata:Main_Page";
}

function wikidataHitMatchesTerm(hit, term) {
  const target = normalizeLookupText(term);
  if (!target || !hit) return false;
  const candidates = [
    hit.label,
    hit.title,
    hit.match && hit.match.text,
    hit.display && hit.display.label && hit.display.label.value,
  ];
  if (Array.isArray(hit.aliases)) {
    candidates.push(...hit.aliases);
  }
  return candidates.some((candidate) => normalizeLookupText(candidate) === target);
}

async function fetchWikidataConceptSummary(term, language) {
  if (typeof fetch !== "function") return null;
  const ordered = [language, "en"].filter(
    (value, index, array) => value && array.indexOf(value) === index,
  );
  for (const lang of ordered) {
    const url = `${WIKIDATA_API}?action=wbsearchentities&format=json&origin=*&type=item&limit=5&language=${encodeURIComponent(
      lang,
    )}&search=${encodeURIComponent(term)}`;
    try {
      const response = await fetch(url, {
        headers: {
          accept: "application/json",
          "api-user-agent":
            "formal-ai-demo (https://github.com/link-assistant/formal-ai)",
        },
      });
      if (!response || !response.ok) continue;
      const data = await response.json();
      const hits = data && Array.isArray(data.search) ? data.search : [];
      const hit = hits.find((candidate) =>
        wikidataHitMatchesTerm(candidate, term),
      );
      if (!hit) continue;
      const display = hit.display || {};
      return {
        sourceKind: "wikidata",
        qid: hit.id || "",
        title:
          (display.label && display.label.value) ||
          hit.label ||
          (hit.match && hit.match.text) ||
          term,
        description:
          (display.description && display.description.value) ||
          hit.description ||
          "",
        url: wikidataConceptUrl(hit),
        language: lang,
      };
    } catch (_error) {
      // Try the next Wikidata language.
    }
  }
  return null;
}

function wiktionaryFallbackDescription(title, language) {
  if (language === "ru") {
    return `В Wiktionary есть словарная статья «${title}».`;
  }
  if (language === "zh") {
    return `Wiktionary 有“${title}”这个词条。`;
  }
  if (language === "hi") {
    return `Wiktionary में "${title}" के लिए शब्दकोश प्रविष्टि है।`;
  }
  return `Wiktionary has a dictionary entry for "${title}".`;
}

async function fetchWiktionaryEntry(term, language) {
  if (typeof fetch !== "function") return null;
  const ordered = [language, "en"].filter(
    (value, index, array) => value && array.indexOf(value) === index,
  );
  const target = normalizeLookupText(term);
  for (const lang of ordered) {
    const base = WIKTIONARY_SEARCH_HOSTS[lang] || WIKTIONARY_SEARCH_HOSTS.en;
    const url = `${base}?action=opensearch&search=${encodeURIComponent(
      term,
    )}&limit=5&format=json&origin=*`;
    try {
      const response = await fetch(url, {
        headers: {
          accept: "application/json",
          "api-user-agent":
            "formal-ai-demo (https://github.com/link-assistant/formal-ai)",
        },
      });
      if (!response || !response.ok) continue;
      const data = await response.json();
      if (!Array.isArray(data) || !Array.isArray(data[1])) continue;
      const titles = data[1];
      const descriptions = Array.isArray(data[2]) ? data[2] : [];
      const urls = Array.isArray(data[3]) ? data[3] : [];
      const index = titles.findIndex(
        (title) => normalizeLookupText(title) === target,
      );
      if (index < 0) continue;
      const title = titles[index] || term;
      return {
        sourceKind: "wiktionary",
        title,
        description:
          descriptions[index] || wiktionaryFallbackDescription(title, lang),
        url:
          urls[index] ||
          `https://${lang}.wiktionary.org/wiki/${encodeURIComponent(title)}`,
        language: lang,
      };
    } catch (_error) {
      // Try the next Wiktionary language.
    }
  }
  return null;
}

function renderExternalLookupContent(result, requestedTerm) {
  const humanUrl = humanizeUrl(result.url);
  const title = result.title || requestedTerm;
  const heading =
    requestedTerm && normalizeLookupText(requestedTerm) !== normalizeLookupText(title)
      ? `${requestedTerm}: ${title}`
      : title;
  const description = String(result.description || "").trim();
  const body = description ? `${heading}: ${description}` : `${heading}.`;
  return `${body}\n\nSource: [${humanUrl}](${result.url}) (${result.sourceKind}).`;
}

function externalLookupResponse(result, requestedTerm, rejectedSummary) {
  const humanUrl = humanizeUrl(result.url);
  const evidence = [
    `${result.sourceKind}_lookup:${result.qid || result.title}`,
    `source:${humanUrl}`,
    `language:${result.language}`,
  ];
  if (result.qid) evidence.push(`wikidata:${result.qid}`);
  if (rejectedSummary && rejectedSummary.title) {
    evidence.push(`wikipedia_lookup:rejected:${rejectedSummary.title}`);
  }
  return {
    intent: `${result.sourceKind}_lookup`,
    content: renderExternalLookupContent(result, requestedTerm),
    confidence: result.sourceKind === "wikidata" ? 0.82 : 0.75,
    evidence,
  };
}

async function tryTermKnowledgeFallback(term, language, rejectedSummary) {
  const wikidata = await fetchWikidataConceptSummary(term, language);
  if (wikidata) {
    return externalLookupResponse(wikidata, term, rejectedSummary);
  }
  const wiktionary = await fetchWiktionaryEntry(term, language);
  if (wiktionary) {
    return externalLookupResponse(wiktionary, term, rejectedSummary);
  }
  return null;
}

async function wikidataFetchEntityClaim(qid, property, language) {
  if (typeof fetch !== "function") return null;
  const url = `${WIKIDATA_API}?action=wbgetentities&format=json&origin=*&ids=${encodeURIComponent(
    qid,
  )}&props=claims%7Clabels%7Csitelinks&languages=${encodeURIComponent(
    language,
  )}%7Cen`;
  try {
    const response = await fetch(url, {
      headers: {
        accept: "application/json",
        "api-user-agent":
          "formal-ai-demo (https://github.com/link-assistant/formal-ai)",
      },
    });
    if (!response || !response.ok) return null;
    const data = await response.json();
    if (!data || !data.entities) return null;
    const entity = data.entities[qid];
    if (!entity) return null;
    const claims = (entity.claims || {})[property] || [];
    const subjectLabel =
      (entity.labels && (entity.labels[language] || entity.labels.en) || {})
        .value || "";
    const sitelinks = entity.sitelinks || {};
    return { claims, subjectLabel, sitelinks };
  } catch (_error) {
    return null;
  }
}

async function wikidataResolveLabel(qid, language) {
  if (typeof fetch !== "function") return null;
  const url = `${WIKIDATA_API}?action=wbgetentities&format=json&origin=*&ids=${encodeURIComponent(
    qid,
  )}&props=labels%7Csitelinks&languages=${encodeURIComponent(language)}%7Cen`;
  try {
    const response = await fetch(url, {
      headers: {
        accept: "application/json",
        "api-user-agent":
          "formal-ai-demo (https://github.com/link-assistant/formal-ai)",
      },
    });
    if (!response || !response.ok) return null;
    const data = await response.json();
    if (!data || !data.entities) return null;
    const entity = data.entities[qid];
    if (!entity) return null;
    const label =
      (entity.labels && (entity.labels[language] || entity.labels.en) || {})
        .value || "";
    const sitelinks = entity.sitelinks || {};
    return { label, sitelinks };
  } catch (_error) {
    return null;
  }
}

function wikipediaSitelinkUrl(sitelinks, language) {
  if (!sitelinks || typeof sitelinks !== "object") return "";
  const key = `${language}wiki`;
  const fallback = "enwiki";
  const entry = sitelinks[key] || sitelinks[fallback];
  if (!entry) return "";
  if (entry.url) return entry.url;
  if (entry.title) {
    const lang = sitelinks[key] ? language : "en";
    return `https://${lang}.wikipedia.org/wiki/${encodeURIComponent(
      String(entry.title).replace(/\s+/g, "_"),
    ).replace(/%2F/gi, "/")}`;
  }
  return "";
}

// Localized templates for rendering the final answer. The seed value is
// inserted via `{value}`; the subject is inserted via `{subject}`.
const FACT_RESPONSE_TEMPLATES = {
  capital: {
    en: "The capital of {subject} is {value}.",
    ru: "Столица {subject} — {value}.",
    hi: "{subject} की राजधानी {value} है।",
    zh: "{subject}的首都是{value}。",
  },
  population: {
    en: "The population of {subject} is approximately {value}.",
    ru: "Население {subject} составляет примерно {value}.",
    hi: "{subject} की जनसंख्या लगभग {value} है।",
    zh: "{subject}的人口约为 {value}。",
  },
  currency: {
    en: "The currency of {subject} is the {value}.",
    ru: "Валюта {subject} — {value}.",
    hi: "{subject} की मुद्रा {value} है।",
    zh: "{subject}的货币是{value}。",
  },
  official_language: {
    en: "The official language of {subject} is {value}.",
    ru: "Государственный язык {subject} — {value}.",
    hi: "{subject} की राजभाषा {value} है।",
    zh: "{subject}的官方语言是{value}。",
  },
  continent: {
    en: "{subject} is located on the continent of {value}.",
    ru: "{subject} расположена на континенте {value}.",
    hi: "{subject} {value} महाद्वीप पर स्थित है।",
    zh: "{subject}位于{value}。",
  },
  author_of_book: {
    en: "{subject} was written by {value}.",
    ru: "Автор произведения «{subject}»: {value}.",
    hi: "{subject} को {value} ने लिखा था।",
    zh: "《{subject}》由{value}创作。",
  },
  area: {
    en: "The area of {subject} is approximately {value}.",
    ru: "Площадь {subject} составляет примерно {value}.",
    hi: "{subject} का क्षेत्रफल लगभग {value} है।",
    zh: "{subject}的面积约为 {value}。",
  },
  head_of_state: {
    en: "The head of state of {subject} is {value}.",
    ru: "Глава государства {subject} — {value}.",
    hi: "{subject} के राष्ट्राध्यक्ष {value} हैं।",
    zh: "{subject}的国家元首是{value}。",
  },
  head_of_government: {
    en: "The head of government of {subject} is {value}.",
    ru: "Глава правительства {subject} — {value}.",
    hi: "{subject} के सरकार के प्रमुख {value} हैं।",
    zh: "{subject}的政府首脑是{value}。",
  },
};

function renderFactSummary(relation, subjectLabel, valueLabel, language) {
  const templates =
    FACT_RESPONSE_TEMPLATES[relation] || FACT_RESPONSE_TEMPLATES.capital;
  const template = templates[language] || templates.en;
  return template
    .replace("{subject}", subjectLabel || "")
    .replace("{value}", valueLabel || "");
}

function factQueryEvidence(record, language) {
  const evidence = [
    `fact_query:relation:${record.relation}`,
    `fact_query:subject:${record.subjectLabel || record.subjectTerm}`,
    `language:${language}`,
  ];
  if (record.subjectQid) evidence.push(`wikidata:${record.subjectQid}`);
  if (record.valueQid) evidence.push(`wikidata:${record.valueQid}`);
  if (record.source) evidence.push(`source:${humanizeUrl(record.source)}`);
  if (record.fromSeed) evidence.push("fact_query:cache:seed");
  else if (record.fromCache) evidence.push("fact_query:cache:hit");
  else evidence.push("fact_query:cache:miss");
  return evidence;
}

async function resolveFactQueryViaWikidata(query, log) {
  // Stage 1: subject resolution via wbsearchentities.
  if (log) log.push(`fact_query:wbsearchentities:request:${query.subjectTerm}`);
  const subject = await wikidataSearchEntity(query.subjectTerm, query.language);
  if (!subject) {
    if (log) log.push("fact_query:wbsearchentities:miss");
    return null;
  }
  if (log) log.push(`fact_query:wbsearchentities:resolved:${subject.qid}`);

  const config = relationConfig(query.relation);
  if (!config) return null;

  // Stage 2: claim fetch via wbgetentities.
  if (log) log.push(`fact_query:wbgetentities:request:${config.property}`);
  const claimData = await wikidataFetchEntityClaim(
    subject.qid,
    config.property,
    query.language,
  );
  if (!claimData || !claimData.claims || claimData.claims.length === 0) {
    if (log) log.push("fact_query:wbgetentities:no_claim");
    return null;
  }
  const claim = claimData.claims[0];
  const mainsnak = claim && claim.mainsnak;
  if (!mainsnak || !mainsnak.datavalue) {
    if (log) log.push("fact_query:wbgetentities:no_datavalue");
    return null;
  }

  // Stage 3: value resolution.
  let valueLabel = "";
  let valueQid = "";
  if (config.valueType === "entity") {
    const value = mainsnak.datavalue.value || {};
    valueQid = value.id || "";
    if (!valueQid) {
      if (log) log.push("fact_query:wbgetentities:value_not_entity");
      return null;
    }
    if (log) log.push(`fact_query:label_resolve:request:${valueQid}`);
    const labelResult = await wikidataResolveLabel(valueQid, query.language);
    if (!labelResult || !labelResult.label) {
      if (log) log.push("fact_query:label_resolve:miss");
      return null;
    }
    valueLabel = labelResult.label;
    if (log) log.push(`fact_query:label_resolve:${valueLabel}`);
    // Capture the Wikipedia sitelink for the value entity as the canonical
    // evidence source — that's the human-readable artifact users can verify.
    const url =
      wikipediaSitelinkUrl(labelResult.sitelinks, query.language) ||
      wikipediaSitelinkUrl(claimData.sitelinks, query.language);
    return {
      relation: query.relation,
      subjectTerm: query.subjectTerm,
      subjectLabel: claimData.subjectLabel || subject.label,
      subjectQid: subject.qid,
      valueLabel,
      valueQid,
      summary: renderFactSummary(
        query.relation,
        claimData.subjectLabel || subject.label,
        valueLabel,
        query.language,
      ),
      source: url,
      sourceKind: "wikidata",
      language: query.language,
      fromCache: false,
      fromSeed: false,
    };
  }

  // Quantity values (population, area) are not Q-IDs.
  const value = mainsnak.datavalue.value || {};
  const rawAmount = String(value.amount || "").replace(/^\+/, "");
  if (!rawAmount) {
    if (log) log.push("fact_query:wbgetentities:value_empty");
    return null;
  }
  valueLabel = rawAmount;
  if (log) log.push(`fact_query:quantity:${valueLabel}`);
  const url = wikipediaSitelinkUrl(claimData.sitelinks, query.language);
  return {
    relation: query.relation,
    subjectTerm: query.subjectTerm,
    subjectLabel: claimData.subjectLabel || subject.label,
    subjectQid: subject.qid,
    valueLabel,
    valueQid: "",
    summary: renderFactSummary(
      query.relation,
      claimData.subjectLabel || subject.label,
      valueLabel,
      query.language,
    ),
    source: url,
    sourceKind: "wikidata",
    language: query.language,
    fromCache: false,
    fromSeed: false,
  };
}

async function tryFactQuery(prompt, normalized, preferences) {
  const query = parseFactQuestion(prompt, normalized);
  if (!query) return null;

  // Trace events: every step of the reasoning pipeline is recorded so the
  // browser memory log shows the structured query, the cache decision, and
  // any Wikidata calls.
  const trace = [];
  trace.push(`fact_query:request:${prompt}`);
  trace.push(`fact_query:relation:${query.relation}`);
  trace.push(`fact_query:subject:${query.subjectTerm}`);
  trace.push(`fact_query:language:${query.language}`);
  if (query.forceFresh) trace.push("fact_query:force_fresh");

  // Stage 1: cache check (skipped when the user asked for fresh data).
  if (!query.forceFresh) {
    trace.push("fact_query:cache:check");
    const cached = factCacheGet(
      query.relation,
      query.subjectTerm,
      query.language,
    );
    if (cached) {
      trace.push(`fact_query:cache:hit:${cached.fromSeed ? "seed" : "runtime"}`);
      const evidence = factQueryEvidence(
        Object.assign({}, cached, { fromCache: true }),
        query.language,
      );
      return {
        intent: "fact_query",
        content: cached.summary,
        confidence: 0.92,
        evidence,
        trace,
        formalizedObject: cached.subjectQid || "",
      };
    }
    trace.push("fact_query:cache:miss");
  } else {
    trace.push("fact_query:cache:bypass");
  }

  // Stage 2: Wikidata resolution.
  const resolved = await resolveFactQueryViaWikidata(query, trace);
  if (!resolved) {
    trace.push("fact_query:wikidata:no_match");
    return null;
  }

  // Stage 3: cache the resolution.
  factCachePut(query.relation, query.subjectTerm, query.language, resolved);
  trace.push(`fact_query:cache:store:${factCacheKey(
    query.relation,
    query.subjectTerm,
    query.language,
  )}`);

  trace.push(`fact_query:response:${resolved.summary}`);
  return {
    intent: "fact_query",
    content: resolved.summary,
    confidence: 0.92,
    evidence: factQueryEvidence(resolved, query.language),
    trace,
    formalizedObject: resolved.subjectQid || "",
  };
}

async function tryWikipediaLookup(prompt, language, preferences) {
  const query = extractConceptQuery(prompt);
  if (!query) return null;
  // Avoid hitting the network for terms that already resolved in CONCEPTS;
  // that path is handled by `tryConceptLookup`. We try the full
  // `(term, context)` query first so that "what is iir in ml" doesn't waste
  // a network call when a context-aware record exists.
  if (lookupConceptQuery(query)) return null;
  // Pass the original-case term to Wikipedia: non-Latin scripts (e.g. Cyrillic
  // for "Илон Маск") require correct capitalization in the REST URL because
  // ru.wikipedia.org does not redirect the all-lowercase slug.
  const wikiTerm = query.termOriginal || query.term;
  const wikiContext = query.contextOriginal || query.context;
  const summary = await fetchWikipediaSummary(wikiTerm, language, wikiContext, {
    includeDefinitionDisambiguation: !wikiContext,
  });
  if (!summary) {
    return tryTermKnowledgeFallback(wikiTerm, language, null);
  }
  const isClosestMatch = isClosestWikipediaMatch(summary);
  const requiresPlausibleSearchMatch =
    isClosestMatch || summary.matchKind === "context_search";
  if (
    requiresPlausibleSearchMatch &&
    !isPlausibleWikipediaSearchMatch(summary, wikiTerm)
  ) {
    const fallback = await tryTermKnowledgeFallback(wikiTerm, language, summary);
    if (fallback) return fallback;
    return null;
  }
  const guessProbability = numericPreference(
    preferences && preferences.guessProbability,
    0.8,
    0,
    1,
  );
  const humanUrl = humanizeUrl(summary.url);
  const evidence = [
    `wikipedia_lookup:${summary.title}`,
    `source:${humanUrl}`,
    `language:${summary.language}`,
  ];
  if (wikiContext) evidence.push(`wikipedia_lookup:context:${wikiContext}`);
  if (isClosestMatch) {
    evidence.push(`wikipedia_lookup:closest_match:${summary.title}`);
  }
  if (summary.matchKind === "disambiguation") {
    const entryCount = Array.isArray(summary.disambiguationEntries)
      ? summary.disambiguationEntries.length
      : 0;
    evidence.push(`wikipedia_lookup:disambiguation:${summary.title}`);
    evidence.push(`wikipedia_lookup:disambiguation_entries:${entryCount}`);
    return {
      intent: "wikipedia_lookup",
      content: wikipediaDisambiguationMessage(summary, language),
      confidence: 0.84,
      evidence,
    };
  }
  if (isClosestMatch && guessProbability < 0.5) {
    evidence.push("ambiguity:ask");
    return {
      intent: "clarification",
      content: wikipediaClarificationMessage(summary, language),
      confidence: 0.65,
      evidence,
    };
  }
  const bodyLines = [
    `${summary.title}: ${summary.extract}\n\n` +
      `Source: [${humanUrl}](${summary.url}) (wikipedia).`,
  ];
  if (isClosestMatch) {
    bodyLines.push(closestMatchNote(summary, language));
    evidence.push("ambiguity:guess");
  }
  return {
    intent: "wikipedia_lookup",
    content: bodyLines.join("\n\n"),
    confidence: 0.85,
    evidence,
  };
}

async function tryWikipediaArticleQuestion(prompt, language, preferences) {
  const term = extractWikipediaArticleQuestionTerm(prompt);
  if (!term) return null;
  const query = refineWikipediaArticleQuestionLookup(term, language);
  if (!query.exactTerm) return null;

  const exactSummary = await fetchWikipediaSummary(query.exactTerm, language, null);
  let summary = exactSummary;
  const exactMatch = exactSummary && exactSummary.matchKind === "direct";
  if (!exactMatch && (query.lookupTerm !== query.exactTerm || query.contextOriginal)) {
    const refinedSummary = await fetchWikipediaSummary(
      query.lookupTerm,
      language,
      query.contextOriginal,
    );
    if (refinedSummary) summary = refinedSummary;
  }
  if (!summary) {
    return tryTermKnowledgeFallback(query.exactTerm, language, null);
  }
  if (!exactMatch && !isArticleQuestionWikipediaMatch(summary, query)) {
    const fallback = await tryTermKnowledgeFallback(
      query.exactTerm,
      language,
      summary,
    );
    if (fallback) return fallback;
    return null;
  }

  const guessProbability = numericPreference(
    preferences && preferences.guessProbability,
    0.8,
    0,
    1,
  );
  const humanUrl = humanizeUrl(summary.url);
  const evidence = [
    `wikipedia_article_question:${query.exactTerm}`,
    `source:${humanUrl}`,
    `language:${summary.language}`,
  ];
  if (query.lookupTerm !== query.exactTerm) {
    evidence.push(`wikipedia_article_question:lookup:${query.lookupTerm}`);
  }
  if (query.contextOriginal) {
    evidence.push(`wikipedia_article_question:context:${query.contextOriginal}`);
  }
  if (exactMatch) {
    evidence.push("wikipedia_article_question:exact");
  } else {
    evidence.push(`wikipedia_article_question:closest_match:${summary.title}`);
  }
  if (!exactMatch && guessProbability < 0.5) {
    evidence.push("ambiguity:ask");
    return {
      intent: "wikipedia_article_question",
      content: wikipediaClarificationMessage(summary, language),
      confidence: 0.65,
      evidence,
    };
  }
  if (!exactMatch) evidence.push("ambiguity:guess");
  return {
    intent: "wikipedia_article_question",
    content: wikipediaArticleQuestionMessage(summary, query, language, exactMatch),
    confidence: exactMatch ? 0.88 : 0.82,
    evidence,
    query: query.exactTerm,
    formalizedObject: summary.title,
  };
}

// Issue #386 software-authoring roles — mirror ROLE_SOFTWARE_AUTHORING_ACTION
// and ROLE_SOFTWARE_ARTIFACT_KIND in src/seed/meanings.rs. The surface words (in
// every supported language) live in data/seed/meanings.lino and
// data/seed/meanings-software-project.lino (loaded into MEANINGS_LINO);
// this module names no word in any single language.
const ROLE_SOFTWARE_AUTHORING_ACTION = "software_authoring_action";
const ROLE_SOFTWARE_ARTIFACT_KIND = "software_artifact_kind";
const ROLE_SOFTWARE_REQUIREMENT_CATEGORY = "software_requirement_category";
// Issue #386 software-delivery / language / game-tracker / approval roles —
// mirror the matching ROLE_* consts in src/seed/meanings.rs. Their surface
// words (every supported language) live in
// data/seed/meanings-software-project.lino (loaded into MEANINGS_LINO);
// this module names no word in any single language.
const ROLE_SOFTWARE_FEATURE = "software_feature";
const ROLE_SOFTWARE_DELIVERY_MODE = "software_delivery_mode";
const ROLE_SOFTWARE_IMPLEMENTATION_LANGUAGE = "software_implementation_language";
const ROLE_GAME_TRACKER_DOMAIN = "game_tracker_domain";
const ROLE_GAME_TRACKER_MECHANIC = "game_tracker_mechanic";
const ROLE_SOFTWARE_STEP_GRANULARITY = "software_step_granularity";
const ROLE_SOFTWARE_BASH_COMMAND = "software_bash_command";
const ROLE_SOFTWARE_APPROVAL_TRIGGER = "software_approval_trigger";

// Map a software-requirement-category meaning slug to its canonical category
// label. Mirrors requirement_category_label in
// src/solver_handlers/software_project.rs: recognition words live in the
// lexicon; the canonical category label lives here. A slug absent here is
// skipped rather than mislabelled.
const SOFTWARE_REQUIREMENT_CATEGORY_LABELS = {
  requirement_state_tracking: "state_tracking",
  requirement_data_exchange: "data_exchange",
  requirement_automation: "automation",
  requirement_validation: "validation",
  requirement_integration: "integration",
  requirement_user_interface: "user_interface",
  requirement_project_behavior: "project_behavior",
};

// Map a software_delivery_mode meaning slug to its canonical delivery-mode
// label. Mirrors DeliveryMode::from_slug in
// src/solver_handlers/software_project.rs: the lexicon owns the surface words;
// this resolver owns the slug→label mapping. The default (code_generation) has
// no slug — it is the fallback when no mode meaning is evidenced.
const SOFTWARE_DELIVERY_MODE_LABELS = {
  delivery_manual_instructions: "manual_instructions",
  delivery_immediate_execution: "immediate_execution",
  delivery_script_generation: "script_generation",
};

// Map a software_implementation_language meaning slug to its canonical target
// label. Mirrors implementation_language_from_slug in
// src/solver_handlers/software_project.rs. The default (typescript) has no slug.
const SOFTWARE_IMPLEMENTATION_LANGUAGE_LABELS = {
  language_python: "python",
  language_rust: "rust",
  language_javascript: "javascript",
};

// Map a software-artifact-kind meaning slug to its canonical English label.
// Mirrors artifact_label in src/solver_handlers/software_project.rs: the lexicon
// owns the surface words a prompt is matched against (every language); this
// resolver owns only the stable slug→label mapping. A slug absent here is
// skipped rather than mislabelled.
const SOFTWARE_ARTIFACT_LABELS = {
  artifact_browser_extension: "browser extension",
  artifact_command_line_tool: "command-line tool",
  artifact_github_action: "action",
  artifact_mobile_app: "mobile app",
  artifact_web_app: "web app",
  artifact_application: "application",
  artifact_extension: "extension",
  artifact_dashboard: "dashboard",
  artifact_scraper: "scraper",
  artifact_library: "library",
  artifact_website: "website",
  artifact_plugin: "plugin",
  artifact_service: "service",
  artifact_bot: "bot",
  artifact_app: "app",
  artifact_api: "API",
  artifact_sdk: "SDK",
  artifact_tool: "tool",
  artifact_mod: "mod",
};

// [surface, label] recognition table, sourced from the lexicon in declaration
// order. Mirrors artifact_surface_table in
// src/solver_handlers/software_project.rs.
function softwareArtifactTable() {
  const table = [];
  for (const meaning of meaningsWithRole(ROLE_SOFTWARE_ARTIFACT_KIND)) {
    const label = SOFTWARE_ARTIFACT_LABELS[meaning.slug];
    if (!label) continue;
    for (const surface of meaning.words) table.push([surface, label]);
  }
  return table;
}

// [surface, action-slug] recognition table for software-authoring verbs. The
// matched slug is stored verbatim as the request's `action`, so the verb is
// recognised in every language it is lexicalised in. Mirrors
// action_surface_table in src/solver_handlers/software_project.rs.
function softwareActionTable() {
  const table = [];
  for (const meaning of meaningsWithRole(ROLE_SOFTWARE_AUTHORING_ACTION)) {
    for (const surface of meaning.words) table.push([surface, meaning.slug]);
  }
  return table;
}

// Whether `character` is a "word character" for the recognition scan:
// alphanumeric but not CJK. Mirrors is_word_character in
// src/solver_handlers/software_project.rs — CJK scripts have no inter-word
// spaces so they match as substrings, while Latin/Cyrillic/Devanagari keep
// whole-token boundaries so a short surface like `апи` never matches inside
// `напиши`.
function isSoftwareWordCharacter(character) {
  return /[\p{Alphabetic}\p{Number}]/u.test(character) && !containsCjk(character);
}

function isSoftwareStartBoundary(value, index) {
  if (index === 0) return true;
  return !isSoftwareWordCharacter(value[index - 1]);
}

function isSoftwareEndBoundary(value, index) {
  if (index >= value.length) return true;
  return !isSoftwareWordCharacter(value[index]);
}

// Position-major scan: the surface appearing earliest in `normalized` wins,
// ties at one position broken by table order (prefix collisions like app vs
// application are resolved by the end-boundary check, not order). Returns the
// matched { surface, payload } or null. Mirrors scan_match in
// src/solver_handlers/software_project.rs.
function scanSoftwareSurface(normalized, table) {
  const text = String(normalized || "");
  for (let index = 0; index < text.length; index += 1) {
    if (!isSoftwareStartBoundary(text, index)) continue;
    for (const [surface, payload] of table) {
      if (surface && text.startsWith(surface, index)) {
        if (isSoftwareEndBoundary(text, index + surface.length)) {
          return { surface, payload };
        }
      }
    }
  }
  return null;
}

// Lowercased union of every surface word (all languages) of the meanings that
// carry ROLE_SOFTWARE_REQUIREMENT_CATEGORY. A clause containing any of them
// states a feature requirement. Mirrors requirement_marker_words in
// src/solver_handlers/software_project.rs — no hardcoded marker list; the
// vocabulary lives in data/seed/meanings-software-project.lino.
function requirementMarkerWords() {
  return wordsForRole(ROLE_SOFTWARE_REQUIREMENT_CATEGORY)
    .filter((word) => word.length > 0)
    .map((word) => word.toLowerCase());
}
