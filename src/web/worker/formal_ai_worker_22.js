// Statement-level captured-search fusion mirror for issue #709. The worker
// reads the same Wikidata anchors, function words, negation cues, and source
// tiers as Rust; only structural role names and language codes live in code.

const SEARCH_FUSION_ENTITY_ROLE = "wikidata_entity_anchor";
const SEARCH_FUSION_RELATION_ROLE = "binary_relation_property";
const SEARCH_FUSION_FUNCTION_WORD_ROLE = "statement_function_word";
const SEARCH_FUSION_NEGATION_ROLE = "statement_negation_cue";
const SEARCH_FUSION_MAX_MEANINGS = 3;
const SEARCH_FUSION_TIERS = new Map([
  ["original_first_party", 1.0],
  ["original_journalism", 0.85],
  ["independent_corroboration", 0.5],
  ["unoriginal", 0.0],
]);

function searchFusionTier(raw) {
  const slug = String(raw || "").toLowerCase();
  return SEARCH_FUSION_TIERS.has(slug) ? slug : "independent_corroboration";
}

function searchFusionSurface(value) {
  return normalizePrompt(String(value || "").replace(/[.!?。！？।॥]+$/gu, ""))
    .replace(/\s+/gu, " ").trim();
}

function searchFusionSentences(value) {
  const sentences = [];
  let buffer = "";
  for (const character of String(value || "")) {
    buffer += character;
    if (/[.!?。！？।॥\n]/u.test(character)) {
      if (buffer.trim()) sentences.push(buffer.trim());
      buffer = "";
    }
  }
  if (buffer.trim()) sentences.push(buffer.trim());
  return sentences;
}

function searchFusionVocabulary(role) {
  const out = new Set();
  for (const meaning of meaningsWithRole(role)) {
    for (const word of meaning.words || []) {
      const normalized = searchFusionSurface(word);
      for (const token of normalized.match(/[\p{L}\p{N}]+/gu) || []) out.add(token);
    }
  }
  return out;
}

function searchFusionEntity(surface, language) {
  const wanted = searchFusionSurface(surface);
  if (!wanted) return null;
  return meaningsWithRole(SEARCH_FUSION_ENTITY_ROLE).find((meaning) =>
    (meaning.lexemes || []).some((lexeme) =>
      lexeme.language === language && lexeme.words.some(
        (word) => searchFusionSurface(word) === wanted,
      ),
    ),
  ) || null;
}

function searchFusionRelation(text, language) {
  const normalized = ` ${searchFusionSurface(text)} `;
  const matches = [];
  for (const meaning of meaningsWithRole(SEARCH_FUSION_RELATION_ROLE)) {
    for (const lexeme of meaning.lexemes || []) {
      if (lexeme.language !== language) continue;
      for (const surface of lexeme.words || []) {
        const relation = searchFusionSurface(surface);
        const needle = ` ${relation} `;
        const at = normalized.indexOf(needle);
        if (relation && at >= 0) matches.push({ meaning, relation, at });
      }
    }
  }
  matches.sort((left, right) => right.relation.length - left.relation.length);
  const match = matches[0];
  if (!match) return null;
  const body = normalized.trim();
  const at = body.indexOf(match.relation);
  return {
    meaning: match.meaning,
    subject: body.slice(0, at).trim(),
    object: body.slice(at + match.relation.length).trim(),
  };
}

function searchFusionTargetWord(meaning, language, predicate) {
  const lexeme = (meaning.lexemes || []).find((item) => item.language === language);
  if (!lexeme || !lexeme.words.length) return null;
  if (predicate) {
    const grammatical = lexeme.words.find((word) =>
      (meaning.wordForms || []).some((form) => form.text === word && form.action),
    );
    if (grammatical) return grammatical;
  }
  return lexeme.words[0] || null;
}

function searchFusionFormalize(text, sourceLanguage, targetLanguage) {
  const language = sourceLanguage || detectLanguageSlug(text) || "en";
  const negations = searchFusionVocabulary(SEARCH_FUSION_NEGATION_ROLE);
  const tokens = searchFusionSurface(text).match(/[\p{L}\p{N}]+/gu) || [];
  const denied = tokens.some((token) => negations.has(token));
  const withoutNegation = tokens.filter((token) => !negations.has(token)).join(" ");
  const relation = searchFusionRelation(withoutNegation, language);
  if (relation) {
    const subject = searchFusionEntity(relation.subject, language);
    const object = searchFusionEntity(relation.object, language);
    if (subject && object && subject.groundedIn && object.groundedIn && relation.meaning.groundedIn) {
      const semantic = [
        `subject=wikidata:${subject.groundedIn}`,
        `predicate=wikidata:${relation.meaning.groundedIn}`,
        `object=wikidata:${object.groundedIn}`,
      ];
      let rendered = text.trim();
      if (language !== targetLanguage) {
        const parts = [
          searchFusionTargetWord(subject, targetLanguage, false),
          searchFusionTargetWord(relation.meaning, targetLanguage, true),
          searchFusionTargetWord(object, targetLanguage, false),
        ];
        if (parts.every(Boolean)) rendered = capitalizeAsciiFirst(parts.join(" "));
      }
      if (!/[.!?。！？।॥]$/u.test(rendered)) rendered += ".";
      return {
        polarity: denied ? "denied" : "asserted",
        meaning: semantic.join("|"), semantic, rendered,
      };
    }
  }
  const ignored = searchFusionVocabulary(SEARCH_FUSION_FUNCTION_WORD_ROLE);
  for (const cue of negations) ignored.add(cue);
  const terms = tokens.filter((token) => !ignored.has(token)).sort();
  const unique = [...new Set(terms)];
  let rendered = text.trim();
  if (!/[.!?。！？।॥]$/u.test(rendered)) rendered += ".";
  return {
    polarity: denied ? "denied" : "asserted",
    meaning: unique.join(" "), semantic: [], rendered,
  };
}

function searchFusionSources(entries) {
  const sources = [];
  for (const entry of entries) {
    sources.push(entry);
    for (const alternate of entry.alternateUrls || []) sources.push(alternate);
  }
  return sources.filter((source) => source && source.url);
}

function searchFusionSourceFragments(source, query, targetLanguage, evidence) {
  const title = String(source.title || source.url).trim();
  let excerpt = String(source.excerpt || title).trim();
  const titledPrefix = `${title} - `;
  if (excerpt.startsWith(titledPrefix)) excerpt = excerpt.slice(titledPrefix.length).trim();
  const sourceLanguage = source.sourceLanguage || detectLanguageSlug(excerpt) || "en";
  const tier = searchFusionTier(source.sourceTier);
  const fragments = searchFusionSentences(excerpt);
  if (!fragments.length) fragments.push(title);
  return fragments.map((fragment) => {
    const formal = searchFusionFormalize(fragment, sourceLanguage, targetLanguage);
    evidence.push(
      `search_fusion:formalization:${source.url}:${formal.polarity}:${formal.meaning || "empty"}`,
    );
    if (tier === "unoriginal") evidence.push(`search_fusion:ignored:${source.url}:unoriginal`);
    return {
      source: {
        url: source.url, title, quote: fragment.trim(), readMore: source.url,
        language: sourceLanguage, tier,
        providers: Array.isArray(source.providers) ? source.providers : [],
      },
      formal,
      query,
    };
  });
}

function searchFusionPosterior(support, oppose) {
  if (oppose <= 0) return Math.min(1, 0.6 + 0.4 * Math.min(1, support));
  const total = support + oppose;
  return total <= 0 ? 0.5 : support / total;
}

function searchFusionNodes(entries, query, language, evidence) {
  const observations = searchFusionSources(entries).flatMap((source) =>
    searchFusionSourceFragments(source, query, language, evidence));
  const nodes = new Map();
  const trustedUrls = new Set();
  for (const observation of observations) {
    const weight = SEARCH_FUSION_TIERS.get(observation.source.tier) || 0;
    if (weight <= 0 || !observation.formal.meaning) continue;
    trustedUrls.add(observation.source.url);
    const key = `${observation.formal.polarity}:${observation.formal.meaning}`;
    let node = nodes.get(key);
    if (!node) {
      node = {
        key, polarity: observation.formal.polarity, meaning: observation.formal.meaning,
        semantic: observation.formal.semantic, text: observation.formal.rendered,
        sources: [], support: 0,
      };
      nodes.set(key, node);
    }
    if (!node.sources.some((source) => source.url === observation.source.url)) {
      if (node.sources.length) evidence.push(`search_fusion:merge:${key}:${observation.source.url}`);
      node.sources.push(observation.source);
      node.support += weight;
    }
  }
  for (const node of nodes.values()) {
    const opposite = nodes.get(
      `${node.polarity === "asserted" ? "denied" : "asserted"}:${node.meaning}`,
    );
    node.conflict = !!opposite;
    node.posterior = searchFusionPosterior(node.support, opposite ? opposite.support : 0);
    const authority = node.support / Math.max(1, node.sources.length);
    const coverage = node.sources.length / Math.max(1, trustedUrls.size);
    const agreement = node.support / Math.max(node.support, node.support + (opposite ? opposite.support : 0));
    node.weight = Math.floor((60 + 200 * coverage * authority * agreement) / 3);
    node.sources.sort((left, right) =>
      (SEARCH_FUSION_TIERS.get(right.tier) - SEARCH_FUSION_TIERS.get(left.tier)) ||
      left.url.localeCompare(right.url));
  }
  return [...nodes.values()].sort((left, right) =>
    (right.weight - left.weight) || (right.sources.length - left.sources.length) ||
    left.key.localeCompare(right.key));
}

function fuseBrowserSearchStatements(entries, query, language, texts, evidence, rrfK) {
  const ranked = searchFusionNodes(entries, query, language || "en", evidence);
  const selectedMeanings = [];
  const selected = ranked.filter((node) => {
    const seen = selectedMeanings.includes(node.meaning);
    if (!seen && selectedMeanings.length >= SEARCH_FUSION_MAX_MEANINGS) return false;
    if (!seen) selectedMeanings.push(node.meaning);
    return true;
  });
  const lines = [texts.header(query, selected.length, rrfK), ""];
  selected.forEach((node, index) => {
    evidence.push(`search_fusion:rank:${index + 1}:${node.key}:${node.weight}`);
    if (node.conflict) evidence.push(`conflict:source_disagreement:${node.key}`);
    lines.push(`${index + 1}. ${node.text}`);
    const tiers = node.sources.map((source) => source.tier).join("|");
    const conflict = node.conflict ? " conflict=source_disagreement" : "";
    lines.push(
      `   \`posterior=${node.posterior.toFixed(6)} source_count=${node.sources.length} ` +
      `source_tier=${tiers}${conflict}\``,
    );
    for (const source of node.sources) {
      const domain = extractDomain(source.url);
      lines.push(`   - **[${source.title}](${source.url})**${domain ? `  \`${domain}\`` : ""}`);
      lines.push(`     > ${source.quote}`);
      const tags = source.providers.map((provider) => `${provider.id}#${provider.rank}`).join(", ");
      lines.push(`     [${texts.readMore}](${source.readMore})${tags ? ` — _${texts.via} ${tags}_` : ""}`);
    }
    lines.push("");
  });
  while (lines.length && !lines[lines.length - 1]) lines.pop();
  return {
    lines,
    statements: selected.map((node) => ({
      id: node.key, text: node.text, semanticLinks: node.semantic,
      posterior: node.posterior, weight: node.weight, conflict: node.conflict,
      sources: node.sources,
    })),
  };
}
