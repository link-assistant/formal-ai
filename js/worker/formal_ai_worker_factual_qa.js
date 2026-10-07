// Subject-verified factual Q&A (issue #1172 R2, R6, R7).
//
// Browser twin of rust/src/solver_handlers/factual_qa.rs and
// rust/src/solver_handlers/prompt_text_question.rs.
//
// R2: a seeded fact answers only when the question's formalized subject
// resolves to the record's `subjectQid` (and its relation to the record's
// relation). The subject slot is the one `parseFactQuestion` formalizes; its
// Q-id comes from the fact store's own label index (aliases and localized
// subject labels -> Q-id), the longest whole-word label winning and a tie
// between different Q-ids resolving to nothing. Alias + keyword matching
// stays only as the last-resort surface hint, for records that carry no
// Q-id at all.
//
// R6: "Compare X and Y" over two seeded subjects aligns their facts by
// relation and states the differences; one seeded side and one unknown side
// is an honest gap naming the missing side (offered only after the research
// route had its turn). R7: a question over a quoted, prompt-supplied passage
// is answered from that passage's own sentences, with no network call.
//
// Recognition cues are the `fact_comparison_cue`, `fact_comparison_joiner`
// and `prompt_text_question_cue` meanings of data/seed/meanings-facts.lino;
// the wording is data/seed/multilingual-responses-entities.lino.

const FACTUAL_QA_ROLE_COMPARISON_CUE = "fact_comparison_cue";
const FACTUAL_QA_ROLE_COMPARISON_JOINER = "fact_comparison_joiner";
const FACTUAL_QA_ROLE_TEXT_CUE = "prompt_text_question_cue";
const FACTUAL_QA_ROLE_FUNCTION_WORD = "statement_function_word";
const FACTUAL_QA_ROLE_INTERROGATIVE = "interrogative_opener";
/** Prepositions opening a time / place phrase of the statement projection. */
const FACTUAL_QA_ROLE_TIME_PREPOSITION = "statement_time_preposition";
const FACTUAL_QA_ROLE_PLACE_PREPOSITION = "statement_place_preposition";
/** Roles whose surfaces make a phrase a time. */
const FACTUAL_QA_TEMPORAL_ROLES = ["calendar_weekday", "calendar_month_name", "calendar_hour_reference", "statement_clock_suffix"];
/** The statement slots in rendering order, with the role of the question words asking each. */
const FACTUAL_QA_SLOTS = [
  ["subject", "statement_subject_question"], ["predicate", null], ["object", null],
  ["time", "statement_time_question"], ["place", "statement_place_question"],
];
/** Fewest words an uncued quoted passage needs to count as a text to read. */
const FACTUAL_QA_MIN_PASSAGE_WORDS = 6;
/** Shortest shared stem two inflected tokens need to count as one word. */
const FACTUAL_QA_MIN_STEM = 4;
/** Opening and closing quote marks a passage may be wrapped in. */
const FACTUAL_QA_QUOTE_PAIRS = [
  ["«", "»"], ["“", "”"], ["„", "“"], ["「", "」"], ["『", "』"], ["\"", "\""], ["'", "'"],
];

/**
 * The normalized surfaces of every meaning carrying `role`, longest first.
 * @param {string} role
 * @returns {Array<string>}
 */
function factualQaSurfaces(role) {
  const surfaces = [];
  for (const meaning of meaningsWithRole(role)) {
    for (const word of meaning.words) {
      const surface = containsCjk(word) ? String(word).toLowerCase().replace(/[…\s]+/gu, "") : normalizePrompt(word);
      if (surface && !surfaces.includes(surface)) surfaces.push(surface);
    }
  }
  return surfaces.sort((left, right) => right.length - left.length);
}

/**
 * Where `surface` starts in `text` as a whole word (CJK: as a substring), or -1.
 * @param {string} text
 * @param {string} surface
 * @returns {number}
 */
function factualQaLocate(text, surface) {
  if (!surface) return -1;
  if (containsCjk(surface)) return text.indexOf(surface);
  return ` ${text} `.indexOf(` ${surface} `);
}

/**
 * The fact store as a label index: every alias and localized subject label
 * of a record that carries a subject Q-id.
 * @returns {Array<{qid: string, surface: string}>}
 */
function factSubjectLabelIndex() {
  const index = [];
  for (const record of Array.isArray(FACTS) ? FACTS : []) {
    if (!record || !record.subjectQid) continue;
    const labels = (record.subjectAliases || []).concat(
      [record.subjectLabel],
      (record.localized || []).map((entry) => entry && entry.subjectLabel),
    );
    for (const label of labels) {
      const surface = normalizePrompt(label || "");
      if (surface) index.push({ qid: record.subjectQid, surface });
    }
  }
  return index;
}

/**
 * Resolve a subject slot to one Q-id: the longest whole-word label wins; a
 * tie between different Q-ids (or no label at all) resolves to null.
 * @param {string} slot
 * @returns {{qid: string, surface: string}|null}
 */
function resolveFactSubject(slot) {
  const normalized = normalizePrompt(slot);
  if (!normalized) return null;
  let best = null;
  const qids = new Set();
  for (const entry of factSubjectLabelIndex()) {
    if (!surfacePresent(normalized, entry.surface)) continue;
    if (!best || entry.surface.length > best.surface.length) {
      best = entry;
      qids.clear();
    }
    if (entry.surface.length === best.surface.length) qids.add(entry.qid);
  }
  return best && qids.size === 1 ? { qid: best.qid, surface: best.surface } : null;
}

/**
 * The formalized question: its relation and subject slot (parseFactQuestion
 * over the text, punctuation intact so a clause boundary ends the slot) and
 * the slot's resolved Q-id ("" when unresolved).
 * @param {string} text
 * @returns {{relation: string, slot: string, qid: string}}
 */
function factSubjectGate(text) {
  const normalized = normalizePrompt(text);
  const query = parseFactQuestion(text, normalized);
  const slot = query ? query.subjectTerm : normalized;
  const subject = resolveFactSubject(slot);
  return { relation: query ? query.relation : "", slot, qid: subject ? subject.qid : "" };
}

/**
 * The seeded record allowed to answer, and the gate that admitted it:
 * `subject_qid:<Q>` when the formalized subject matched the record's Q-id,
 * `surface_hint` for a record with no Q-id matched by alias and keyword. The
 * slot is formalized from `prompt` unless `normalized` is a rewrite of it
 * (a resolved coreference), which is then the text to formalize.
 * @param {string} normalized
 * @param {string} prompt
 * @returns {{record: object, gate: string}|null}
 */
function gatedFactRecord(normalized, prompt) {
  const facts = Array.isArray(FACTS) ? FACTS : [];
  const hasSurface = (values) => (values || []).some((value) => surfacePresent(normalized, normalizePrompt(value)));
  const gate = factSubjectGate(normalizePrompt(prompt) === normalized ? prompt : normalized);
  const verified = gate.qid && facts.find((fact) => fact.subjectQid === gate.qid &&
    (gate.relation ? fact.relation === gate.relation : hasSurface(fact.questionKeywords)));
  if (verified) return { record: verified, gate: `subject_qid:${gate.qid}` };
  const hinted = facts.find((fact) => !fact.subjectQid && hasSurface(fact.subjectAliases) && hasSurface(fact.questionKeywords));
  return hinted ? { record: hinted, gate: "surface_hint" } : null;
}

/**
 * A structured fact question answered from the seeded record the subject
 * gate admits (intent from the record, as the native fact_lookup row).
 * @param {{relation: string, subjectTerm: string, language: string}} query
 * @param {string} prompt
 * @param {Array<string>} trace
 * @returns {object|null}
 */
function seededFactQueryAnswer(query, prompt, trace) {
  const gated = gatedFactRecord(normalizePrompt(prompt), prompt);
  if (!gated || gated.record.relation !== query.relation) return null;
  const record = gated.record;
  const localized = localizedFactFor(record, query.language) || {};
  trace.push("fact_query:cache:hit:seed", `fact_lookup:subject_gate:${gated.gate}`);
  const evidence = [`fact_lookup:hit:${record.slug}`, `fact_lookup:subject_gate:${gated.gate}`].concat(factQueryEvidence({
    relation: record.relation, subjectLabel: localized.subjectLabel || record.subjectLabel, subjectTerm: query.subjectTerm,
    subjectQid: record.subjectQid, valueQid: record.valueQid, source: localized.source || record.source, fromSeed: true,
  }, query.language));
  return {
    intent: record.intent || "fact_lookup",
    content: localized.summary || record.summary,
    confidence: 0.92,
    evidence,
    trace,
    formalizedObject: record.subjectQid,
  };
}

/**
 * A seed response template for `intent` with its `{name}` slots filled.
 * @param {string} intent
 * @param {string} language
 * @param {Object<string, string>} bindings
 * @returns {string}
 */
function factualQaRender(intent, language, bindings) {
  const table = MULTILINGUAL_ANSWERS[intent] || {};
  const entry = table[language] || table.en;
  let rendered = entry ? normalizeEntry(entry, intent).text : "";
  for (const name of Object.keys(bindings || {})) rendered = rendered.split(`{${name}}`).join(bindings[name]);
  return rendered;
}

/**
 * The relation's word in `language` (the fact_relation meaning's surface).
 * @param {string} relation
 * @param {string} language
 * @returns {string}
 */
function factRelationLabel(relation, language) {
  const meaning = findMeaning(relation);
  return (meaning && (wordInLanguage(meaning, language) || wordInLanguage(meaning, "en"))) || relation;
}

/**
 * The comparison's two sides: the prompt minus its cue, split at the first joiner.
 * @param {string} normalized
 * @returns {{left: string, right: string}|null}
 */
function factComparisonSides(normalized) {
  const cue = factualQaSurfaces(FACTUAL_QA_ROLE_COMPARISON_CUE).find((surface) => factualQaLocate(normalized, surface) >= 0);
  if (!cue) return null;
  const at = factualQaLocate(normalized, cue);
  const rest = `${normalized.slice(0, at)} ${normalized.slice(at + cue.length)}`.replace(/\s+/gu, " ").trim();
  let split = null;
  for (const joiner of factualQaSurfaces(FACTUAL_QA_ROLE_COMPARISON_JOINER)) {
    const index = factualQaLocate(rest, joiner);
    if (index > 0 && (!split || index < split.index)) split = { index, joiner };
  }
  if (!split) return null;
  const left = rest.slice(0, split.index).trim();
  const right = rest.slice(split.index + split.joiner.length).trim();
  return left && right ? { left, right } : null;
}

/**
 * Both sides of a comparison prompt with their resolved subjects (or null).
 * @param {string} normalized
 * @returns {{sides: {left: string, right: string}, left: object|null, right: object|null}|null}
 */
function factComparisonPlan(normalized) {
  const sides = factComparisonSides(normalized);
  if (!sides) return null;
  return { sides, left: resolveFactSubject(sides.left), right: resolveFactSubject(sides.right) };
}

/**
 * The seeded facts of one subject, grouped by relation (first record wins).
 * @param {string} qid
 * @param {string} language
 * @returns {Map<string, {value: string, label: string, record: object}>}
 */
function factRelationsFor(qid, language) {
  const relations = new Map();
  for (const record of Array.isArray(FACTS) ? FACTS : []) {
    if (record.subjectQid !== qid || !record.relation || relations.has(record.relation)) continue;
    const localized = localizedFactFor(record, language) || {};
    relations.set(record.relation, {
      value: localized.valueLabel || record.valueLabel || "",
      label: localized.subjectLabel || record.subjectLabel || "",
      record,
    });
  }
  return relations;
}

/**
 * The display label of a resolved subject.
 * @param {Map<string, object>} facts
 * @param {{surface: string}} subject
 * @returns {string}
 */
function factSubjectDisplay(facts, subject) {
  const first = facts.values().next().value;
  return (first && first.label) || subject.surface;
}

/**
 * "Compare X and Y" over two seeded subjects: aligned rows and differences.
 * @param {string} prompt
 * @param {string} normalized
 * @returns {object|null}
 */
function tryFactComparison(prompt, normalized) {
  const plan = factComparisonPlan(normalized);
  if (!plan || !plan.left || !plan.right || plan.left.qid === plan.right.qid) return null;
  const language = detectLanguage(prompt);
  const leftFacts = factRelationsFor(plan.left.qid, language);
  const rightFacts = factRelationsFor(plan.right.qid, language);
  if (leftFacts.size === 0 || rightFacts.size === 0) return null;
  const left = factSubjectDisplay(leftFacts, plan.left);
  const right = factSubjectDisplay(rightFacts, plan.right);
  const missing = factualQaRender("fact_comparison_missing_value", language, {});
  const evidence = [`fact_comparison:left:${plan.left.qid}`, `fact_comparison:right:${plan.right.qid}`];
  const rows = [];
  const differences = [];
  for (const relation of new Set([...leftFacts.keys(), ...rightFacts.keys()])) {
    const label = factRelationLabel(relation, language);
    const leftValue = leftFacts.has(relation) ? leftFacts.get(relation).value : "";
    const rightValue = rightFacts.has(relation) ? rightFacts.get(relation).value : "";
    rows.push(factualQaRender("fact_comparison_row", language, {
      relation: label, left, right, left_value: leftValue || missing, right_value: rightValue || missing,
    }));
    let kind = "unaligned";
    if (leftValue && rightValue) kind = leftValue === rightValue ? "same" : "difference";
    differences.push(factualQaRender(`fact_comparison_${kind}`, language, {
      relation: label, left, right, left_value: leftValue, right_value: rightValue,
      value: leftValue, subject: leftValue ? left : right,
    }));
    evidence.push(`fact_comparison:relation:${relation}`, `fact_comparison:${kind}:${relation}`);
  }
  evidence.push(`wikidata:${plan.left.qid}`, `wikidata:${plan.right.qid}`, `language:${language}`, "response:fact_comparison");
  return {
    intent: "fact_comparison",
    content: factualQaRender("fact_comparison", language, { left, right, rows: rows.join("\n"), differences: differences.join("\n") }),
    confidence: 0.9,
    evidence,
  };
}

/**
 * The honest gap of a comparison whose one side is seeded and the other is
 * not: names the missing side and lists what the known side has.
 * @param {string} prompt
 * @param {string} normalized
 * @returns {object|null}
 */
function tryFactComparisonGap(prompt, normalized) {
  const plan = factComparisonPlan(normalized);
  if (!plan || (plan.left && plan.right) || (!plan.left && !plan.right)) return null;
  const subject = plan.left || plan.right;
  const missingNormalized = plan.left ? plan.sides.right : plan.sides.left;
  // Name the missing side as the user spelled it when the prompt carries it verbatim.
  const at = String(prompt).toLowerCase().indexOf(missingNormalized);
  const missingSide = at >= 0 ? String(prompt).slice(at, at + missingNormalized.length) : missingNormalized;
  const language = detectLanguage(prompt);
  const facts = factRelationsFor(subject.qid, language);
  if (facts.size === 0) return null;
  const known = factSubjectDisplay(facts, subject);
  const rows = [...facts.keys()].map((relation) => factualQaRender("fact_comparison_known_row", language, {
    relation: factRelationLabel(relation, language), value: facts.get(relation).value,
  }));
  return {
    intent: "fact_comparison_gap",
    content: factualQaRender("fact_comparison_gap", language, { known, missing: missingSide, rows: rows.join("\n") }),
    confidence: 0.6,
    evidence: [`fact_comparison:known:${subject.qid}`, `fact_comparison:missing:${missingSide}`, `language:${language}`, "response:fact_comparison_gap"],
  };
}

/**
 * The longest quoted passage of the prompt (an ASCII apostrophe only counts
 * as a quote when no letter touches it from outside).
 * @param {string} prompt
 * @returns {{body: string, start: number, end: number}|null}
 */
function promptTextPassage(prompt) {
  const text = String(prompt || "");
  const letter = (character) => /\p{L}/u.test(character || "");
  let best = null;
  for (const [open, close] of FACTUAL_QA_QUOTE_PAIRS) {
    let start = text.indexOf(open);
    while (start >= 0) {
      const apostrophe = open === "'";
      let end = text.indexOf(close, start + open.length);
      while (apostrophe && end >= 0 && letter(text[end + 1])) end = text.indexOf(close, end + 1);
      if (end < 0) break;
      const body = text.slice(start + open.length, end).trim();
      if ((!apostrophe || !letter(text[start - 1])) && body && (!best || body.length > best.body.length)) {
        best = { body, start, end: end + close.length };
      }
      start = text.indexOf(open, end + close.length);
    }
  }
  return best;
}

/**
 * The sentences of a passage, terminators kept.
 * @param {string} passage
 * @returns {Array<string>}
 */
function promptTextSentences(passage) {
  return passage.split(/(?<=[.!?])\s+|(?<=[。！？])/u).map((sentence) => sentence.trim()).filter(Boolean);
}

/**
 * Do two tokens name the same word (equal, or one inflected stem)?
 * @param {string} left
 * @param {string} right
 * @returns {boolean}
 */
function promptTextTokensMatch(left, right) {
  if (left === right) return true;
  const shorter = Math.min(left.length, right.length);
  if (shorter < FACTUAL_QA_MIN_STEM) return false;
  let common = 0;
  while (common < shorter && left[common] === right[common]) common += 1;
  return common >= Math.max(FACTUAL_QA_MIN_STEM, shorter - 2);
}

/**
 * The question's content units: words (CJK: character pairs) left after the
 * function words, interrogatives and text-cue words are removed.
 * @param {string} question
 * @returns {Array<string>}
 */
function promptTextContentWords(question) {
  const stop = factualQaSurfaces(FACTUAL_QA_ROLE_FUNCTION_WORD).concat(
    factualQaSurfaces(FACTUAL_QA_ROLE_INTERROGATIVE),
    factualQaSurfaces(FACTUAL_QA_ROLE_TEXT_CUE).flatMap((surface) => surface.split(" ")),
  );
  let normalized = normalizePrompt(question);
  if (!containsCjk(normalized)) return normalized.split(" ").filter((word) => word && !stop.includes(word));
  for (const surface of stop) if (containsCjk(surface)) normalized = normalized.split(surface).join(" ");
  const units = [];
  for (const run of normalized.split(" ").filter(Boolean)) {
    const characters = [...run];
    if (characters.length === 1) units.push(run);
    for (let index = 0; index + 1 < characters.length; index += 1) units.push(characters[index] + characters[index + 1]);
  }
  return units;
}

/**
 * The content units of `question` a sentence covers.
 * @param {string} sentence
 * @param {Array<string>} units
 * @returns {Array<string>}
 */
function promptTextCovered(sentence, units) {
  const normalized = normalizePrompt(sentence);
  const tokens = normalized.split(" ").filter(Boolean);
  return units.filter((unit) => (containsCjk(unit)
    ? normalized.includes(unit)
    : tokens.some((token) => promptTextTokensMatch(token, unit))));
}

/**
 * A question over a quoted, prompt-supplied passage, answered from the
 * passage's own sentences with no network call; an honest "the text does
 * not say" when no sentence covers the question.
 * @param {string} prompt
 * @param {string} normalized
 * @returns {object|null}
 */
function tryPromptTextQuestion(prompt, normalized) {
  const passage = promptTextPassage(prompt);
  if (!passage) return null;
  const lead = normalizePrompt(String(prompt).slice(0, passage.start));
  const question = String(prompt).slice(passage.end).replace(/^[\s.,;:!?。，；：！？)\]—-]+/u, "").trim();
  if (!question) return null;
  const cued = factualQaSurfaces(FACTUAL_QA_ROLE_TEXT_CUE).some((surface) => factualQaLocate(lead, surface) >= 0);
  const asks = /[?？]\s*$/u.test(question) ||
    factualQaSurfaces(FACTUAL_QA_ROLE_INTERROGATIVE).some((surface) => factualQaLocate(normalizePrompt(question), surface) >= 0);
  const passageNormalized = normalizePrompt(passage.body);
  const passageWords = containsCjk(passageNormalized) ? [...passageNormalized].length / 2 : passageNormalized.split(" ").length;
  if (!asks || (!cued && (lead || passageWords < FACTUAL_QA_MIN_PASSAGE_WORDS))) return null;
  const language = detectLanguage(question);
  const sentences = promptTextSentences(passage.body);
  const units = promptTextContentWords(question);
  let best = null;
  sentences.forEach((sentence, index) => {
    const covered = promptTextCovered(sentence, units);
    if (!best || covered.length > best.covered.length) best = { sentence, index, covered };
  });
  const evidence = [`prompt_text:sentences:${sentences.length}`, "prompt_text:network:none", `language:${language}`];
  const answered = best && (units.length === 0
    ? sentences.length === 1
    : best.covered.length > 0 && best.covered.length * 2 >= units.length);
  if (!answered) {
    const focus = units.length > 0 ? units.join(", ") : question;
    return {
      intent: "prompt_text_gap",
      content: factualQaRender("prompt_text_gap", language, { focus }),
      confidence: 0.7,
      evidence: evidence.concat("prompt_text:gap", "response:prompt_text_gap"),
    };
  }
  const quote = factualQaRender("prompt_text_answer", language, { sentence: best.sentence });
  const projected = promptTextStatementAnswer(promptTextStatement(best.sentence), quote, [normalizePrompt(question), units], language);
  return {
    intent: "prompt_text_answer",
    content: projected.content,
    confidence: 0.85,
    evidence: evidence.concat(
      `prompt_text:selected:${best.index + 1}`,
      `prompt_text:covered:${best.covered.join(",")}`,
      projected.evidence,
      "response:prompt_text_answer",
    ),
  };
}

/**
 * Project one sentence into a statement (twin of `project_statement`): a
 * seeded time or place preposition opens an adjunct phrase (a comma closes
 * it); a phrase naming a weekday, month, hour or clock suffix (by stem), or
 * a clock shape, is a time, another place phrase a place, the rest object.
 * The clause before the adjuncts splits into subject and predicate.
 * @param {string} sentence
 * @returns {Array<Array<string>>|null} the words of each slot, or null without an adjunct
 */
function promptTextStatement(sentence) {
  if (containsCjk(sentence) || formalGrammar().natural.some((natural) => natural.language === detectLanguage(sentence) && natural.verbFinal)) return null;
  const timePrepositions = factualQaSurfaces(FACTUAL_QA_ROLE_TIME_PREPOSITION);
  const placePrepositions = factualQaSurfaces(FACTUAL_QA_ROLE_PLACE_PREPOSITION);
  const temporal = FACTUAL_QA_TEMPORAL_ROLES.flatMap((role) => factualQaSurfaces(role));
  const words = [];
  for (const raw of sentence.split(/\s+/u)) {
    const text = raw.replace(/^[^\p{L}\p{N}\p{M}]+|[^\p{L}\p{N}\p{M}]+$/gu, "");
    const trailing = raw.slice(raw.replace(/[^\p{L}\p{N}\p{M}]+$/u, "").length);
    if (text) words.push({ text, key: normalizePrompt(text), closes: /[,;，；]/u.test(trailing) });
  }
  const core = [];
  const phrases = [];
  let open = null;
  for (const word of words) {
    const place = placePrepositions.includes(word.key);
    if (place || timePrepositions.includes(word.key)) {
      if (open) phrases.push(open);
      open = { place, words: [word] };
    } else if (open) open.words.push(word);
    else core.push(word);
    if (word.closes && open) {
      phrases.push(open);
      open = null;
    }
  }
  if (open) phrases.push(open);
  if (phrases.length === 0) return null;
  const functionWords = factualQaSurfaces(FACTUAL_QA_ROLE_FUNCTION_WORD);
  const contentAfter = (from) => {
    for (let at = from; at < core.length; at += 1) if (!functionWords.includes(core[at].key)) return at + 1;
    return core.length;
  };
  const subjectEnd = core.length === 2 ? 1 : contentAfter(0);
  const predicateEnd = contentAfter(subjectEnd);
  const text = (list) => list.map((word) => word.text);
  const slots = [text(core.slice(0, subjectEnd)), text(core.slice(subjectEnd, predicateEnd)), text(core.slice(predicateEnd)), [], []];
  for (const phrase of phrases) {
    const body = phrase.words.slice(1);
    const timed = body.some((word) => /^\d+:\d+$/u.test(word.text)
      || temporal.some((surface) => promptTextTokensMatch(word.key, surface)));
    const slot = body.length === 0 ? 2 : timed ? 3 : phrase.place ? 4 : 2;
    slots[slot].push(...text(phrase.words));
  }
  return slots;
}

/**
 * The answer over a projected statement (twin of `statement_answer`): the
 * asked slots first when each tells a content word the question does not
 * already say, the quote, then the statement.
 * @param {Array<Array<string>>|null} slots
 * @param {string} quote
 * @param {[string, Array<string>]} question the normalized question and its content units
 * @param {string} language
 * @returns {{content: string, evidence: Array<string>}}
 */
function promptTextStatementAnswer(slots, quote, [question, units], language) {
  if (!slots) return { content: quote, evidence: [] };
  const functionWords = factualQaSurfaces(FACTUAL_QA_ROLE_FUNCTION_WORD);
  const tells = (index) => slots[index].some((word) => {
    const key = normalizePrompt(word);
    return !functionWords.includes(key) && !units.some((unit) => promptTextTokensMatch(key, unit));
  });
  const render = (indices) => indices.filter((index) => slots[index].length > 0)
    .map((index) => factualQaRender(`prompt_text_slot_${FACTUAL_QA_SLOTS[index][0]}`, language, { value: slots[index].join(" ") }))
    .join("; ");
  const asked = FACTUAL_QA_SLOTS.map(([, role], index) => (role && factualQaSurfaces(role)
    .some((surface) => factualQaLocate(question, surface) >= 0) ? index : -1)).filter((index) => index >= 0);
  const fields = FACTUAL_QA_SLOTS.map(([name], index) => (slots[index].length > 0 ? `${name}=${slots[index].join(" ")}` : ""))
    .filter(Boolean).join(";");
  const evidence = [`prompt_text:statement:${fields}`];
  const lines = [];
  if (asked.length > 0 && asked.every(tells)) {
    evidence.push(`prompt_text:asked:${asked.map((index) => FACTUAL_QA_SLOTS[index][0]).join(",")}`);
    lines.push(factualQaRender("prompt_text_slot_answer", language, { slots: render(asked) }));
  }
  lines.push(quote, factualQaRender("prompt_text_statement", language, { statement: render([0, 1, 2, 3, 4]) }));
  return { content: lines.join("\n"), evidence };
}

// Issue #1172 R3: the live answer, twin of
// rust/src/solver_handlers/fact_live_answer.rs. The question is formalized
// from seed data alone (the relation's fact_relation surfaces, the
// interrogative openers and the function words are removed from the
// relation's clause; what remains is the subject term), and the answer is the
// seeded fact_live_answer template citing the claim's reference URL.
const FACT_LIVE_MAX_SUBJECT_WORDS = 4;
let FACT_LIVE_PROPERTIES = null;

/**
 * The Wikidata property a meaning is `grounded-in` ("" when none).
 * @param {string} slug
 * @returns {string}
 */
function factRelationProperty(slug) {
  if (!FACT_LIVE_PROPERTIES) {
    FACT_LIVE_PROPERTIES = new Map();
    for (const container of parseLinoTree(MEANINGS_LINO).children) {
      for (const node of container.children || []) {
        const grounded = (node.children || []).find((child) => child.name === "grounded-in");
        if (grounded) FACT_LIVE_PROPERTIES.set(meaningSlug(node), grounded.value);
      }
    }
  }
  return FACT_LIVE_PROPERTIES.get(slug) || "";
}

/**
 * The content words of `text` without `removed` surfaces and stop words, or
 * null when nothing (or a whole sentence) is left.
 * @param {string} text
 * @param {Array<string>} removed
 * @returns {string|null}
 */
function factLiveContentWords(text, removed) {
  let tokens = normalizePrompt(text).split(" ").filter(Boolean);
  for (const surface of removed) {
    const parts = surface.split(" ").filter(Boolean);
    if (parts.length === 0) continue;
    for (let index = 0; index + parts.length <= tokens.length; ) {
      if (parts.every((part, offset) => tokens[index + offset] === part)) tokens.splice(index, parts.length);
      else index += 1;
    }
  }
  const stop = factualQaSurfaces(FACTUAL_QA_ROLE_FUNCTION_WORD).concat(factualQaSurfaces(FACTUAL_QA_ROLE_INTERROGATIVE));
  tokens = tokens.filter((token) => !stop.includes(token));
  return tokens.length > 0 && tokens.length <= FACT_LIVE_MAX_SUBJECT_WORDS ? tokens.join(" ") : null;
}

/**
 * Formalize a fact question for the live path: the relation (whose
 * grounded-in must be a Wikidata property) and the subject term.
 * @param {string} prompt
 * @returns {{relation: string, subjectTerm: string}|null}
 */
function factLiveQuestion(prompt) {
  const normalized = normalizePrompt(prompt);
  if (!normalized || containsCjk(normalized)) return null;
  const mentions = (text, meaning) => meaning.words.some((word) => surfacePresent(text, normalizePrompt(word)));
  const relation = meaningsWithRole("fact_relation").find((meaning) => mentions(normalized, meaning));
  if (!relation || !/^P\d+$/.test(factRelationProperty(relation.slug))) return null;
  const clause = String(prompt).split(/[,;:()—!?，；：。？]/).find((part) => mentions(normalizePrompt(part), relation));
  const subjectTerm = factLiveContentWords(clause === undefined ? prompt : clause, relation.words.map(normalizePrompt));
  return subjectTerm ? { relation: relation.slug, subjectTerm } : null;
}

/**
 * The URL a Wikidata claim cites through `property` ("" when it cites none).
 * @param {object} claim
 * @param {string} property
 * @returns {string}
 */
function factClaimReference(claim, property) {
  for (const reference of (claim && claim.references) || []) {
    const snak = ((reference.snaks || {})[property] || [])[0];
    const value = snak && snak.datavalue && snak.datavalue.value;
    if (typeof value === "string" && value) return value;
  }
  return "";
}

/**
 * The live answer (`fact_live_answer_<relation>` when seeded, else the generic
 * one) citing the claim's reference URL, else the registry's subject snapshot.
 * @param {{relation: string, language: string}} query
 * @param {string} subjectQid
 * @param {string} subjectLabel
 * @param {object} claim
 * @param {string} valueLabel
 * @returns {{summary: string, source: string}}
 */
function factLiveSummary(query, subjectQid, subjectLabel, claim, valueLabel) {
  const row = pageSeedRecords(seedRawText(SEED_RAW, "sources-registry.lino"))
    .find((record) => record.name === "source" && record.value === "wikidata") || { children: [] };
  const source = factClaimReference(claim, childValue(row, "reference_url_property")) ||
    childValue(row, "api").split("{id}").join(subjectQid);
  const relation = factRelationLabel(query.relation, query.language), own = `fact_live_answer_${query.relation}`;
  const summary = factualQaRender(MULTILINGUAL_ANSWERS[own] ? own : "fact_live_answer", query.language, { relation, subject: subjectLabel, value: valueLabel, reference: source });
  return { summary, source };
}

// Issue #1172 R8, browser twin of `explanation_concept` and
// `try_explanation_research` (rust/src/solver_handlers/fact_live_answer.rs)
// over `research_page_senses` (rust/src/concept_lookup.rs, the issue #1163 R4
// retrieve-and-formalize path). The concept is what follows a
// `capability_act_explain` surface opening the prompt. Web search is the
// DuckDuckGo Instant Answer boundary (`searchDuckDuckGo`, Rust's CORS-readable
// fallback); each result page is captured with its SHA-256, remembered with
// its trust score and read most-trusted first by the registry row whose
// extractor is the generic page formalizer: every paragraph or list item that
// mentions the concept is a statement, quoted with its URL and digest through
// the `explanation_research_answer` / `explanation_research_row` templates.
const EXPLANATION_ROLE_CUE = "capability_act_explain";
const EXPLANATION_PAGE_EXTRACTOR = "generic_page_v1";
const EXPLANATION_SEARCH_PROVIDER = "duckduckgo";
const EXPLANATION_STATEMENT_KINDS = ["paragraph", "list_item"];
let EXPLANATION_PAGE_MEMORY = null;

/**
 * The concept an explanation request asks about, or null.
 * @param {string} prompt
 * @returns {string|null}
 */
function explanationConcept(prompt) {
  const normalized = normalizePrompt(prompt);
  if (!normalized || containsCjk(normalized)) return null;
  const cue = factualQaSurfaces(EXPLANATION_ROLE_CUE).find((surface) => factualQaLocate(normalized, surface) === 0);
  return cue ? factLiveContentWords(normalized.slice(cue.length), []) : null;
}

/**
 * Retrieve and formalize pages about `subject`: `{sourceId, senses}`, or null
 * when no registry row uses the generic page formalizer or the search could
 * not be read.
 * @param {string} subject
 * @param {{maxPagesPerService: number, maxItems: number}} bounds
 * @returns {Promise<{sourceId: string, senses: Array<object>}|null>}
 */
async function researchPageSenses(subject, bounds) {
  const record = sourceWalkRegistry().find((row) => row.extractor === EXPLANATION_PAGE_EXTRACTOR);
  if (!record || webSearchIsDisabled(EXPLANATION_SEARCH_PROVIDER)) return null;
  const limit = webSearchProviderLimit();
  const search = await searchDuckDuckGo(subject, "", limit);
  if (!search || !search.ok) return null;
  if (!EXPLANATION_PAGE_MEMORY) EXPLANATION_PAGE_MEMORY = createFormalizedPageStore();
  const store = EXPLANATION_PAGE_MEMORY;
  const pages = [];
  for (const result of search.results.slice(0, Math.min(bounds.maxPagesPerService, limit))) {
    // eslint-disable-next-line no-await-in-loop -- capture order is the fused order.
    const capture = await sourceWalkFetchCapture(result.url);
    if (!capture.ok) continue;
    const key = pageKey(capture.url, capture.sha256);
    if (!store.pages.has(key)) {
      const trust = pageTrustScore(pageTrustFeatures(capture.url, capture.text, store, []));
      formalizedPageStoreInsert(store, formalizedPageFromCapture(capture, subject, pages.length + 1, trust, null));
    }
    pages.push({ capture, trust: store.pages.get(key).trust });
  }
  pages.sort((left, right) => right.trust - left.trust);
  const needle = subject.trim().toLowerCase();
  const senses = [];
  for (const { capture } of pages) {
    const room = bounds.maxItems - senses.length;
    if (!needle || room <= 0) continue;
    const statements = formalizePage(capture.text, null, null).blocks
      .filter((block) => EXPLANATION_STATEMENT_KINDS.includes(block.kind) && block.text.toLowerCase().includes(needle))
      .slice(0, room);
    for (const block of statements) senses.push({ gloss: block.text, sourceUrl: capture.url, sha256: capture.sha256 });
  }
  return { sourceId: record.id, senses };
}

/**
 * Answer an explanation request from retrieved pages (intent
 * `explanation_research`), or null so the dispatch keeps looking.
 * @param {string} prompt
 * @returns {Promise<object|null>}
 */
async function tryExplanationResearch(prompt) {
  const concept = explanationConcept(prompt);
  if (!concept) return null;
  const language = detectLanguage(prompt);
  const researched = await researchPageSenses(concept, CONCEPT_LOOKUP_BOUNDS);
  if (!researched || researched.senses.length === 0) return null;
  const solverEvents = [solverEvent("explanation_research:concept", concept), solverEvent("explanation_research:source", researched.sourceId)];
  const rows = researched.senses.map((sense) => {
    solverEvents.push(solverEvent("source", sense.sourceUrl));
    return factualQaRender("explanation_research_row", language, { statement: sense.gloss, url: sense.sourceUrl, sha256: sense.sha256 });
  });
  const content = factualQaRender("explanation_research_answer", language, { concept, rows: rows.join("\n") });
  if (!content) return null;
  const evidence = [`explanation_research:concept:${concept}`, `explanation_research:source:${researched.sourceId}`]
    .concat(researched.senses.map((sense) => `source:${sense.sourceUrl}`));
  return { intent: "explanation_research", content, confidence: 0.8, evidence, trace: evidence.slice(), solverEvents };
}

// ---------------------------------------------------------------------------
// Release timelines (issue #892; browser twin for issue #918). Mirrors
// rust/src/release_timeline.rs over data/seed/release-timelines.lino: a
// timeline answer is never a stored sentence but the snapshot rendered
// against the day it is asked, every word from the seed's phrasing blocks.
// ---------------------------------------------------------------------------

let cachedReleaseTimelines = null;

/**
 * The phrasing blocks and timelines of data/seed/release-timelines.lino.
 * Mirrors parse_release_timelines in rust/src/seed/release_timelines.rs.
 * @returns {{phrasings: object[], timelines: object[]}}
 */
function releaseTimelineRegistry() {
  if (cachedReleaseTimelines) return cachedReleaseTimelines;
  const text = seedRawText(SEED_RAW, "release-timelines.lino");
  const root = text ? parseLinoTree(text).children.find((node) => node.name === "release_timelines") : null;
  const records = root ? root.children : [];
  const localizedPairs = (node, field) => node.children
    .filter((child) => child.name === "localized")
    .map((child) => ({ language: child.value, text: childValue(child, field) }))
    .filter((pair) => pair.language && pair.text);
  const registry = {
    phrasings: records.filter((node) => node.name === "phrasing").map((node) => ({
      language: node.value,
      releasedHeading: childValue(node, "released-heading"),
      releasedItem: childValue(node, "released-item"),
      announcedHeading: childValue(node, "announced-heading"),
      announcedItem: childValue(node, "announced-item"),
      undatedItem: childValue(node, "undated-item"),
      itemSeparator: childValue(node, "item-separator"),
      sectionEnd: childValue(node, "section-end"),
      provenanceNote: childValue(node, "provenance-note"),
      staleNote: childValue(node, "stale-note"),
    })),
    timelines: records.filter((node) => node.name === "timeline").map((node) => ({
      slug: node.value,
      sourceLabel: childValue(node, "source-label"),
      retrievedAt: String(childValue(node, "retrieved-at") || ""),
      freshForDays: Number.parseInt(String(childValue(node, "fresh-for-days") || "").trim(), 10) || 0,
      subjects: localizedPairs(node, "subject"),
      entries: node.children.filter((child) => child.name === "entry").map((entry) => ({
        qid: entry.value,
        releaseDate: String(childValue(entry, "release-date") || ""),
        titles: localizedPairs(entry, "title"),
      })),
    })),
  };
  if (text) cachedReleaseTimelines = registry;
  return registry;
}

/** The pair for `language`, else the English one (Rust `title_for` / `subject_for`). */
function releaseTimelineLocalized(pairs, language, fallback) {
  const hit = pairs.find((pair) => pair.language === language) || pairs.find((pair) => pair.language === "en");
  return hit ? hit.text : fallback;
}

/** Days since the Unix epoch for an ISO day, or null (Rust `days_from_iso_date`). */
function releaseTimelineDays(date) {
  const parts = String(date || "").split("-").map((part) => Number.parseInt(part, 10));
  if (parts.length < 3 || parts.some((part) => Number.isNaN(part))) return null;
  const [year, month, day] = parts;
  if (month < 1 || month > 12 || day < 1 || day > 31) return null;
  const adjustedYear = year - (month <= 2 ? 1 : 0);
  const era = Math.floor(adjustedYear / 400);
  const yearOfEra = adjustedYear - era * 400;
  const dayOfYear = Math.floor((153 * (month + (month > 2 ? -3 : 9)) + 2) / 5) + day - 1;
  const dayOfEra = yearOfEra * 365 + Math.floor(yearOfEra / 4) - Math.floor(yearOfEra / 100) + dayOfYear;
  return era * 146097 + dayOfEra - 719468;
}

/**
 * Render the timeline `slug` in `language` as of `today` (an ISO day), or
 * null for an unknown slug. Mirrors release_timeline::render_from.
 * @param {string} slug
 * @param {string} language
 * @param {string} today
 * @returns {{text: string, released: object[], announced: object[], stale: boolean}|null}
 */
function renderReleaseTimeline(slug, language, today) {
  const registry = releaseTimelineRegistry();
  const timeline = registry.timelines.find((candidate) => candidate.slug === slug);
  const phrasing = registry.phrasings.find((candidate) => candidate.language === language) ||
    registry.phrasings.find((candidate) => candidate.language === "en");
  if (!timeline || !phrasing) return null;
  const isReleased = (entry) => entry.releaseDate !== "" && entry.releaseDate <= today;
  const released = timeline.entries.filter(isReleased).sort((left, right) =>
    (left.releaseDate < right.releaseDate ? -1 : left.releaseDate > right.releaseDate ? 1
      : left.qid < right.qid ? -1 : left.qid > right.qid ? 1 : 0));
  const announced = timeline.entries.filter((entry) => !isReleased(entry));
  const retrieved = releaseTimelineDays(timeline.retrievedAt);
  const now = releaseTimelineDays(today);
  const stale = retrieved === null || now === null || now - retrieved > timeline.freshForDays;
  const subject = releaseTimelineLocalized(timeline.subjects, language, "");
  const fill = (template, entry, position) => String(template)
    .split("{position}").join(String(position))
    .split("{title}").join(releaseTimelineLocalized(entry.titles, language, entry.qid))
    .split("{year}").join(entry.releaseDate.split("-")[0] || "")
    .split("{date}").join(entry.releaseDate);
  const sections = [];
  if (released.length > 0) {
    const items = released.map((entry, index) => fill(phrasing.releasedItem, entry, index + 1));
    sections.push(`${phrasing.releasedHeading.split("{subject}").join(subject)} ${items.join(phrasing.itemSeparator)}${phrasing.sectionEnd}`);
  }
  if (announced.length > 0) {
    const items = announced.map((entry) =>
      fill(entry.releaseDate === "" ? phrasing.undatedItem : phrasing.announcedItem, entry, 0));
    sections.push(`${phrasing.announcedHeading.split("{subject}").join(subject)} ${items.join(phrasing.itemSeparator)}${phrasing.sectionEnd}`);
  }
  sections.push(String(stale ? phrasing.staleNote : phrasing.provenanceNote)
    .split("{source}").join(timeline.sourceLabel)
    .split("{retrieved}").join(timeline.retrievedAt));
  return { text: sections.join(" "), released, announced, stale };
}

// ---------------------------------------------------------------------------
// Topic summaries (the `summarization` precedence row; browser twin for issue
// #918). Mirrors try_summarization_request in
// rust/src/solver_handlers/benchmark_prompts.rs over
// data/seed/summary-topics.lino: a triggered request names a seeded topic,
// and the answer is that topic's canonical concept summary or heaviest project
// statement, never a sentence of its own.
// ---------------------------------------------------------------------------

let cachedSummaryTopicSeeds = null;

/** The items of a seed list value: every quoted item, else the bare value. */
function summaryTopicList(value) {
  const text = String(value || "").trim();
  if (!text) return [];
  const quoted = [...text.matchAll(/"([^"]*)"/gu)].map((match) => match[1]);
  return (quoted.length > 0 ? quoted : [text]).map((item) => item.toLowerCase()).filter(Boolean);
}

/**
 * The summary-topic seed (Rust `summary_topic_seeds`).
 * @returns {object}
 */
function summaryTopicSeeds() {
  if (cachedSummaryTopicSeeds) return cachedSummaryTopicSeeds;
  const text = seedRawText(SEED_RAW, "summary-topics.lino");
  const root = text ? parseLinoTree(text).children.find((node) => node.name === "summary_topics") : null;
  const records = root ? root.children : [];
  const values = (name) => records.filter((node) => node.name === name).flatMap((node) => summaryTopicList(node.value));
  const seeds = {
    triggers: summaryTopicList(childValue(root || { children: [] }, "trigger")),
    rejectSubstrings: values("reject_substring"),
    rejectExact: values("reject_exact"),
    constraintMarkers: values("constraint_marker"),
    constraintLabel: String(childValue(root || { children: [] }, "constraint_label") || ""),
    fallbackResponse: String(childValue(root || { children: [] }, "fallback_response") || ""),
    topics: records.filter((node) => node.name === "topic" && node.value).map((node) => ({
      displayName: node.value,
      detectionKeywords: summaryTopicList(childValue(node, "detection_keywords")),
      sourceKind: String(childValue(node, "source_kind") || ""),
      sourceId: String(childValue(node, "source_id") || ""),
    })).filter((topic) => topic.sourceKind && topic.sourceId),
  };
  if (text) cachedSummaryTopicSeeds = seeds;
  return seeds;
}

/** A localized block for `language`, else the English one (Rust `localized_for`). */
function summaryTopicLocalized(record, language) {
  const localized = Array.isArray(record.localized) ? record.localized : [];
  return localized.find((entry) => entry && entry.language === language) ||
    localized.find((entry) => entry && entry.language === "en") || null;
}

/**
 * The topic's canonical statement and its source (Rust `derived_topic_summary`).
 * @returns {{body: string, source: string}|null}
 */
function summaryTopicDerive(topic, language) {
  if (topic.sourceKind === "concept") {
    const record = (CONCEPTS || []).find((candidate) => candidate.slug === topic.sourceId);
    if (!record) return null;
    const localized = summaryTopicLocalized(record, language);
    return {
      body: (localized && localized.summary) || record.summary || "",
      source: (localized && localized.source) || record.source || "",
    };
  }
  if (topic.sourceKind === "project") {
    const record = (PROJECTS || []).find((candidate) => candidate.slug === topic.sourceId);
    if (!record) return null;
    const localized = summaryTopicLocalized(record, language);
    const statements = localized && Array.isArray(localized.statements) && localized.statements.length > 0
      ? localized.statements : (record.statements || []);
    let best = null;
    for (const statement of statements) {
      if (!best || statement.weight >= best.weight) best = statement; // the last of equal maxima, as Iterator::max_by_key
    }
    return best ? { body: best.text, source: record.url || "" } : null;
  }
  return null;
}

/**
 * The `summarization` row: a summary request over a seeded topic.
 * @param {string} prompt
 * @returns {object|null}
 */
function trySummarizationTopic(prompt) {
  const seeds = summaryTopicSeeds();
  const lower = String(prompt || "").toLowerCase();
  if (!seeds.triggers.some((trigger) => lower.includes(trigger))) return null;
  if (seeds.rejectExact.some((exact) => lower === exact)) return null;
  if (seeds.rejectSubstrings.some((substring) => lower.includes(substring))) return null;
  const language = detectLanguage(prompt);
  const topic = seeds.topics.find((candidate) =>
    candidate.detectionKeywords.some((keyword) => lower.includes(keyword)));
  const derived = topic ? summaryTopicDerive(topic, language) : null;
  const evidence = [];
  let label = topic ? topic.displayName : "";
  let body = derived ? derived.body : "";
  let source = derived ? derived.source : "";
  if (!derived) {
    evidence.push("summarization:refusal:no_seeded_topic");
    label = String(prompt || "").replace(/^[\s!-/:-@[-`{-~]+|[\s!-/:-@[-`{-~]+$/gu, "");
    body = answerFor(seeds.fallbackResponse || "unknown", language);
    source = "none";
  }
  evidence.push(`summarization:topic:${label}`, `summarization:source:${source}`);
  if (seeds.constraintMarkers.some((marker) => lower.includes(marker))) {
    evidence.push(`summarization:constraint:${seeds.constraintLabel}`);
  }
  return { intent: "summarize_topic", content: body, confidence: 0.85, evidence };
}
