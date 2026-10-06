// Statistics over stated number lists, and the price-times-count word
// problems that share their vocabulary (issue #1176).
//
// Browser twin of rust/src/solver_handlers/statistics.rs. Every operation
// runs on exact decimals (a BigInt mantissa with a decimal scale, bounded like
// the native i128 arithmetic); only the standard deviation takes a
// double-precision root, exactly as the native handler does. Recognition
// vocabulary is read from data/seed/meanings-statistics.lino through the
// meaning lexicon and the prose from
// data/seed/multilingual-responses-quantities.lino, so this file owns only
// the arithmetic. The exact-decimal helpers are shared with
// formal_ai_worker_units.js.

const EXACT_I128_MAX = (1n << 127n) - 1n;
const EXACT_I128_MIN = -(1n << 127n);
/** Largest accepted mantissa magnitude (10^12), as in statistics.rs. */
const EXACT_MAX_MANTISSA = 1000000000000n;
/** Largest accepted number of fractional digits in a stated value. */
const EXACT_MAX_SCALE = 9;
/** Largest accepted list length. */
const STATISTICS_MAX_VALUES = 100;

const WORD_PROBLEM_MARKER_UNIT_PRICE = "word_problem_unit_price";
const WORD_PROBLEM_MARKER_PAYMENT = "word_problem_payment";
const WORD_PROBLEM_MARKER_CHANGE = "word_problem_change";
const WORD_PROBLEM_MARKER_TOTAL = "word_problem_total";

/**
 * Canonical statistics operations in answer order, each with its seed
 * meaning slug and evidence kind.
 * @type {Array<{op: string, slug: string, evidence: string}>}
 */
const STATISTICS_OPERATIONS = [
  { op: "mean", slug: "statistics_mean", evidence: "statistics:mean" },
  { op: "median", slug: "statistics_median", evidence: "statistics:median" },
  { op: "mode", slug: "statistics_mode", evidence: "statistics:mode" },
  { op: "variance", slug: "statistics_variance", evidence: "statistics:variance" },
  {
    op: "standard_deviation",
    slug: "statistics_standard_deviation",
    evidence: "statistics:standard_deviation",
  },
  { op: "percentile", slug: "statistics_percentile", evidence: "statistics:percentile" },
  { op: "range", slug: "statistics_range", evidence: "statistics:range" },
];

/** Most characters an ordinal suffix glued to a percentile rank may carry. */
const STATISTICS_MAX_RANK_SUFFIX_CHARS = 4;
/** Most whitespace characters between a percentile surface and a later rank. */
const STATISTICS_MAX_RANK_LEAD_CHARS = 3;

/**
 * The bigint when it fits i128, otherwise null (the native checked ops).
 * @param {bigint} value
 * @returns {bigint|null}
 */
function exactI128(value) {
  if (value > EXACT_I128_MAX || value < EXACT_I128_MIN) return null;
  return value;
}

/**
 * 10^exponent as an i128, or null on overflow.
 * @param {number} exponent
 * @returns {bigint|null}
 */
function exactPow10(exponent) {
  return exactI128(10n ** BigInt(exponent));
}

/**
 * Build a decimal value `mantissa × 10^-scale`.
 * @param {bigint} mantissa
 * @param {number} scale
 * @returns {{mantissa: bigint, scale: number}}
 */
function exactDecimal(mantissa, scale) {
  return { mantissa: mantissa, scale: scale };
}

/**
 * Parse a plain decimal literal ("4", "26.2", "-3.5") within the bounds.
 * @param {string} text
 * @returns {{mantissa: bigint, scale: number}|null}
 */
function exactDecimalParse(text) {
  const source = String(text);
  let negative = false;
  let digits = source;
  if (source.startsWith("-")) {
    negative = true;
    digits = source.slice(1);
  } else if (source.startsWith("+")) {
    digits = source.slice(1);
  }
  let mantissa = 0n;
  let scale = 0;
  let seenDigit = false;
  let seenPoint = false;
  for (const character of digits) {
    if (character >= "0" && character <= "9") {
      seenDigit = true;
      if (seenPoint) {
        scale += 1;
        if (scale > EXACT_MAX_SCALE) return null;
      }
      const next = exactI128(mantissa * 10n + BigInt(character.charCodeAt(0) - 48));
      if (next === null) return null;
      mantissa = next;
    } else if (character === "." && !seenPoint) {
      seenPoint = true;
    } else {
      return null;
    }
  }
  if (!seenDigit || mantissa > EXACT_MAX_MANTISSA) return null;
  return exactDecimal(negative ? -mantissa : mantissa, scale);
}

/**
 * Re-express at a higher scale; lowering is refused.
 * @param {{mantissa: bigint, scale: number}} value
 * @param {number} scale
 * @returns {{mantissa: bigint, scale: number}|null}
 */
function exactDecimalRescaled(value, scale) {
  if (scale < value.scale) return null;
  const factor = exactPow10(scale - value.scale);
  if (factor === null) return null;
  const mantissa = exactI128(value.mantissa * factor);
  return mantissa === null ? null : exactDecimal(mantissa, scale);
}

/**
 * @param {{mantissa: bigint, scale: number}} left
 * @param {{mantissa: bigint, scale: number}} right
 * @returns {{mantissa: bigint, scale: number}|null}
 */
function exactDecimalAdd(left, right) {
  const scale = Math.max(left.scale, right.scale);
  const a = exactDecimalRescaled(left, scale);
  const b = exactDecimalRescaled(right, scale);
  if (a === null || b === null) return null;
  const mantissa = exactI128(a.mantissa + b.mantissa);
  return mantissa === null ? null : exactDecimal(mantissa, scale);
}

/**
 * @param {{mantissa: bigint, scale: number}} left
 * @param {{mantissa: bigint, scale: number}} right
 * @returns {{mantissa: bigint, scale: number}|null}
 */
function exactDecimalSub(left, right) {
  const scale = Math.max(left.scale, right.scale);
  const a = exactDecimalRescaled(left, scale);
  const b = exactDecimalRescaled(right, scale);
  if (a === null || b === null) return null;
  const mantissa = exactI128(a.mantissa - b.mantissa);
  return mantissa === null ? null : exactDecimal(mantissa, scale);
}

/**
 * Exact product of two decimals.
 * @param {{mantissa: bigint, scale: number}} left
 * @param {{mantissa: bigint, scale: number}} right
 * @returns {{mantissa: bigint, scale: number}|null}
 */
function exactDecimalMul(left, right) {
  const mantissa = exactI128(left.mantissa * right.mantissa);
  return mantissa === null ? null : exactDecimal(mantissa, left.scale + right.scale);
}

/**
 * `numerator / denominator` at `scale` fractional digits, rounded half away
 * from zero, with a flag saying whether the quotient terminated.
 * @param {bigint} numerator
 * @param {bigint} denominator
 * @param {number} scale
 * @returns {{value: {mantissa: bigint, scale: number}, exact: boolean}|null}
 */
function exactDecimalFromQuotient(numerator, denominator, scale) {
  if (denominator === 0n) return null;
  const power = exactPow10(scale);
  if (power === null) return null;
  const scaled = exactI128(numerator * power);
  if (scaled === null) return null;
  const sign = (scaled < 0n) === (denominator < 0n) ? 1n : -1n;
  const magnitude = scaled < 0n ? -scaled : scaled;
  const divisor = denominator < 0n ? -denominator : denominator;
  let mantissa = magnitude / divisor;
  if (exactI128(mantissa) === null) return null;
  const remainder = magnitude % divisor;
  const exact = remainder === 0n;
  if (!exact && remainder * 2n >= divisor) mantissa += 1n;
  return { value: exactDecimal(sign * mantissa, scale), exact: exact };
}

/**
 * Divide, carrying the quotient to `precision` fractional digits.
 * @param {{mantissa: bigint, scale: number}} left
 * @param {{mantissa: bigint, scale: number}} right
 * @param {number} precision
 * @returns {{value: {mantissa: bigint, scale: number}, exact: boolean}|null}
 */
function exactDecimalDiv(left, right, precision) {
  if (right.mantissa === 0n) return null;
  const rightPower = exactPow10(right.scale);
  const leftPower = exactPow10(left.scale);
  if (rightPower === null || leftPower === null) return null;
  const numerator = exactI128(left.mantissa * rightPower);
  const denominator = exactI128(right.mantissa * leftPower);
  if (numerator === null || denominator === null) return null;
  return exactDecimalFromQuotient(numerator, denominator, precision);
}

/**
 * Multiply by the exact rational `numerator / denominator`.
 * @param {{mantissa: bigint, scale: number}} value
 * @param {bigint} numerator
 * @param {bigint} denominator
 * @param {number} precision
 * @returns {{value: {mantissa: bigint, scale: number}, exact: boolean}|null}
 */
function exactDecimalMulDiv(value, numerator, denominator, precision) {
  if (denominator === 0n) return null;
  const scaledNumerator = exactI128(value.mantissa * numerator);
  const power = exactPow10(value.scale);
  if (scaledNumerator === null || power === null) return null;
  const scaledDenominator = exactI128(power * denominator);
  if (scaledDenominator === null) return null;
  return exactDecimalFromQuotient(scaledNumerator, scaledDenominator, value.scale + precision);
}

/**
 * Order two decimals by value: -1, 0 or 1.
 * @param {{mantissa: bigint, scale: number}} left
 * @param {{mantissa: bigint, scale: number}} right
 * @returns {number}
 */
function exactDecimalCompare(left, right) {
  const scale = Math.max(left.scale, right.scale);
  const a = exactDecimalRescaled(left, scale);
  const b = exactDecimalRescaled(right, scale);
  if (a === null || b === null) throw new Error("bounded decimals align");
  if (a.mantissa < b.mantissa) return -1;
  if (a.mantissa > b.mantissa) return 1;
  return 0;
}

/**
 * @param {{mantissa: bigint, scale: number}} value
 * @returns {{mantissa: bigint, scale: number}}
 */
function exactDecimalNegate(value) {
  return exactDecimal(-value.mantissa, value.scale);
}

/**
 * Minimal decimal form: no trailing fraction zeros, no `-0`.
 * @param {{mantissa: bigint, scale: number}} value
 * @returns {string}
 */
function exactDecimalRender(value) {
  const negative = value.mantissa < 0n;
  const digits = (negative ? -value.mantissa : value.mantissa).toString();
  const scale = value.scale;
  let text;
  if (scale === 0) {
    text = digits;
  } else if (digits.length <= scale) {
    text = `0.${"0".repeat(scale - digits.length)}${digits}`;
  } else {
    const split = digits.length - scale;
    text = `${digits.slice(0, split)}.${digits.slice(split)}`;
  }
  if (text.includes(".")) {
    text = text.replace(/0+$/u, "");
    if (text.endsWith(".")) text = text.slice(0, -1);
  }
  return negative && text !== "0" ? `-${text}` : text;
}

/**
 * Render a decimal, marking an inexact quotient with a leading `≈`.
 * @param {{mantissa: bigint, scale: number}} value
 * @param {boolean} exact
 * @returns {string}
 */
function exactDecimalRenderApprox(value, exact) {
  return exact ? exactDecimalRender(value) : `≈${exactDecimalRender(value)}`;
}

/**
 * Rust `char::is_alphanumeric` or `_`: the word-boundary test the native
 * `contains_term` uses (marks such as Devanagari matras are alphabetic).
 * @param {string} character
 * @returns {boolean}
 */
function quantityIsWordCharacter(character) {
  return /[\p{Alphabetic}\p{N}_]/u.test(character);
}

/**
 * The code point that ends `text`, or "".
 * @param {string} text
 * @returns {string}
 */
function quantityLastCharacter(text) {
  const characters = Array.from(text);
  return characters.length > 0 ? characters[characters.length - 1] : "";
}

/**
 * The code point that starts `text`, or "".
 * @param {string} text
 * @returns {string}
 */
function quantityFirstCharacter(text) {
  const point = text.codePointAt(0);
  return point === undefined ? "" : String.fromCodePoint(point);
}

/**
 * Whole-term presence with word boundaries; a CJK surface matches as a
 * substring. Mirrors `contains_term` in solver_handlers/calendar.rs.
 * @param {string} haystack
 * @param {string} needle
 * @returns {boolean}
 */
function quantityContainsTerm(haystack, needle) {
  if (!needle) return false;
  if (/[一-鿿]/u.test(needle)) return haystack.includes(needle);
  let index = haystack.indexOf(needle);
  while (index !== -1) {
    const before = quantityLastCharacter(haystack.slice(0, index));
    const after = quantityFirstCharacter(haystack.slice(index + needle.length));
    if ((before === "" || !quantityIsWordCharacter(before)) &&
      (after === "" || !quantityIsWordCharacter(after))) {
      return true;
    }
    index = haystack.indexOf(needle, index + needle.length);
  }
  return false;
}

/**
 * UTF-8 byte offset of a UTF-16 index, so positions compare exactly as the
 * native byte offsets do.
 * @param {string} text
 * @param {number} index
 * @returns {number}
 */
function quantityByteOffset(text, index) {
  return new TextEncoder().encode(text.slice(0, index)).length;
}

/**
 * Byte offset of the first raw occurrence of a present term, or null.
 * Mirrors `term_position` in solver_handlers/calendar.rs.
 * @param {string} haystack
 * @param {string} needle
 * @returns {number|null}
 */
function quantityTermPosition(haystack, needle) {
  if (!quantityContainsTerm(haystack, needle)) return null;
  return quantityByteOffset(haystack, haystack.indexOf(needle));
}

/**
 * Surface words of the seed meaning `slug`, in declaration order.
 * @param {string} slug
 * @returns {Array<string>}
 */
function quantityMeaningWords(slug) {
  const meaning = findMeaning(slug);
  return meaning ? meaning.words : [];
}

/**
 * The seed template for `intent` in `language` (falling back to English),
 * or null when the seed has none.
 * @param {string} intent
 * @param {string} language
 * @returns {string|null}
 */
function quantityLocalizedTemplate(intent, language) {
  const table = MULTILINGUAL_ANSWERS[intent];
  if (!table) return null;
  const entry = table[language] || table.en;
  if (!entry) return null;
  return normalizeEntry(entry, intent).text;
}

/**
 * Replace each `{name}` placeholder in order, like chained `str::replace`.
 * @param {string} template
 * @param {Array<Array<string>>} bindings
 * @returns {string}
 */
function quantityFillTemplate(template, bindings) {
  let rendered = template;
  for (const binding of bindings) {
    rendered = rendered.split(`{${binding[0]}}`).join(binding[1]);
  }
  return rendered;
}

/**
 * The answer object every quantity handler returns.
 * @param {string} intent
 * @param {string} body
 * @param {Array<string>} log
 * @param {string} language
 * @returns {{intent: string, content: string, confidence: number, evidence: Array<string>}}
 */
function quantityAnswer(intent, body, log, language) {
  return {
    intent: intent,
    content: body,
    confidence: 1.0,
    evidence: [`handler:${intent}`].concat(log, [`response:${intent}`, `language:${language}`]),
  };
}

/**
 * Does the normalized prompt mention any surface of the marker meaning?
 * @param {string} normalized
 * @param {string} slug
 * @returns {boolean}
 */
function wordProblemMentionsMarker(normalized, slug) {
  return quantityMeaningWords(slug).some((word) => quantityContainsTerm(normalized, word));
}

/**
 * The numbers the prompt states, as exact decimals, with each surface's byte
 * offset in the lowercased prompt; null when one exceeds the bounds.
 * @param {string} lowered
 * @returns {{values: Array<{mantissa: bigint, scale: number}>, positions: Array<number>}|null}
 */
function statisticsStatedNumbers(lowered) {
  const values = [];
  const positions = [];
  for (const item of parseNumericListNumbers(lowered)) {
    const value = exactDecimalParse(item.text);
    if (value === null) return null;
    const index = lowered.indexOf(item.text);
    if (index === -1) return null;
    values.push(value);
    positions.push(quantityByteOffset(lowered, index));
    if (values.length > STATISTICS_MAX_VALUES) return null;
  }
  return { values: values, positions: positions };
}

/**
 * Index of the largest position still under `limit` (the last one on a tie,
 * as Rust's `max_by_key`), or null.
 * @param {Array<number>} positions
 * @param {number} limit
 * @returns {number|null}
 */
function wordProblemNearestBefore(positions, limit) {
  let best = null;
  for (let index = 0; index < positions.length; index += 1) {
    if (positions[index] >= limit) continue;
    if (best === null || positions[index] >= positions[best]) best = index;
  }
  return best;
}

/**
 * Sum of a non-empty list of decimals, or null on overflow.
 * @param {Array<{mantissa: bigint, scale: number}>} values
 * @returns {{mantissa: bigint, scale: number}|null}
 */
function statisticsSum(values) {
  let sum = values[0];
  for (let index = 1; index < values.length; index += 1) {
    sum = exactDecimalAdd(sum, values[index]);
    if (sum === null) return null;
  }
  return sum;
}

/**
 * A copy sorted by value.
 * @param {Array<{mantissa: bigint, scale: number}>} values
 * @returns {Array<{mantissa: bigint, scale: number}>}
 */
function statisticsSorted(values) {
  return values.slice().sort(exactDecimalCompare);
}

/**
 * @param {Array<{mantissa: bigint, scale: number}>} values
 * @returns {{value: string, derivation: string|null}|null}
 */
function statisticsMean(values) {
  const sum = statisticsSum(values);
  if (sum === null) return null;
  const quotient = exactDecimalDiv(sum, exactDecimal(BigInt(values.length), 0), 7);
  if (quotient === null) return null;
  const joiner = quotient.exact ? "=" : "≈";
  return {
    value: exactDecimalRenderApprox(quotient.value, quotient.exact),
    derivation: `${exactDecimalRender(sum)} / ${values.length} ${joiner} ${exactDecimalRender(quotient.value)}`,
  };
}

/**
 * @param {Array<{mantissa: bigint, scale: number}>} values
 * @returns {{value: string, derivation: string|null}|null}
 */
function statisticsMedian(values) {
  const sorted = statisticsSorted(values);
  const middle = Math.floor(sorted.length / 2);
  if (sorted.length % 2 === 1) {
    return { value: exactDecimalRender(sorted[middle]), derivation: null };
  }
  const total = exactDecimalAdd(sorted[middle - 1], sorted[middle]);
  if (total === null) return null;
  const quotient = exactDecimalDiv(total, exactDecimal(2n, 0), 7);
  if (quotient === null) return null;
  const joiner = quotient.exact ? "=" : "≈";
  return {
    value: exactDecimalRenderApprox(quotient.value, quotient.exact),
    derivation: `(${exactDecimalRender(sorted[middle - 1])} + ${exactDecimalRender(sorted[middle])}) / 2 ${joiner} ${exactDecimalRender(quotient.value)}`,
  };
}

/**
 * @param {Array<{mantissa: bigint, scale: number}>} values
 * @returns {{value: string, derivation: string|null}|null}
 */
function statisticsMode(values) {
  const counts = [];
  for (const value of values) {
    const entry = counts.find((candidate) => exactDecimalCompare(candidate.value, value) === 0);
    if (entry) entry.count += 1;
    else counts.push({ value: value, count: 1 });
  }
  if (counts.length === 0) return null;
  const best = counts.reduce((max, entry) => Math.max(max, entry.count), 0);
  const modes = counts
    .filter((entry) => entry.count === best)
    .map((entry) => exactDecimalRender(entry.value));
  return { value: modes.join(", "), derivation: null };
}

/**
 * The exact population variance `(n·Σx² − (Σx)²) / n²` at the common scale.
 * @param {Array<{mantissa: bigint, scale: number}>} values
 * @returns {{numerator: bigint, denominator: bigint, scale: number}|null}
 */
function statisticsVarianceFraction(values) {
  if (values.length === 0) return null;
  const scale = values.reduce((max, value) => Math.max(max, value.scale), 0);
  let sum = 0n;
  let sumSquares = 0n;
  for (const value of values) {
    const rescaled = exactDecimalRescaled(value, scale);
    if (rescaled === null) return null;
    const mantissa = rescaled.mantissa;
    const nextSum = exactI128(sum + mantissa);
    const square = exactI128(mantissa * mantissa);
    if (nextSum === null || square === null) return null;
    const nextSquares = exactI128(sumSquares + square);
    if (nextSquares === null) return null;
    sum = nextSum;
    sumSquares = nextSquares;
  }
  const count = BigInt(values.length);
  const left = exactI128(count * sumSquares);
  const right = exactI128(sum * sum);
  if (left === null || right === null) return null;
  const numerator = exactI128(left - right);
  const denominator = exactI128(count * count);
  if (numerator === null || denominator === null) return null;
  return { numerator: numerator, denominator: denominator, scale: 2 * scale };
}

/**
 * @param {Array<{mantissa: bigint, scale: number}>} values
 * @returns {{value: string, derivation: string|null}|null}
 */
function statisticsVariance(values) {
  const fraction = statisticsVarianceFraction(values);
  if (fraction === null) return null;
  const count = exactDecimal(BigInt(values.length), 0);
  const variance = exactDecimalDiv(
    exactDecimal(fraction.numerator, fraction.scale),
    exactDecimal(fraction.denominator, 0),
    7,
  );
  if (variance === null) return null;
  const sum = statisticsSum(values);
  if (sum === null) return null;
  let derivation = null;
  const mean = exactDecimalDiv(sum, count, 7);
  if (mean !== null && mean.exact) {
    let squaredDifferences = exactDecimal(0n, 0);
    let derivable = true;
    for (const value of values) {
      const difference = exactDecimalSub(value, mean.value);
      const square = difference === null ? null : exactDecimalMul(difference, difference);
      if (square === null) {
        derivable = false;
      } else {
        squaredDifferences = exactDecimalAdd(squaredDifferences, square);
        if (squaredDifferences === null) return null;
      }
    }
    if (derivable) {
      const quotient = exactDecimalDiv(squaredDifferences, count, 7);
      if (quotient !== null) {
        const joiner = quotient.exact ? "=" : "≈";
        derivation = `${exactDecimalRender(squaredDifferences)} / ${values.length} ${joiner} ${exactDecimalRender(quotient.value)}`;
      }
    }
  }
  return { value: exactDecimalRenderApprox(variance.value, variance.exact), derivation: derivation };
}

/**
 * The standard deviation: a double-precision root of the exact variance,
 * rendered to seven fractional digits like Rust's `{:.7}`.
 * @param {Array<{mantissa: bigint, scale: number}>} values
 * @returns {{value: string, derivation: string|null}|null}
 */
function statisticsStandardDeviation(values) {
  const fraction = statisticsVarianceFraction(values);
  if (fraction === null) return null;
  const variance = Number(fraction.numerator) / Number(fraction.denominator) /
    Math.pow(10, fraction.scale);
  if (!(variance >= 0 && variance <= 1000000000)) return null;
  const root = Math.sqrt(variance);
  const exact = root * root === variance;
  let text = root.toFixed(7);
  if (text.includes(".")) {
    text = text.replace(/0+$/u, "");
    if (text.endsWith(".")) text = text.slice(0, -1);
  }
  return { value: exact ? text : `≈${text}`, derivation: null };
}

/**
 * @param {Array<{mantissa: bigint, scale: number}>} values
 * @returns {{value: string, derivation: string|null}|null}
 */
function statisticsRange(values) {
  const sorted = statisticsSorted(values);
  const smallest = sorted[0];
  const largest = sorted[sorted.length - 1];
  const span = exactDecimalSub(largest, smallest);
  if (span === null) return null;
  return {
    value: exactDecimalRender(span),
    derivation: `${exactDecimalRender(largest)} - ${exactDecimalRender(smallest)} = ${exactDecimalRender(span)}`,
  };
}

/**
 * The percentile by linear interpolation between closest ranks (C = 1): the
 * rank 1 + (n - 1) * p / 100 = k + f over the sorted values gives
 * x(k) + f * (x(k + 1) - x(k)), exact within `scale + 2` digits. Mirrors
 * `percentile_of` in statistics.rs.
 * @param {Array<{mantissa: bigint, scale: number}>} values
 * @param {{mantissa: bigint, scale: number}} rank
 * @param {string} language
 * @returns {{value: string, derivation: string|null}|null}
 */
function statisticsPercentile(values, rank, language) {
  const sorted = statisticsSorted(values);
  const numerator = exactI128(BigInt(sorted.length - 1) * rank.mantissa);
  const power = exactPow10(rank.scale);
  if (numerator === null || power === null) return null;
  const denominator = exactI128(100n * power);
  if (denominator === null) return null;
  const lowerIndex = Number(numerator / denominator);
  const remainder = numerator % denominator;
  const lower = sorted[lowerIndex];
  if (lower === undefined) return null;
  const upper = sorted[lowerIndex + 1] === undefined ? lower : sorted[lowerIndex + 1];
  const precision = rank.scale + 2;
  const fraction = exactDecimalFromQuotient(remainder, denominator, precision);
  const offset = exactDecimalFromQuotient(numerator, denominator, precision);
  if (fraction === null || offset === null) return null;
  const position = exactDecimalAdd(offset.value, exactDecimal(1n, 0));
  const difference = exactDecimalSub(upper, lower);
  if (position === null || difference === null) return null;
  const step = exactDecimalMulDiv(difference, remainder, denominator, precision);
  if (step === null) return null;
  const value = exactDecimalAdd(lower, step.value);
  if (value === null) return null;
  const template = quantityLocalizedTemplate("statistics_percentile_derivation", language);
  const derivation = template === null
    ? null
    : quantityFillTemplate(template, [
      ["p", exactDecimalRender(rank)],
      ["count", String(sorted.length)],
      ["rank", exactDecimalRender(position)],
      ["lower", exactDecimalRender(lower)],
      ["fraction", exactDecimalRender(fraction.value)],
      ["upper", exactDecimalRender(upper)],
      ["value", exactDecimalRender(value)],
    ]);
  return { value: exactDecimalRender(value), derivation: derivation };
}

/**
 * UTF-16 span of the earliest surface of the meaning `slug` (the longest on
 * a tie), or null. Mirrors `surface_span` in statistics.rs.
 * @param {string} lowered
 * @param {string} slug
 * @returns {{start: number, end: number}|null}
 */
function statisticsSurfaceSpan(lowered, slug) {
  let best = null;
  for (const word of quantityMeaningWords(slug)) {
    if (!quantityContainsTerm(lowered, word)) continue;
    const start = lowered.indexOf(word);
    const end = start + word.length;
    if (best === null || start < best.start || (start === best.start && end > best.end)) {
      best = { start: start, end: end };
    }
  }
  return best;
}

/**
 * Index, among the stated numbers, of the percentile rank glued to the
 * percentile surface, or null. Mirrors `percentile_rank_index`.
 * @param {string} lowered
 * @returns {number|null}
 */
function statisticsPercentileRankIndex(lowered) {
  const surface = statisticsSurfaceSpan(lowered, "statistics_percentile");
  if (surface === null) return null;
  const spans = [];
  let cursor = 0;
  for (const item of parseNumericListNumbers(lowered)) {
    const start = lowered.indexOf(item.text, cursor);
    if (start === -1) return null;
    cursor = start + item.text.length;
    spans.push({ start: start, end: cursor });
  }
  let before = -1;
  for (let index = 0; index < spans.length; index += 1) {
    if (spans[index].end <= surface.start) before = index;
  }
  if (before !== -1) {
    const suffix = Array.from(lowered.slice(spans[before].end, surface.start).replace(/\s+$/u, ""));
    if (suffix.length <= STATISTICS_MAX_RANK_SUFFIX_CHARS &&
      suffix.every((character) => /\p{Alphabetic}/u.test(character) || character === "-")) {
      return before;
    }
  }
  const after = spans.findIndex((span) => span.start >= surface.end);
  if (after === -1) return null;
  const lead = Array.from(lowered.slice(surface.end, spans[after].start));
  const next = quantityFirstCharacter(lowered.slice(spans[after].end));
  const opensList = [",", "，", "、", ";"].includes(next);
  const whitespace = lead.every((character) => /\s/u.test(character));
  return lead.length <= STATISTICS_MAX_RANK_LEAD_CHARS && whitespace && !opensList ? after : null;
}

/**
 * Compute one named operation.
 * @param {string} op
 * @param {Array<{mantissa: bigint, scale: number}>} values
 * @param {{mantissa: bigint, scale: number}|null} rank the percentile rank
 * @param {string} language
 * @returns {{value: string, derivation: string|null}|null}
 */
function statisticsCompute(op, values, rank, language) {
  switch (op) {
    case "mean":
      return statisticsMean(values);
    case "median":
      return statisticsMedian(values);
    case "mode":
      return statisticsMode(values);
    case "variance":
      return statisticsVariance(values);
    case "standard_deviation":
      return statisticsStandardDeviation(values);
    case "percentile":
      return statisticsPercentile(values, rank, language);
    case "range":
      return statisticsRange(values);
    default:
      throw new Error("unknown statistics operation");
  }
}

/**
 * The operation's label in the answer language, from the seed lexeme.
 * @param {string} slug
 * @param {string} language
 * @returns {string}
 */
function statisticsLabel(slug, language) {
  const meaning = findMeaning(slug);
  if (!meaning) return slug;
  return wordInLanguage(meaning, language) || wordInLanguage(meaning, "en") || slug;
}

/**
 * Statistics entry point (issue #1176): a question naming a statistics
 * operation over a stated list of at least two numbers. Mirrors
 * `handle_statistics` in rust/src/solver_handlers/statistics.rs.
 * @param {string} prompt
 * @param {string} normalized
 * @param {string} language
 * @returns {{intent: string, content: string, confidence: number, evidence: Array<string>}|null}
 */
function tryStatistics(prompt, normalized, language) {
  // R1017: a number property of one stated number ("is 97 prime?").
  const primality = tryNumberPrimality(prompt, language);
  if (primality) return primality;
  const lowered = String(prompt || "").toLowerCase();
  if (!lowered.includes("?") && !lowered.includes("？")) return null;
  const ops = STATISTICS_OPERATIONS.filter((operation) =>
    quantityMeaningWords(operation.slug).some((word) => quantityContainsTerm(normalized, word)));
  if (ops.length === 0) return null;
  const stated = statisticsStatedNumbers(lowered);
  if (stated === null) return null;
  const values = stated.values;
  // The percentile rank is a parameter, not a data point (statistics.rs).
  let rank = null;
  if (ops.some((operation) => operation.op === "percentile")) {
    const index = statisticsPercentileRankIndex(lowered);
    if (index === null || index >= values.length) return null;
    rank = values.splice(index, 1)[0];
    if (rank.mantissa < 0n || exactDecimalCompare(rank, exactDecimal(100n, 0)) > 0) return null;
  }
  if (values.length < 2) return null;
  const valuesText = values.map(exactDecimalRender).join(", ");
  const log = [`statistics:values:${valuesText}`, `statistics:count:${values.length}`];
  const lines = [];
  for (const operation of ops) {
    const result = statisticsCompute(operation.op, values, rank, language);
    if (result === null) return null;
    const label = statisticsLabel(operation.slug, language);
    const line = result.derivation === null
      ? `${label}: ${result.value}`
      : `${label}: ${result.value} (${result.derivation})`;
    log.push(`${operation.evidence}:${line}`);
    lines.push(line);
  }
  const results = lines.join("\n");
  const template = quantityLocalizedTemplate("statistics", language);
  const body = template === null
    ? results
    : quantityFillTemplate(template, [
      ["values", valuesText],
      ["count", String(values.length)],
      ["results", results],
    ]);
  return quantityAnswer("statistics", body, log, language);
}

/**
 * Byte offset of the earliest surface of the marker meaning, or null.
 * @param {string} lowered
 * @param {string} slug
 * @returns {number|null}
 */
function wordProblemMarkerPosition(lowered, slug) {
  let best = null;
  for (const word of quantityMeaningWords(slug)) {
    const position = quantityTermPosition(lowered, word);
    if (position !== null && (best === null || position < best)) best = position;
  }
  return best;
}

/**
 * Word-problem entry point (issue #1176): the price×count change and total
 * patterns. Mirrors `handle_word_problem` in statistics.rs.
 * @param {string} prompt
 * @param {string} normalized
 * @param {string} language
 * @returns {{intent: string, content: string, confidence: number, evidence: Array<string>}|null}
 */
function tryWordProblem(prompt, normalized, language) {
  if (!wordProblemMentionsMarker(normalized, WORD_PROBLEM_MARKER_UNIT_PRICE)) return null;
  const lowered = String(prompt || "").toLowerCase();
  const stated = statisticsStatedNumbers(lowered);
  if (stated === null) return null;
  const values = stated.values;
  const positions = stated.positions;
  const eachMarker = wordProblemMarkerPosition(lowered, WORD_PROBLEM_MARKER_UNIT_PRICE);
  const eachAt = eachMarker === null ? 0 : eachMarker;
  const nearest = wordProblemNearestBefore(positions, eachAt);
  const priceIndex = nearest === null ? 0 : nearest;

  if (wordProblemMentionsMarker(normalized, WORD_PROBLEM_MARKER_PAYMENT) &&
    wordProblemMentionsMarker(normalized, WORD_PROBLEM_MARKER_CHANGE) &&
    values.length === 3) {
    const paymentIndex = values.length - 1;
    let countIndex = -1;
    for (let index = 0; index < values.length; index += 1) {
      if (index !== priceIndex && index !== paymentIndex) {
        countIndex = index;
        break;
      }
    }
    if (countIndex === -1) return null;
    const price = values[priceIndex];
    const count = values[countIndex];
    const payment = values[paymentIndex];
    const cost = exactDecimalMul(count, price);
    if (cost === null) return null;
    const change = exactDecimalSub(payment, cost);
    if (change === null || change.mantissa < 0n) return null;
    const derivation = `${exactDecimalRender(payment)} - ${exactDecimalRender(count)} × ${exactDecimalRender(price)} = ${exactDecimalRender(change)}`;
    const log = [
      `word_problem:unit_price:${exactDecimalRender(price)}`,
      `word_problem:count:${exactDecimalRender(count)}`,
      `word_problem:payment:${exactDecimalRender(payment)}`,
      `word_problem:change:${exactDecimalRender(change)}`,
      `word_problem:derivation:${derivation}`,
    ];
    const template = quantityLocalizedTemplate("word_problem_change", language);
    const body = template === null
      ? derivation
      : quantityFillTemplate(template, [
        ["change", exactDecimalRender(change)],
        ["derivation", derivation],
      ]);
    return quantityAnswer("word_problem_change", body, log, language);
  }

  if (wordProblemMentionsMarker(normalized, WORD_PROBLEM_MARKER_TOTAL) && values.length === 2) {
    const countIndex = priceIndex === 0 ? 1 : 0;
    const price = values[priceIndex];
    const count = values[countIndex];
    const total = exactDecimalMul(count, price);
    if (total === null) return null;
    const derivation = `${exactDecimalRender(count)} × ${exactDecimalRender(price)} = ${exactDecimalRender(total)}`;
    const log = [
      `word_problem:unit_price:${exactDecimalRender(price)}`,
      `word_problem:count:${exactDecimalRender(count)}`,
      `word_problem:total:${exactDecimalRender(total)}`,
      `word_problem:derivation:${derivation}`,
    ];
    const template = quantityLocalizedTemplate("word_problem_total", language);
    const body = template === null
      ? derivation
      : quantityFillTemplate(template, [
        ["total", exactDecimalRender(total)],
        ["derivation", derivation],
      ]);
    return quantityAnswer("word_problem_total", body, log, language);
  }
  return null;
}

// Primality of one stated whole number (R1017), mirroring
// rust/src/solver_handlers/statistics/primality.rs: exactly one standalone
// whole number, the `number_property_prime` surfaces named after it, no code
// request; trial division up to floor(sqrt(n)) decides, and a composite names
// its smallest factor with the cofactor.

const NUMBER_PROPERTY_PRIME_SLUG = "number_property_prime";
const NUMBER_PROPERTY_MAX_TESTED = 1000000000000;

/**
 * Offsets of the prompt's only standalone whole number, or null (Rust
 * `single_stated_integer`).
 * @param {string} lowered
 * @returns {{start: number, end: number}|null}
 */
function numberPropertySingleInteger(lowered) {
  const glued = (character) => /[A-Za-z0-9.]/u.test(character);
  let found = null;
  for (const match of lowered.matchAll(/[0-9]+/gu)) {
    const start = match.index;
    const end = start + match[0].length;
    const before = start > 0 ? lowered[start - 1] : "";
    const after = end < lowered.length ? lowered[end] : "";
    if (before === "-") return null;
    if ((before && glued(before)) || (after && glued(after))) continue;
    if (found !== null) return null;
    found = { start: start, end: end };
  }
  return found;
}

/**
 * Answer "is <n> prime?" (Rust `handle_primality`), or null.
 * @param {string} prompt
 * @param {string} language
 * @returns {{intent: string, content: string, confidence: number, evidence: Array<string>}|null}
 */
function tryNumberPrimality(prompt, language) {
  const lowered = String(prompt || "").toLowerCase();
  const span = numberPropertySingleInteger(lowered);
  if (span === null) return null;
  const after = lowered.slice(span.end);
  if (!quantityMeaningWords(NUMBER_PROPERTY_PRIME_SLUG).some((word) => quantityContainsTerm(after, word))) {
    return null;
  }
  const normalized = normalizePrompt(prompt);
  if (["code_request", "function", "implement"].some((slug) => operationMatchesSlug(slug, normalized))) {
    return null;
  }
  const number = Number(lowered.slice(span.start, span.end));
  if (!Number.isSafeInteger(number) || number > NUMBER_PROPERTY_MAX_TESTED) return null;
  const limit = Math.floor(Math.sqrt(number));
  let factor = null;
  for (let divisor = 2; divisor <= limit; divisor += 1) {
    if (number % divisor === 0) {
      factor = divisor;
      break;
    }
  }
  let intent = "number_primality_prime";
  if (number < 2) intent = "number_primality_below_two";
  else if (factor !== null) intent = "number_primality_composite";
  else if (limit < 2) intent = "number_primality_prime_small";
  const subject = String(number);
  const log = [
    `number_property:subject:${subject}`,
    `number_property:property:${NUMBER_PROPERTY_PRIME_SLUG}`,
    `number_property:trial_division:2..=${limit}`,
  ];
  if (factor !== null) log.push(`number_property:factor:${subject} = ${factor} × ${number / factor}`);
  log.push(`number_property:result:${factor === null && number >= 2}`);
  const template = quantityLocalizedTemplate(intent, language);
  if (template === null) return null;
  const body = quantityFillTemplate(template, [
    ["n", subject],
    ["limit", String(limit)],
    ["factor", factor === null ? "" : String(factor)],
    ["cofactor", factor === null ? "" : String(number / factor)],
  ]);
  return quantityAnswer("number_primality", body, log, language);
}
