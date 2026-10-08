// Hidden-number interval riddles (issues #403, #699, #890).
//
// Browser twin of rust/src/number_constraints.rs. Recognition reads the
// number-constraint roles of data/seed/meanings-number-constraints.lino; the
// canonical proof statement renders through
// data/seed/proof-program-templates.lino; the formal check is the
// satisfiability branch of the quantifier-free linear real-arithmetic
// decision procedure (rust/src/proof_engine/decision/linear.rs) and its
// presenter (rust/src/proof_engine/presenter.rs, default render config:
// interpretation header on, follow-up questions on). Interval endpoints are
// i64 values held as BigInt; the decision procedure works on doubles, as the
// native one does.

const ROLE_NUMBER_CONSTRAINT_ENTITY = "number_constraint_entity";
const ROLE_NUMBER_CONSTRAINT_QUERY = "number_constraint_query";
const ROLE_NUMBER_CONSTRAINT_HIDDEN = "number_constraint_hidden";
const ROLE_NUMBER_CONSTRAINT_LOWER = "number_constraint_lower";
const ROLE_NUMBER_CONSTRAINT_UPPER = "number_constraint_upper";
const ROLE_NUMBER_CONSTRAINT_LOWER_INCLUSIVE = "number_constraint_lower_inclusive";
const ROLE_NUMBER_CONSTRAINT_LOWER_STRICT = "number_constraint_lower_strict";
const ROLE_NUMBER_CONSTRAINT_UPPER_INCLUSIVE = "number_constraint_upper_inclusive";
const ROLE_NUMBER_CONSTRAINT_UPPER_STRICT = "number_constraint_upper_strict";
const NUMBER_CONSTRAINT_I64_MAX = (1n << 63n) - 1n;
const NUMBER_CONSTRAINT_I64_MIN = -(1n << 63n);
const LINEAR_EPSILON = 1e-9;

/**
 * Parse a decimal integer that fits i64, or null (Rust `str::parse::<i64>`).
 * @param {string} text
 * @returns {bigint|null}
 */
function numberConstraintParseI64(text) {
  if (!/^[+-]?\d+$/u.test(text)) return null;
  const value = BigInt(text);
  if (value > NUMBER_CONSTRAINT_I64_MAX || value < NUMBER_CONSTRAINT_I64_MIN) return null;
  return value;
}

/**
 * Rust `char::is_alphanumeric`.
 * @param {string} character
 * @returns {boolean}
 */
function numberConstraintIsAlphanumeric(character) {
  return /[\p{Alphabetic}\p{N}]/u.test(character);
}

/**
 * The CJK test of `crate::coding::contains_cjk`.
 * @param {string} text
 * @returns {boolean}
 */
function numberConstraintContainsCjk(text) {
  return /[㐀-䶿一-鿿豈-﫿぀-ヿ㄀-ㄯ]/u.test(text);
}

/**
 * A "больше"/"more than" bound directly preceded by "не"/"not" is negated.
 * @param {string} text
 * @param {number} index
 * @param {string} phrase
 * @returns {boolean}
 */
function numberConstraintIsNegatedStrictBound(text, index, phrase) {
  if (!["больше", "меньше", "more than", "less than"].includes(phrase)) return false;
  const words = text.slice(0, index).split(/\s+/u).filter(Boolean);
  if (words.length === 0) return false;
  const last = words[words.length - 1];
  return last === "не" || last === "not";
}

/**
 * Word boundaries around a phrase occurrence (CJK phrases always pass).
 * @param {string} text
 * @param {number} index
 * @param {string} phrase
 * @returns {boolean}
 */
function numberConstraintPhraseHasBoundary(text, index, phrase) {
  if (numberConstraintContainsCjk(phrase)) return true;
  const before = quantityLastCharacter(text.slice(0, index));
  const after = quantityFirstCharacter(text.slice(index + phrase.length));
  const beforeOk = before === "" || !numberConstraintIsAlphanumeric(before);
  const afterOk = after === "" || !numberConstraintIsAlphanumeric(after);
  return beforeOk && afterOk && !numberConstraintIsNegatedStrictBound(text, index, phrase);
}

/**
 * Every non-overlapping occurrence index of `phrase`, like `match_indices`.
 * @param {string} text
 * @param {string} phrase
 * @returns {Array<number>}
 */
function numberConstraintMatchIndices(text, phrase) {
  const indices = [];
  if (!phrase) return indices;
  let index = text.indexOf(phrase);
  while (index !== -1) {
    indices.push(index);
    index = text.indexOf(phrase, index + phrase.length);
  }
  return indices;
}

/**
 * The signed integer leading `text` after `:`, `,`, `=` and whitespace.
 * @param {string} text
 * @returns {bigint|null}
 */
function numberConstraintParseLeadingInteger(text) {
  const trimmed = text.replace(/^[\s:,=]+/u, "");
  const match = /^-?[0-9]*/u.exec(trimmed);
  const head = match ? match[0] : "";
  if (head === "" || head.endsWith("-")) return null;
  return numberConstraintParseI64(head);
}

/**
 * @param {string} text
 * @param {string} phrase
 * @returns {bigint|null}
 */
function numberConstraintNumberAfterPhrase(text, phrase) {
  for (const index of numberConstraintMatchIndices(text, phrase)) {
    if (!numberConstraintPhraseHasBoundary(text, index, phrase)) continue;
    const value = numberConstraintParseLeadingInteger(text.slice(index + phrase.length));
    if (value !== null) return value;
  }
  return null;
}

/**
 * @param {string} text
 * @param {string} phrase
 * @returns {bigint|null}
 */
function numberConstraintNumberBeforePhrase(text, phrase) {
  for (const index of numberConstraintMatchIndices(text, phrase)) {
    if (!numberConstraintPhraseHasBoundary(text, index, phrase)) continue;
    const head = text.slice(0, index).replace(/\s+$/u, "");
    const match = /[0-9-]*$/u.exec(head);
    const candidate = match ? match[0] : "";
    if (candidate === "" || candidate === "-") continue;
    const value = numberConstraintParseI64(candidate);
    if (value !== null) return value;
  }
  return null;
}

/**
 * The first role word form with a number after (or before) it.
 * @param {string} text
 * @param {string} role
 * @param {boolean} inclusive
 * @returns {{value: bigint, inclusive: boolean}|null}
 */
function numberConstraintRoleBound(text, role, inclusive) {
  for (const form of roleWordForms(role)) {
    const after = numberConstraintNumberAfterPhrase(text, form.text);
    const value = after !== null ? after : numberConstraintNumberBeforePhrase(text, form.text);
    if (value !== null) return { value: value, inclusive: inclusive };
  }
  return null;
}

/**
 * A symbolic bound (`>=`, `>`, `<=`, `<`) followed by a number.
 * @param {string} text
 * @param {Array<Array<*>>} phrases
 * @returns {{value: bigint, inclusive: boolean}|null}
 */
function numberConstraintSymbolBound(text, phrases) {
  for (const entry of phrases) {
    const value = numberConstraintNumberAfterPhrase(text, entry[0]);
    if (value !== null) return { value: value, inclusive: entry[1] };
  }
  return null;
}

/**
 * @param {string} cleaned
 * @param {string} source
 * @returns {boolean}
 */
function numberConstraintLooksLikeRiddle(cleaned, source) {
  const mentionsNumber = lexiconMentionsRole(ROLE_NUMBER_CONSTRAINT_ENTITY, cleaned);
  const asksIdentity = lexiconMentionsRole(ROLE_NUMBER_CONSTRAINT_QUERY, cleaned);
  const hiddenNumber = lexiconMentionsRole(ROLE_NUMBER_CONSTRAINT_HIDDEN, cleaned);
  const hasLower = lexiconMentionsRole(ROLE_NUMBER_CONSTRAINT_LOWER, cleaned) ||
    source.includes(">") || source.includes("≥");
  const hasUpper = lexiconMentionsRole(ROLE_NUMBER_CONSTRAINT_UPPER, cleaned) ||
    source.includes("<") || source.includes("≤");
  return mentionsNumber && hasLower && hasUpper && (asksIdentity || hiddenNumber);
}

/**
 * @param {string} wordText
 * @param {string} symbolText
 * @returns {{lower: {value: bigint, inclusive: boolean}, upper: {value: bigint, inclusive: boolean}}|null}
 */
function numberConstraintIntervalBounds(wordText, symbolText) {
  let lower = numberConstraintRoleBound(wordText, ROLE_NUMBER_CONSTRAINT_LOWER_INCLUSIVE, true);
  if (lower === null) lower = numberConstraintRoleBound(wordText, ROLE_NUMBER_CONSTRAINT_LOWER_STRICT, false);
  if (lower === null) lower = numberConstraintSymbolBound(symbolText, [[">=", true], [">", false]]);
  if (lower === null) return null;
  let upper = numberConstraintRoleBound(wordText, ROLE_NUMBER_CONSTRAINT_UPPER_INCLUSIVE, true);
  if (upper === null) upper = numberConstraintRoleBound(wordText, ROLE_NUMBER_CONSTRAINT_UPPER_STRICT, false);
  if (upper === null) upper = numberConstraintSymbolBound(symbolText, [["<=", true], ["<", false]]);
  if (upper === null) return null;
  return { lower: lower, upper: upper };
}

/**
 * @param {{value: bigint, inclusive: boolean}} bound
 * @returns {string}
 */
function numberConstraintLowerOperator(bound) {
  return bound.inclusive ? ">=" : ">";
}

/**
 * @param {{value: bigint, inclusive: boolean}} bound
 * @returns {string}
 */
function numberConstraintUpperOperator(bound) {
  return bound.inclusive ? "<=" : "<";
}

/**
 * The canonical proof statement from the seed's proof-program template.
 * Mirrors `FormalProof::integer_interval(..).statement()`.
 * @param {{lower: {value: bigint, inclusive: boolean}, upper: {value: bigint, inclusive: boolean}}} bounds
 * @returns {string}
 */
function numberConstraintProofStatement(bounds) {
  const root = parseLinoTree(seedRawText(SEED_RAW, "proof-program-templates.lino"));
  const section = root.children.find((child) => child.name === "proof_program_templates");
  if (!section) throw new Error("proof-program-templates.lino must declare proof_program_templates");
  const statementNode = section.children.find((child) => child.name === "statement");
  const template = statementNode ? String(statementNode.value || "") : "";
  if (template === "") throw new Error("proof statement template is required");
  const first = bounds.lower.inclusive ? bounds.lower.value : bounds.lower.value + 1n;
  const last = bounds.upper.inclusive ? bounds.upper.value : bounds.upper.value - 1n;
  const witness = first <= last && first <= NUMBER_CONSTRAINT_I64_MAX ? first.toString() : "";
  const bindings = [
    ["variable", "x"],
    ["lower_operator", numberConstraintLowerOperator(bounds.lower)],
    ["lower_value", bounds.lower.value.toString()],
    ["upper_operator", numberConstraintUpperOperator(bounds.upper)],
    ["upper_value", bounds.upper.value.toString()],
    ["witness", witness],
    ["first", first.toString()],
    ["last", last.toString()],
    ["result", witness !== "" ? "satisfiable" : "unsatisfiable"],
  ];
  return quantityFillTemplate(template, bindings);
}

/**
 * The integer solutions: none, unique, a short list, or a long range.
 * @param {{lower: {value: bigint, inclusive: boolean}, upper: {value: bigint, inclusive: boolean}}} bounds
 * @returns {{kind: string, start: bigint, end: bigint}}
 */
function numberConstraintIntegerSolutions(bounds) {
  const start = bounds.lower.inclusive ? bounds.lower.value : bounds.lower.value + 1n;
  const end = bounds.upper.inclusive ? bounds.upper.value : bounds.upper.value - 1n;
  if (start > end) return { kind: "none", start: 0n, end: 0n };
  if (start > NUMBER_CONSTRAINT_I64_MAX || end < NUMBER_CONSTRAINT_I64_MIN) {
    return { kind: "none", start: 0n, end: 0n };
  }
  if (start === end) return { kind: "unique", start: start, end: end };
  if (end - start > 20n) return { kind: "range", start: start, end: end };
  return { kind: "multiple", start: start, end: end };
}

/**
 * "a, b, c" for every integer from start to end.
 * @param {bigint} start
 * @param {bigint} end
 * @returns {string}
 */
function numberConstraintFormatCandidates(start, end) {
  const rendered = [];
  for (let value = start; value <= end; value += 1n) rendered.push(value.toString());
  return rendered.join(", ");
}

/**
 * `value / 2` rendered as an integer or a `.5` half.
 * @param {bigint} halfSteps
 * @returns {string}
 */
function numberConstraintFormatHalf(halfSteps) {
  const sign = halfSteps < 0n ? "-" : "";
  const magnitude = halfSteps < 0n ? -halfSteps : halfSteps;
  const whole = magnitude / 2n;
  return magnitude % 2n === 0n ? `${sign}${whole}` : `${sign}${whole}.5`;
}

// --- linear real-arithmetic satisfiability (proof_engine/decision/linear.rs) ---

/**
 * @param {number} value
 * @returns {boolean}
 */
function linearNearlyZero(value) {
  return Math.abs(value) < LINEAR_EPSILON;
}

/**
 * @param {number} left
 * @param {number} right
 * @returns {boolean}
 */
function linearNearlyEqual(left, right) {
  return Math.abs(left - right) < LINEAR_EPSILON;
}

/**
 * Rust `format_number`: integers without a fraction, otherwise at most six
 * fractional digits with trailing zeros trimmed.
 * @param {number} value
 * @returns {string}
 */
function linearFormatNumber(value) {
  if (linearNearlyZero(value)) return "0";
  if (linearNearlyZero(value % 1)) return value.toFixed(0);
  return value.toFixed(6).replace(/0+$/u, "").replace(/\.$/u, "");
}

/**
 * Rust `f64::midpoint`.
 * @param {number} a
 * @param {number} b
 * @returns {number}
 */
function linearMidpoint(a, b) {
  const low = 2.2250738585072014e-308 * 2;
  const high = Number.MAX_VALUE / 2;
  const absA = Math.abs(a);
  const absB = Math.abs(b);
  if (absA <= high && absB <= high) return (a + b) / 2;
  if (absA < low) return a + b / 2;
  if (absB < low) return a / 2 + b;
  return a / 2 + b / 2;
}

/**
 * Parse one generated atom `x <op> <integer>` into its threshold relation.
 * The atoms come from `formal_statement`, so this is the slice of the
 * native affine parser that statement shape exercises.
 * @param {string} text
 * @returns {{original: string, comparison: string, threshold: number}|null}
 */
function linearParseIntervalAtom(text) {
  const tokens = ["!=", "<=", ">=", "==", "=", "<", ">"];
  for (const token of tokens) {
    const index = text.indexOf(token);
    if (index === -1) continue;
    const left = text.slice(0, index).trim();
    const right = text.slice(index + token.length).trim();
    if (!/^[A-Za-z_][A-Za-z0-9_]*$/u.test(left) || !/^-?\d+$/u.test(right)) return null;
    const comparison = token === "==" ? "=" : token;
    // lhs - rhs = x - c, so the threshold is -(-c) / 1 = c.
    const constant = 0 - Number(right.replace(/^-/u, "")) * (right.startsWith("-") ? -1 : 1);
    return { original: `${left} ${comparison} ${right}`, comparison: comparison, threshold: -constant / 1 };
  }
  return null;
}

/**
 * Apply the atoms to an interval exactly as `build_interval_system` does.
 * @param {Array<{original: string, comparison: string, threshold: number}>} atoms
 * @returns {{lower: {value: number, strict: boolean}|null, upper: {value: number, strict: boolean}|null, contradiction: {lower: {value: number, strict: boolean}, upper: {value: number, strict: boolean}}|null}}
 */
function linearIntervalSystem(atoms) {
  const interval = { lower: null, upper: null, contradiction: null };
  for (const atom of atoms) {
    const value = atom.threshold;
    switch (atom.comparison) {
      case ">":
      case ">=": {
        const candidate = { value: value, strict: atom.comparison === ">" };
        const current = interval.lower;
        if (current === null || candidate.value > current.value + LINEAR_EPSILON ||
          (linearNearlyEqual(candidate.value, current.value) && candidate.strict && !current.strict)) {
          interval.lower = candidate;
        }
        break;
      }
      case "<":
      case "<=": {
        const candidate = { value: value, strict: atom.comparison === "<" };
        const current = interval.upper;
        if (current === null || candidate.value < current.value - LINEAR_EPSILON ||
          (linearNearlyEqual(candidate.value, current.value) && candidate.strict && !current.strict)) {
          interval.upper = candidate;
        }
        break;
      }
      default:
        throw new Error("unsupported interval comparison");
    }
    const lower = interval.lower;
    const upper = interval.upper;
    if (interval.contradiction === null && lower !== null && upper !== null &&
      (lower.value > upper.value + LINEAR_EPSILON ||
        (linearNearlyEqual(lower.value, upper.value) && (lower.strict || upper.strict)))) {
      interval.contradiction = { lower: lower, upper: upper };
    }
  }
  return interval;
}

/**
 * @param {{lower: {value: number, strict: boolean}|null, upper: {value: number, strict: boolean}|null}} interval
 * @param {number} value
 * @returns {boolean}
 */
function linearIntervalContains(interval, value) {
  const lower = interval.lower;
  if (lower !== null && (value < lower.value - LINEAR_EPSILON ||
    (lower.strict && linearNearlyEqual(value, lower.value)))) {
    return false;
  }
  const upper = interval.upper;
  if (upper !== null && (value > upper.value + LINEAR_EPSILON ||
    (upper.strict && linearNearlyEqual(value, upper.value)))) {
    return false;
  }
  return true;
}

/**
 * The witness `witness_value` picks, or null.
 * @param {{lower: {value: number, strict: boolean}|null, upper: {value: number, strict: boolean}|null, contradiction: object|null}} interval
 * @returns {number|null}
 */
function linearWitness(interval) {
  if (interval.contradiction !== null) return null;
  const lower = interval.lower;
  const upper = interval.upper;
  if (upper !== null && !upper.strict && linearIntervalContains(interval, upper.value)) return upper.value;
  if (lower !== null && !lower.strict && linearIntervalContains(interval, lower.value)) return lower.value;
  let candidate = 0;
  if (lower !== null && upper !== null) candidate = linearMidpoint(lower.value, upper.value);
  else if (lower !== null) candidate = lower.value + (lower.strict ? 1 : 0);
  else if (upper !== null) candidate = upper.value - (upper.strict ? 1 : 0);
  if (linearIntervalContains(interval, candidate)) return candidate;
  const probes = [-10, -1, 0, 1, 10];
  const found = probes.find((probe) => linearIntervalContains(interval, probe));
  return found === undefined ? null : found;
}

/**
 * `x: > 1 and < 3`.
 * @param {{lower: {value: number, strict: boolean}|null, upper: {value: number, strict: boolean}|null}} interval
 * @returns {string}
 */
function linearIntervalSummary(interval) {
  const lower = interval.lower === null
    ? "-infinity"
    : `${interval.lower.strict ? ">" : ">="} ${linearFormatNumber(interval.lower.value)}`;
  const upper = interval.upper === null
    ? "infinity"
    : `${interval.upper.strict ? "<" : "<="} ${linearFormatNumber(interval.upper.value)}`;
  return `x: ${lower} and ${upper}`;
}

let PROOF_LIBRARY_TEMPLATES = null;

/**
 * The `template` records of data/seed/proof-library.lino by id, read once the
 * seed is loaded. Mirrors `proof_library()` in rust/src/seed/proof_library.rs.
 * @returns {Map<string, object>}
 */
function proofLibraryTemplates() {
  if (PROOF_LIBRARY_TEMPLATES !== null) return PROOF_LIBRARY_TEMPLATES;
  const root = parseLinoTree(seedRawText(SEED_RAW, "proof-library.lino"));
  const section = root.children.find((child) => child.name === "proof_library");
  if (!section) return new Map();
  PROOF_LIBRARY_TEMPLATES = new Map(
    section.children.filter((child) => child.name === "template").map((child) => [child.value, child]),
  );
  return PROOF_LIBRARY_TEMPLATES;
}

/**
 * A record's surface for `language`, falling back to English. Mirrors
 * `LocalizedText::get`.
 * @param {object} node
 * @param {string} language
 * @returns {string}
 */
function proofLibrarySurface(node, language) {
  const surface = (slug) =>
    node.children.find((child) => child.name === slug && child.children.length === 0 && child.value !== "");
  const found = surface(language) || surface("en");
  return found ? found.value : "";
}

/**
 * The phrase `id` in `language`, each `{name}` slot filled in one pass; a
 * missing record degrades to its id. Mirrors `ProofLibrary::text`.
 * @param {string} id
 * @param {string} language
 * @param {Object<string, string>} [values]
 * @returns {string}
 */
function proofLibraryText(id, language, values = {}) {
  const node = proofLibraryTemplates().get(id);
  if (!node) return id;
  return proofLibrarySurface(node, language).replace(/\{([^{}]*)\}/gu, (slot, name) =>
    Object.prototype.hasOwnProperty.call(values, name) ? values[name] : slot);
}

/**
 * The item phrases of the list template `id`. Mirrors `ProofLibrary::items`.
 * @param {string} id
 * @param {string} language
 * @returns {Array<string>}
 */
function proofLibraryItems(id, language) {
  const node = proofLibraryTemplates().get(id);
  if (!node) return [];
  return node.children.filter((child) => child.name === "item").map((item) => proofLibrarySurface(item, language));
}

/**
 * The interpretation header for a proven or disproven decision. Mirrors
 * `render_interpretation` in rust/src/proof_engine/presenter.rs.
 * @param {boolean} proven
 * @param {string} statement
 * @param {string} language
 * @returns {string}
 */
function proofPresenterInterpretation(proven, statement, language) {
  const method = proofLibraryText("method_decision_procedure", language);
  const detail = proven
    ? proofLibraryText("interpretation_proven", language, { statement, method })
    : proofLibraryText("interpretation_disproven", language, { method });
  return `${proofLibraryText("interpretation_label", language)}: ${detail}`;
}

/**
 * `Heading (method: label).`, as `method_line`.
 * @param {string} headingId
 * @param {string} language
 * @returns {string}
 */
function proofPresenterMethodLine(headingId, language) {
  return `${proofLibraryText(headingId, language)} (${proofLibraryText("method_intro", language)}: ${proofLibraryText("method_decision_procedure", language)}).`;
}

/**
 * `\n1. Definition: …` lines, as `render_steps`.
 * @param {Array<{kind: string, text: string}>} steps
 * @param {string} language
 * @returns {string}
 */
function proofPresenterSteps(steps, language) {
  return steps.map((step, index) =>
    `\n${index + 1}. ${proofLibraryText(`step_${step.kind}`, language)}: ${step.text}`).join("");
}

/**
 * Why an interval system has no model, in `language`. Mirrors
 * `IntervalSystem::contradiction` with the `linear_empty_intersection`
 * fallback; the bound conflict is the only contradiction interval riddles
 * produce.
 * @param {{lower: {value: number, strict: boolean}, upper: {value: number, strict: boolean}}|null} contradiction
 * @param {string} language
 * @returns {string}
 */
function linearContradiction(contradiction, language) {
  if (contradiction === null) return proofLibraryText("linear_empty_intersection", language);
  return proofLibraryText("linear_conflicting_bounds", language, {
    lower_symbol: contradiction.lower.strict ? ">" : ">=",
    lower: linearFormatNumber(contradiction.lower.value),
    upper_symbol: contradiction.upper.strict ? "<" : "<=",
    upper: linearFormatNumber(contradiction.upper.value),
  });
}

/**
 * Decide and render `<constraints> is satisfiable` the way
 * `attempt_proof_with_config` + `render_outcome_with_config` do for an
 * interval claim under the default render config.
 * @param {string} claim
 * @param {string} language
 * @returns {string}
 */
function numberConstraintFormalCheck(claim, language) {
  const suffix = " is satisfiable";
  if (!claim.endsWith(suffix)) throw new Error("interval claim must end with is satisfiable");
  const constraints = claim.slice(0, -suffix.length).trim();
  const atoms = [];
  for (const part of constraints.split(" and ")) {
    const atom = linearParseIntervalAtom(part.trim());
    if (atom === null) throw new Error("interval claim atom must be a bound");
    atoms.push(atom);
  }
  const interval = linearIntervalSystem(atoms);
  const constraintsText = atoms.map((atom) => atom.original).join(" and ");
  const statement = `${constraints} is satisfiable`;
  const witness = linearWitness(interval);
  const values = {
    constraints: constraintsText,
    interval: linearIntervalSummary(interval),
    assignment: witness === null ? "" : `x = ${linearFormatNumber(witness)}`,
  };
  const delegated = { kind: "definition", text: proofLibraryText("linear_delegate", language) };
  const constraintsStep = { kind: "definition", text: proofLibraryText("linear_constraints", language, values) };
  if (interval.contradiction === null && witness !== null) {
    const steps = [
      delegated,
      constraintsStep,
      { kind: "inference", text: proofLibraryText("linear_constraints_reduce", language, values) },
      { kind: "inference", text: proofLibraryText("linear_witness", language, values) },
    ];
    const core = `${proofPresenterMethodLine("proof_heading", language)}\n\n${proofLibraryText("statement_label", language)}: ${statement}\n${proofPresenterSteps(steps, language)}\n${proofLibraryText("linear_satisfiable", language)}`;
    return `${proofPresenterInterpretation(true, statement, language)}\n\n${core}`;
  }
  values.contradiction = linearContradiction(interval.contradiction, language);
  const steps = [
    delegated,
    constraintsStep,
    { kind: "inference", text: proofLibraryText("linear_empty_model_set", language, values) },
  ];
  const core = `${proofPresenterMethodLine("disproof_heading", language)}\n\n${proofLibraryText("counterexample_label", language)}: ${proofLibraryText("linear_no_assignment", language, values)}\n\n${proofPresenterSteps(steps, language)}\n${proofLibraryText("linear_unsatisfiable", language)}`;
  // `enforce_questions` (rust/src/question_necessity.rs) authorizes one
  // question per answer: the first follow-up stays, and every later one loses
  // its sentence, which starts after the item's "N. " prefix (the seeded
  // questions carry no inner sentence terminator), leaving the bare "N.".
  const questions = proofLibraryItems("disproven_follow_ups", language)
    .map((question, index) => (index === 0 ? `${index + 1}. ${question}` : `${index + 1}.`))
    .join("\n");
  return `${proofPresenterInterpretation(false, statement, language)}\n\n${core}\n\n${proofLibraryText("follow_up_label", language)}\n${questions}`;
}

/**
 * The compaction `enforce_questions` applies once it removed a question:
 * line ends trimmed, runs of blank lines collapsed to one, the whole body
 * trimmed. Mirrors `remove_ranges` in rust/src/question_necessity.rs.
 * @param {string} body
 * @returns {string}
 */
function numberConstraintCompactBody(body) {
  const kept = [];
  let blankLines = 0;
  for (const line of body.split("\n")) {
    if (line.trim() === "") {
      blankLines += 1;
      if (blankLines > 1) continue;
    } else {
      blankLines = 0;
    }
    kept.push(line.replace(/\s+$/u, ""));
  }
  return kept.join("\n").trim();
}

/**
 * The integer-solution sentence in the answer language, a
 * `number_constraint_integer_*` phrase of data/seed/proof-library.lino.
 * @param {{kind: string, start: bigint, end: bigint}} solutions
 * @param {string} language
 * @returns {string}
 */
function numberConstraintIntegerLine(solutions, language) {
  switch (solutions.kind) {
    case "unique":
      return proofLibraryText("number_constraint_integer_unique", language, { only: String(solutions.start) });
    case "none":
      return proofLibraryText("number_constraint_integer_none", language);
    case "range":
      return proofLibraryText("number_constraint_integer_range", language, {
        start: String(solutions.start),
        end: String(solutions.end),
      });
    case "multiple":
      return proofLibraryText("number_constraint_integer_multiple", language, {
        candidates: numberConstraintFormatCandidates(solutions.start, solutions.end),
      });
    default:
      throw new Error("unknown integer solution kind");
  }
}

/**
 * The real-domain sentence in the answer language. Mirrors
 * `real_domain_line` in rust/src/number_constraints.rs.
 * @param {{lower: {value: bigint, inclusive: boolean}, upper: {value: bigint, inclusive: boolean}}} bounds
 * @param {string} language
 * @returns {string}
 */
function numberConstraintRealLine(bounds, language) {
  if (bounds.lower.value < bounds.upper.value) {
    const example = numberConstraintFormatHalf(bounds.lower.value * 2n + 1n);
    return proofLibraryText("number_constraint_real_multiple", language, { example });
  }
  if (bounds.lower.value === bounds.upper.value && bounds.lower.inclusive && bounds.upper.inclusive) {
    return proofLibraryText("number_constraint_real_single", language, { value: String(bounds.lower.value) });
  }
  return proofLibraryText("number_constraint_real_inconsistent", language);
}

/**
 * Hidden-number interval riddle entry point. Mirrors
 * `solve_number_constraints` in rust/src/number_constraints.rs.
 * @param {string} prompt
 * @param {string} normalized
 * @param {string} language
 * @returns {{intent: string, content: string, confidence: number, evidence: Array<string>}|null}
 */
function tryNumberConstraintReasoning(prompt, normalized, language) {
  const lowercased = String(prompt || "").toLowerCase();
  const cleaned = normalizePrompt(normalized);
  if (!numberConstraintLooksLikeRiddle(cleaned, lowercased)) return null;
  const bounds = numberConstraintIntervalBounds(cleaned, lowercased);
  if (bounds === null) return null;
  const statement = numberConstraintProofStatement(bounds);
  const lowerOperator = numberConstraintLowerOperator(bounds.lower);
  const upperOperator = numberConstraintUpperOperator(bounds.upper);
  const decisionStatement = `x ${lowerOperator} ${bounds.lower.value} and x ${upperOperator} ${bounds.upper.value} is satisfiable`;
  const formalCheck = numberConstraintFormalCheck(decisionStatement, language);
  const solutions = numberConstraintIntegerSolutions(bounds);
  const body = proofLibraryText("number_constraint_answer", language, {
    integer_line: numberConstraintIntegerLine(solutions, language),
    formalization: `x ${lowerOperator} ${bounds.lower.value}, x ${upperOperator} ${bounds.upper.value}`,
    statement: statement,
    real_line: numberConstraintRealLine(bounds, language),
    formal_check: formalCheck,
  });
  // Only the disproof carries follow-up questions, so only it is compacted.
  const content = formalCheck.includes(`${proofLibraryText("follow_up_label", language)}\n`)
    ? numberConstraintCompactBody(body)
    : body;
  return {
    intent: "number_constraint_reasoning",
    content: content,
    confidence: 0.86,
    evidence: [
      "handler:number_constraint_reasoning",
      "reasoning:number_constraint:hidden_number_interval",
      `formalization:linear_constraint:${statement}`,
      "response:number_constraint_reasoning",
      `language:${language}`,
    ],
  };
}
