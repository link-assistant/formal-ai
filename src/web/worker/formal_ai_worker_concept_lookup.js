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
