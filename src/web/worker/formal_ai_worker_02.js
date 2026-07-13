// Worker module 3 of 21. Loaded by ../formal_ai_worker.js.
function formatLinearSymbolicSolution(constant, terms) {
  const parts = [];
  if (!equationNearlyZero(constant)) {
    parts.push(formatArithmeticResult(constant));
  }
  for (const [variable, coefficient] of terms) {
    if (equationNearlyZero(coefficient)) continue;
    const absolute = Math.abs(coefficient);
    const body = equationNearlyZero(absolute - 1)
      ? variable
      : `${formatArithmeticResult(absolute)}*${variable}`;
    if (parts.length === 0) {
      parts.push(coefficient < 0 ? `-${body}` : body);
    } else {
      parts.push(`${coefficient < 0 ? "-" : "+"} ${body}`);
    }
  }
  return parts.length > 0 ? parts.join(" ") : "0";
}

function solveLinearEquation(expression) {
  const parts = String(expression).split("=");
  if (parts.length !== 2) throw new Error("expression could not be parsed");
  const left = parseLinearExpression(parts[0]);
  const right = parseLinearExpression(parts[1]);
  const variableOrder = left.variables.concat(
    right.variables.filter((name) => !left.variables.includes(name)),
  );
  const combined = linearSubtract(left.value, right.value);
  const activeVariables = variableOrder.filter((name) =>
    !equationNearlyZero(combined.terms[name] || 0),
  );
  const variable =
    activeVariables.find((name) => isEquationUnknownPlaceholder(name)) ||
    activeVariables[0];
  if (!variable) throw new Error("expression could not be parsed");
  const coefficient = combined.terms[variable] || 0;
  if (equationNearlyZero(coefficient)) throw new Error("expression could not be parsed");

  const constant = -combined.constant / coefficient;
  const symbolicTerms = [];
  for (const name of activeVariables) {
    if (name === variable) continue;
    symbolicTerms.push([name, -(combined.terms[name] || 0) / coefficient]);
  }
  const hasSymbolicTerms = symbolicTerms.some((entry) => !equationNearlyZero(entry[1]));
  const rendered = hasSymbolicTerms
    ? formatLinearSymbolicSolution(constant, symbolicTerms)
    : formatArithmeticResult(constant);
  return `${variable} = ${rendered}`;
}

function polynomialConstant(value) {
  const coefficients = Object.create(null);
  if (!equationNearlyZero(value)) coefficients[0] = value;
  return { coefficients };
}

function polynomialVariable() {
  const coefficients = Object.create(null);
  coefficients[1] = 1;
  return { coefficients };
}

function polynomialCoefficient(poly, degree) {
  return poly.coefficients[degree] || 0;
}

function polynomialEntries(poly) {
  return Object.entries(poly.coefficients)
    .map(([degree, coefficient]) => [Number(degree), coefficient])
    .filter((entry) => !equationNearlyZero(entry[1]));
}

function polynomialDegree(poly) {
  return polynomialEntries(poly).reduce(
    (degree, entry) => Math.max(degree, entry[0]),
    0,
  );
}

function polynomialAdd(left, right) {
  const coefficients = Object.create(null);
  for (const [degree, coefficient] of polynomialEntries(left)) {
    coefficients[degree] = (coefficients[degree] || 0) + coefficient;
  }
  for (const [degree, coefficient] of polynomialEntries(right)) {
    coefficients[degree] = (coefficients[degree] || 0) + coefficient;
  }
  return { coefficients };
}

function polynomialSubtract(left, right) {
  const coefficients = Object.create(null);
  for (const [degree, coefficient] of polynomialEntries(left)) {
    coefficients[degree] = (coefficients[degree] || 0) + coefficient;
  }
  for (const [degree, coefficient] of polynomialEntries(right)) {
    coefficients[degree] = (coefficients[degree] || 0) - coefficient;
  }
  return { coefficients };
}

function polynomialScale(poly, scalar) {
  const coefficients = Object.create(null);
  for (const [degree, coefficient] of polynomialEntries(poly)) {
    coefficients[degree] = coefficient * scalar;
  }
  return { coefficients };
}

function polynomialMultiply(left, right) {
  const coefficients = Object.create(null);
  for (const [leftDegree, leftCoefficient] of polynomialEntries(left)) {
    for (const [rightDegree, rightCoefficient] of polynomialEntries(right)) {
      const degree = leftDegree + rightDegree;
      coefficients[degree] =
        (coefficients[degree] || 0) + leftCoefficient * rightCoefficient;
    }
  }
  return { coefficients };
}

function polynomialDivide(left, right) {
  if (polynomialDegree(right) !== 0) throw new Error("variable denominator");
  const denominator = polynomialCoefficient(right, 0);
  if (equationNearlyZero(denominator)) throw new Error("division by zero");
  return polynomialScale(left, 1 / denominator);
}

function polynomialPower(poly, exponent) {
  if (
    polynomialDegree(exponent) !== 0 ||
    !Number.isInteger(polynomialCoefficient(exponent, 0)) ||
    polynomialCoefficient(exponent, 0) < 0 ||
    polynomialCoefficient(exponent, 0) > 6
  ) {
    throw new Error("unsupported polynomial exponent");
  }
  let result = polynomialConstant(1);
  for (let i = 0; i < polynomialCoefficient(exponent, 0); i += 1) {
    result = polynomialMultiply(result, poly);
  }
  return result;
}

function parsePolynomialExpression(input) {
  const state = { position: 0 };
  let variable = null;

  function peek() {
    return input[state.position] || "";
  }

  function skipWhitespace() {
    while (/\s/.test(peek())) state.position += 1;
  }

  function consume(expected) {
    if (peek() === expected) {
      state.position += 1;
      return true;
    }
    return false;
  }

  function rememberVariable(name) {
    if (variable && variable !== name) throw new Error("multiple variables");
    variable = name;
  }

  function parseExpression() {
    let value = parseTerm();
    while (true) {
      skipWhitespace();
      if (consume("+")) {
        value = polynomialAdd(value, parseTerm());
      } else if (consume("-") || consume("−")) {
        value = polynomialSubtract(value, parseTerm());
      } else {
        return value;
      }
    }
  }

  function parseTerm() {
    let value = parseUnary();
    while (true) {
      skipWhitespace();
      if (consume("*") || consume("×") || consume("·")) {
        value = polynomialMultiply(value, parseUnary());
      } else if (consume("/") || consume("÷")) {
        value = polynomialDivide(value, parseUnary());
      } else {
        return value;
      }
    }
  }

  function parseUnary() {
    skipWhitespace();
    if (consume("+")) return parseUnary();
    if (consume("-") || consume("−")) return polynomialScale(parseUnary(), -1);
    return parsePower();
  }

  function parsePower() {
    let value = parsePrimary();
    skipWhitespace();
    if (consume("^")) {
      value = polynomialPower(value, parseUnary());
    }
    return value;
  }

  function parsePrimary() {
    skipWhitespace();
    if (consume("(")) {
      const value = parseExpression();
      skipWhitespace();
      if (!consume(")")) throw new Error("unbalanced parentheses");
      return value;
    }
    if (/[0-9.]/.test(peek())) return parseEquationNumber(input, state, polynomialConstant);
    if (/\p{L}/u.test(peek()) || isEquationUnknownPlaceholder(peek())) {
      parseEquationVariable(input, state, rememberVariable);
      return polynomialVariable();
    }
    throw new Error("expression could not be parsed");
  }

  const value = parseExpression();
  skipWhitespace();
  if (state.position !== input.length) throw new Error("expression could not be parsed");
  return { value, variable };
}

function evaluatePolynomial(poly, value) {
  return polynomialEntries(poly).reduce(
    (total, entry) => total + entry[1] * value ** entry[0],
    0,
  );
}

function findPolynomialRealRoots(poly) {
  const roots = [];
  for (let denominator = 1; denominator <= 20; denominator += 1) {
    for (let numerator = -200; numerator <= 200; numerator += 1) {
      const candidate = numerator / denominator;
      if (roots.some((root) => Math.abs(root - candidate) < 1e-8)) continue;
      if (Math.abs(evaluatePolynomial(poly, candidate)) < 1e-8) {
        roots.push(candidate);
      }
    }
  }
  return roots.sort((left, right) => left - right);
}

function solvePolynomialEquation(expression) {
  const parts = String(expression).split("=");
  if (parts.length !== 2) throw new Error("expression could not be parsed");
  const left = parsePolynomialExpression(parts[0]);
  const right = parsePolynomialExpression(parts[1]);
  if (!left.variable && !right.variable) throw new Error("expression could not be parsed");
  if (left.variable && right.variable && left.variable !== right.variable) {
    throw new Error("expression could not be parsed");
  }
  const variable = left.variable || right.variable;
  const combined = polynomialSubtract(left.value, right.value);
  if (polynomialDegree(combined) < 2) throw new Error("expression could not be parsed");
  const roots = findPolynomialRealRoots(combined);
  if (roots.length === 0) throw new Error("expression could not be parsed");
  return roots
    .map((root) => `${variable} = ${formatArithmeticResult(root)}`)
    .join(" or ");
}

function solveEquation(expression) {
  try {
    return solveLinearEquation(expression);
  } catch (_linearError) {
    return solvePolynomialEquation(expression);
  }
}

function hasArithmeticWordOperator(expression) {
  // Issue #386: the spelled operators come from the arithmetic_operation
  // meanings (addition, subtraction, multiplication, division, modulo) by
  // role, not a literal array. Each surface is matched as a whole token — CJK
  // surfaces as a substring — the boundary contract the former space-padded
  // `.includes` checks enforced. The match goes through the *spelled* query so
  // a meaning's value surface (the operator symbol "+") stays neutral here and
  // is caught by the symbol parser instead, mirroring contains_word_operator in
  // src/calculation.rs.
  return lexiconMentionsRoleSpelled(
    ROLE_ARITHMETIC_OPERATOR_WORD,
    String(expression).toLowerCase(),
  );
}

function hasSpelledArithmetic(expression) {
  // Issue #386: the cardinal number words come from the cardinal_number
  // meanings by role, not a literal array. Pure-numeral surfaces ("10") are
  // skipped — a bare digit run is handled by the numeric parser — and each
  // spelled surface is matched as a space-bounded whole token, mirroring
  // contains_spelled_arithmetic in src/calculation.rs.
  const lower = ` ${String(expression).toLowerCase()} `;
  const hasNumberWord = roleWordForms(ROLE_CARDINAL_NUMBER_WORD).some((form) => {
    if (
      [...form.text].every(
        (character) => character >= "0" && character <= "9",
      )
    ) {
      return false;
    }
    return lower.includes(` ${form.text} `);
  });
  return hasNumberWord && hasArithmeticWordOperator(expression);
}

// Issue #334 step 2: the website demo's second agent step was "calculate the
// 10th Fibonacci number and multiply it by 8% of 500. Show me the code and the
// final result." It is not a calculator expression, but it reduces to one once
// the symbolic Fibonacci reference is resolved (F(10) = 55), the spelled-out
// operator is rewritten to `*`, and the trailing instruction sentence is
// dropped — yielding `55 * 8% of 500` = 2200. The helpers below mirror
// `fibonacci_value`, `parse_ordinal`, `bare_word`, `resolve_fibonacci_references`,
// `split_sentences` and `normalize_word_problem` in `src/calculation.rs`.
function fibonacciValue(n) {
  if (n === 0) return 0;
  let previous = 0;
  let current = 1;
  for (let step = 1; step < n; step += 1) {
    const next = previous + current;
    previous = current;
    current = next;
  }
  return current;
}

const ORDINAL_WORDS = {
  first: 1,
  second: 2,
  third: 3,
  fourth: 4,
  fifth: 5,
  sixth: 6,
  seventh: 7,
  eighth: 8,
  ninth: 9,
  tenth: 10,
};

// Lowercased, punctuation-trimmed view of a token for keyword comparisons.
function trimNonAlnum(token) {
  return String(token || "")
    .replace(/^[^\p{L}\p{N}]+/u, "")
    .replace(/[^\p{L}\p{N}]+$/u, "");
}

function bareWord(token) {
  return trimNonAlnum(token).toLowerCase();
}

// Parse a leading ordinal/cardinal token such as "10th", "10", "3rd" or the
// spelled-out "tenth" into its numeric value. Returns null for anything else.
function parseOrdinal(token) {
  const trimmed = trimNonAlnum(token);
  if (!trimmed) return null;
  const digits = (trimmed.match(/^[0-9]+/) || [""])[0];
  if (digits) {
    const suffix = trimmed.slice(digits.length);
    if (suffix === "" || ["st", "nd", "rd", "th"].includes(suffix)) {
      return Number.parseInt(digits, 10);
    }
    return null;
  }
  const value = ORDINAL_WORDS[trimmed.toLowerCase()];
  return value === undefined ? null : value;
}

// Replace "(the) N-th Fibonacci number" references with their numeric value.
function resolveFibonacciReferences(text) {
  if (!text.toLowerCase().includes("fibonacci")) return text;
  const tokens = text.split(/\s+/).filter(Boolean);
  const out = [];
  let index = 0;
  while (index < tokens.length) {
    const n = parseOrdinal(tokens[index]);
    if (
      n !== null &&
      tokens[index + 1] !== undefined &&
      bareWord(tokens[index + 1]) === "fibonacci"
    ) {
      // Drop a determiner we already emitted ("the 10th" -> "55").
      if (out.length > 0 && bareWord(out[out.length - 1]) === "the") {
        out.pop();
      }
      out.push(String(fibonacciValue(n)));
      index += 2;
      // Absorb a trailing "number" / "term" / "sequence" noun.
      const noun = tokens[index] !== undefined ? bareWord(tokens[index]) : "";
      if (noun === "number" || noun === "term" || noun === "sequence") {
        index += 1;
      }
      continue;
    }
    out.push(tokens[index]);
    index += 1;
  }
  return out.join(" ");
}

// Split text into sentences on a period that ends a sentence (followed by
// whitespace or the end of the string). A period flanked by digits ("3.14") is
// kept inside its sentence so decimals are never broken apart.
function splitSentences(text) {
  const chars = Array.from(String(text || ""));
  const sentences = [];
  let current = "";
  for (let i = 0; i < chars.length; i += 1) {
    const ch = chars[i];
    const next = chars[i + 1];
    if (ch === "." && (next === undefined || /\s/.test(next))) {
      const sentence = current.trim();
      if (sentence) sentences.push(sentence);
      current = "";
      continue;
    }
    current += ch;
  }
  const last = current.trim();
  if (last) sentences.push(last);
  return sentences;
}

function wordProblemWords(sentence) {
  return String(sentence || "")
    .split(/[^\p{L}\p{N}]+/u)
    .filter(Boolean)
    .map((token) => token.toLowerCase());
}

function parseWordProblemInteger(token) {
  const text = String(token || "").toLowerCase();
  if (/^[+-]?\d+$/.test(text)) return Number.parseInt(text, 10);
  const numbers = {
    zero: 0,
    one: 1,
    a: 1,
    an: 1,
    two: 2,
    three: 3,
    four: 4,
    five: 5,
    six: 6,
    seven: 7,
    eight: 8,
    nine: 9,
    ten: 10,
    eleven: 11,
    twelve: 12,
    thirteen: 13,
    fourteen: 14,
    fifteen: 15,
    sixteen: 16,
    seventeen: 17,
    eighteen: 18,
    nineteen: 19,
    twenty: 20,
  };
  return Object.prototype.hasOwnProperty.call(numbers, text) ? numbers[text] : null;
}

function canonicalBoxId(token) {
  const cleaned = trimNonAlnum(token).toUpperCase();
  return cleaned && cleaned.length <= 3 ? cleaned : "";
}

function parseDeclaredBoxCount(words) {
  if (
    words.length >= 4 &&
    words[0] === "i" &&
    words[1] === "have" &&
    (words[3] === "box" || words[3] === "boxes")
  ) {
    const count = parseWordProblemInteger(words[2]);
    return Number.isInteger(count) ? count : null;
  }
  return null;
}

function parseBoxRule(words) {
  let index = words[0] === "if" ? 1 : 0;
  if (words[index] !== "box" || words[index + 2] !== "has") return null;
  const target = canonicalBoxId(words[index + 1]);
  if (!target) return null;
  index += 3;

  if (
    words[index] === "twice" &&
    words[index + 1] === "as" &&
    words[index + 2] === "many" &&
    words[index + 4] === "as" &&
    words[index + 5] === "box"
  ) {
    const source = canonicalBoxId(words[index + 6]);
    if (!source) return null;
    return {
      target,
      rule: { kind: "multiple", factor: 2, source },
      item: words[index + 3] || "",
    };
  }

  const value = parseWordProblemInteger(words[index]);
  if (!Number.isInteger(value)) return null;
  if (
    words[index + 1] === "more" &&
    words[index + 3] === "than" &&
    words[index + 4] === "box"
  ) {
    const source = canonicalBoxId(words[index + 5]);
    if (!source) return null;
    return {
      target,
      rule: { kind: "add", source, addend: value },
      item: words[index + 2] || "",
    };
  }

  return {
    target,
    rule: { kind: "known", value },
    item: words[index + 1] || "",
  };
}

function resolveBoxValue(id, rules, memo, stack, reasoningSteps, resultLabel) {
  if (memo.has(id)) return memo.get(id);
  if (stack.includes(id)) return null;
  const rule = rules.get(id);
  if (!rule) return null;
  stack.push(id);
  let value = null;
  if (rule.kind === "known") {
    value = rule.value;
    reasoningSteps.push(`Box ${id} = ${value} ${resultLabel}.`);
  } else if (rule.kind === "multiple") {
    const sourceValue = resolveBoxValue(
      rule.source,
      rules,
      memo,
      stack,
      reasoningSteps,
      resultLabel,
    );
    if (!Number.isFinite(sourceValue)) return null;
    value = sourceValue * rule.factor;
    reasoningSteps.push(
      `Box ${id} = ${rule.factor} * ${sourceValue} = ${value} ${resultLabel}.`,
    );
  } else if (rule.kind === "add") {
    const sourceValue = resolveBoxValue(
      rule.source,
      rules,
      memo,
      stack,
      reasoningSteps,
      resultLabel,
    );
    if (!Number.isFinite(sourceValue)) return null;
    value = sourceValue + rule.addend;
    reasoningSteps.push(
      `Box ${id} = ${sourceValue} + ${rule.addend} = ${value} ${resultLabel}.`,
    );
  }
  stack.pop();
  if (!Number.isFinite(value)) return null;
  memo.set(id, value);
  return value;
}

function normalizeBoxTotalProblem(text) {
  const lower = String(text || "").toLowerCase();
  if (
    !lower.includes("box") ||
    !lower.includes("how many") ||
    !lower.includes("total") ||
    (!lower.includes("twice as many") &&
      !(lower.includes("more") && lower.includes("than")))
  ) {
    return null;
  }

  let declaredCount = null;
  let resultLabel = "";
  const rules = new Map();
  for (const sentence of splitSentences(text)) {
    const words = wordProblemWords(sentence);
    if (words.length === 0) continue;
    if (declaredCount === null) {
      declaredCount = parseDeclaredBoxCount(words);
    }
    const parsed = parseBoxRule(words);
    if (!parsed) continue;
    if (parsed.item && parsed.item !== "box" && parsed.item !== "boxes") {
      resultLabel = parsed.item;
    }
    rules.set(parsed.target, parsed.rule);
  }
  if (rules.size < 2) return null;
  if (declaredCount !== null && rules.size < declaredCount) return null;

  const label = resultLabel || "items";
  const memo = new Map();
  const reasoningSteps = [];
  const ids = Array.from(rules.keys()).sort();
  for (const id of ids) {
    if (
      !Number.isFinite(
        resolveBoxValue(id, rules, memo, [], reasoningSteps, label),
      )
    ) {
      return null;
    }
  }
  const values = ids.map((id) => memo.get(id));
  if (values.some((value) => !Number.isFinite(value))) return null;
  const expression = values.join(" + ");
  reasoningSteps.push(`Total = ${expression} ${label}.`);
  return { expression, reasoningSteps, resultLabel: label };
}

function parseMotionDecimalToken(token) {
  const cleaned = String(token || "")
    .replace(/^[^0-9.+-]+/, "")
    .replace(/[^0-9.]+$/, "");
  if (!cleaned || cleaned === "." || cleaned === "-" || cleaned === "+") return null;
  if ((cleaned.match(/\./g) || []).length > 1) return null;
  const value = Number.parseFloat(cleaned);
  return Number.isFinite(value) ? value : null;
}

function motionUnitToken(token) {
  return String(token || "")
    .replace(/^[^A-Za-z0-9/]+/, "")
    .replace(/[^A-Za-z0-9/]+$/, "")
    .toLowerCase();
}

function isMotionSpeedUnit(unit) {
  return unit === "km/h" || unit === "kph" || unit === "kmh";
}

function isMotionDistanceUnit(unit) {
  return (
    unit === "km" ||
    unit === "kilometer" ||
    unit === "kilometers" ||
    unit === "kilometre" ||
    unit === "kilometres"
  );
}

function formatMotionQuantity(value) {
  if (Math.abs(value % 1) < 1e-10) return value.toFixed(0);
  return value.toFixed(10).replace(/0+$/, "").replace(/\.$/, "");
}

function cleanMotionOriginToken(token, isLast) {
  let cleaned = String(token || "")
    .replace(/^[^\p{L}\p{N}.]+/u, "")
    .replace(/[^\p{L}\p{N}.]+$/u, "");
  if (isLast) cleaned = cleaned.replace(/[.?!]+$/, "");
  return cleaned;
}

function extractMotionOriginBeforeSpeed(tokens, speedIndex) {
  if (speedIndex === 0) return "";
  let start = Math.max(0, speedIndex - 1);
  for (let index = speedIndex - 1; index >= 0; index -= 1) {
    const word = bareWord(tokens[index]);
    if (
      word === "leaves" ||
      word === "leave" ||
      word === "left" ||
      word === "departs" ||
      word === "depart" ||
      word === "starts" ||
      word === "start"
    ) {
      start = index + 1;
      break;
    }
  }
  if (start >= speedIndex) return "";

  const originTokens = tokens.slice(start, speedIndex);
  let firstOriginToken = 0;
  while (
    originTokens[firstOriginToken] !== undefined &&
    bareWord(originTokens[firstOriginToken]) === "from"
  ) {
    firstOriginToken += 1;
  }
  let lastOriginToken = originTokens.length;
  while (lastOriginToken > firstOriginToken) {
    const word = bareWord(originTokens[lastOriginToken - 1]);
    if (word !== "at" && word !== "with") break;
    lastOriginToken -= 1;
  }
  const selected = originTokens.slice(firstOriginToken, lastOriginToken);
  return selected
    .map((token, index) => cleanMotionOriginToken(token, index + 1 === selected.length))
    .filter(Boolean)
    .join(" ");
}

function normalizeTrainMeetingProblem(text) {
  const lower = String(text || "").toLowerCase();
  if (
    !lower.includes("meet") ||
    !lower.includes("distance") ||
    !["km/h", "kph", "kmh"].some((unit) => lower.includes(unit))
  ) {
    return null;
  }

  const tokens = String(text || "").split(/\s+/).filter(Boolean);
  const speeds = [];
  let statedDistance = null;
  let fallbackDistance = null;
  for (let index = 0; index + 1 < tokens.length; index += 1) {
    const value = parseMotionDecimalToken(tokens[index]);
    if (value === null) continue;
    const unit = motionUnitToken(tokens[index + 1]);
    if (isMotionSpeedUnit(unit)) {
      const origin =
        extractMotionOriginBeforeSpeed(tokens, index) || `train ${speeds.length + 1}`;
      speeds.push({ value, origin });
    } else if (isMotionDistanceUnit(unit)) {
      const previous = index > 0 ? bareWord(tokens[index - 1]) : "";
      if (previous === "distance") {
        statedDistance = value;
      } else if (fallbackDistance === null) {
        fallbackDistance = value;
      }
    }
  }

  if (speeds.length < 2) return null;
  const distance = statedDistance !== null ? statedDistance : fallbackDistance;
  if (!Number.isFinite(distance)) return null;
  const first = speeds[0];
  const second = speeds[1];
  const relativeSpeed = first.value + second.value;
  if (!Number.isFinite(relativeSpeed) || relativeSpeed <= 0) return null;

  const time = distance / relativeSpeed;
  const firstDistance = first.value * time;
  const secondDistance = second.value * time;
  const distanceText = formatMotionQuantity(distance);
  const firstSpeedText = formatMotionQuantity(first.value);
  const secondSpeedText = formatMotionQuantity(second.value);
  const relativeSpeedText = formatMotionQuantity(relativeSpeed);
  const timeText = formatMotionQuantity(time);
  const firstDistanceText = formatMotionQuantity(firstDistance);
  const secondDistanceText = formatMotionQuantity(secondDistance);
  const expression = `${distanceText} / (${firstSpeedText} + ${secondSpeedText})`;

  return {
    expression,
    reasoningSteps: [
      `[STEP 1] Define variables: distance = ${distanceText} km, ${first.origin} train speed = ${firstSpeedText} km/h, ${second.origin} train speed = ${secondSpeedText} km/h, and t = meeting time in hours. [VERIFY] Units are consistent: kilometers divided by kilometers per hour gives hours.`,
      `[STEP 2] Write equation: (${firstSpeedText} + ${secondSpeedText}) * t = ${distanceText}. [VERIFY] The trains move toward each other, so their relative speed is ${relativeSpeedText} km/h.`,
      `[STEP 3] Solve algebraically: t = ${distanceText} / (${firstSpeedText} + ${secondSpeedText}) = ${timeText} hours. [VERIFY] ${firstSpeedText} + ${secondSpeedText} = ${relativeSpeedText} and ${distanceText} / ${relativeSpeedText} = ${timeText}.`,
      `[STEP 4] Interpret result: the ${first.origin} train travels ${firstSpeedText} * ${timeText} = ${firstDistanceText} km; the ${second.origin} train travels ${secondSpeedText} * ${timeText} = ${secondDistanceText} km. [VERIFY] ${firstDistanceText} + ${secondDistanceText} = ${distanceText} km.`,
      `[STEP 5] Convert to user-friendly format: they meet after ${timeText} hours, ${firstDistanceText} km from ${first.origin} and ${secondDistanceText} km from ${second.origin}. [VERIFY] Both distances add to the stated route length.`,
      "[COMPARE] Formal-ai uses the same relative-speed equation as the direct solution; the verification tags make each assumption and arithmetic check explicit.",
    ],
    resultLabel: "",
  };
}

// Rewrite a natural-language "word problem" into a calculator expression, or
// return null when no rewrite applies so callers fall through unchanged.
function normalizeWordProblemDetailed(expression) {
  const trimmed = String(expression || "").trim();
  if (!trimmed) return null;
  const boxProblem = normalizeBoxTotalProblem(trimmed);
  if (boxProblem) return boxProblem;
  const trainMeetingProblem = normalizeTrainMeetingProblem(trimmed);
  if (trainMeetingProblem) return trainMeetingProblem;
  // Keep only sentence fragments that carry arithmetic content, dropping pure
  // instruction clauses such as "Show me the code and the final result".
  const arithmetic = splitSentences(trimmed).filter(
    (sentence) => sentence && (/[0-9]/.test(sentence) || sentence.includes("%")),
  );
  if (arithmetic.length === 0) return null;
  let working = resolveFibonacciReferences(arithmetic.join(". "));
  // Rewrite spelled-out operators the calculator does not accept. Longer phrases
  // come first so "and multiply it by" wins over "multiply by".
  const operatorPhrases = [
    [" and multiply it by ", " * "],
    [" and multiply by ", " * "],
    [" multiply it by ", " * "],
    [" multiplied by ", " * "],
    [" multiply by ", " * "],
    [" and divide it by ", " / "],
    [" and divide by ", " / "],
    [" divide it by ", " / "],
    [" divided by ", " / "],
    [" divide by ", " / "],
  ];
  for (const [phrase, symbol] of operatorPhrases) {
    const position = working.toLowerCase().indexOf(phrase);
    if (position !== -1) {
      working =
        working.slice(0, position) + symbol + working.slice(position + phrase.length);
    }
  }
  working = working.split(/\s+/).filter(Boolean).join(" ");
  if (!working || working.toLowerCase() === trimmed.toLowerCase()) return null;
  return { expression: working, reasoningSteps: [], resultLabel: "" };
}

// Issue #386: the calculator-domain signal set is rebuilt from three seed roles
// instead of a 62-entry literal array, so the router reasons over the same
// self-describing lexicon every other handler reads. Each surface is shaped into
// a match pattern by its script and role, mirroring calculator_domain_signals in
// src/calculation.rs:
//
// - math_function_name ("sqrt", "sin", "логарифм", "对数", …): an ASCII name gains
//   only a leading space so it still fires when glued to its argument's
//   parenthesis ("sqrt(16)"); a non-ASCII name matches as a raw substring because
//   those scripts have no inter-word spaces.
// - calculation_domain_term (currencies and measurement units: "usd", "kg",
//   "ms", "доллар", "公斤", "месяцев", …): an ASCII surface matches as a whole
//   token (leading and trailing space) so a short code like "ms" never fires
//   inside "items" nor "mb" inside "number"; a non-ASCII surface matches as a raw
//   substring.
// - quantity_conversion_cue, CJK members only ("换成", …): the Chinese conversion
//   verbs match as raw substrings; the Latin cues ("to", "into") are excluded
//   here because they are far too common to mark a calculation on their own.
//
// The caller pads the lowercased expression with surrounding spaces and tests
// includes against each signal, so the set — not its order — decides.
function calculatorDomainSignals() {
  const signals = [];
  for (const surface of wordsForRole(ROLE_MATH_FUNCTION_NAME)) {
    signals.push(isAsciiText(surface) ? ` ${surface}` : surface);
  }
  for (const surface of wordsForRole(ROLE_CALCULATION_DOMAIN_TERM)) {
    signals.push(isAsciiText(surface) ? ` ${surface} ` : surface);
  }
  for (const surface of wordsForRole(ROLE_QUANTITY_CONVERSION_CUE)) {
    if (containsCjk(surface)) {
      signals.push(surface);
    }
  }
  return signals;
}

function extractArithmeticExpression(prompt) {
  return extractArithmeticExpressionInternal(prompt, true);
}

function calculationRequestPrefixes() {
  return wordsForRole(ROLE_CALCULATION_REQUEST_CUE).map((surface) =>
    containsCjk(surface) ? surface : `${surface} `,
  );
}

function hasEmbeddedCalculationPrefixBoundary(value, start) {
  if (start === 0) return true;
  const previous = Array.from(String(value || "").slice(0, start)).pop() || "";
  return !/[\p{L}\p{N}]/u.test(previous);
}

function embeddedCalculationRequestSlices(prompt, prefixes) {
  const text = String(prompt || "");
  const lower = text.toLowerCase();
  const matches = [];
  for (const prefix of prefixes) {
    let searchStart = 0;
    while (searchStart < lower.length) {
      const start = lower.indexOf(prefix, searchStart);
      if (start === -1) break;
      if (containsCjk(prefix) || hasEmbeddedCalculationPrefixBoundary(text, start)) {
        matches.push({ start, length: prefix.length });
      }
      searchStart = start + Math.max(prefix.length, 1);
    }
  }
  matches.sort((left, right) => left.start - right.start || right.length - left.length);
  const seen = new Set();
  return matches
    .filter((match) => {
      if (seen.has(match.start)) return false;
      seen.add(match.start);
      return true;
    })
    .map((match) => text.slice(match.start));
}

function clockTimeMentions(value) {
  const text = String(value || "");
  const mentions = [];
  const pattern = /(^|[^\p{L}\p{N}])([0-9]{1,2}):([0-9]{2})(?=$|[^\p{L}\p{N}])/gu;
  for (const match of text.matchAll(pattern)) {
    const hour = Number(match[2]);
    const minute = Number(match[3]);
    if (hour > 23 || minute > 59) continue;
    const start = match.index + match[1].length;
    const end = start + match[2].length + 1 + match[3].length;
    mentions.push({
      start,
      end,
      text: text.slice(start, end),
    });
  }
  return mentions;
}

function explicitTimeSubtractionBetween(value, left, right) {
  const between = String(value || "").slice(left.end, right.start);
  return /[-−]/u.test(between) || hasArithmeticWordOperator(between);
}

function elapsedTimeExpression(prompt) {
  const text = String(prompt || "");
  if (!lexiconMentionsRole(ROLE_TIME_DURATION_CUE, normalizePrompt(text))) {
    return null;
  }
  const mentions = clockTimeMentions(text);
  if (mentions.length !== 2) return null;
  const [first, second] = mentions;
  if (explicitTimeSubtractionBetween(text, first, second)) {
    return `${first.text} - ${second.text}`;
  }
  return `${second.text} - ${first.text}`;
}

function extractArithmeticExpressionInternal(prompt, allowEmbedded) {
  const trimmed = String(prompt || "").trim();
  if (!trimmed) return null;
  const interpretations = [];
  // Issue #386: the leading calculation cues come from the calculation_request
  // meaning by role, not a literal array. Each bare surface is rebuilt into a
  // strip prefix following its script — space-delimited scripts gain a trailing
  // space so a cue strips only on a word boundary ("calculate" never eats the
  // start of "calculated"), while CJK surfaces strip as-is because those
  // scripts have no inter-word spaces. wordsForRole preserves declaration
  // order, and the Chinese cues are stored longest first, so a more specific
  // cue strips before a shorter one it contains. Mirrors
  // strip_calculation_wrappers in src/calculation.rs.
  const prefixes = calculationRequestPrefixes();
  let working = trimmed;
  let strippedLeadingCue = false;
  let changed = true;
  while (changed) {
    changed = false;
    const stripped = stripKnownPrefix(working, prefixes);
    if (stripped) {
      working = stripped.value;
      if (stripped.interpretation) interpretations.push(stripped.interpretation);
      changed = true;
      strippedLeadingCue = true;
    }
  }
  working = working.replace(/[?.!]+$/g, "").trim();
  // Issue #386: the trailing calculation cues come from the
  // calculation_result_query and politeness meanings by role, not a literal
  // array of regexes. Each surface is rebuilt into a strip suffix following its
  // script — CJK surfaces strip as-is because those scripts have no inter-word
  // spaces; a pure-symbol surface like the equals sign strips both bare and on a
  // word boundary (so a compact "2*2+2=" is recognised); every other surface
  // gains a leading space so the cue strips only on a word boundary. Mirrors
  // calculation_wrapper_suffixes in src/calculation.rs.
  const suffixes = [];
  for (const role of [ROLE_CALCULATION_RESULT_QUERY_CUE, ROLE_POLITENESS_CUE]) {
    for (const surface of wordsForRole(role)) {
      if (containsCjk(surface)) {
        suffixes.push(surface);
      } else if (!/\p{L}/u.test(surface)) {
        suffixes.push(` ${surface}`);
        suffixes.push(surface);
      } else {
        suffixes.push(` ${surface}`);
      }
    }
  }
  changed = true;
  while (changed) {
    changed = false;
    for (const suffix of suffixes) {
      // Mirror strip_suffix_case_insensitive in src/calculation.rs: a
      // case-insensitive endsWith followed by trimming the trailing whitespace.
      if (working.toLowerCase().endsWith(suffix)) {
        working = working.slice(0, working.length - suffix.length).trim();
        changed = true;
        break;
      }
    }
  }
  if (!working) return null;
  if (allowEmbedded && !strippedLeadingCue) {
    for (const slice of embeddedCalculationRequestSlices(trimmed, prefixes)) {
      const extracted = extractArithmeticExpressionInternal(slice, false);
      if (extracted) return extracted;
    }
  }
  const durationExpression = elapsedTimeExpression(working);
  if (durationExpression) {
    return {
      expression: durationExpression,
      interpretations,
      reasoningSteps: [],
      resultLabel: "",
    };
  }
  // Issue #334 step 2: rewrite a natural-language word problem into a calculator
  // expression ("the 10th Fibonacci number and multiply it by 8% of 500. Show me
  // the code ..." -> "55 * 8% of 500") before the symbolic checks below run.
  const wordProblem = normalizeWordProblemDetailed(working);
  let reasoningSteps = [];
  let resultLabel = "";
  if (wordProblem) {
    working = wordProblem.expression;
    reasoningSteps = Array.isArray(wordProblem.reasoningSteps)
      ? wordProblem.reasoningSteps
      : [];
    resultLabel = wordProblem.resultLabel || "";
  }
  const workingLower = working.toLowerCase();
  const hasLetter = /\p{L}/u.test(working);
  const hasSymbolic = /[+*/%^=×·÷−$€¥₹₽]/.test(working) || (!hasLetter && /-/.test(working));
  const hasWordOperator = hasArithmeticWordOperator(working);
  const hasSpelled = hasSpelledArithmetic(working);
  const hasPercentOf = evaluatePercentOfExpression(working) !== null;
  const hasWord =
    hasWordOperator ||
    calculatorDomainSignals().some((signal) => ` ${workingLower} `.includes(signal));
  const hasDigit = /[0-9]/.test(working);
  if (!hasDigit && !hasSpelled) return null;
  if (!hasSymbolic && !hasWord && hasLetter) return null;
  const extracted = { expression: working, interpretations, reasoningSteps, resultLabel };
  if (hasPercentOf) return extracted;
  if (evaluateCurrencyConversionExpression(working) !== null) {
    return extracted;
  }
  if (working.includes(":") && evaluateClockDifferenceExpression(working) === null) {
    return null;
  }
  const allowed = /^[0-9:+\-*/%^().=?\s_×·÷−,a-zA-Z]+$/;
  if (!allowed.test(working) && !hasWordOperator) return null;
  return extracted;
}

function extractFencedBlock(text, languages) {
  const fence = "```";
  let cursor = 0;
  while (true) {
    const open = text.indexOf(fence, cursor);
    if (open === -1) return null;
    const infoStart = open + fence.length;
    const newlineRel = text.indexOf("\n", infoStart);
    const infoEnd = newlineRel === -1 ? text.length : newlineRel;
    const info = text.slice(infoStart, infoEnd).trim().toLowerCase();
    const bodyStart = Math.min(infoEnd + 1, text.length);
    const closeRel = text.indexOf(fence, bodyStart);
    if (closeRel === -1) return null;
    const body = text.slice(bodyStart, closeRel).replace(/\n+$/, "");
    if (info === "" || languages.some((lang) => info === lang)) {
      return body;
    }
    cursor = closeRel + fence.length;
  }
}

function extractJavaScriptProgram(prompt) {
  const lower = String(prompt || "").toLowerCase();
  const asksToRun =
    lower.includes("run this javascript") ||
    lower.includes("run this js") ||
    lower.includes("execute this javascript") ||
    lower.includes("execute this js") ||
    lower.includes("run the following javascript") ||
    lower.includes("run the following js") ||
    lower.includes("evaluate this javascript") ||
    lower.includes("evaluate this js");
  if (!asksToRun) return null;
  const fenced = extractFencedBlock(prompt, ["javascript", "js"]);
  if (fenced !== null) return fenced;
  const backticks = prompt.match(/`([^`]+)`/);
  if (backticks) return backticks[1];
  const quoted = prompt.match(/"([^"]+)"/);
  return quoted ? quoted[1] : null;
}

// Look up an intent route by id (e.g. "intent_greeting"). Returns `null`
// when the routing table is empty (no `.lino` seed) so callers can decide
// whether to fall back to legacy hardcoded matching.
function findIntentRoute(id) {
  if (!INTENT_ROUTING || !Array.isArray(INTENT_ROUTING.intents)) return null;
  for (const route of INTENT_ROUTING.intents) {
    if (route && route.id === id) return route;
  }
  return null;
}

function tokensOf(normalized) {
  return normalized ? normalized.split(/\s+/).filter(Boolean) : [];
}

function tokenContains(normalized, expected) {
  return tokensOf(normalized).includes(String(expected || ""));
}

// Match a normalized prompt against an intent route using the same
// semantics as `src/engine.rs::matches_intent_route`:
//   - `keywords` / `phrases`: exact whole-prompt match
//   - `tokens`: any whitespace-separated token equals the value
//   - `combos`: every combo entry must appear as a token
function matchesIntentRoute(normalized, rawPrompt, id) {
  const route = findIntentRoute(id);
  if (!route) return false;
  const fromWasm = wasmMatchIntentRoute(normalized, rawPrompt, route);
  if (fromWasm !== null) return fromWasm;
  const raw = String(rawPrompt || "")
    .toLowerCase()
    .replace(/[?。.!!,,;:]+$/g, "")
    .trim();
  if (route.keywords && route.keywords.some((kw) => kw === normalized || kw === raw)) {
    return true;
  }
  if (route.phrases && route.phrases.some((ph) => ph === normalized || ph === raw)) {
    return true;
  }
  if (route.tokens && route.tokens.some((tok) => tokenContains(normalized, tok))) {
    return true;
  }
  if (
    route.combos &&
    route.combos.some(
      (combo) =>
        Array.isArray(combo) &&
        combo.length > 0 &&
        combo.every((tok) => tokenContains(normalized, tok)),
    )
  ) {
    return true;
  }
  return false;
}

function isIdentityPrompt(normalized, rawPrompt) {
  if (repositoryFromPrompt(rawPrompt)) return false;
  return matchesIntentRoute(normalized, rawPrompt, "intent_identity");
}

function isAssistantNamePrompt(normalized, rawPrompt) {
  return matchesIntentRoute(normalized, rawPrompt, "intent_assistant_name");
}

function isGreetingPrompt(normalized, rawPrompt) {
  return matchesIntentRoute(normalized, rawPrompt, "intent_greeting");
}

function isWellbeingPrompt(normalized, rawPrompt) {
  return matchesIntentRoute(normalized, rawPrompt, "intent_wellbeing");
}

function isAssistantFreeTimePrompt(normalized, rawPrompt) {
  return matchesIntentRoute(normalized, rawPrompt, "intent_assistant_free_time");
}

function isFarewellPrompt(normalized, rawPrompt) {
  return matchesIntentRoute(normalized, rawPrompt, "intent_farewell");
}

function isTestStatusPrompt(normalized, rawPrompt) {
  return matchesIntentRoute(normalized, rawPrompt, "intent_test_status");
}

function isCourtesyResponsePrompt(normalized, rawPrompt) {
  return matchesIntentRoute(normalized, rawPrompt, "intent_courtesy_response");
}

function isPunctuationOnlyPrompt(prompt) {
  const trimmed = String(prompt || "").trim();
  return /^[.!?…。？！]+$/.test(trimmed);
}
