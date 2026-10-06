// The recursive meta reasoner, JavaScript first (2026-10-06 meta-algorithm
// doctrine; VISION.md "Universal Problem-Solving Algorithm").
//
// Nothing here knows a task. Every word of a request starts as an Unknown.
// The loop grounds unknowns into hypotheses from four places, in this order:
// a definition the request itself gives, a chunk learned on an earlier turn,
// the documentation of the instruction set (data/seed/meta-reasoning.lino),
// and dictionary captures fetched through the source walk. A gloss is itself
// text, so its words are grounded by the same function one level deeper
// (impasse -> subgoal), bounded by depth and a cycle stack. The goal state is
// the request's own examples: programs are enumerated over the typed
// instruction set, shortest first (Occam), ordered by grounded evidence, and
// each candidate's difference from the goal is the number of failing
// examples. The first failing example is the counterexample that rejects it
// (CEGIS). The program the answer shows is the exact source the verifier ran.
// Every decision is a trace event; the derivation serialises to links
// notation.

const META_REASONING_FILE = "meta-reasoning.lino";
const META_BOUNDS = Object.freeze({
  groundDepth: 2,
  glossesPerWord: 3,
  specificGloss: 0.5,
  glossSupport: 0.2,
  programLength: 4,
  candidateBudget: 20000,
  lookupRounds: 3,
  lookupsPerRound: 4,
  rejectionsTraced: 5,
  measureLength: 3,
  requiredGroupSize: 2,
});

let metaSeedCache = null;
const metaLearnedChunks = new Map();
// Chunks learned since the app last persisted them (see metaTakeLearned).
const metaNewChunks = new Map();
const metaCompiled = new Map();

/**
 * The parsed seed: cues, grammatical words, affixes, primitives, probes and
 * response templates.
 * @returns {object}
 */
function metaSeed() {
  if (metaSeedCache) return metaSeedCache;
  const text = typeof SEED_RAW === "object" ? seedRawText(SEED_RAW, META_REASONING_FILE) : "";
  const seed = { cues: {}, grammatical: {}, affixes: {}, degrees: {}, primitives: [], combinators: [], filters: [], selectors: [], probes: {}, responses: {}, notes: {}, impasse: { intents: [], suffixes: [] } };
  if (!text) return seed;
  const root = parseLinoTree(text);
  for (const top of root.children) {
    for (const record of top.children) {
      const field = (name) => {
        const child = record.children.find((item) => item.name === name);
        return child ? child.value : "";
      };
      const fields = (name) => record.children.filter((item) => item.name === name).map((item) => item.value);
      const language = field("language");
      if (record.name === "cue") {
        const role = field("role");
        seed.cues[role] = seed.cues[role] || {};
        seed.cues[role][language] = fields("marker");
      } else if (record.name === "grammatical") {
        seed.grammatical[language] = new Set(fields("word"));
      } else if (record.name === "affix") {
        seed.affixes[language] = fields("ending").map((value) => {
          const parts = String(value).split(/\s+/u);
          return [parts[0], parts[1] === "\"\"" || parts[1] === undefined ? "" : parts[1]];
        });
      } else if (record.name === "degree") {
        seed.degrees[language] = seed.degrees[language] || [];
        seed.degrees[language].push({
          id: record.value,
          endings: fields("ending").map((value) => {
            const parts = String(value).split(/\s+/u);
            return [parts[0], parts[1] === "\"\"" || parts[1] === undefined ? "" : parts[1]];
          }),
          gloss: field("gloss"),
        });
      } else if (record.name === "primitive") {
        seed.primitives.push({
          id: record.value,
          from: field("from"),
          to: field("to"),
          doc: field("doc"),
          code: field("code"),
          infer: field("infer"),
          environment: field("environment"),
          effect: field("effect") === "true",
          takes: field("takes").split(/\s+/u).filter(Boolean),
        });
      } else if (record.name === "combinator") {
        seed.combinators.push({ id: record.value, doc: field("doc"), code: field("code") });
      } else if (record.name === "filter") {
        seed.filters.push({ id: record.value, doc: field("doc"), test: field("test"), measure: field("measure") || "number" });
      } else if (record.name === "selector") {
        seed.selectors.push({ id: record.value, doc: field("doc"), test: field("test") });
      } else if (record.name === "impasse") {
        seed.impasse.intents.push(...fields("intent"));
        seed.impasse.suffixes.push(...fields("suffix"));
      } else if (record.name === "probe") {
        seed.probes[field("type")] = (seed.probes[field("type")] || []).concat(fields("value"));
      } else if (record.name === "note") {
        seed.notes[record.value] = field("text");
      } else if (record.name === "response") {
        seed.responses[record.value] = seed.responses[record.value] || {};
        seed.responses[record.value][language] = field("text");
      }
    }
  }
  metaSeedCache = seed;
  return seed;
}

/**
 * One response template with its {slots} filled.
 * @param {string} id
 * @param {string} language
 * @param {object} slots
 * @returns {string}
 */
function metaResponse(id, language, slots) {
  const variants = metaSeed().responses[id] || {};
  let text = variants[language] || variants.en || "";
  for (const name of Object.keys(slots || {})) text = text.split(`{${name}}`).join(String(slots[name]));
  return text;
}

/**
 * One trace detail from its seeded template (`note` records): the
 * derivation's wording is data, like every answer's.
 * @param {string} id
 * @param {object} slots
 * @returns {string}
 */
function metaNote(id, slots) {
  const template = metaSeed().notes[id];
  if (template === undefined) return id;
  let text = template;
  for (const name of Object.keys(slots || {})) text = text.split(`{${name}}`).join(String(slots[name]));
  return text;
}

/**
 * The trace recorder: numbered events in the order decisions were taken.
 * @returns {{events: Array<object>, emit: function}}
 */
function metaTrace() {
  const events = [];
  return {
    events,
    emit(kind, detail, data) {
      events.push({ seq: events.length + 1, kind, detail: String(detail), data: data || null });
    },
  };
}

/**
 * Lowercased word tokens of a text.
 * @param {string} text
 * @returns {Array<string>}
 */
function metaWords(text) {
  return (String(text || "").toLowerCase().match(/[\p{L}\p{N}_]+/gu) || []);
}

/**
 * A word and the base forms its seeded inflection endings yield.
 * @param {string} word
 * @param {string} language
 * @returns {Array<string>}
 */
function metaLemmas(word, language) {
  const out = [word];
  const affixes = metaSeed().affixes[language] || metaSeed().affixes.en || [];
  for (const pair of affixes) {
    const ending = pair[0];
    if (word.length > ending.length + 2 && word.endsWith(ending)) {
      const base = word.slice(0, word.length - ending.length) + pair[1];
      if (!out.includes(base)) out.push(base);
    }
  }
  return out;
}

/**
 * True when the word is a closed-class (grammatical) word in any language.
 * @param {string} word
 * @returns {boolean}
 */
function metaIsGrammatical(word) {
  const sets = metaSeed().grammatical;
  for (const language of Object.keys(sets)) {
    if (sets[language].has(word)) return true;
  }
  return /^\p{N}+$/u.test(word);
}

/**
 * The cue markers of one role across every language.
 * @param {string} role
 * @returns {Array<string>}
 */
function metaCueMarkers(role) {
  const byLanguage = metaSeed().cues[role] || {};
  const out = [];
  for (const language of Object.keys(byLanguage)) {
    for (const marker of byLanguage[language]) if (!out.includes(marker)) out.push(marker);
  }
  return out;
}

/**
 * The value literals of a request with their positions: quoted strings,
 * bracketed lists, paths and bare numbers.
 * @param {string} prompt
 * @returns {Array<{start: number, end: number, value: *}>}
 */
function metaValueLiterals(prompt) {
  const pattern = /(?<![\p{L}\p{N}])'([^'\n]*)'(?![\p{L}\p{N}])|"([^"\n]*)"|“([^”\n]*)”|«([^»\n]*)»|(\[[^\]\n]*\])|(?<![\p{L}\p{N}_./-])((?:\.{1,2}\/|\/)?[\p{L}\p{N}_.-]+(?:\/[\p{L}\p{N}_.-]+)+\/?)(?![\p{L}\p{N}_/])|(?<![\p{L}\p{N}.])(-?\d+(?:\.\d+)?)(?![\p{L}\p{N}])/gu;
  const out = [];
  for (const match of String(prompt).matchAll(pattern)) {
    let value;
    let kind = "value";
    if (match[6] !== undefined) {
      // A slash-separated name is a path ("js/worker"), unless every segment
      // is a grammatical word ("and/or").
      const segments = match[6].split("/").filter(Boolean);
      if (segments.every((segment) => metaIsGrammatical(segment.toLowerCase()))) continue;
      kind = "path";
      value = match[6];
    } else if (match[5] !== undefined) {
      try {
        value = JSON.parse(match[5].replace(/'/gu, "\""));
      } catch {
        continue;
      }
    } else if (match[7] !== undefined) {
      value = Number(match[7]);
    } else {
      value = match[1] ?? match[2] ?? match[3] ?? match[4];
    }
    out.push({ start: match.index, end: match.index + match[0].length, value, kind });
  }
  return out;
}

/**
 * Input/output examples: two adjacent values joined by an example marker.
 * @param {string} prompt
 * @param {Array<object>} literals
 * @returns {Array<{input: *, output: *, symbolic: boolean}>}
 */
function metaExamples(prompt, literals) {
  const markers = metaCueMarkers("example");
  const examples = [];
  for (let index = 0; index + 1 < literals.length; index += 1) {
    const between = ` ${prompt.slice(literals[index].end, literals[index + 1].start).toLowerCase()} `;
    if (between.length > 28) continue;
    const marker = markers.find((item) => between.includes(item));
    if (!marker) continue;
    // An arrow is unambiguous; a word marker ("into", "в") may be prose.
    examples.push({ input: literals[index].value, output: literals[index + 1].value, symbolic: !/\p{L}/u.test(marker) });
    index += 1;
  }
  return examples;
}

/**
 * True when a word is a question word: a surface of the seed lexicon's
 * `interrogative_opener` meaning, in any language.
 * @param {string} word
 * @returns {boolean}
 */
function metaIsInterrogative(word) {
  return typeof wordsForRole === "function" && wordsForRole("interrogative_opener").includes(word);
}

/**
 * Definitions the request gives itself: "<term> is <definition>".
 * @param {string} prompt
 * @returns {Array<{term: string, definition: string}>}
 */
function metaRequestDefinitions(prompt) {
  const markers = metaCueMarkers("definition");
  const out = [];
  for (const sentence of String(prompt).split(/[.;!?\n]+/u)) {
    const padded = ` ${sentence.trim()} `;
    for (const marker of markers) {
      const at = padded.toLowerCase().indexOf(marker);
      if (at < 0) continue;
      const head = metaWords(padded.slice(0, at)).filter((word) => !metaIsGrammatical(word));
      const definition = padded.slice(at + marker.length).trim();
      // A question word ("what is ...") asks for the unknown; it is never a
      // term the request defines.
      if (head.length === 1 && definition && !metaIsInterrogative(head[0])) out.push({ term: head[0], definition });
      break;
    }
  }
  return out;
}

/**
 * Document frequency of every doc token over the instruction set, so a word
 * shared by most operations ("list", "returns") carries almost no evidence.
 * @returns {{operations: Array<object>, frequency: Map<string, number>}}
 */
function metaDocIndex() {
  const seed = metaSeed();
  if (seed.docIndex) return seed.docIndex;
  const operations = seed.primitives
    .map((primitive) => ({ id: primitive.id, kind: "primitive", text: `${primitive.id.replace(/_/gu, " ")} ${primitive.doc}` }))
    .concat(seed.combinators.map((combinator) => ({ id: combinator.id, kind: "combinator", text: `${combinator.id.replace(/_/gu, " ")} ${combinator.doc}` })))
    .concat(seed.filters.map((filter) => ({ id: filter.id, kind: "filter", text: `${filter.id.replace(/_/gu, " ")} ${filter.doc}` })))
    .concat(seed.selectors.map((selector) => ({ id: selector.id, kind: "selector", text: `${selector.id.replace(/_/gu, " ")} ${selector.doc}` })));
  const frequency = new Map();
  for (const operation of operations) {
    const forms = new Set();
    for (const word of metaWords(operation.text)) for (const lemma of metaLemmas(word, "en")) forms.add(lemma);
    operation.forms = forms;
    for (const form of forms) frequency.set(form, (frequency.get(form) || 0) + 1);
  }
  seed.docIndex = { operations, frequency };
  return seed.docIndex;
}

/**
 * Operations whose documentation shares a form with the word, weighted by
 * inverse document frequency.
 * @param {string} word
 * @param {string} language
 * @returns {Array<{operation: string, score: number}>}
 */
function metaDocHypotheses(word, language) {
  const index = metaDocIndex();
  const limit = Math.max(2, Math.floor(index.operations.length / 4));
  const scores = new Map();
  for (const lemma of metaLemmas(word, language)) {
    const frequency = index.frequency.get(lemma) || 0;
    if (frequency === 0 || frequency > limit || metaIsGrammatical(lemma)) continue;
    for (const operation of index.operations) {
      if (!operation.forms.has(lemma)) continue;
      // An operation's own name is stronger evidence than its prose.
      const named = operation.id.split("_").some((part) => metaLemmas(part, "en").includes(lemma)) ? 2 : 1;
      scores.set(operation.id, Math.max(scores.get(operation.id) || 0, named / frequency));
    }
  }
  return Array.from(scores, ([operation, score]) => ({ operation, score })).sort((a, b) => b.score - a.score || a.operation.localeCompare(b.operation));
}

/**
 * Ground one word: request definition, learned chunk, documentation, then the
 * glosses of its dictionary captures, whose own words are grounded one level
 * deeper. Returns hypotheses and the words a lookup would have to open.
 * @param {string} word
 * @param {object} context
 * @param {number} depth
 * @param {Array<string>} stack
 * @returns {{word: string, status: string, origin: string, hypotheses: Array<object>, needs: Array<string>}}
 */
function metaGround(word, context, depth, stack) {
  const trace = context.trace;
  const indent = depth > 0 ? `${"↳".repeat(depth)} ` : "";
  if (stack.includes(word)) {
    trace.emit("impasse", `${indent}cycle ${stack.concat(word).join(" → ")}`);
    return { word, status: "open", origin: "cycle", hypotheses: [], needs: [] };
  }
  const definition = context.definitions.find((item) => item.term === word);
  if (definition) {
    trace.emit("hypothesis", `${indent}${word} := "${definition.definition}" (request definition)`);
    const hypotheses = metaGroundText(definition.definition, context, depth + 1, stack.concat(word));
    return { word, status: hypotheses.length ? "grounded" : "defined", origin: "request", hypotheses, needs: [] };
  }
  for (const lemma of metaLemmas(word, context.language)) {
    const chunk = metaLearnedChunks.get(lemma);
    if (chunk) {
      trace.emit("recall", `${indent}${word} → ${chunk.operation} (learned from ${chunk.via})`);
      return { word, status: "grounded", origin: "chunk", hypotheses: [{ operation: chunk.operation, score: chunk.score, via: chunk.via }], needs: [] };
    }
  }
  const index = metaDocIndex();
  const structuralLimit = Math.max(2, Math.floor(index.operations.length / 4));
  if (metaLemmas(word, context.language).some((lemma) => (index.frequency.get(lemma) || 0) > structuralLimit)) {
    trace.emit("known", metaNote("structural", { indent, word }));
    return { word, status: "structural", origin: "documentation", hypotheses: [], needs: [] };
  }
  const documented = metaDocHypotheses(word, context.language);
  if (documented.length) {
    trace.emit("hypothesis", `${indent}${word} → ${documented.slice(0, 3).map((item) => `${item.operation}:${item.score.toFixed(2)}`).join(" | ")} (documentation)`);
    return { word, status: "grounded", origin: "documentation", hypotheses: documented.map((item) => ({ ...item, via: "documentation" })), needs: [] };
  }
  // A word in a degree of comparison is its stem's measure and the degree's
  // selection ("longest": long, superlative), read before its own entry.
  const degree = depth === 0 ? metaGroundDegree(word, context, stack, indent) : { grounding: null, needs: [] };
  if (degree.grounding) return degree.grounding;
  const senses = context.knowledge.get(word) || [];
  // Captures ground only the request's own words. A gloss word is grounded
  // from documentation and the request alone, so meaning cannot drift
  // through chains of loosely related senses.
  if (senses.length && depth === 0) {
    trace.emit("subgoal", `${indent}understand ${word} through ${senses.length} gloss(es)`);
    const frameMarkers = metaCueMarkers("artifact");
    // The word names the artifact's frame when a salient gloss is mostly
    // about it ("JavaScript: a programming language"), not when one word of
    // a long gloss happens to match.
    const frame = senses.slice(0, META_BOUNDS.glossesPerWord).find((sense) => {
      const content = metaWords(sense.gloss).filter((token) => !metaIsGrammatical(token));
      const framing = content.filter((token) => frameMarkers.some((marker) => token.startsWith(marker.trim()))).length;
      return content.length > 0 && framing / content.length >= META_BOUNDS.glossSupport;
    });
    if (frame) {
      trace.emit("grounded", metaNote("frame", { indent, word, gloss: frame.gloss }));
      return { word, status: "frame", origin: "capture", hypotheses: [], needs: [] };
    }
    const scores = new Map();
    const deeperNeeds = [];
    let via = "";
    for (const sense of senses.slice(0, META_BOUNDS.glossesPerWord)) {
      // Lesk-style support: the share of the gloss's content words whose
      // specific meaning (an operation's name or a rare documentation word)
      // is the operation. A synonym-like gloss ("Reversed.") vouches fully;
      // one incidental word in a long sentence barely does.
      const content = metaWords(sense.gloss).filter((token) => !metaIsGrammatical(token));
      if (!content.length) continue;
      const support = new Map();
      let grounded = 0;
      for (const token of new Set(content)) {
        const grounding = metaGround(token, context, depth + 1, stack.concat(word));
        for (const need of grounding.needs) if (!deeperNeeds.includes(need)) deeperNeeds.push(need);
        const specific = new Set(grounding.hypotheses.filter((item) => item.score >= META_BOUNDS.specificGloss).map((item) => item.operation));
        if (specific.size) grounded += 1;
        for (const operation of specific) support.set(operation, (support.get(operation) || 0) + 1);
      }
      for (const [operation, count] of support) {
        // A majority of the gloss's grounded words, and a real share of all
        // its words; words grounded nowhere are neutral, rivals dilute.
        const score = count / content.length;
        if (score < META_BOUNDS.glossSupport || count * 2 <= grounded) continue;
        if (score > (scores.get(operation) || 0)) {
          scores.set(operation, score);
          if (!via || score >= Math.max(...scores.values())) via = sense.sourceUrl || sense.gloss;
        }
      }
    }
    const hypotheses = Array.from(scores, ([operation, score]) => ({ operation, score, via })).sort((a, b) => b.score - a.score || a.operation.localeCompare(b.operation));
    if (hypotheses.length) {
      trace.emit("grounded", `${indent}${word} → ${hypotheses[0].operation} via ${via}`);
      return { word, status: "grounded", origin: "capture", hypotheses, needs: [] };
    }
    // A gloss that quotes a symbol ("The punctuation mark ( : )") defines
    // the word as that symbol: the word names a value, not an operation.
    const symbol = metaGlossSymbol(senses);
    if (symbol) {
      trace.emit("grounded", metaNote("symbol", { indent, word, value: JSON.stringify(symbol.value) }));
      return { word, status: "value", origin: "capture", hypotheses: [], value: symbol.value, via: symbol.via, needs: [] };
    }
    // Lazy grounding: a gloss word is worth a lookup only when no gloss of
    // its parent reached an operation.
    trace.emit("impasse", metaNote("no_gloss", { indent, word }));
    return { word, status: "open", origin: "capture", hypotheses: [], needs: deeperNeeds.concat(degree.needs.filter((need) => !deeperNeeds.includes(need))) };
  }
  if (depth > 0) return { word, status: "open", origin: "none", hypotheses: [], needs: [] };
  if (!context.knowledge.has(word)) {
    trace.emit("impasse", metaNote("lookup_needed", { word }));
    return { word, status: "open", origin: "none", hypotheses: [], needs: [word].concat(degree.needs) };
  }
  return { word, status: "open", origin: "none", hypotheses: [], needs: degree.needs };
}

/**
 * The symbol a dictionary gloss defines its word as: a token of the gloss's
 * first sentence that, stripped of enclosing brackets and quotation marks,
 * is one character that is neither a letter, a digit nor a space ("The
 * punctuation mark ( : )"). A symbol a later sentence mentions in passing
 * does not define the word. The first salient gloss defining one decides.
 * @param {Array<{gloss: string, sourceUrl?: string}>} senses
 * @returns {{value: string, via: string}|null}
 */
function metaGlossSymbol(senses) {
  for (const sense of senses.slice(0, META_BOUNDS.glossesPerWord)) {
    const head = String(sense.gloss || "").split(/[.;](?:\s|$)/u)[0];
    for (const token of head.split(/\s+/u)) {
      const core = token.replace(/^[()[\]{}"'“”‘’«»⟨⟩]+|[()[\]{}"'“”‘’«»⟨⟩]+$/gu, "");
      if (Array.from(core).length === 1 && !/[\p{L}\p{N}\s]/u.test(core)) return { value: core, via: sense.sourceUrl || sense.gloss };
    }
  }
  return null;
}

/**
 * A word in a degree of comparison ("longest" = long + superlative): its stem
 * names a measure, grounded like any word but kept only where it reaches an
 * operation to a number; the degree's seeded gloss ("the greatest") names the
 * selection by that measure. Returns the grounding, or the stems a lookup
 * would have to open.
 * @param {string} word
 * @param {object} context
 * @param {Array<string>} stack
 * @param {string} indent
 * @returns {{grounding: object|null, needs: Array<string>}}
 */
function metaGroundDegree(word, context, stack, indent) {
  const seed = metaSeed();
  const needs = [];
  for (const degree of seed.degrees[context.language] || seed.degrees.en || []) {
    const gloss = degree.gloss || ((seed.degrees.en || []).find((item) => item.id === degree.id) || {}).gloss || "";
    for (const [ending, replacement] of degree.endings) {
      if (!gloss || word.length <= ending.length + 2 || !word.endsWith(ending)) continue;
      const stem = word.slice(0, word.length - ending.length) + replacement;
      if (stem === word || metaIsGrammatical(stem) || stack.includes(stem)) continue;
      const measures = metaGroundMeasure(stem, context, stack.concat(word), indent);
      if (measures === null) {
        if (!needs.includes(stem)) needs.push(stem);
        continue;
      }
      if (!measures.length) continue;
      const hypotheses = metaGroundText(gloss, context, 1, stack.concat(word), []);
      if (!hypotheses.length) continue;
      context.trace.emit("hypothesis", metaNote("degree", { word, stem, degree: degree.id }));
      return { grounding: { word, status: "grounded", origin: "degree", hypotheses, measures, needs: [] }, needs: [] };
    }
  }
  return { grounding: null, needs };
}

/**
 * True when an operation is a measure: a parameter-free primitive from some
 * type to a number with no effect on the world.
 * @param {string} id
 * @returns {boolean}
 */
function metaIsMeasure(id) {
  const primitive = metaSeed().primitives.find((item) => item.id === id);
  return Boolean(primitive && primitive.to === "number" && !primitive.effect && !primitive.infer && !primitive.takes.length);
}

/**
 * The measures a word names: its documentation, or the words of its first
 * gloss that reaches one, kept only where they are measures. Null when the
 * word has neither and was not looked up yet.
 * @param {string} stem
 * @param {object} context
 * @param {Array<string>} stack
 * @param {string} indent
 * @returns {Array<object>|null}
 */
function metaGroundMeasure(stem, context, stack, indent) {
  const documented = metaDocHypotheses(stem, context.language).filter((item) => metaIsMeasure(item.operation));
  if (documented.length) return documented.map((item) => ({ ...item, via: "documentation" }));
  if (!context.knowledge.has(stem)) return null;
  for (const sense of context.knowledge.get(stem).slice(0, META_BOUNDS.glossesPerWord)) {
    const scores = new Map();
    for (const token of new Set(metaWords(sense.gloss).filter((item) => !metaIsGrammatical(item)))) {
      for (const hypothesis of metaGround(token, context, 1, stack.concat(stem)).hypotheses) {
        if (hypothesis.score < META_BOUNDS.specificGloss || !metaIsMeasure(hypothesis.operation)) continue;
        if (hypothesis.score > (scores.get(hypothesis.operation) || 0)) scores.set(hypothesis.operation, hypothesis.score);
      }
    }
    if (!scores.size) continue;
    const measures = Array.from(scores, ([operation, score]) => ({ operation, score, via: sense.sourceUrl || sense.gloss }))
      .sort((a, b) => b.score - a.score || a.operation.localeCompare(b.operation));
    context.trace.emit("grounded", metaNote("degree_measure", { indent: `${indent}↳ `, stem, operations: measures.map((item) => item.operation).join(" | ") }));
    return measures;
  }
  return [];
}

/**
 * Ground every content word of a text and merge the operation evidence.
 * @param {string} text
 * @param {object} context
 * @param {number} depth
 * @param {Array<string>} stack
 * @param {Array<string>} [needs] where lookups this text asks for go
 * @returns {Array<{operation: string, score: number, via: string}>}
 */
function metaGroundText(text, context, depth, stack, needs) {
  const scores = new Map();
  const sink = Array.isArray(needs) ? needs : context.needs;
  for (const word of metaWords(text)) {
    if (metaIsGrammatical(word)) continue;
    const grounding = metaGround(word, context, depth, stack);
    for (const need of grounding.needs) if (!sink.includes(need)) sink.push(need);
    for (const hypothesis of grounding.hypotheses) {
      const previous = scores.get(hypothesis.operation);
      if (!previous || hypothesis.score > previous.score) scores.set(hypothesis.operation, hypothesis);
    }
  }
  return Array.from(scores.values()).sort((a, b) => b.score - a.score || a.operation.localeCompare(b.operation));
}

/**
 * The environment one program step needs ("node" for the file system).
 * @param {object} step
 * @returns {string}
 */
function metaStepEnvironment(step) {
  if (step.rewrite) return "node";
  return step.primitive.environment || "";
}

/**
 * True when this runtime offers an environment a primitive needs.
 * @param {string} environment
 * @returns {boolean}
 */
function metaEnvironmentAvailable(environment) {
  if (environment === "node") return typeof require === "function";
  return !environment;
}

/**
 * True when an operation only changes representation between a text and the
 * list of its parts (split / join), which carries no ordering constraint.
 * @param {string} id
 * @returns {boolean}
 */
function metaIsView(id) {
  const primitive = metaSeed().primitives.find((item) => item.id === id);
  if (!primitive) return false;
  const pair = [primitive.from, primitive.to].sort().join(" ");
  return pair === "list_text text";
}

/**
 * The request's coordinated clauses, each with its head action (the first
 * grounded word's operations) and the type its object noun denotes: a plural
 * noun grounded only in representation changes names the list, a singular
 * one names the text.
 * @param {string} text
 * @param {Array<object>} groundings
 * @param {object} trace
 * @returns {Array<{head: Array<string>, others: Array<string>, objectType: string|null}>}
 */
function metaClauses(text, groundings, trace) {
  let parts = [` ${String(text).toLowerCase()} `];
  for (const marker of metaCueMarkers("sequence")) parts = parts.flatMap((part) => part.split(marker));
  const byWord = new Map(groundings.map((grounding) => [grounding.word, grounding]));
  const clauses = [];
  for (const part of parts) {
    const grounded = metaWords(part).map((word) => byWord.get(word)).filter((grounding) => grounding && grounding.hypotheses.length);
    if (!grounded.length) continue;
    const top = (grounding) => grounding.hypotheses.filter((item) => item.score >= grounding.hypotheses[0].score * 0.99).map((item) => item.operation);
    // The head is the first specific action: not a representation change,
    // and not a word tied across many operations (that one names data).
    const headAt = grounded.findIndex((grounding) => !top(grounding).every(metaIsView) && top(grounding).length <= META_BOUNDS.requiredGroupSize);
    if (headAt < 0) continue;
    const head = top(grounded[headAt]);
    const others = grounded
      .filter((grounding, position) => position !== headAt && top(grounding).length <= META_BOUNDS.requiredGroupSize)
      .flatMap(top)
      .filter((id) => !head.includes(id));
    let objectType = null;
    const noun = grounded.slice(headAt + 1).find((grounding) => top(grounding).every(metaIsView));
    if (noun) {
      const plural = metaLemmas(noun.word, "en").some((lemma) => lemma !== noun.word);
      const views = top(noun).map((id) => metaSeed().primitives.find((item) => item.id === id));
      const split = views.find((primitive) => primitive.from === "text");
      objectType = plural && split ? split.to : "text";
    }
    clauses.push({ head, others, objectType });
    trace.emit("clause", `head {${head.join("|")}}${objectType ? ` acting on ${objectType}` : ""}${others.length ? `; modifiers {${others.join("|")}}` : ""}`);
  }
  return clauses;
}

/**
 * The synchronous core: parse, open unknowns, ground them, plan and verify.
 * `knowledge` maps a word to dictionary senses already fetched.
 * @param {string} prompt
 * @param {string} language
 * @param {Map<string, Array<object>>} knowledge
 * @returns {object}
 */
function metaReasonCore(prompt, language, knowledge) {
  const trace = metaTrace();
  trace.emit("impulse", prompt);
  const literals = metaValueLiterals(prompt);
  const examples = metaExamples(prompt, literals);
  const definitions = metaRequestDefinitions(prompt);
  let unquoted = String(prompt);
  for (const literal of literals.slice().reverse()) unquoted = unquoted.slice(0, literal.start) + " " + unquoted.slice(literal.end);
  // A bare name after a locative cue that follows a file noun ("any file in
  // data") is a location operand, a path like "js/worker".
  const located = metaLocationOperands(unquoted, language);
  for (const operand of located.slice().reverse()) unquoted = unquoted.slice(0, operand.start) + " " + unquoted.slice(operand.end);
  const lowered = ` ${unquoted.toLowerCase()} `;
  const artifact = metaCueMarkers("artifact").some((marker) => lowered.includes(marker));
  const question = metaCueMarkers("question").find((marker) => lowered.includes(marker));
  // An imperative that opens with an operation its documentation names
  // ("replace every colon ...") and gives no data to act on asks for the
  // procedure itself, as an artifact request does.
  const opening = metaWords(unquoted)[0];
  const openingOperations = opening && !metaIsGrammatical(opening) ? metaDocHypotheses(opening, language) : [];
  const openingTop = openingOperations.filter((item) => item.score >= openingOperations[0].score * 0.99).map((item) => item.operation);
  const imperative = !artifact && !question && literals.every((literal) => literal.kind === "path")
    && openingTop.length > 0 && openingTop.length <= META_BOUNDS.requiredGroupSize && !openingTop.every(metaIsView);
  // Examples embedded in prose that asks for no artifact ("on the 18th at
  // 17:00") are not a specification: an arrow, an artifact request or bare
  // examples make the examples the goal.
  const prose = metaWords(unquoted).some((word) => !metaIsGrammatical(word));
  const specified = examples.length && (artifact || !prose || examples.some((example) => example.symbolic));
  const goal = specified ? "synthesize_from_examples" : artifact || imperative ? "synthesize_from_meaning" : question ? "explain" : "understand";
  trace.emit("formalize", metaNote("formalized", { goal, examples: examples.length, definitions: definitions.length }));
  if (imperative && !artifact && goal === "synthesize_from_meaning") trace.emit("evidence", metaNote("imperative", { word: opening }));
  for (const operand of located) trace.emit("evidence", metaNote("location", { word: operand.value, cue: operand.cue }));
  const context = { trace, definitions, knowledge, language, needs: [] };
  const artifactWords = metaCueMarkers("artifact");
  const contentWords = [];
  for (const word of metaWords(unquoted)) {
    if (metaIsGrammatical(word) || contentWords.includes(word)) continue;
    if (artifactWords.some((marker) => word.startsWith(marker.trim()))) continue;
    if (definitions.some((item) => item.term === word) && goal !== "explain") continue;
    contentWords.push(word);
  }
  trace.emit("unknowns", contentWords.join(", ") || "none");
  const groundings = goal === "understand" ? [] : contentWords.map((word) => metaGround(word, context, 0, []));
  for (const grounding of groundings) {
    for (const need of grounding.needs) if (!context.needs.includes(need)) context.needs.push(need);
  }
  for (const definition of definitions) {
    if (goal === "explain") continue;
    for (const hypothesis of metaGroundText(definition.definition, context, 1, [definition.term])) {
      groundings.push({ word: definition.term, status: "grounded", origin: "request", hypotheses: [hypothesis], needs: [] });
    }
  }
  const evidence = new Map();
  for (const grounding of groundings) {
    for (const hypothesis of grounding.hypotheses) evidence.set(hypothesis.operation, Math.max(evidence.get(hypothesis.operation) || 0, hypothesis.score));
  }
  const result = {
    goal,
    language,
    examples,
    definitions,
    unknowns: groundings.map((grounding) => ({ word: grounding.word, status: grounding.status, origin: grounding.origin, best: grounding.hypotheses[0] || null })),
    needs: context.needs.slice(),
    program: null,
    verification: null,
    status: "open",
    imperative,
    trace,
  };
  if (goal === "explain") {
    const defined = definitions.find((item) => contentWords.includes(item.term));
    if (defined) {
      result.status = "solved";
      result.explanation = defined;
      trace.emit("goal_achieved", metaNote("self_defined", { term: defined.term }));
    }
    return result;
  }
  if (goal === "synthesize_from_examples") {
    const found = metaSynthesizeFromExamples(examples, evidence, trace);
    if (found) {
      const source = metaRender(found.steps, found.parameter);
      const verification = metaVerify(source, examples);
      trace.emit(verification.failures.length ? "impasse" : "verified", metaNote("verified", { passed: verification.passed, total: verification.total }));
      result.program = { steps: found.steps.map(metaStepLabel), parameter: found.parameter, source, alternatives: found.alternatives, evaluated: found.evaluated };
      result.verification = verification;
      result.status = verification.failures.length ? "open" : "solved";
      if (result.status === "solved") metaLearn(groundings, found.steps, trace);
    }
    return result;
  }
  if (goal === "synthesize_from_meaning") {
    const groups = [];
    for (const grounding of groundings) {
      if (!grounding.hypotheses.length) continue;
      const top = grounding.hypotheses[0].score;
      const group = grounding.hypotheses.filter((hypothesis) => hypothesis.score >= top * 0.99).map((hypothesis) => hypothesis.operation);
      // A word that ties between many operations describes data, not an
      // action; it stays evidence instead of a required operation.
      if (group.length > META_BOUNDS.requiredGroupSize) {
        trace.emit("evidence", metaNote("evidence_only", { word: grounding.word, count: group.length }));
        continue;
      }
      if (!groups.some((existing) => existing.join() === group.join())) groups.push(group);
    }
    if (!groups.length) {
      trace.emit("impasse", metaNote("nothing_grounded", {}));
      return result;
    }
    const open = groundings.filter((grounding) => grounding.status === "open").length;
    if (open > groups.length) {
      trace.emit("impasse", metaNote("open_outweigh", { open, grounded: groups.length }));
      return result;
    }
    const stated = literals.find((literal) => literal.kind !== "path" && typeof literal.value === "number");
    // The values the request names, in order: the numbers and texts it
    // states, and the symbols its words name ("a colon": ":").
    const values = literals
      .filter((literal) => literal.kind !== "path" && (typeof literal.value === "number" || typeof literal.value === "string"))
      .map((literal) => ({ value: literal.value, type: typeof literal.value === "number" ? "number" : "text", at: literal.start }))
      .concat(groundings.filter((grounding) => grounding.status === "value").map((grounding) => ({ value: grounding.value, type: "text", at: metaWordPosition(prompt, grounding.word) })))
      .sort((a, b) => a.at - b.at);
    // Words that tie across many operations describe data; they neither
    // require nor credit an operation.
    const words = groundings
      .map((grounding) => grounding.hypotheses.filter((hypothesis) => hypothesis.score >= grounding.hypotheses[0].score * 0.99))
      .filter((hypotheses) => hypotheses.length > 0 && hypotheses.length <= META_BOUNDS.requiredGroupSize);
    // A word tied across operations that all take one type names the
    // argument: "the lines of a text" is a function of a text.
    const inputTypes = [];
    for (const grounding of groundings) {
      const top = grounding.hypotheses.filter((hypothesis) => grounding.hypotheses.length && hypothesis.score >= grounding.hypotheses[0].score * 0.99);
      if (top.length <= META_BOUNDS.requiredGroupSize) continue;
      const froms = new Set(top.map((hypothesis) => (metaSeed().primitives.find((primitive) => primitive.id === hypothesis.operation) || {}).from));
      if (froms.size === 1) {
        const type = Array.from(froms)[0];
        if (type && !inputTypes.includes(type)) {
          inputTypes.push(type);
          trace.emit("evidence", metaNote("argument_type", { word: grounding.word, type }));
        }
      }
    }
    const statedPath = literals.find((literal) => literal.kind === "path") || located[0];
    if (statedPath && !inputTypes.includes("path")) {
      inputTypes.unshift("path");
      trace.emit("evidence", metaNote("stated_path", { path: statedPath.value }));
    }
    const clauses = metaClauses(unquoted, groundings, trace);
    // Words tied across many operations still break ties among programs
    // equal in everything else, and may name a filter's measure.
    const tied = groundings
      .map((grounding) => ({ word: grounding.word, hypotheses: grounding.hypotheses.filter((hypothesis) => hypothesis.score >= grounding.hypotheses[0].score * 0.99) }))
      .filter((item) => item.hypotheses.length > META_BOUNDS.requiredGroupSize);
    const weakWords = tied.map((item) => item.hypotheses);
    // A tied plural noun names a representation ("the lines"): the view
    // producing it should be taken, ahead of a shorter program.
    const weakViews = tied
      .filter((item) => metaLemmas(item.word, language).some((lemma) => lemma !== item.word))
      .map((item) => item.hypotheses.filter((hypothesis) => metaIsView(hypothesis.operation)))
      .filter((hypotheses) => hypotheses.length);
    // A quantifier over a value ("every colon") ranges over its occurrences,
    // which the operation reading the value handles; over anything else
    // ("every file") it asks the program to iterate.
    const quantifier = metaCueMarkers("universal").find((marker) => {
      const at = lowered.indexOf(marker);
      if (at < 0) return false;
      const quantified = metaWords(lowered.slice(at + marker.length))[0];
      return !groundings.some((grounding) => grounding.word === quantified && grounding.status === "value");
    });
    if (quantifier) trace.emit("evidence", metaNote("universal", { word: quantifier.trim() }));
    // The grounded word right after a stated bound is its unit ("longer
    // than 80 characters"): it names what a filter measures.
    const byWord = new Map(groundings.map((grounding) => [grounding.word, grounding]));
    const unit = stated ? metaWords(prompt.slice(stated.end)).map((word) => byWord.get(word)).find((grounding) => grounding && grounding.hypotheses.length) : null;
    if (unit) trace.emit("evidence", metaNote("measure_unit", { word: unit.word, bound: stated.value }));
    // The unit credits the measure, not the main program.
    const mainWords = unit ? words.filter((hypotheses) => !hypotheses.every((hypothesis) => unit.hypotheses.includes(hypothesis))) : words;
    // A superlative's stem names the measure its selection compares.
    const measureWords = (unit ? [unit.hypotheses] : []).concat(groundings.filter((grounding) => grounding.measures).map((grounding) => grounding.measures));
    const found = metaSynthesizeFromMeaning(groups, evidence, trace, values, mainWords, inputTypes, clauses, {
      weakWords,
      weakViews,
      universal: Boolean(quantifier),
      measureWords: measureWords.length ? measureWords : null,
    });
    if (found) {
      if (found.parameter !== null) trace.emit("bind", metaNote("parameter", { value: metaLiteral(found.parameter) }));
      const source = metaRender(found.steps, found.parameter);
      // Probe with the seeded samples of the argument type; the first one the
      // program changes is shown. A program no sample changes is a no-op.
      // eslint-disable-next-line no-new-func -- the probe runs the shown source.
      const solution = new Function(`${source}\nreturn solution;`)();
      let probe = null;
      // An operation that needs an environment this runtime lacks (the file
      // system outside Node) is not probed here; the answer says so.
      const needs = found.steps.map(metaStepEnvironment).find(Boolean);
      const argument = statedPath ? statedPath.value : null;
      if (needs && !metaEnvironmentAvailable(needs)) {
        trace.emit("probe", metaNote("probe_skipped", { environment: needs }));
        result.program = { steps: found.steps.map(metaStepLabel), parameter: found.parameter, source, alternatives: 0, evaluated: 0, environment: needs, argument };
        result.probe = null;
        result.status = result.unknowns.some((unknown) => unknown.status === "open") ? "partial" : "solved";
        metaLearn(groundings, found.steps, trace);
        return result;
      }
      // A program reading a value from the request is also probed with
      // samples holding that value ("hello:world" for a colon).
      const samples = (metaSeed().probes[found.fromType] || []).slice();
      const texts = [].concat(found.parameter === null ? [] : found.parameter).filter((value) => typeof value === "string");
      if (found.fromType === "text" && texts.length) for (const sample of metaSeed().probes.text || []) samples.push(sample.split(" ").join(texts[0]));
      for (const sample of samples) {
        try {
          const input = found.fromType === "text" ? sample : JSON.parse(sample);
          const output = solution(input);
          if (!probe) probe = { input, output };
          if (JSON.stringify(input) !== JSON.stringify(output)) {
            probe = { input, output };
            break;
          }
        } catch {
          // A sample the program cannot take is skipped.
        }
      }
      trace.emit("probe", probe ? `${JSON.stringify(probe.input)} → ${JSON.stringify(probe.output)}` : "no probe");
      if (!probe || JSON.stringify(probe.input) === JSON.stringify(probe.output)) {
        trace.emit("impasse", metaNote("no_op", {}));
        return result;
      }
      result.program = { steps: found.steps.map(metaStepLabel), parameter: found.parameter, source, alternatives: 0, evaluated: 0, argument };
      result.probe = probe;
      result.status = result.unknowns.some((unknown) => unknown.status === "open") ? "partial" : "solved";
      metaLearn(groundings, found.steps, trace);
    }
    return result;
  }
  return result;
}

/**
 * Location operands: the bare name right after a locative cue ("in", "в")
 * that follows a file noun, a word whose documented operations all take or
 * give a path. A name with no determiner is a proper name ("in data"); "in a
 * folder" names no operand.
 * @param {string} text the request with its literals blanked
 * @param {string} language
 * @returns {Array<{start: number, end: number, value: string, cue: string}>}
 */
function metaLocationOperands(text, language) {
  const out = [];
  const lowered = String(text).toLowerCase();
  for (const marker of metaCueMarkers("location")) {
    let from = 0;
    for (;;) {
      const at = lowered.indexOf(marker, from);
      if (at < 0) break;
      from = at + 1;
      const before = metaWords(lowered.slice(0, at));
      const noun = before[before.length - 1];
      const after = /^[\p{L}\p{N}_]+(?:[.-][\p{L}\p{N}_]+)*/u.exec(text.slice(at + marker.length));
      if (!noun || !after || metaIsGrammatical(after[0].toLowerCase())) continue;
      const types = metaDocHypotheses(noun, language)
        .map((item) => metaSeed().primitives.find((primitive) => primitive.id === item.operation))
        .filter(Boolean);
      if (!types.length || !types.every((primitive) => [primitive.from, primitive.to].some((type) => type === "path" || type === "list_path"))) continue;
      const start = at + marker.length;
      if (out.some((operand) => operand.start === start)) continue;
      out.push({ start, end: start + after[0].length, value: after[0], cue: marker.trim() });
    }
  }
  return out.sort((a, b) => a.start - b.start);
}

/**
 * Where a word first stands in the request, so values named by words and
 * values stated as literals keep the request's order.
 * @param {string} prompt
 * @param {string} word
 * @returns {number}
 */
function metaWordPosition(prompt, word) {
  const match = new RegExp(`(?<![\\p{L}\\p{N}_])${word}(?![\\p{L}\\p{N}_])`, "u").exec(String(prompt).toLowerCase());
  return match ? match.index : String(prompt).length;
}

/**
 * Explanation-based learning: a word grounded only through a capture becomes
 * a chunk keyed by its lemma once a verified program used its operation, so
 * the next turn recalls it without a lookup.
 * @param {Array<object>} groundings
 * @param {Array<object>} steps
 * @param {object} trace
 */
function metaLearn(groundings, steps, trace) {
  const used = new Set(steps.flatMap(metaStepOperations));
  for (const grounding of groundings) {
    if (grounding.origin !== "capture") continue;
    const hypothesis = grounding.hypotheses.find((item) => used.has(item.operation));
    if (!hypothesis) continue;
    const chunk = { operation: hypothesis.operation, score: hypothesis.score, via: hypothesis.via };
    metaLearnedChunks.set(grounding.word, chunk);
    metaNewChunks.set(grounding.word, chunk);
    trace.emit("chunk", `learned ${grounding.word} → ${hypothesis.operation}`);
  }
}

/**
 * Learned chunks as links notation, so a session can persist them.
 * @returns {string}
 */
function metaLearnedLino() {
  const lines = ["meta_learned_chunks"];
  for (const [word, chunk] of Array.from(metaLearnedChunks).sort((a, b) => a[0].localeCompare(b[0]))) {
    lines.push(`  chunk ${JSON.stringify(word)}`, `    operation ${chunk.operation}`, `    via ${JSON.stringify(chunk.via)}`);
  }
  return lines.join("\n");
}

/**
 * The derivation as links notation: goal, unknowns, program, verification and
 * every trace event in order.
 * @param {object} result
 * @returns {string}
 */
function metaDerivationLino(result) {
  const lines = ["derivation", `  goal ${result.goal}`, `  status ${result.status}`];
  for (const unknown of result.unknowns) {
    lines.push(`  unknown ${JSON.stringify(unknown.word)}`, `    status ${unknown.status}`, `    origin ${unknown.origin}`);
    if (unknown.best) lines.push(`    operation ${unknown.best.operation}`);
  }
  if (result.program) lines.push(`  program ${JSON.stringify(result.program.steps.join(" ∘ "))}`);
  if (result.verification) lines.push(`  verified ${result.verification.passed}/${result.verification.total}`);
  for (const event of result.trace.events) lines.push(`  event ${event.seq}`, `    kind ${event.kind}`, `    detail ${JSON.stringify(event.detail)}`);
  for (const [index, sub] of (result.subgoals || []).entries()) {
    lines.push(`  subgoal ${index + 1}`);
    for (const line of metaDerivationLino(sub).split("\n").slice(1)) lines.push(`  ${line}`);
  }
  return lines.join("\n");
}

/**
 * The full loop: run the core, open the lookups its impasses asked for, run
 * again with the new knowledge, until solved or the budget is spent.
 * `lookup(word, language)` resolves to dictionary senses `{gloss, sourceUrl}`.
 * @param {string} prompt
 * @param {string} language
 * @param {{lookup?: function, knowledge?: Map}} options
 * @returns {Promise<object>}
 */
async function metaReason(prompt, language, options) {
  const settings = options || {};
  const knowledge = settings.knowledge instanceof Map ? settings.knowledge : new Map();
  const requests = metaSubRequests(prompt);
  if (requests.length > 1) return metaReasonComposite(prompt, requests, language, { ...settings, knowledge });
  let result = metaReasonCore(prompt, language, knowledge);
  const rounds = [];
  for (let round = 0; round < META_BOUNDS.lookupRounds; round += 1) {
    if (result.status === "solved" || !result.needs.length || typeof settings.lookup !== "function") break;
    if (result.goal === "understand" || result.goal === "explain") break;
    const opened = [];
    for (const word of result.needs.slice(0, META_BOUNDS.lookupsPerRound)) {
      // A dictionary lists base forms: the surface first, then each lemma.
      let senses = [];
      let looked = word;
      for (const lemma of metaLemmas(word, language)) {
        try {
          // eslint-disable-next-line no-await-in-loop -- each lookup is bounded by the budget.
          senses = (await settings.lookup(lemma, language)) || [];
        } catch {
          senses = [];
        }
        looked = lemma;
        if (senses.length) break;
      }
      knowledge.set(word, senses);
      opened.push(`${word}${looked === word ? "" : `(${looked})`}:${senses.length}`);
    }
    rounds.push(opened.join(" "));
    result = metaReasonCore(prompt, language, knowledge);
  }
  result.lookups = rounds;
  result.derivationLino = metaDerivationLino(result);
  return result;
}

/**
 * The reader-facing answer for a solved or partial result, or null. With
 * `allowOpen`, an unsolved synthesis goal answers with what is still unknown.
 * @param {object} result
 * @param {boolean} [allowOpen]
 * @returns {{intent: string, content: string, confidence: number, evidence: Array<string>}|null}
 */
function metaAnswer(result, allowOpen) {
  if (result.subgoals) return metaCompositeAnswer(result, allowOpen);
  // An imperative names a procedure only when every word of it is
  // understood; otherwise the turn is not evidently a request for one.
  if (result.imperative && result.status !== "solved") return null;
  const language = result.language === "ru" ? "ru" : "en";
  const reasoning = [metaResponse("reasoning_heading", language, {})];
  for (const event of result.trace.events) {
    if (["impulse", "unknowns"].includes(event.kind)) continue;
    reasoning.push(`${event.seq}. ${event.kind}: ${event.detail}`);
  }
  if (result.lookups && result.lookups.length) reasoning.push(`lookups: ${result.lookups.join("; ")}`);
  const evidence = ["meta_reasoner:derivation", ...result.trace.events.map((event) => `meta:${event.kind}:${event.detail}`)];
  if (result.status === "solved" && result.explanation) {
    return {
      intent: "meta_reasoned_definition",
      content: metaResponse("defined_in_request", language, { term: result.explanation.term, definition: result.explanation.definition }),
      confidence: 0.8,
      evidence,
    };
  }
  if (!result.program) {
    if (!allowOpen || !result.goal.startsWith("synthesize")) return null;
    const unresolved = result.unknowns.filter((unknown) => unknown.status === "open").map((unknown) => unknown.word);
    if (!unresolved.length) return null;
    return {
      intent: "meta_reasoning_open",
      content: [metaResponse("open_unknowns", language, { terms: unresolved.join(", "), question: metaResponse("decide_question", language, {}) }), "", reasoning.join("\n")].join("\n"),
      confidence: 0.3,
      evidence,
    };
  }
  const open = result.unknowns.filter((unknown) => unknown.status === "open").map((unknown) => unknown.word);
  const head = result.verification
    ? metaResponse("solved_by_examples", language, { passed: result.verification.passed, total: result.verification.total })
    : metaResponse("solved_by_meaning", language, { probe: result.probe ? `${JSON.stringify(result.probe.input)} → ${JSON.stringify(result.probe.output)}` : "—" });
  const parts = [head, "", "```javascript", result.program.source, "```"];
  if (result.program.argument) parts.push("", metaResponse("usage", language, { call: `solution(${JSON.stringify(result.program.argument)})` }));
  if (result.program.environment) parts.push("", metaResponse("environment_note", language, { environment: result.program.environment }));
  parts.push("", reasoning.join("\n"));
  if (open.length && !result.verification) {
    parts.push("", metaResponse("open_unknowns", language, { terms: open.join(", "), question: metaResponse("decide_question", language, {}) }));
  }
  return {
    intent: result.verification ? "meta_reasoned_program" : "meta_reasoned_program_unverified",
    content: parts.join("\n"),
    confidence: result.verification ? 0.9 : 0.55,
    evidence,
  };
}

/**
 * The worker's turn entry: discovery goes through the registry source walk.
 * @param {string} prompt
 * @param {string} language
 * @param {object} preferences
 * @returns {Promise<object>}
 */
async function metaReasonTurn(prompt, language, preferences) {
  const lookup = typeof lookupConceptSurface === "function"
    ? async (word, lang) => {
      const outcome = await lookupConceptSurface(word, lang, preferences || {});
      return (outcome.items || []).map((sense) => ({ gloss: sense.gloss, sourceUrl: sense.sourceUrl, sha256: sense.sha256 }));
    }
    : null;
  return metaReason(prompt, language, { lookup });
}

/**
 * True when a handler's answer admits it could not do the task.
 * @param {string} intent
 * @returns {boolean}
 */
function metaIsImpasseIntent(intent) {
  const impasse = metaSeed().impasse;
  const value = String(intent || "");
  return impasse.intents.includes(value) || impasse.suffixes.some((suffix) => value.endsWith(suffix));
}

/**
 * A handler impasse is a subgoal for the general loop (Soar's universal
 * subgoaling): when the loop derives a program, its answer replaces the
 * admission; otherwise the handler's answer stands, carrying the derivation.
 * @param {string} prompt
 * @param {object} answer
 * @param {object} preferences
 * @returns {Promise<object>}
 */
async function metaResolveImpasse(prompt, answer, preferences) {
  if (!answer || !metaIsImpasseIntent(answer.intent)) return answer;
  const language = typeof detectLanguage === "function" ? detectLanguage(prompt) : "en";
  const meta = await metaReasonTurn(prompt, language, preferences);
  const resolved = metaAnswer(meta, true);
  const impasseStep = { step: "meta_impasse", detail: `${answer.intent} → general loop: ${meta.goal} ${meta.status}`, derivation: meta.derivationLino, level: "high" };
  // The trace still ends by projecting the answer out of the formalization.
  const steps = (answer.steps || []).slice();
  const closing = steps.length && steps[steps.length - 1].step === "deformalize" ? steps.pop() : null;
  steps.push(impasseStep);
  if (closing) steps.push(resolved ? { ...closing, answer: resolved.content.split(String.fromCharCode(10), 1)[0] } : closing);
  if (!resolved) return { ...answer, steps, derivation: meta.derivationLino };
  return {
    ...answer,
    intent: resolved.intent,
    content: resolved.content,
    confidence: resolved.confidence,
    evidence: resolved.evidence.concat(`meta:impasse:${answer.intent}`, (answer.evidence || []).filter((entry) => entry.startsWith("trace:"))),
    steps,
    derivation: meta.derivationLino,
  };
}
