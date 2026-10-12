// Word problems as quantity relations (issue #1176 R2): the stated numbers of
// a question are joined by the relation its words declare, read from the seed
// meanings in data/seed/meanings-statistics.lino:
//   word_problem_gain  -> the next quantity is added      (5 + 3 = 8)
//   word_problem_loss  -> the next quantity is subtracted (20 - 7 = 13)
//   word_problem_group -> a per-group size times a count  (6 × 4 = 24)
//   word_problem_share -> a total shared equally          (24 ÷ 6 = 4)
// A question cue (word_problem_question) must be present, and a prompt whose
// numbers no relation word joins is declined rather than guessed.

const WORD_RELATION_QUESTION = "word_problem_question";
const WORD_RELATION_GAIN = "word_problem_gain";
const WORD_RELATION_LOSS = "word_problem_loss";
const WORD_RELATION_GROUP = "word_problem_group";
const WORD_RELATION_SHARE = "word_problem_share";
const WORD_RELATION_MAX_VALUES = 6;
const WORD_RELATION_DIVISION_PRECISION = 6;

/**
 * Character index of a UTF-8 byte offset into `text`.
 * @param {string} text
 * @param {number} byteOffset
 * @returns {number}
 */
function wordRelationCharIndex(text, byteOffset) {
  let bytes = 0;
  let index = 0;
  for (const character of text) {
    if (bytes >= byteOffset) return index;
    const point = character.codePointAt(0);
    bytes += point < 0x80 ? 1 : point < 0x800 ? 2 : point < 0x10000 ? 3 : 4;
    index += character.length;
  }
  return index;
}

/**
 * Does any surface of the meaning `slug` occur in `text`?
 * @param {string} text
 * @param {string} slug
 * @returns {boolean}
 */
function wordRelationMentions(text, slug) {
  return quantityMeaningWords(slug).some((word) => quantityContainsTerm(text, word));
}

/**
 * The sign the relation word between two stated numbers gives the later one:
 * "+" for a gain, "-" for a loss, null when no relation word joins them.
 * @param {string} window
 * @returns {string|null}
 */
function wordRelationSign(window) {
  const gain = wordRelationMentions(window, WORD_RELATION_GAIN);
  const loss = wordRelationMentions(window, WORD_RELATION_LOSS);
  if (gain === loss) return null;
  return gain ? "+" : "-";
}

/**
 * Answer a word problem whose numbers are joined by seeded relation words.
 * @param {string} prompt
 * @param {string} normalized
 * @param {string} language
 * @returns {{intent: string, content: string, confidence: number, evidence: Array<string>}|null}
 */
function tryRelationWordProblem(prompt, normalized, language) {
  const lowered = String(prompt || "").toLowerCase();
  if (!wordRelationMentions(lowered, WORD_RELATION_QUESTION) &&
    !wordRelationMentions(String(normalized || ""), WORD_RELATION_QUESTION)) {
    return null;
  }
  const stated = statisticsStatedNumbers(lowered);
  if (stated === null) return null;
  const values = stated.values;
  if (values.length < 2 || values.length > WORD_RELATION_MAX_VALUES) return null;
  const render = exactDecimalRender;
  const log = [];
  let result = null;
  let derivation = "";
  let exact = true;

  if (values.length === 2 && wordRelationMentions(lowered, WORD_RELATION_SHARE)) {
    const quotient = exactDecimalDiv(values[0], values[1], WORD_RELATION_DIVISION_PRECISION);
    if (quotient === null) return null;
    result = quotient.value;
    exact = quotient.exact;
    log.push("word_problem:relation:share");
    derivation = `${render(values[0])} ÷ ${render(values[1])} = ${exactDecimalRenderApprox(result, quotient.exact)}`;
  } else if (values.length === 2 && wordRelationMentions(lowered, WORD_RELATION_GROUP)) {
    result = exactDecimalMul(values[0], values[1]);
    if (result === null) return null;
    log.push("word_problem:relation:group");
    derivation = `${render(values[0])} × ${render(values[1])} = ${render(result)}`;
  } else {
    result = values[0];
    derivation = render(values[0]);
    for (let index = 1; index < values.length; index += 1) {
      const from = wordRelationCharIndex(lowered, stated.positions[index - 1]);
      const to = wordRelationCharIndex(lowered, stated.positions[index]);
      const sign = wordRelationSign(lowered.slice(from, to));
      if (sign === null) return null;
      result = sign === "+" ? exactDecimalAdd(result, values[index]) : exactDecimalSub(result, values[index]);
      if (result === null) return null;
      log.push(`word_problem:relation:${sign === "+" ? "gain" : "loss"}:${render(values[index])}`);
      derivation += ` ${sign} ${render(values[index])}`;
    }
    if (result.mantissa < 0n) return null;
    derivation += ` = ${render(result)}`;
  }

  log.push(`word_problem:derivation:${derivation}`);
  const template = quantityLocalizedTemplate("word_problem_relation", language);
  const body = template === null
    ? derivation
    : quantityFillTemplate(template, [
      ["result", exactDecimalRenderApprox(result, exact)],
      ["derivation", derivation],
    ]);
  return quantityAnswer("word_problem_relation", body, log, language);
}
