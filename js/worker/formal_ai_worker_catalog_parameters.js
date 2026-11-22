// Algorithm operands and output, mirrored in rust/src/coding/catalog_parameters.rs.
const CATALOG_PARAMETER_GUIDANCE = Object.freeze({
  count_to_three: "request-program-range",
  fizzbuzz: "request-program-range",
  sum_to_ten: "request-program-range",
  factorial: "request-factorial-operand",
  reverse_string: "request-reversal-operand",
  fibonacci: "request-fibonacci-index",
});
const MAXIMUM_CATALOG_RANGE_SPAN = 999;
const MAXIMUM_CATALOG_RANGE_MAGNITUDE = 1_000_000;
const MAXIMUM_CATALOG_FACTORIAL = 18;
const MAXIMUM_CATALOG_FIBONACCI = 77;

function catalogNumberText(prompt) {
  return obligationTextOutsideQuotes(prompt).toLowerCase();
}

function catalogIntegers(text, language) {
  const tokens = /[-+]?[0-9]+(?:\.[0-9]+)?(?:st|nd|rd|th)?|\p{Script=Han}|(?:(?!\p{Script=Han})[\p{L}\p{M}])+/gu;
  const words = new Map(arithmeticNormalizationTables().tokens);
  const aliases = wordsForMeaning(`program_language_${language}`);
  const result = [];
  let previousEnd = 0;
  for (const token of text.matchAll(tokens)) {
    const word = token[0];
    const before = text.slice(0, token.index).trimEnd();
    const afterLanguage = aliases.some((alias) => before.endsWith(alias) && before.length - alias.length <= previousEnd);
    previousEnd = token.index + word.length;
    if (/^[-+]?[0-9]/u.test(word)) {
      if (afterLanguage && /\s/u.test(text[token.index - 1] || "")) continue;
      const value = Number(word.replace(/(?:st|nd|rd|th)$/u, ""));
      if (!Number.isSafeInteger(value) || word.includes(".")) return null;
      result.push({ value, start: token.index, end: previousEnd });
    } else if (!(word === "一" && /^[个個]/u.test(text.slice(previousEnd))) && word !== "un" && word !== "una" && /^\d+$/u.test(words.get(word) || "")) {
      result.push({ value: Number(words.get(word)), start: token.index, end: previousEnd });
    }
  }
  return result;
}

function catalogIntegerRange(text, operands) {
  const connectors = wordsForMeaning("line_range_connector");
  if (operands?.length === 1) {
    const [end] = operands;
    const prefix = text.slice(0, end.start).trimEnd();
    return connectors.some((word) => prefix.endsWith(word)
      && (prefix.length === word.length || /\s/u.test(prefix[prefix.length - word.length - 1]) || /^[到至]$/u.test(word)))
      ? [1, end.value] : null;
  }
  if (operands?.length !== 2) return null;
  const [start, end] = operands;
  const separator = text.slice(start.end, end.start).trim();
  return separator === ".." || separator === "..=" || connectors.includes(separator)
    ? [start.value, end.value] : null;
}

function catalogReversalOperand(prompt) {
  const quoted = obligationQuotedSegmentSpans(prompt);
  if (quoted.length === 1) return quoted[0].text;
  if (quoted.length) return null;
  const lower = prompt.toLowerCase();
  if (lower.length !== prompt.length) return null;
  for (const alias of wordsForMeaning("program-reverse-operand-introducer")) {
    const start = lower.lastIndexOf(alias);
    if (start < 0) continue;
    const suffix = prompt.slice(start + alias.length);
    if (/^\s+[\p{L}\p{M}\p{N}_]+\s*$/u.test(suffix)) return suffix.trim();
  }
  return null;
}

function bindCatalogProgram(prompt, task, language, template) {
  if (typeof template !== "string") return null;
  const values = new Map();
  let output = null;
  if (["count_to_three", "fizzbuzz", "sum_to_ten"].includes(task)) {
    const text = catalogNumberText(prompt);
    const range = catalogIntegerRange(text, catalogIntegers(text, language));
    if (!range) return null;
    const [start, end] = range;
    if (start < -MAXIMUM_CATALOG_RANGE_MAGNITUDE || end > MAXIMUM_CATALOG_RANGE_MAGNITUDE
        || end < start || end - start > MAXIMUM_CATALOG_RANGE_SPAN) return null;
    values.set("start", start);
    values.set("end", end);
    values.set("end_exclusive", end + 1);
    if (task === "sum_to_ten") {
      output = String((start + end) * (end - start + 1) / 2);
    } else {
      const lines = [];
      for (let n = start; n <= end; n += 1) {
        const token = task === "fizzbuzz"
          ? (n % 3 === 0 ? "Fizz" : "") + (n % 5 === 0 ? "Buzz" : "")
          : "";
        lines.push(token || String(n));
      }
      output = lines.join("\n");
    }
  } else if (task === "factorial") {
    const operands = catalogIntegers(catalogNumberText(prompt), language);
    if (operands?.length !== 1) return null;
    const [{ value: n }] = operands;
    if (n < 0 || n > MAXIMUM_CATALOG_FACTORIAL) return null;
    values.set("n", n);
    values.set("end_exclusive", n + 1);
    let factorial = 1;
    for (let factor = 2; factor <= n; factor += 1) factorial *= factor;
    output = String(factorial);
  } else if (task === "reverse_string") {
    const text = catalogReversalOperand(prompt);
    if (text === null) return null;
    const literal = programStringLiteral(language, text);
    if (literal === null) return null;
    values.set("text", literal);
    output = Array.from(text).reverse().join("");
  } else if (task === "fibonacci") {
    const operands = catalogIntegers(catalogNumberText(prompt), language);
    if (!operands || operands.length > 1) return null;
    if (operands.length === 0) {
      return {
        code: template.split("\n").filter((line) => !line.includes("{{index}}") && !line.includes("{{go_fmt}}")).join("\n").trimEnd(),
        output,
      };
    }
    const [{ value: n }] = operands;
    if (n < 0 || n > MAXIMUM_CATALOG_FIBONACCI) return null;
    let previous = 0;
    let current = 1;
    for (let i = 0; i < n; i += 1) [previous, current] = [current, previous + current];
    values.set("index", n);
    values.set("go_fmt", 'import "fmt"');
    output = String(previous);
  }
  return { code: template.replace(/\{\{([a-z_]+)\}\}/gu, (slot, name) => values.get(name) ?? slot), output };
}

/** Encode caller text using the target language's literal policy. */
function programStringLiteral(language, output) {
  if (typeof output !== "string" || /[\x00-\x08\x0a-\x1f\x7f-\x9f]/u.test(output)) return null;
  const renderer = PROGRAM_PRINT_RENDERERS.get(language);
  if (!renderer) return null;
  const { style } = renderer;
  let quoted;
  if (style === "shell") {
    quoted = "'" + output.replaceAll("'", "'\\''") + "'";
  } else if (style === "php_single") {
    quoted = "'" + output.replace(/[\\']/gu, (value) => "\\" + value) + "'";
  } else {
    quoted = '"' + output.replace(/[\\"\t$#]/gu, (value) => {
      if (value === "\t") return "\\t";
      if (value === "$" && style !== "dollar" || value === "#" && style !== "ruby") return value;
      return "\\" + value;
    }) + '"';
  }
  return quoted;
}

function catalogParameterGap(parameterGuidance, responseLanguage, language, task, diagnostics) {
  return {
    intent: "write_program",
    content: codingGuidance(parameterGuidance, responseLanguage),
    confidence: 0.9,
    evidence: [`program_parameter:language:${language}`, `program_parameter:task:${task}`],
    steps: diagnostics.steps,
    trace: diagnostics.trace,
  };
}
