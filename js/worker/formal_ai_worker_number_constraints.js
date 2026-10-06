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
const LINEAR_DELEGATED_STEP =
  "Delegate the normalized claim to the relative-meta-logic / SMT decision procedure for quantifier-free linear real arithmetic.";

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
 * @returns {{lower: {value: number, strict: boolean}|null, upper: {value: number, strict: boolean}|null, contradiction: string|null}}
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
      interval.contradiction = `requires x ${lower.strict ? ">" : ">="} ${linearFormatNumber(lower.value)} and x ${upper.strict ? "<" : "<="} ${linearFormatNumber(upper.value)}`;
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
 * @param {{lower: {value: number, strict: boolean}|null, upper: {value: number, strict: boolean}|null, contradiction: string|null}} interval
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

/**
 * Seed-free localized labels of the native proof presenter.
 * @param {string} key
 * @param {string} language
 * @returns {string}
 */
function proofPresenterLabel(key, language) {
  const table = {
    method: { ru: "процедура разрешения relative-meta-logic / SMT", zh: "relative-meta-logic / SMT 判定过程", hi: "relative-meta-logic / SMT निर्णय प्रक्रिया", en: "relative-meta-logic / SMT decision procedure" },
    definition: { ru: "Определение", zh: "定义", hi: "परिभाषा", en: "Definition" },
    inference: { ru: "Вывод", zh: "推理", hi: "निष्कर्षण", en: "Inference" },
    interpretation: { ru: "Как я понял запрос", hi: "मैंने प्रश्न को कैसे समझा", zh: "对问题的理解", en: "How I interpreted the request" },
    proof: { ru: "Доказательство", hi: "प्रमाण", zh: "证明", en: "Proof" },
    disproof: { ru: "Опровержение", hi: "खंडन", zh: "反驳", en: "Disproof" },
    statement: { ru: "Утверждение", hi: "कथन", zh: "命题", en: "Statement" },
    counterexample: { ru: "Контрпример", hi: "प्रतिउदाहरण", zh: "反例", en: "Counterexample" },
    method_intro: { ru: "метод", hi: "विधि", zh: "方法", en: "method" },
    follow_up: { ru: "Уточняющие вопросы:", hi: "स्पष्टीकरण के प्रश्न:", zh: "澄清问题:", en: "Clarifying questions:" },
  };
  const row = table[key];
  if (!row) throw new Error("unknown proof presenter label");
  return row[language] || row.en;
}

/**
 * The interpretation detail for a proven or disproven decision.
 * @param {boolean} proven
 * @param {string} statement
 * @param {string} language
 * @returns {string}
 */
function proofPresenterInterpretation(proven, statement, language) {
  const method = proofPresenterLabel("method", language);
  let detail;
  if (proven) {
    switch (language) {
      case "ru":
        detail = `трактуем запрос как формальное утверждение «${statement}» и доказываем методом «${method}» в relative-meta-logic.`;
        break;
      case "hi":
        detail = `प्रश्न को औपचारिक कथन "${statement}" मानकर relative-meta-logic में "${method}" विधि से प्रमाणित कर रहे हैं।`;
        break;
      case "zh":
        detail = `把问题视为形式命题“${statement}”,在 relative-meta-logic 中用“${method}”方法证明。`;
        break;
      default:
        detail = `treating the request as the formal claim "${statement}" and discharging it by ${method} inside relative-meta-logic.`;
        break;
    }
  } else {
    switch (language) {
      case "ru":
        detail = `трактуем запрос как утверждение, которое нужно опровергнуть; используем ${method} и приводим контрпример.`;
        break;
      case "hi":
        detail = `प्रश्न को खंडन योग्य कथन मानकर ${method} का उपयोग कर रहे हैं और प्रतिउदाहरण देते हैं।`;
        break;
      case "zh":
        detail = `把问题视为应予反驳的断言,用${method}并给出反例。`;
        break;
      default:
        detail = `treating the request as a claim to be refuted; applying ${method} and producing a counterexample.`;
        break;
    }
  }
  return `${proofPresenterLabel("interpretation", language)}: ${detail}`;
}

/**
 * The follow-up questions after a disproof.
 * @param {string} language
 * @returns {Array<string>}
 */
function proofPresenterDisprovenQuestions(language) {
  switch (language) {
    case "ru":
      return [
        "хотите ли вы ослабить утверждение до проверяемой формы (например, заменить равенство неравенством или ограничить область)?",
        "если требуется ровно это утверждение, нужно ли добавить аксиому, при которой контрпример исключается?",
      ];
    case "hi":
      return [
        "क्या आप कथन को जाँचने योग्य रूप तक शिथिल करना चाहते हैं (जैसे समता को असमिका से बदलना या क्षेत्र सीमित करना)?",
        "यदि वही कथन ज़रूरी है, क्या आप कोई अभिगृहीत जोड़ना चाहते हैं जिससे प्रतिउदाहरण बाहर रहे?",
      ];
    case "zh":
      return [
        "是否希望把命题弱化为可证形式(例如把等式改为不等式,或限制定义域)?",
        "若需保留原命题,是否要新增一条公理以排除该反例?",
      ];
    default:
      return [
        "do you want to weaken the claim into a checkable form (e.g. replace equality with an inequality, or restrict the domain)?",
        "if the exact claim is required, should we add an axiom under which the counterexample is excluded?",
      ];
  }
}

/**
 * `\n1. Definition: …` lines, as `render_steps`.
 * @param {Array<{kind: string, text: string}>} steps
 * @param {string} language
 * @returns {string}
 */
function proofPresenterSteps(steps, language) {
  return steps.map((step, index) =>
    `\n${index + 1}. ${proofPresenterLabel(step.kind, language)}: ${step.text}`).join("");
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
  if (interval.contradiction === null && witness !== null) {
    const steps = [
      { kind: "definition", text: LINEAR_DELEGATED_STEP },
      { kind: "definition", text: `Constraints: ${constraintsText}.` },
      { kind: "inference", text: `The constraints reduce to ${linearIntervalSummary(interval)}.` },
      { kind: "inference", text: `Witness found: x = ${linearFormatNumber(witness)}.` },
    ];
    const core = `${proofPresenterLabel("proof", language)} (${proofPresenterLabel("method_intro", language)}: ${proofPresenterLabel("method", language)}).\n\n${proofPresenterLabel("statement", language)}: ${statement}\n${proofPresenterSteps(steps, language)}\nTherefore the constraint system is satisfiable. ∎`;
    return `${proofPresenterInterpretation(true, statement, language)}\n\n${core}`;
  }
  const contradiction = interval.contradiction === null
    ? "the interval constraints have empty intersection"
    : interval.contradiction;
  const steps = [
    { kind: "definition", text: LINEAR_DELEGATED_STEP },
    { kind: "definition", text: `Constraints: ${constraintsText}.` },
    {
      kind: "inference",
      text: `The interval solver reports an empty model set: ${contradiction}. Last interval state: ${linearIntervalSummary(interval)}.`,
    },
  ];
  const core = `${proofPresenterLabel("disproof", language)} (${proofPresenterLabel("method_intro", language)}: ${proofPresenterLabel("method", language)}).\n\n${proofPresenterLabel("counterexample", language)}: No assignment exists: ${contradiction}.\n\n${proofPresenterSteps(steps, language)}\nTherefore the constraint system is unsatisfiable. ∎`;
  // `enforce_questions` (rust/src/question_necessity.rs) authorizes one
  // question per answer: the first follow-up stays, and every later one loses
  // its sentence, which starts after the item's "N. " prefix (the seeded
  // questions carry no inner sentence terminator), leaving the bare "N.".
  const questions = proofPresenterDisprovenQuestions(language)
    .map((question, index) => (index === 0 ? `${index + 1}. ${question}` : `${index + 1}.`))
    .join("\n");
  return `${proofPresenterInterpretation(false, statement, language)}\n\n${core}\n\n${proofPresenterLabel("follow_up", language)}\n${questions}`;
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
 * The integer-solution sentence in the answer language.
 * @param {{kind: string, start: bigint, end: bigint}} solutions
 * @param {boolean} russian
 * @returns {string}
 */
function numberConstraintIntegerLine(solutions, russian) {
  switch (solutions.kind) {
    case "unique":
      return russian
        ? `Если это задача про целое число, единственный ответ: ${solutions.start}.`
        : `If this is an integer-number riddle, the unique answer is ${solutions.start}.`;
    case "none":
      return russian
        ? "Если это задача про целое число, решения нет."
        : "If this is an integer-number riddle, there is no solution.";
    case "range":
      return russian
        ? `Если это задача про целые числа, ответ не единственный: подходит любое целое от ${solutions.start} до ${solutions.end}.`
        : `If this is an integer-number riddle, the answer is not unique: every integer from ${solutions.start} through ${solutions.end} fits.`;
    case "multiple": {
      const candidates = numberConstraintFormatCandidates(solutions.start, solutions.end);
      return russian
        ? `Если это задача про целые числа, ответ не единственный: подходят ${candidates}.`
        : `If this is an integer-number riddle, the answer is not unique: ${candidates} all fit.`;
    }
    default:
      throw new Error("unknown integer solution kind");
  }
}

/**
 * The real-domain sentence in the answer language.
 * @param {{lower: {value: bigint, inclusive: boolean}, upper: {value: bigint, inclusive: boolean}}} bounds
 * @param {boolean} russian
 * @returns {string}
 */
function numberConstraintRealLine(bounds, russian) {
  if (bounds.lower.value < bounds.upper.value) {
    const example = numberConstraintFormatHalf(bounds.lower.value * 2n + 1n);
    return russian
      ? `Если разрешены вещественные числа, ответ не единственный: например, x = ${example} тоже подходит.`
      : `If real numbers are allowed, the answer is not unique; for example, x = ${example} also fits.`;
  }
  if (bounds.lower.value === bounds.upper.value && bounds.lower.inclusive && bounds.upper.inclusive) {
    return russian
      ? `На вещественных числах тоже есть единственное решение: x = ${bounds.lower.value}.`
      : `Over the real numbers there is also a single solution: x = ${bounds.lower.value}.`;
  }
  return russian
    ? "На вещественных числах эти ограничения несовместимы."
    : "Over the real numbers, these constraints are inconsistent.";
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
  const russian = language === "ru";
  const integerLine = numberConstraintIntegerLine(solutions, russian);
  const realLine = numberConstraintRealLine(bounds, russian);
  const formalization = `x ${lowerOperator} ${bounds.lower.value}, x ${upperOperator} ${bounds.upper.value}`;
  const body = russian
    ? `${integerLine}\n\nФормализация над целыми: x in Z, ${formalization}. Проверяемая форма для решателя: \`${statement}\`.\n\n${realLine}\n\nФормальная проверка relative-meta-logic / SMT:\n${formalCheck}`
    : `${integerLine}\n\nInteger formalization: x in Z, ${formalization}. Solver form: \`${statement}\`.\n\n${realLine}\n\nFormal relative-meta-logic / SMT check:\n${formalCheck}`;
  // Only the disproof carries follow-up questions, so only it is compacted.
  const content = formalCheck.includes(`${proofPresenterLabel("follow_up", language)}\n`)
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
