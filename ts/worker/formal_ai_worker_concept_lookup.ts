// Issue #1138 plan 01 browser mirror of `source_walk` + `concept_lookup`.
// Source choice, endpoints, extractors, licenses, and opt-outs come from the
// source registry. Adding a dictionary remains a data change plus an extractor;
// no host name is used to decide what captured bytes mean.

const CONCEPT_LOOKUP_BOUNDS = Object.freeze({
  maxDepth: 2,
  maxPagesPerService: 4,
  maxServices: 4,
  maxItems: 12,
});

function conceptCompact(value) {
  return String(value == null ? "" : value).split(/\s+/u).filter(Boolean).join(" ");
}

function conceptRespelling(candidate, headword) {
  const letters = (value) => String(value || "")
    .normalize("NFD")
    .replace(/[^\p{L}\p{N}]/gu, "")
    .toLowerCase();
  const normalized = letters(candidate);
  return normalized.length > 0 && normalized === letters(headword);
}

function conceptWiktionaryExtract(value, surface) {
  const out = [];
  const pages = value && value.query && value.query.pages;
  if (!pages || typeof pages !== "object") return out;
  for (const page of Object.values(pages)) {
    if (!page || Object.hasOwn(page, "missing") || typeof page.extract !== "string") continue;
    const lemma = String(page.title || surface);
    let seenHeading = false;
    for (const raw of page.extract.split(/\r?\n/u)) {
      const line = raw.trim();
      if (!line) continue;
      if (line.startsWith("=") && line.endsWith("=") && line.length > 1) {
        if (seenHeading) break;
        seenHeading = true;
        continue;
      }
      if (!seenHeading) continue;
      const gloss = conceptCompact(line);
      if (!gloss || conceptRespelling(gloss, lemma) || conceptRespelling(gloss, surface)) continue;
      out.push({ lemma, gloss, partOfSpeech: "", synonyms: [] });
    }
  }
  return out;
}

function conceptReadGlosses(extractor, text, surface) {
  let value;
  try {
    value = JSON.parse(String(text || "").trim());
  } catch {
    return [];
  }
  if (extractor === "wiktionary_entry_v1") {
    if (!Array.isArray(value)) return conceptWiktionaryExtract(value, surface);
    return value.flatMap((entry) => (entry.meanings || []).flatMap((meaning) =>
      (meaning.definitions || []).flatMap((definition) => {
        const gloss = conceptCompact(definition && definition.definition);
        return gloss ? [{
          lemma: String(entry.word || surface),
          gloss,
          partOfSpeech: String(meaning.partOfSpeech || ""),
          synonyms: Array.isArray(meaning.synonyms) ? meaning.synonyms.map(String) : [],
        }] : [];
      }),
    ));
  }
  if (extractor === "wordnet_sense_v1" && Array.isArray(value)) {
    return value.flatMap((synset) => (synset.definition || []).flatMap((definition) => {
      const gloss = conceptCompact(definition);
      return gloss ? [{
        lemma: surface,
        gloss,
        partOfSpeech: String(synset.partOfSpeech || ""),
        synonyms: (synset.members || [])
          .map((member) => String(member.lemma || ""))
          .filter((lemma) => lemma && lemma.toLowerCase() !== surface.toLowerCase()),
      }] : [];
    }));
  }
  if (extractor === "mediawiki_summary_v1") {
    const gloss = conceptCompact(value && value.extract);
    return gloss ? [{
      lemma: String(value.title || surface),
      gloss,
      partOfSpeech: "",
      synonyms: [],
    }] : [];
  }
  if (extractor === "wikidata_entity_v1") {
    return Object.entries((value && value.entities) || {}).flatMap(([id, entity]) =>
      Object.values((entity && entity.descriptions) || {}).flatMap((description) => {
        const gloss = conceptCompact(description && description.value);
        return gloss ? [{ lemma: String(entity.id || id), gloss, partOfSpeech: "", synonyms: [] }] : [];
      }),
    );
  }
  return [];
}

function conceptStableId(prefix, text) {
  const wasmId = typeof wasmStableId === "function" ? wasmStableId(prefix, text) : null;
  if (wasmId) return wasmId;
  let hash = 0xcbf29ce484222325n;
  const prime = 0x100000001b3n;
  const mask = 0xffffffffffffffffn;
  for (const byte of new TextEncoder().encode(String(text || ""))) {
    hash ^= BigInt(byte);
    hash = (hash * prime) & mask;
  }
  return `${prefix}_${hash.toString(16).padStart(16, "0")}`;
}

function conceptSense(source, capture, surface, language, raw, depth) {
  const identity = [capture.sha256, raw.lemma, language, raw.gloss].join("\u001f");
  return {
    contentId: conceptStableId("sense", identity),
    surface,
    lemma: raw.lemma,
    language,
    gloss: raw.gloss,
    partOfSpeech: raw.partOfSpeech,
    synonyms: raw.synonyms,
    sourceId: source.id,
    sourceUrl: capture.url,
    sha256: capture.sha256,
    fetchedAt: capture.fetchedAt,
    cached: Boolean(capture.cached),
    tier: source.tier,
    licenseName: source.licenseName,
    licenseUrl: source.licenseUrl,
    depth,
  };
}

async function lookupConceptSurface(surface, language = "en", preferences = {}, bounds) {
  const subject = String(surface || "").trim();
  const limits = { ...CONCEPT_LOOKUP_BOUNDS, ...(bounds || {}) };
  return sourceWalkSources("concept", subject, language, preferences, limits, {
    entryUrl: sourceWalkEntryUrl,
    emptyStatus: "no_items",
    stopAtMaxItems: true,
    read: (source, capture, depth) => ({
      items: conceptReadGlosses(source.extractor, capture.text, subject)
        .map((raw) => conceptSense(source, capture, subject, language, raw, depth)),
      follow: [],
    }),
  });
}

function conceptKnownSurfaces() {
  const known = new Set();
  if (typeof meaningLexicon !== "function") return known;
  for (const meaning of meaningLexicon()) {
    for (const word of meaning.words || []) known.add(normalizePrompt(word));
  }
  return known;
}

function conceptUnknownTokens(text) {
  const known = conceptKnownSurfaces();
  const out = [];
  for (const compound of String(text || "").match(/[\p{L}\p{N}_]+/gu) || []) {
    for (const token of compound.split("_")) {
      const folded = normalizePrompt(token);
      if (folded.length < 3 || known.has(folded) || out.includes(token)) continue;
      out.push(token);
    }
  }
  return out;
}

async function composeFromConcepts(prompt, preferences = {}) {
  const language = typeof detectLanguage === "function" ? detectLanguage(prompt) : "en";
  const senses = [];
  for (const token of conceptUnknownTokens(prompt)) {
    // eslint-disable-next-line no-await-in-loop -- stop at the first evidenced unknown concept.
    const outcome = await lookupConceptSurface(token, language, preferences);
    if (outcome.items.length > 0) {
      senses.push(...outcome.items);
      break;
    }
  }
  return {
    source: "// composition requires runtime verification\n",
    verified: false,
    senses,
    sources: Array.from(new Set(senses.map((sense) => sense.sourceUrl))),
  };
}

// ---------------------------------------------------------------------------
// Issue #1172 R1172-5: "What does X mean?" resolves through a dictionary
// source of the registry (Wiktionary), never through a canned paragraph.
// The frames that read the word, the registry kind that answers, and the
// per-edition section titles, example markers and annotation labels are
// `word_definition` rows of data/seed/prompt-patterns.lino. Mirrors
// rust/src/solver_handlers/word_definition.rs.
// ---------------------------------------------------------------------------

const WORD_DEFINITION_MAX_SENSES = 3;

/**
 * The `word_definition` prompt-pattern texts of one kind (and language).
 * @param {string} kind pattern kind
 * @param {string|null} language language filter, or null for all
 * @returns {Array<object>} {language, text} rows in seed order
 */
function wordDefinitionPatterns(kind, language) {
  return (typeof PROMPT_PATTERNS === "undefined" ? [] : PROMPT_PATTERNS)
    .filter((row) => row && row.intent === "word_definition" && row.kind === kind && row.text
      && (language === null || row.language === language))
    .map((row) => ({ language: String(row.language || "en"), text: String(row.text) }));
}

/**
 * The word a definition request asks about, read from the longest seeded
 * frame that encloses it (Rust `definition_term`).
 * @param {string} prompt raw prompt
 * @returns {{term: string, language: string}|null}
 */
function wordDefinitionTerm(prompt) {
  let lowered = String(prompt || "").toLowerCase().trim();
  while (lowered && "?？.。!！".includes(lowered[lowered.length - 1])) lowered = lowered.slice(0, -1).trimEnd();
  while (lowered && "¿¡".includes(lowered[0])) lowered = lowered.slice(1).trimStart();
  const frames = wordDefinitionPatterns("frame", null)
    .filter((row) => row.text.includes("{term}"))
    .sort((a, b) => b.text.length - a.text.length);
  for (const frame of frames) {
    const at = frame.text.indexOf("{term}");
    const before = frame.text.slice(0, at);
    const after = frame.text.slice(at + 6);
    if (lowered.length <= before.length + after.length) continue;
    if (!lowered.startsWith(before) || !lowered.endsWith(after)) continue;
    const term = lowered.slice(before.length, lowered.length - after.length).trim()
      .replace(/^["'`“”‘’«»]+|["'`“”‘’«»]+$/gu, "").trim();
    const words = term.split(/\s+/u).filter(Boolean);
    if (words.length === 0 || words.length > 3 || !/^[\p{L}\p{M}\s'-]+$/u.test(term)) continue;
    return { term: term, language: frame.language };
  }
  return null;
}

/**
 * Definitions stated by one dictionary capture: the Free Dictionary entry
 * array, or a MediaWiki extract read by its seeded definition sections
 * (Rust `read_senses`).
 * @param {string} text captured bytes
 * @param {string} term the word
 * @param {string} language edition language
 * @returns {Array<{pos: string, gloss: string}>}
 */
function wordDefinitionReadSenses(text, term, language) {
  let value;
  try {
    value = JSON.parse(String(text || "").trim());
  } catch {
    return [];
  }
  const out = [];
  if (Array.isArray(value)) {
    for (const entry of value) {
      for (const meaning of (entry && entry.meanings) || []) {
        for (const definition of (meaning && meaning.definitions) || []) {
          const gloss = conceptCompact(definition && definition.definition);
          if (gloss) out.push({ pos: String(meaning.partOfSpeech || ""), gloss: gloss });
        }
      }
    }
    return out;
  }
  const pages = value && value.query && value.query.pages;
  if (!pages || typeof pages !== "object") return out;
  const titles = wordDefinitionPatterns("definition_section", language).map((row) => row.text.toLowerCase());
  const markers = wordDefinitionPatterns("example_marker", language).map((row) => row.text);
  const labels = wordDefinitionPatterns("annotation_prefix", language).map((row) => row.text.toLowerCase());
  const inflections = wordDefinitionPatterns("inflection_marker", language).map((row) => row.text);
  for (const page of Object.values(pages)) {
    if (!page || typeof page.extract !== "string") continue;
    const lines = page.extract.split(/\r?\n/u).map((line) => line.trim());
    const isHeading = (line) => line.length > 1 && line.startsWith("=") && line.endsWith("=");
    const titleOf = (line) => line.replace(/^=+|=+$/gu, "").trim().toLowerCase();
    const sectioned = titles.length > 0 && lines.some((line) => isHeading(line)
      && titles.some((title) => titleOf(line).startsWith(title)));
    let inSection = false;
    let headings = 0;
    for (const line of lines) {
      if (isHeading(line)) {
        headings += 1;
        inSection = sectioned
          ? titles.some((title) => titleOf(line).startsWith(title))
          : headings === 1;
        continue;
      }
      if (!inSection || line === "" || /^[0-9]+$/u.test(line)) continue;
      if (labels.some((label) => line.toLowerCase().startsWith(label))) continue;
      if (inflections.some((marker) => line.includes(marker))) continue;
      let gloss = line;
      for (const marker of markers) {
        const cut = gloss.indexOf(marker);
        if (cut !== -1) gloss = gloss.slice(0, cut);
      }
      gloss = conceptCompact(gloss);
      if (!gloss || conceptRespelling(gloss, term)) continue;
      out.push({ pos: "", gloss: gloss });
    }
  }
  return out;
}

/**
 * Fill a localized `word_definition_*` template (English when the language
 * has none), as Rust `localized_response` does.
 * @param {string} intent template intent
 * @param {string} language response language
 * @param {object} values placeholder values
 * @returns {string} the rendered text
 */
function wordDefinitionRender(intent, language, values) {
  const table = MULTILINGUAL_ANSWERS[intent];
  const entry = table ? table[language] || table.en : null;
  let out = entry ? String(normalizeEntry(entry, intent).text || "") : "";
  for (const [name, value] of Object.entries(values)) out = out.split(`{${name}}`).join(String(value));
  return out;
}

/**
 * Answer a definition request from the registry's dictionary sources.
 * @param {string} prompt raw prompt
 * @param {object} preferences service settings
 * @returns {Promise<object|null>} the worker answer, or null to fall through
 */
async function tryWordDefinition(prompt, preferences = {}) {
  const request = wordDefinitionTerm(prompt);
  if (request === null) return null;
  // A seeded concept record answers first, as the native concept_lookup row does.
  if (typeof tryConceptLookup === "function" && tryConceptLookup(prompt)) return null;
  const kinds = wordDefinitionPatterns("source_kind", null).map((row) => row.text);
  const evidence = [`word_definition:term:${request.term}`];
  for (const record of sourceWalkRegistry()) {
    if (!kinds.includes(record.kind) || !sourceWalkServiceAllowed(preferences, record)) continue;
    const url = sourceWalkEntryUrl(record, request.term, request.language);
    if (!url) {
      evidence.push(`word_definition:unbound:${record.id}`);
      continue;
    }
    // eslint-disable-next-line no-await-in-loop -- dictionary sources are consulted in registry order.
    const capture = await sourceWalkFetchCapture(url);
    if (!capture.ok) {
      evidence.push(`word_definition:miss:${record.id}:${capture.error}`);
      continue;
    }
    const senses = wordDefinitionReadSenses(capture.text, request.term, request.language)
      .slice(0, WORD_DEFINITION_MAX_SENSES);
    if (senses.length === 0) {
      evidence.push(`word_definition:no_sense:${record.id}`);
      continue;
    }
    let rendered = "";
    senses.forEach((sense, index) => {
      rendered += wordDefinitionRender("word_definition_sense", request.language, {
        n: String(index + 1),
        pos: sense.pos ? `(${sense.pos}) ` : "",
        gloss: sense.gloss,
      });
    });
    const content = wordDefinitionRender("word_definition_answer", request.language, {
      term: request.term,
      source: record.name,
      senses: rendered,
      url: capture.url,
      sha256: String(capture.sha256 || "").slice(0, 16),
      license: record.licenseName,
    });
    evidence.push(`word_definition:source:${record.id}`, `source:${capture.url}`, "response:word_definition");
    return { intent: "concept_lookup", content: content, confidence: 0.85, evidence: evidence };
  }
  return null;
}
