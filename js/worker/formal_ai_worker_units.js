// Exact conversion between the measurement units the seed knows (issues
// #1176 and #700).
//
// Browser twin of rust/src/solver_handlers/unit_conversion.rs and
// rust/src/si_units.rs. Every factor and formula is read from
// data/seed/meanings-units.lino (`conversion` rows) and every SI dimension
// and factor from data/seed/si-unit-dimensions.lino, so adding a unit is a
// data edit. Arithmetic is exact: the shared decimal helpers live in
// formal_ai_worker_statistics.js and the SI path is reduced BigInt rationals
// bounded to i128 exactly as the native engine is.

const ROLE_UNIT_CONVERSION_QUESTION = "unit_conversion_question";
const SI_BASE_SYMBOLS = ["L", "M", "T", "I", "K", "N", "J"];

/**
 * Parse "9/5" (or a plain "2") as an exact rational, or null.
 * @param {string} text
 * @returns {{numerator: bigint, denominator: bigint}|null}
 */
function unitParseRational(text) {
  const parseInteger = (value) => (/^[+-]?\d+$/u.test(value) ? exactI128(BigInt(value)) : null);
  const slash = text.indexOf("/");
  if (slash !== -1) {
    const numerator = parseInteger(text.slice(0, slash));
    const denominator = parseInteger(text.slice(slash + 1));
    if (numerator === null || denominator === null) return null;
    return { numerator: numerator, denominator: denominator };
  }
  const whole = parseInteger(text);
  return whole === null ? null : { numerator: whole, denominator: 1n };
}

/**
 * Parse one formula step: `multiply 9/5`, `divide 9/5`, `add 32`,
 * `subtract 273.15`.
 * @param {string} word
 * @param {string} operand
 * @returns {{kind: string, numerator: bigint, denominator: bigint, shift: {mantissa: bigint, scale: number}|null}|null}
 */
function unitParseFormulaStep(word, operand) {
  switch (word) {
    case "multiply": {
      const rational = unitParseRational(operand);
      if (rational === null) return null;
      return { kind: "scale", numerator: rational.numerator, denominator: rational.denominator, shift: null };
    }
    case "divide": {
      const rational = unitParseRational(operand);
      if (rational === null) return null;
      return { kind: "scale", numerator: rational.denominator, denominator: rational.numerator, shift: null };
    }
    case "add": {
      const constant = exactDecimalParse(operand);
      if (constant === null) return null;
      return { kind: "shift", numerator: 0n, denominator: 1n, shift: constant };
    }
    case "subtract": {
      const constant = exactDecimalParse(operand);
      if (constant === null) return null;
      return { kind: "shift", numerator: 0n, denominator: 1n, shift: exactDecimalNegate(constant) };
    }
    default:
      return null;
  }
}

/**
 * Every `conversion` row of data/seed/meanings-units.lino as
 * `{source, target, linear | steps}`, in document order.
 * @returns {Array<{source: string, target: string, linear: {mantissa: bigint, scale: number}|null, steps: Array<object>|null}>}
 */
function unitConversionTable() {
  const table = [];
  const root = parseLinoTree(seedRawText(SEED_RAW, "meanings-units.lino"));
  for (const record of root.children) {
    for (const unit of record.children) {
      for (const child of unit.children) {
        if (child.name !== "conversion") continue;
        const tokens = String(child.value || "").split(/\s+/u).filter(Boolean);
        if (tokens.length < 2) continue;
        const target = tokens[0];
        const rest = tokens.slice(1);
        if (rest[0] === "formula") {
          const operands = rest.slice(1);
          const steps = [];
          let valid = true;
          for (let index = 0; index + 1 < operands.length; index += 2) {
            const step = unitParseFormulaStep(operands[index], operands[index + 1]);
            if (step === null) {
              valid = false;
              break;
            }
            steps.push(step);
          }
          if (!valid || steps.length === 0) continue;
          table.push({ source: unit.name, target: target, linear: null, steps: steps });
        } else {
          const factor = exactDecimalParse(rest[0]);
          if (factor === null) continue;
          table.push({ source: unit.name, target: target, linear: factor, steps: null });
        }
      }
    }
  }
  return table;
}

/**
 * Greatest common divisor (non-negative).
 * @param {bigint} left
 * @param {bigint} right
 * @returns {bigint}
 */
function siGcd(left, right) {
  let a = left;
  let b = right;
  while (b !== 0n) {
    const remainder = a % b;
    a = b;
    b = remainder;
  }
  return a < 0n ? -a : a;
}

/**
 * Reduce a rational to lowest terms.
 * @param {bigint} numerator
 * @param {bigint} denominator
 * @returns {{numerator: bigint, denominator: bigint}}
 */
function siReduce(numerator, denominator) {
  const common = siGcd(numerator, denominator);
  if (common <= 1n) return { numerator: numerator, denominator: denominator };
  return { numerator: numerator / common, denominator: denominator / common };
}

/**
 * Parse a factor written as a terminating decimal or a `num/den` fraction
 * into an exact reduced positive rational. Mirrors `parse_factor`.
 * @param {string} text
 * @returns {{numerator: bigint, denominator: bigint}|null}
 */
function siParseFactor(text) {
  const trimmed = String(text || "").trim();
  const slash = trimmed.indexOf("/");
  if (slash !== -1) {
    const left = trimmed.slice(0, slash).trim();
    const right = trimmed.slice(slash + 1).trim();
    if (!/^[+-]?\d+$/u.test(left) || !/^[+-]?\d+$/u.test(right)) return null;
    const numerator = exactI128(BigInt(left));
    const denominator = exactI128(BigInt(right));
    if (numerator === null || denominator === null || numerator <= 0n || denominator <= 0n) return null;
    return siReduce(numerator, denominator);
  }
  const point = trimmed.indexOf(".");
  const whole = point === -1 ? trimmed : trimmed.slice(0, point);
  const fractional = point === -1 ? "" : trimmed.slice(point + 1);
  if (whole === "" && fractional === "") return null;
  if (!/^\d*$/u.test(whole) || !/^\d*$/u.test(fractional)) return null;
  if (whole === "") return null;
  let numerator = exactI128(BigInt(whole));
  let denominator = 1n;
  if (numerator === null) return null;
  for (const digit of fractional) {
    numerator = exactI128(numerator * 10n + BigInt(digit.charCodeAt(0) - 48));
    denominator = exactI128(denominator * 10n);
    if (numerator === null || denominator === null) return null;
  }
  if (numerator <= 0n) return null;
  return siReduce(numerator, denominator);
}

/**
 * Parse an exponent string (`L`, `LT-1`, `L2MT-3`, `1`) into seven
 * exponents, or null.
 * @param {string} expression
 * @returns {Array<number>|null}
 */
function siParseDimension(expression) {
  const trimmed = String(expression || "").trim();
  const exponents = [0, 0, 0, 0, 0, 0, 0];
  if (trimmed === "1") return exponents;
  const characters = Array.from(trimmed);
  let index = 0;
  while (index < characters.length) {
    const slot = SI_BASE_SYMBOLS.indexOf(characters[index]);
    if (slot === -1) return null;
    index += 1;
    let sign = 1;
    if (index < characters.length && characters[index] === "-") {
      sign = -1;
      index += 1;
    }
    let magnitude = 0;
    let digits = false;
    while (index < characters.length && /^[0-9]$/u.test(characters[index])) {
      magnitude = magnitude * 10 + Number(characters[index]);
      if (magnitude > 127) return null;
      digits = true;
      index += 1;
    }
    const exponent = digits ? sign * magnitude : sign;
    const next = exponents[slot] + exponent;
    if (next > 127 || next < -128) return null;
    exponents[slot] = next;
  }
  return exponents;
}

/**
 * Render seven exponents back to the seed notation.
 * @param {Array<number>} exponents
 * @returns {string}
 */
function siDimensionString(exponents) {
  if (exponents.every((exponent) => exponent === 0)) return "1";
  let out = "";
  for (let slot = 0; slot < SI_BASE_SYMBOLS.length; slot += 1) {
    const exponent = exponents[slot];
    if (exponent === 0) continue;
    out += exponent === 1 ? SI_BASE_SYMBOLS[slot] : `${SI_BASE_SYMBOLS[slot]}${exponent}`;
  }
  return out;
}

/**
 * The first child value named `name`, or "".
 * @param {{children: Array<{name: string, value: string}>}} node
 * @param {string} name
 * @returns {string}
 */
function siChildValue(node, name) {
  const child = node.children.find((candidate) => candidate.name === name);
  return child ? String(child.value || "") : "";
}

/**
 * Every `si_unit` record of data/seed/si-unit-dimensions.lino. Mirrors
 * `si_units()` in rust/src/si_units.rs.
 * @returns {Array<{unit: string, dimension: Array<number>, numerator: bigint, denominator: bigint, surfaces: Array<string>}>}
 */
function siUnits() {
  const out = [];
  const root = parseLinoTree(seedRawText(SEED_RAW, "si-unit-dimensions.lino"));
  const section = root.children.find((child) => child.name === "si_units");
  if (!section) return out;
  for (const record of section.children) {
    if (record.name !== "si_unit") continue;
    const unit = siChildValue(record, "unit").trim();
    if (unit === "") continue;
    const dimension = siParseDimension(siChildValue(record, "dimension"));
    if (dimension === null) continue;
    const factor = siParseFactor(siChildValue(record, "si_factor"));
    if (factor === null) continue;
    const surfaces = record.children
      .filter((child) => child.name === "surface")
      .map((child) => String(child.value || "").trim().toLowerCase())
      .filter((surface) => surface !== "");
    out.push({
      unit: unit,
      dimension: dimension,
      numerator: factor.numerator,
      denominator: factor.denominator,
      surfaces: surfaces,
    });
  }
  return out;
}

/**
 * The canonical unit for a surface word or canonical name, or null.
 * @param {string} surface
 * @returns {string|null}
 */
function siUnitNamedBy(surface) {
  const needle = String(surface || "").trim().toLowerCase();
  const entry = siUnits().find((candidate) =>
    candidate.unit === needle || candidate.surfaces.includes(needle));
  return entry ? entry.unit : null;
}

/**
 * Convert `numerator/denominator` between two units of one dimension
 * through SI. Mirrors `convert_through_si`: the tag is `converted`,
 * `incompatible`, `unknown_unit` or `overflow`.
 * @param {bigint} valueNumerator
 * @param {bigint} valueDenominator
 * @param {string} from
 * @param {string} to
 * @returns {{tag: string, numerator: bigint, denominator: bigint, unit: string, from: string, to: string}}
 */
function siConvertThroughSi(valueNumerator, valueDenominator, from, to) {
  const units = siUnits();
  const resolve = (name) => {
    const needle = String(name).trim().toLowerCase();
    return units.find((entry) => entry.unit === needle || entry.surfaces.includes(needle)) || null;
  };
  const outcome = { tag: "", numerator: 0n, denominator: 1n, unit: "", from: "", to: "" };
  const fromEntry = resolve(from);
  if (fromEntry === null) return Object.assign(outcome, { tag: "unknown_unit", unit: String(from).trim() });
  const toEntry = resolve(to);
  if (toEntry === null) return Object.assign(outcome, { tag: "unknown_unit", unit: String(to).trim() });
  const fromDimension = siDimensionString(fromEntry.dimension);
  const toDimension = siDimensionString(toEntry.dimension);
  if (fromDimension !== toDimension) {
    return Object.assign(outcome, { tag: "incompatible", from: fromDimension, to: toDimension });
  }
  const left = exactI128(valueNumerator * fromEntry.numerator);
  const numerator = left === null ? null : exactI128(left * toEntry.denominator);
  const right = exactI128(valueDenominator * fromEntry.denominator);
  const denominator = right === null ? null : exactI128(right * toEntry.numerator);
  if (numerator === null || denominator === null || denominator <= 0n) {
    return Object.assign(outcome, { tag: "overflow" });
  }
  const reduced = siReduce(numerator, denominator);
  return Object.assign(outcome, {
    tag: "converted",
    numerator: reduced.numerator,
    denominator: reduced.denominator,
  });
}

/**
 * The earliest present surface of a word list, as `{surface, position}`
 * (the first word wins a tie, as Rust's `min_by_key`), or null.
 * @param {string} lowered
 * @param {Array<string>} words
 * @returns {{surface: string, position: number}|null}
 */
function unitEarliestSurface(lowered, words) {
  let best = null;
  for (const word of words) {
    const position = quantityTermPosition(lowered, word);
    if (position === null) continue;
    if (best === null || position < best.position) best = { surface: word, position: position };
  }
  return best;
}

/**
 * Every measurement unit the lowercased prompt names, lexicon units first
 * then the SI catalogue. Mirrors `unit_mentions`.
 * @param {string} lowered
 * @returns {Array<{slug: string, surface: string, position: number}>}
 */
function unitMentions(lowered) {
  const mentions = [];
  for (const meaning of meaningsWithRole(ROLE_MEASUREMENT_UNIT)) {
    const found = unitEarliestSurface(lowered, meaning.words);
    if (found === null) continue;
    if (!mentions.some((mention) => mention.slug === meaning.slug)) {
      mentions.push({ slug: meaning.slug, surface: found.surface, position: found.position });
    }
  }
  for (const unit of siUnits()) {
    const words = unit.surfaces.concat([unit.unit]).filter((word) => word !== "in");
    const found = unitEarliestSurface(lowered, words);
    if (found === null) continue;
    if (!mentions.some((mention) => mention.slug === unit.unit)) {
      mentions.push({ slug: unit.unit, surface: found.surface, position: found.position });
    }
  }
  return mentions;
}

/**
 * The seed conversion from `source` to `target`: forward (linear or
 * formula) or an inverted linear factor; null when the seed has none.
 * @param {Array<object>} table
 * @param {string} source
 * @param {string} target
 * @returns {{kind: string, factor: {mantissa: bigint, scale: number}|null, steps: Array<object>|null}|null}
 */
function unitFindConversion(table, source, target) {
  const forward = table.find((row) => row.source === source && row.target === target);
  if (forward) {
    return forward.linear !== null
      ? { kind: "linear", factor: forward.linear, steps: null }
      : { kind: "formula", factor: null, steps: forward.steps };
  }
  const reverse = table.find((row) => row.source === target && row.target === source);
  if (!reverse || reverse.linear === null) return null;
  return { kind: "reverse_linear", factor: reverse.linear, steps: null };
}

/**
 * Apply the formula steps, tracking exactness and echoing each step.
 * @param {{mantissa: bigint, scale: number}} start
 * @param {Array<object>} steps
 * @returns {{value: {mantissa: bigint, scale: number}, exact: boolean, derivation: string}|null}
 */
function unitApplyFormula(start, steps) {
  let value = start;
  let exact = true;
  let derivation = exactDecimalRender(start);
  for (const step of steps) {
    switch (step.kind) {
      case "scale": {
        const scaled = exactDecimalMulDiv(value, step.numerator, step.denominator, 7);
        if (scaled === null) return null;
        if (derivation.includes(" ")) derivation = `(${derivation})`;
        derivation = `${derivation} × ${step.numerator}/${step.denominator}`;
        value = scaled.value;
        exact = exact && scaled.exact;
        break;
      }
      case "shift": {
        const shifted = exactDecimalAdd(value, step.shift);
        if (shifted === null) return null;
        value = shifted;
        derivation = step.shift.mantissa < 0n
          ? `${derivation} - ${exactDecimalRender(exactDecimalNegate(step.shift))}`
          : `${derivation} + ${exactDecimalRender(step.shift)}`;
        break;
      }
      default:
        throw new Error("unknown formula step");
    }
  }
  const joiner = exact ? "=" : "≈";
  return { value: value, exact: exact, derivation: `${derivation} ${joiner} ${exactDecimalRender(value)}` };
}

/**
 * Unit-conversion entry point (issue #1176): "how many X are N Y",
 * "N Y in X", "convert N Y to X", exactly and with the arithmetic shown.
 * Mirrors `handle_unit_conversion`.
 * @param {string} prompt
 * @param {string} normalized
 * @param {string} language
 * @returns {{intent: string, content: string, confidence: number, evidence: Array<string>}|null}
 */
function tryUnitConversion(prompt, normalized, language) {
  const asksConversion = wordsForRole(ROLE_UNIT_CONVERSION_QUESTION)
    .some((word) => quantityContainsTerm(normalized, word));
  const lowered = String(prompt || "").toLowerCase();
  if (!asksConversion && !lowered.includes("?") && !lowered.includes("？")) return null;
  const items = parseNumericListNumbers(lowered);
  if (items.length === 0) return null;
  const mentions = unitMentions(lowered);
  if (mentions.length !== 2) return null;
  const value = exactDecimalParse(items[0].text);
  if (value === null) return null;
  const numberIndex = lowered.indexOf(items[0].text);
  if (numberIndex === -1) return null;
  const numberPosition = quantityByteOffset(lowered, numberIndex);
  let source = mentions[0];
  for (const mention of mentions) {
    if (Math.abs(mention.position - numberPosition) < Math.abs(source.position - numberPosition)) {
      source = mention;
    }
  }
  const target = mentions.find((mention) => mention.slug !== source.slug);
  if (!target) return null;
  const direction = unitFindConversion(unitConversionTable(), source.slug, target.slug);

  if (direction === null) {
    const log = [];
    const outcome = siConvertThroughSi(value.mantissa, 10n ** BigInt(value.scale), source.slug, target.slug);
    let body;
    switch (outcome.tag) {
      case "converted":
        log.push(`unit_conversion:si_path:${source.slug} -> ${target.slug}`);
        body = `${exactDecimalRender(value)} ${source.surface} = ${outcome.numerator}/${outcome.denominator} ${target.surface}`;
        break;
      case "unknown_unit":
        log.push(`si_units:unknown_unit:${outcome.unit}`);
        body = `unknown unit: ${outcome.unit}`;
        break;
      case "incompatible":
        body = `incompatible dimensions: ${outcome.from} vs ${outcome.to}`;
        break;
      default:
        body = "value out of range for exact conversion";
        break;
    }
    return {
      intent: "unit_conversion",
      content: body,
      confidence: 1.0,
      evidence: ["handler:unit_conversion"].concat(log, ["response:unit_conversion_si", `language:${language}`]),
    };
  }

  const log = [
    `unit_conversion:source_unit:${source.slug}`,
    `unit_conversion:target_unit:${target.slug}`,
    `unit_conversion:value:${exactDecimalRender(value)}`,
  ];
  let result;
  let exact;
  let derivation;
  let intent;
  let factorText;
  switch (direction.kind) {
    case "linear": {
      result = exactDecimalMul(value, direction.factor);
      if (result === null) return null;
      exact = true;
      derivation = `${exactDecimalRender(value)} × ${exactDecimalRender(direction.factor)} = ${exactDecimalRender(result)}`;
      intent = "unit_conversion_multiply";
      factorText = exactDecimalRender(direction.factor);
      break;
    }
    case "reverse_linear": {
      const quotient = exactDecimalDiv(value, direction.factor, 7);
      if (quotient === null) return null;
      result = quotient.value;
      exact = quotient.exact;
      derivation = `${exactDecimalRender(value)} ÷ ${exactDecimalRender(direction.factor)} ${exact ? "=" : "≈"} ${exactDecimalRender(result)}`;
      intent = "unit_conversion_divide";
      factorText = exactDecimalRender(direction.factor);
      break;
    }
    case "formula": {
      const applied = unitApplyFormula(value, direction.steps);
      if (applied === null) return null;
      result = applied.value;
      exact = applied.exact;
      derivation = applied.derivation;
      intent = "unit_conversion_formula";
      factorText = "";
      break;
    }
    default:
      throw new Error("unknown conversion direction");
  }
  const resultText = exactDecimalRenderApprox(result, exact);
  log.push(`unit_conversion:result:${resultText}`);
  log.push(`unit_conversion:derivation:${derivation}`);
  const template = quantityLocalizedTemplate(intent, language);
  const body = template === null
    ? derivation
    : quantityFillTemplate(template, [
      ["value", exactDecimalRender(value)],
      ["source_unit", source.surface],
      ["target_unit", target.surface],
      ["result", resultText],
      ["factor", factorText],
      ["derivation", derivation],
      ["equals", exact ? "=" : "≈"],
    ]);
  return {
    intent: "unit_conversion",
    content: body,
    confidence: 1.0,
    evidence: ["handler:unit_conversion"].concat(log, ["response:unit_conversion", `language:${language}`]),
  };
}
