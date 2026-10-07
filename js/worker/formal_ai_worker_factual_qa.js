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
  return {
    intent: "prompt_text_answer",
    content: factualQaRender("prompt_text_answer", language, { sentence: best.sentence }),
    confidence: 0.85,
    evidence: evidence.concat(
      `prompt_text:selected:${best.index + 1}`,
      `prompt_text:covered:${best.covered.join(",")}`,
      "response:prompt_text_answer",
    ),
  };
}
