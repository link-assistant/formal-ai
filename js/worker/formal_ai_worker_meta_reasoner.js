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
  glossesPerWord: 6,
  programLength: 4,
  candidateBudget: 20000,
  lookupRounds: 3,
  lookupsPerRound: 4,
  rejectionsTraced: 5,
  requiredGroupSize: 2,
});

let metaSeedCache = null;
const metaLearnedChunks = new Map();
const metaCompiled = new Map();

/**
 * The parsed seed: cues, grammatical words, affixes, primitives, probes and
 * response templates.
 * @returns {object}
 */
function metaSeed() {
  if (metaSeedCache) return metaSeedCache;
  const text = typeof SEED_RAW === "object" ? seedRawText(SEED_RAW, META_REASONING_FILE) : "";
  const seed = { cues: {}, grammatical: {}, affixes: {}, primitives: [], combinators: [], filters: [], probes: {}, responses: {}, impasse: { intents: [], suffixes: [] } };
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
      } else if (record.name === "primitive") {
        seed.primitives.push({
          id: record.value,
          from: field("from"),
          to: field("to"),
          doc: field("doc"),
          code: field("code"),
          infer: field("infer"),
        });
      } else if (record.name === "combinator") {
        seed.combinators.push({ id: record.value, doc: field("doc") });
      } else if (record.name === "filter") {
        seed.filters.push({ id: record.value, doc: field("doc"), test: field("test") });
      } else if (record.name === "impasse") {
        seed.impasse.intents.push(...fields("intent"));
        seed.impasse.suffixes.push(...fields("suffix"));
      } else if (record.name === "probe") {
        seed.probes[field("type")] = (seed.probes[field("type")] || []).concat(fields("value"));
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
 * bracketed lists and bare numbers.
 * @param {string} prompt
 * @returns {Array<{start: number, end: number, value: *}>}
 */
function metaValueLiterals(prompt) {
  const pattern = /(?<![\p{L}\p{N}])'([^'\n]*)'(?![\p{L}\p{N}])|"([^"\n]*)"|“([^”\n]*)”|«([^»\n]*)»|(\[[^\]\n]*\])|(?<![\p{L}\p{N}.])(-?\d+(?:\.\d+)?)(?![\p{L}\p{N}])/gu;
  const out = [];
  for (const match of String(prompt).matchAll(pattern)) {
    let value;
    if (match[5] !== undefined) {
      try {
        value = JSON.parse(match[5].replace(/'/gu, "\""));
      } catch {
        continue;
      }
    } else if (match[6] !== undefined) {
      value = Number(match[6]);
    } else {
      value = match[1] ?? match[2] ?? match[3] ?? match[4];
    }
    out.push({ start: match.index, end: match.index + match[0].length, value });
  }
  return out;
}

/**
 * Input/output examples: two adjacent values joined by an example marker.
 * @param {string} prompt
 * @param {Array<object>} literals
 * @returns {Array<{input: *, output: *}>}
 */
function metaExamples(prompt, literals) {
  const markers = metaCueMarkers("example");
  const examples = [];
  for (let index = 0; index + 1 < literals.length; index += 1) {
    const between = ` ${prompt.slice(literals[index].end, literals[index + 1].start).toLowerCase()} `;
    if (between.length > 28) continue;
    if (!markers.some((marker) => between.includes(marker))) continue;
    examples.push({ input: literals[index].value, output: literals[index + 1].value });
    index += 1;
  }
  return examples;
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
      if (head.length === 1 && definition) out.push({ term: head[0], definition });
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
    .concat(seed.filters.map((filter) => ({ id: filter.id, kind: "filter", text: `${filter.id.replace(/_/gu, " ")} ${filter.doc}` })));
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
    trace.emit("known", `${indent}${word} is structural vocabulary of the instruction set`);
    return { word, status: "structural", origin: "documentation", hypotheses: [], needs: [] };
  }
  const documented = metaDocHypotheses(word, context.language);
  if (documented.length) {
    trace.emit("hypothesis", `${indent}${word} → ${documented.slice(0, 3).map((item) => `${item.operation}:${item.score.toFixed(2)}`).join(" | ")} (documentation)`);
    return { word, status: "grounded", origin: "documentation", hypotheses: documented.map((item) => ({ ...item, via: "documentation" })), needs: [] };
  }
  const senses = context.knowledge.get(word) || [];
  if (senses.length && depth < META_BOUNDS.groundDepth) {
    trace.emit("subgoal", `${indent}understand ${word} through ${senses.length} gloss(es)`);
    const frameMarkers = metaCueMarkers("artifact");
    const frame = senses.slice(0, META_BOUNDS.glossesPerWord).find((sense) => frameMarkers.some((marker) => metaWords(sense.gloss).some((token) => token.startsWith(marker.trim()))));
    if (frame) {
      trace.emit("grounded", `${indent}${word} names the artifact's frame: "${frame.gloss}"`);
      return { word, status: "frame", origin: "capture", hypotheses: [], needs: [] };
    }
    const scores = new Map();
    const deeperNeeds = [];
    let via = "";
    for (const sense of senses.slice(0, META_BOUNDS.glossesPerWord)) {
      for (const hypothesis of metaGroundText(sense.gloss, context, depth + 1, stack.concat(word), deeperNeeds)) {
        const score = hypothesis.score / 2;
        if (score > (scores.get(hypothesis.operation) || 0)) {
          scores.set(hypothesis.operation, score);
          if (!via || score >= Math.max(...scores.values())) via = sense.sourceUrl || sense.gloss;
        }
      }
    }
    const hypotheses = Array.from(scores, ([operation, score]) => ({ operation, score, via })).sort((a, b) => b.score - a.score || a.operation.localeCompare(b.operation));
    if (hypotheses.length) {
      trace.emit("grounded", `${indent}${word} → ${hypotheses[0].operation} via ${via}`);
      return { word, status: "grounded", origin: "capture", hypotheses, needs: [] };
    }
    // Lazy grounding: a gloss word is worth a lookup only when no gloss of
    // its parent reached an operation.
    trace.emit("impasse", `${indent}${word}: no gloss reaches an operation`);
    return { word, status: "open", origin: "capture", hypotheses: [], needs: deeperNeeds };
  }
  if (!context.knowledge.has(word) && depth < META_BOUNDS.groundDepth) {
    if (depth === 0) trace.emit("impasse", `${word}: unknown, lookup needed`);
    return { word, status: "open", origin: "none", hypotheses: [], needs: [word] };
  }
  return { word, status: "open", origin: "none", hypotheses: [], needs: [] };
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
  const lowered = ` ${unquoted.toLowerCase()} `;
  const artifact = metaCueMarkers("artifact").some((marker) => lowered.includes(marker));
  const question = metaCueMarkers("question").find((marker) => lowered.includes(marker));
  const goal = examples.length ? "synthesize_from_examples" : artifact ? "synthesize_from_meaning" : question ? "explain" : "understand";
  trace.emit("formalize", `goal ${goal}; ${examples.length} example(s); ${definitions.length} definition(s) in the request`);
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
    trace,
  };
  if (goal === "explain") {
    const defined = definitions.find((item) => contentWords.includes(item.term));
    if (defined) {
      result.status = "solved";
      result.explanation = defined;
      trace.emit("goal_achieved", `${defined.term} is defined by the request itself`);
    }
    return result;
  }
  if (goal === "synthesize_from_examples") {
    const found = metaSynthesizeFromExamples(examples, evidence, trace);
    if (found) {
      const source = metaRender(found.steps, found.parameter);
      const verification = metaVerify(source, examples);
      trace.emit(verification.failures.length ? "impasse" : "verified", `rendered source passes ${verification.passed}/${verification.total}`);
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
        trace.emit("evidence", `${grounding.word} ties ${group.length} operations; evidence only`);
        continue;
      }
      if (!groups.some((existing) => existing.join() === group.join())) groups.push(group);
    }
    if (!groups.length) {
      trace.emit("impasse", "no word of the request grounds an operation");
      return result;
    }
    const open = groundings.filter((grounding) => grounding.status === "open").length;
    if (open > groups.length) {
      trace.emit("impasse", `${open} open unknown(s) outweigh ${groups.length} grounded operation(s); no program is claimed`);
      return result;
    }
    const stated = literals.find((literal) => typeof literal.value === "number");
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
          trace.emit("evidence", `${grounding.word} names the argument type ${type}`);
        }
      }
    }
    const clauses = metaClauses(unquoted, groundings, trace);
    const found = metaSynthesizeFromMeaning(groups, evidence, trace, stated ? stated.value : null, words, inputTypes, clauses);
    if (found) {
      if (found.parameter !== null) trace.emit("bind", `parameter ${found.parameter} from the request`);
      const source = metaRender(found.steps, found.parameter);
      // Probe with the seeded samples of the argument type; the first one the
      // program changes is shown. A program no sample changes is a no-op.
      // eslint-disable-next-line no-new-func -- the probe runs the shown source.
      const solution = new Function(`${source}\nreturn solution;`)();
      let probe = null;
      for (const sample of metaSeed().probes[found.fromType] || []) {
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
        trace.emit("impasse", "no probe input is changed by the program; it does nothing the request asked for");
        return result;
      }
      result.program = { steps: found.steps.map(metaStepLabel), parameter: found.parameter, source, alternatives: 0, evaluated: 0 };
      result.probe = probe;
      result.status = result.unknowns.some((unknown) => unknown.status === "open") ? "partial" : "solved";
      metaLearn(groundings, found.steps, trace);
    }
    return result;
  }
  return result;
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
    metaLearnedChunks.set(grounding.word, { operation: hypothesis.operation, score: hypothesis.score, via: hypothesis.via });
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
  let result = metaReasonCore(prompt, language, knowledge);
  const rounds = [];
  for (let round = 0; round < META_BOUNDS.lookupRounds; round += 1) {
    if (result.status === "solved" || !result.needs.length || typeof settings.lookup !== "function") break;
    if (result.goal === "understand" || result.goal === "explain") break;
    const opened = [];
    for (const word of result.needs.slice(0, META_BOUNDS.lookupsPerRound)) {
      let senses = [];
      try {
        // eslint-disable-next-line no-await-in-loop -- each lookup is bounded by the budget.
        senses = (await settings.lookup(word, language)) || [];
      } catch {
        senses = [];
      }
      knowledge.set(word, senses);
      opened.push(`${word}:${senses.length}`);
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
  const parts = [head, "", "```javascript", result.program.source, "```", "", reasoning.join("\n")];
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
