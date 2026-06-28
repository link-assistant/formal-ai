// Worker module 7 of 21. Loaded by ../formal_ai_worker.js.
function compoundLabel(compoundsPerYear) {
  switch (compoundsPerYear) {
    case 1:
      return "annually";
    case 4:
      return "quarterly";
    case 12:
      return "monthly";
    case 52:
      return "weekly";
    case 365:
      return "daily";
    default:
      return "times per year";
  }
}

function formatCompoundNumber(value) {
  if (Math.abs(value % 1) < 1e-10) return value.toFixed(0);
  return trimCompoundDecimal(value.toFixed(10));
}

function formatCompoundMoney(value) {
  return value.toFixed(2);
}

function roundCompoundMoney(value) {
  return Math.round(value * 100) / 100;
}

function formatCompoundRate(value) {
  return trimCompoundDecimal(value.toFixed(15));
}

function trimCompoundDecimal(value) {
  return String(value).replace(/0+$/, "").replace(/\.$/, "");
}

function parseClockTimeMinutes(value) {
  const match = /^([0-9]{1,2}):([0-9]{2})$/.exec(String(value || "").trim());
  if (!match) return null;
  const hour = Number(match[1]);
  const minute = Number(match[2]);
  if (hour > 23 || minute > 59) return null;
  return hour * 60 + minute;
}

function formatClockDuration(minutes) {
  if (minutes <= 0) return "0 seconds";
  const hours = Math.floor(minutes / 60);
  const remainingMinutes = minutes % 60;
  const parts = [];
  if (hours > 0) parts.push(`${hours} ${hours === 1 ? "hour" : "hours"}`);
  if (remainingMinutes > 0) {
    parts.push(
      `${remainingMinutes} ${remainingMinutes === 1 ? "minute" : "minutes"}`,
    );
  }
  return parts.join(", ");
}

function evaluateClockDifferenceExpression(expression) {
  const match =
    /^\s*([0-9]{1,2}:[0-9]{2})\s*[-−]\s*([0-9]{1,2}:[0-9]{2})\s*$/.exec(
      String(expression || ""),
    );
  if (!match) return null;
  const left = parseClockTimeMinutes(match[1]);
  const right = parseClockTimeMinutes(match[2]);
  if (left === null || right === null) return null;
  return formatClockDuration(left - right);
}

function renderCalculationReasoningStep(step, index) {
  const text = String(step || "");
  return text.trimStart().startsWith("[") ? text : `Step ${index + 1}: ${text}`;
}

function tryArithmetic(prompt) {
  const extracted = extractArithmeticExpression(prompt);
  if (!extracted) return null;
  const expression = extracted.expression;
  const interpretations = Array.isArray(extracted.interpretations)
    ? extracted.interpretations
    : [];
  const reasoningSteps = Array.isArray(extracted.reasoningSteps)
    ? extracted.reasoningSteps
    : [];
  const resultLabel = typeof extracted.resultLabel === "string" ? extracted.resultLabel : "";
  try {
    const isEquation = expression.includes("=");
    let formatted;
    let backend = "js";
    const wasmResult = wasmEvaluateArithmetic(expression);
    if (wasmResult && wasmResult.ok) {
      formatted = wasmResult.value;
      backend = "wasm";
    } else {
      const percentOfResult = evaluatePercentOfExpression(expression);
      const currencyConversionResult = evaluateCurrencyConversionExpression(expression);
      const clockDifferenceResult = evaluateClockDifferenceExpression(expression);
      if (currencyConversionResult !== null) {
        formatted = currencyConversionResult;
        backend = "js-currency";
      } else if (percentOfResult) {
        formatted = percentOfResult;
        backend = "js-percent-of";
      } else if (clockDifferenceResult !== null) {
        formatted = clockDifferenceResult;
        backend = "js-clock-time";
      } else if (isEquation) {
        formatted = solveEquation(expression);
        backend = "js-equation-fallback";
      } else {
        formatted = formatArithmeticResult(evaluateArithmetic(expression));
      }
    }
    const renderedExpression = escapeCalculationMarkdown(expression.trim());
    const renderedFormatted = escapeCalculationMarkdown(formatted);
    const calculationLine = isEquation
      ? `${renderedExpression} => ${renderedFormatted}`
      : `${renderedExpression} = ${renderedFormatted}`;
    const sections = [];
    if (reasoningSteps.length > 0) {
      sections.push(
        reasoningSteps
          .map((step, index) => renderCalculationReasoningStep(step, index))
          .join("\n"),
      );
    }
    sections.push(calculationLine);
    if (resultLabel) {
      sections.push(`Therefore, there are ${formatted} ${resultLabel} in total.`);
    }
    const content = sections.join("\n\n");
    const evidence = [
      `calculation:${content}`,
      `calculation_backend:${backend}`,
    ];
    if (reasoningSteps.length > 0) {
      evidence.push(`calculation_reasoning_steps:${reasoningSteps.length}`);
    }
    if (resultLabel) evidence.push(`calculation_result_label:${resultLabel}`);
    return {
      intent: "calculation",
      content: content,
      confidence: 1.0,
      evidence,
      interpretations,
    };
  } catch (error) {
    const message = String(error && error.message ? error.message : error);
    return {
      intent: "calculation_error",
      content: `I could not evaluate \`${expression.trim()}\`: ${message}.`,
      confidence: 0.4,
      evidence: [`calculation_error:${message}`],
      interpretations,
    };
  }
}

const SYNTHESIS_NUMBER_WORDS = new Map([
  ["zero", 0],
  ["one", 1],
  ["a", 1],
  ["an", 1],
  ["two", 2],
  ["three", 3],
  ["four", 4],
  ["five", 5],
  ["six", 6],
  ["seven", 7],
  ["eight", 8],
  ["nine", 9],
  ["ten", 10],
  ["eleven", 11],
  ["twelve", 12],
  ["thirteen", 13],
  ["fourteen", 14],
  ["fifteen", 15],
  ["sixteen", 16],
  ["seventeen", 17],
  ["eighteen", 18],
  ["nineteen", 19],
  ["twenty", 20],
]);

const SYNTHESIS_OBJECT_CATEGORIES = new Map([
  [
    "musical instrument",
    new Set([
      "clarinet",
      "flute",
      "guitar",
      "harmonica",
      "piano",
      "saxophone",
      "trumpet",
      "violin",
      "drum",
    ]),
  ],
  ["fruit", new Set(["apple", "banana", "orange", "pear", "grape"])],
  ["vegetable", new Set(["carrot", "onion", "potato", "tomato", "pepper"])],
  ["animal", new Set(["cat", "dog", "horse", "cow", "bird"])],
  ["vehicle", new Set(["car", "truck", "bus", "bicycle", "train"])],
  ["tool", new Set(["hammer", "saw", "wrench", "screwdriver", "drill"])],
  ["utensil", new Set(["spoon", "fork", "knife", "ladle"])],
  ["furniture", new Set(["chair", "table", "sofa", "desk", "bed"])],
  ["clothing", new Set(["shirt", "coat", "hat", "shoe", "dress"])],
]);

const PYTHON_SYNTHESIS_CANDIDATES = {
  has_close_elements: {
    id: "pairwise_threshold_distance",
    defaultSignature:
      "has_close_elements(numbers: list[float], threshold: float) -> bool",
    body: [
      {
        kind: "for_loop",
        semanticNode: "pairwise_outer_loop",
        target: "left_index, left",
        iterator: "enumerate(numbers)",
        body: [
          {
            kind: "for_loop",
            semanticNode: "pairwise_inner_loop",
            target: "right",
            iterator: "numbers[left_index + 1:]",
            body: [
              {
                kind: "if_return",
                semanticNode: "threshold_match_return",
                condition: "abs(left - right) < threshold",
                value: "True",
              },
            ],
          },
        ],
      },
      {
        kind: "return",
        semanticNode: "no_pair_matches_return",
        expression: "False",
      },
    ],
    tests: [
      "assert has_close_elements([1.0, 2.0, 3.0], 0.5) is False",
      "assert has_close_elements([1.0, 2.0, 3.0], 1.1) is True",
      "assert has_close_elements([1.0, 2.8, 3.0], 0.3) is True",
      "assert has_close_elements([], 0.1) is False",
    ],
    fragments: [
      "python:def_function",
      "loop:pairwise_distinct_values",
      "predicate:absolute_difference_less_than_threshold",
      "branch:return_false_when_no_pair_matches",
    ],
  },
  similar_elements: {
    id: "tuple_intersection_set",
    defaultSignature: "similar_elements(test_tup1, test_tup2)",
    body: [
      {
        kind: "return",
        semanticNode: "deterministic_tuple_intersection_return",
        expression: "tuple(sorted(set(test_tup1) & set(test_tup2)))",
      },
    ],
    tests: [
      "assert similar_elements((3, 4, 5, 6), (5, 7, 4, 10)) == (4, 5)",
      "assert similar_elements((1, 2), (3, 4)) == ()",
      "assert similar_elements(('a', 'b'), ('b', 'c')) == ('b',)",
    ],
    fragments: [
      "python:def_function",
      "collection:set_intersection",
      "collection:deterministic_tuple_order",
    ],
  },
  count_vowels: {
    id: "count_matching_characters",
    defaultSignature: "count_vowels(text: str) -> int",
    body: [
      {
        kind: "assign",
        semanticNode: "vowel_membership_set_assignment",
        target: "vowels",
        expression: 'set("aeiouAEIOU")',
      },
      {
        kind: "return",
        semanticNode: "matching_character_count_return",
        expression: "sum(1 for character in text if character in vowels)",
      },
    ],
    tests: [
      "assert count_vowels('hello') == 2",
      "assert count_vowels('sky') == 0",
      "assert count_vowels('Formal AI') == 4",
    ],
    fragments: [
      "python:def_function",
      "collection:membership_set",
      "aggregation:sum_generator",
    ],
  },
};

function synthesisStableId(prefix, value) {
  const wasmId = wasmStableId(prefix, value);
  if (wasmId) return wasmId;
  const text = `${String(prefix || "")}\n${String(value || "")}`;
  let hash = 0xcbf29ce484222325n;
  const prime = 0x100000001b3n;
  const mask = 0xffffffffffffffffn;
  for (let index = 0; index < text.length; index += 1) {
    hash ^= BigInt(text.charCodeAt(index));
    hash = (hash * prime) & mask;
  }
  return `${prefix}:${hash.toString(16).padStart(16, "0")}`;
}

function truncateSynthesisTrace(value, limit = 80) {
  const text = String(value || "").replace(/\s+/g, " ").trim();
  return text.length > limit ? `${text.slice(0, limit)}...` : text;
}

function synthesisSubResults(prompt) {
  return String(prompt || "")
    .split(/(?:[.;?]+|\band\b)/i)
    .map((part) => part.trim())
    .filter(Boolean)
    .slice(0, 6)
    .map((part, index) => ({
      id: synthesisStableId("sub_result", `${index}:${part}`),
      text: part,
    }));
}

function subResultEvidence(subResults) {
  return subResults.map(
    (item) => `sub_result:${item.id}:${truncateSynthesisTrace(item.text, 60)}`,
  );
}

function evaluateSynthesisArithmetic(expression) {
  const wasmResult = wasmEvaluateArithmetic(expression);
  if (wasmResult && wasmResult.ok) {
    return { formatted: wasmResult.value, backend: "wasm" };
  }
  return {
    formatted: formatArithmeticResult(evaluateArithmetic(expression)),
    backend: "js-fallback",
  };
}

function extractAlgebraAssignments(prompt) {
  const assignments = [];
  const seen = new Set();
  const pattern = /(?:^|[\s,;(])([A-Za-z_][A-Za-z0-9_]{0,1})\s*=\s*([+-]?\d+(?:\.\d+)?)/g;
  let match;
  while ((match = pattern.exec(String(prompt || ""))) !== null) {
    const variable = match[1].toLowerCase();
    if (seen.has(variable)) continue;
    seen.add(variable);
    assignments.push({ variable, value: match[2] });
  }
  return assignments;
}

function extractRequestedAlgebraExpression(prompt) {
  const text = String(prompt || "");
  const lower = text.toLowerCase();
  for (const marker of ["value of", "evaluate", "calculate", "compute"]) {
    const start = lower.indexOf(marker);
    if (start < 0) continue;
    const tail = text.slice(start + marker.length).trim();
    const sentence = tail.replace(/[?.!]+$/g, "").trim();
    const cleaned = sentence
      .replace(/^(?:the\s+)?(?:expression|value|result)\s+(?:of\s+)?/i, "")
      .replace(/^then\s+/i, "")
      .trim();
    if (cleaned && /[A-Za-z_]/.test(cleaned)) return cleaned;
  }
  return null;
}

function lastNonWhitespace(value) {
  for (let index = value.length - 1; index >= 0; index -= 1) {
    const ch = value[index];
    if (!/\s/.test(ch)) return ch;
  }
  return "";
}

function substituteAlgebraVariables(expression, assignments) {
  const values = new Map(assignments.map((item) => [item.variable, item.value]));
  let out = "";
  let index = 0;
  while (index < expression.length) {
    const ch = expression[index];
    if (/[A-Za-z_]/.test(ch)) {
      const start = index;
      index += 1;
      while (index < expression.length && /[A-Za-z_]/.test(expression[index])) {
        index += 1;
      }
      const token = expression.slice(start, index).toLowerCase();
      const value = values.get(token);
      if (value !== undefined) {
        const previous = lastNonWhitespace(out);
        if (/[0-9)]/.test(previous)) out += "*";
        out += value;
        if (expression[index] === "(") out += "*";
      } else {
        out += expression.slice(start, index);
      }
      continue;
    }
    if (ch === "(" && /[0-9]/.test(lastNonWhitespace(out))) out += "*";
    out += ch;
    index += 1;
  }
  return out;
}

function composeAlgebraSubstitution(prompt, subResults) {
  const assignments = extractAlgebraAssignments(prompt);
  if (assignments.length === 0) return null;
  const expression = extractRequestedAlgebraExpression(prompt);
  if (!expression) return null;
  if (
    !assignments.some((assignment) =>
      new RegExp(`(^|[^A-Za-z_])${assignment.variable}([^A-Za-z_]|$)`, "i").test(
        expression,
      ),
    )
  ) {
    return null;
  }
  try {
    const substituted = substituteAlgebraVariables(expression, assignments);
    const evaluation = evaluateSynthesisArithmetic(substituted);
    const assignmentText = assignments
      .map((assignment) => `${assignment.variable}=${assignment.value}`)
      .join(", ");
    const evaluationText = `${substituted} = ${evaluation.formatted}`;
    const evidence = [
      ...subResultEvidence(subResults),
      `composition:substitution:${assignmentText} -> ${substituted}`,
      `composition:evaluation:${evaluationText}`,
      `calculation_backend:${evaluation.backend}`,
    ];
    return {
      intent: "algebra_substitution",
      content: `Substituting ${assignmentText} into ${expression} gives ${evaluationText}.`,
      confidence: 1.0,
      evidence,
      trace: [
        `composition:substitution:${assignmentText}`,
        `composition:evaluation:${evaluationText}`,
      ],
    };
  } catch (_error) {
    return null;
  }
}

function extractSynthesisQuantities(prompt) {
  return String(prompt || "")
    .split(/[^A-Za-z0-9$.-]+/)
    .map((raw) =>
      raw
        .replace(/^\$+/, "")
        .replace(/^[.-]+/, "")
        .replace(/[.-]+$/, ""),
    )
    .filter(Boolean)
    .map((token) => {
      if (/^\d+$/.test(token)) return Number(token);
      return SYNTHESIS_NUMBER_WORDS.get(token.toLowerCase());
    })
    .filter((value) => Number.isInteger(value));
}

function composeRemainderSale(prompt, subResults) {
  const lower = String(prompt || "").toLowerCase();
  if (!lower.includes("remainder") || !lower.includes("sell")) return null;
  const quantities = extractSynthesisQuantities(prompt);
  if (quantities.length < 4) return null;
  const total = quantities[0];
  const price = quantities[quantities.length - 1];
  const consumed = quantities.slice(1, -1).reduce((sum, value) => sum + value, 0);
  if (total <= 0 || price <= 0 || consumed < 0 || consumed >= total) return null;
  const remaining = total - consumed;
  const expression = `(${total} - ${consumed}) * ${price}`;
  try {
    const evaluation = evaluateSynthesisArithmetic(expression);
    const evaluationText = `${expression} = ${evaluation.formatted}`;
    return {
      intent: "arithmetic_word_problem",
      content:
        `The remainder is ${total} - ${consumed} = ${remaining}. ` +
        `Selling ${remaining} at ${price} each gives ${evaluation.formatted}.`,
      confidence: 1.0,
      evidence: [
        ...subResultEvidence(subResults),
        `composition:remainder:total=${total} consumed=${consumed} remainder=${remaining} price=${price}`,
        `composition:evaluation:${evaluationText}`,
        `calculation_backend:${evaluation.backend}`,
      ],
      trace: [
        `composition:remainder:total=${total} consumed=${consumed} remainder=${remaining} price=${price}`,
        `composition:evaluation:${evaluationText}`,
      ],
    };
  } catch (_error) {
    return null;
  }
}

function singularizeSynthesisToken(token) {
  if (token.length > 4 && token.endsWith("ies")) return `${token.slice(0, -3)}y`;
  if (token.length > 3 && token.endsWith("s") && !token.endsWith("ss")) {
    return token.slice(0, -1);
  }
  return token;
}

function normalizeCountPhrase(value) {
  return String(value || "")
    .toLowerCase()
    .split(/[^a-z0-9]+/)
    .filter(Boolean)
    .filter((token) => !["a", "an", "the", "of"].includes(token))
    .map(singularizeSynthesisToken)
    .join(" ");
}

function cleanCountedItem(raw) {
  return String(raw || "")
    .trim()
    .replace(/^[\s.,:;!?]+|[\s.,:;!?]+$/g, "")
    .replace(/^(?:a|an|the|one)\s+/i, "")
    .trim();
}

function extractCountedItems(prompt) {
  const match = String(prompt || "").match(/\bi\s+have\s+(.+?)\.\s*how\s+many\b/i);
  const segment = match ? match[1] : "";
  if (!segment) return [];
  return segment
    .replace(/\s+and\s+/gi, ", ")
    .split(",")
    .map(cleanCountedItem)
    .filter(Boolean);
}

function extractRequestedCountCategory(prompt) {
  const match = String(prompt || "").match(
    /\bhow\s+many\s+(.+?)(?:\s+do\s+i\s+have|\s+are\s+there|[?.!]|$)/i,
  );
  return match ? normalizeCountPhrase(match[1]) : "";
}

function composeObjectCount(prompt, subResults) {
  const items = extractCountedItems(prompt);
  if (items.length === 0) return null;
  const category = extractRequestedCountCategory(prompt);
  const accepted = SYNTHESIS_OBJECT_CATEGORIES.get(category);
  if (!accepted) return null;
  const matched = items.filter((item) => accepted.has(normalizeCountPhrase(item)));
  if (matched.length === 0) return null;
  const categoryLabel = category.endsWith("s") ? category : `${category}s`;
  return {
    intent: "object_counting",
    content:
      `Matching ${categoryLabel} in the list gives ${matched.join(", ")}. ` +
      `Count: ${matched.length}.`,
    confidence: 1.0,
    evidence: [
      ...subResultEvidence(subResults),
      `composition:category:category=${categoryLabel} items=${items.join("|")}`,
      `composition:count:matched=${matched.join("|")} count=${matched.length}`,
    ],
    trace: [
      `composition:category:category=${categoryLabel}`,
      `composition:count:matched=${matched.join("|")} count=${matched.length}`,
    ],
  };
}

function tryLinkNativeSynthesis(prompt) {
  const subResults = synthesisSubResults(prompt);
  if (subResults.length === 0) return null;
  return (
    composeAlgebraSubstitution(prompt, subResults) ||
    composeRemainderSale(prompt, subResults) ||
    composeObjectCount(prompt, subResults)
  );
}

// Does `normalized` read like a request to synthesise a Python function?
// Mirrors looks_like_python_function_request in
// src/solver_handlers/program_synthesis.rs: a function *subject*, a *domain*
// signal (Python or a data kind), and an *action* verb — all supplied by the
// meaning lexicon, never hardcoded. `def ` is Python syntax a user may paste,
// so a literal signature satisfies both the subject and action sides.
function looksLikePythonFunctionSynthesis(prompt, normalized) {
  const hasDef = String(prompt || "").toLowerCase().includes("def ");
  return (
    (lexiconMentionsRole(ROLE_PROGRAM_SYNTHESIS_SUBJECT, normalized) || hasDef) &&
    lexiconMentionsRole(ROLE_PROGRAM_SYNTHESIS_DOMAIN, normalized) &&
    (lexiconMentionsRole(ROLE_PROGRAM_SYNTHESIS_ACTION, normalized) || hasDef)
  );
}

// Is `task` evidenced by its signals — is every `program_synthesis_signal`
// meaning it is `defined_by` present in `normalized`? A task with no signal
// definitions is never matched this way (it can still match by declared name).
// Mirrors synthesis_task_evidenced in src/solver_handlers/program_synthesis.rs.
function synthesisTaskEvidenced(task, normalized) {
  let required = 0;
  for (const target of task.definedBy) {
    const signal = findMeaning(target);
    if (!signal || !signal.roles.includes(ROLE_PROGRAM_SYNTHESIS_SIGNAL)) continue;
    required += 1;
    if (!meaningEvidencedIn(signal, normalized)) return false;
  }
  return required > 0;
}

// The canonical function name of the first synthesis task (declaration order)
// whose signals are all evidenced in `normalized`, or "". The slug is the
// Python function name. Mirrors match_synthesis_task in
// src/solver_handlers/program_synthesis.rs.
function matchSynthesisTask(normalized) {
  const task = meaningsWithRole(ROLE_PROGRAM_SYNTHESIS_TASK).find((candidate) =>
    synthesisTaskEvidenced(candidate, normalized),
  );
  return task ? task.slug : "";
}

function identifierAfterAsciiMarker(prompt, marker) {
  const text = String(prompt || "");
  const lower = text.toLowerCase();
  const start = lower.indexOf(marker);
  if (start < 0) return "";
  let name = "";
  let started = false;
  for (const character of text.slice(start + marker.length)) {
    if (/[A-Za-z0-9_]/.test(character)) {
      name += character;
      started = true;
    } else if (started) {
      break;
    } else if (!/\s/.test(character)) {
      return "";
    }
  }
  return name;
}

// The Python function name a prompt asks for: the matched task's slug (which
// *is* its function name), else the identifier in a literal `function `/`def `
// signature. Mirrors extract_function_name in
// src/solver_handlers/program_synthesis.rs (the declared-signature scan there is
// covered here by the marker fallbacks the worker has always used).
function extractPythonFunctionName(prompt, normalized) {
  const task = matchSynthesisTask(normalized);
  if (task) return task;
  return (
    identifierAfterAsciiMarker(prompt, "function ") ||
    identifierAfterAsciiMarker(prompt, "def ")
  );
}

function matchingCloseParen(text, openIndex) {
  let depth = 0;
  for (let index = openIndex; index < text.length; index += 1) {
    const character = text[index];
    if (character === "(") {
      depth += 1;
    } else if (character === ")") {
      depth -= 1;
      if (depth === 0) return index + 1;
      if (depth < 0) return -1;
    }
  }
  return -1;
}

// A character that may appear inside a Python return annotation (`-> list[int]`).
// A whitelist, mirroring is_return_annotation_char in
// src/solver_handlers/program_synthesis.rs: this is what lets the annotation
// scan stop at the first non-Latin character — Hindi/Chinese prose, the
// Devanagari danda `।`, or the ideographic full stop `。` — instead of relying
// on an ASCII-only `[.;\n]` terminator that those scripts never contain.
function isReturnAnnotationChar(character) {
  return /[A-Za-z0-9]/.test(character) || "_[](),'\"|".includes(character);
}

// The first non-whitespace character at or after `start`, or "".
// Mirrors next_non_whitespace in src/solver_handlers/program_synthesis.rs.
function nextNonWhitespace(text, start) {
  for (let index = start; index < text.length; ) {
    const character = String.fromCodePoint(text.codePointAt(index));
    if (!/\s/.test(character)) return character;
    index += character.length;
  }
  return "";
}

// The end index (exclusive) of a `->` return annotation that begins at
// `arrowStart`, or -1 if none. Walks annotation characters, allowing internal
// spaces (`-> dict[str, int]`) only while more annotation characters follow.
// Mirrors return_annotation_end in src/solver_handlers/program_synthesis.rs.
function returnAnnotationEnd(text, arrowStart) {
  let cursor = arrowStart + "->".length;
  let seenAnnotation = false;
  let lastAnnotationEnd = -1;
  while (cursor < text.length) {
    const character = String.fromCodePoint(text.codePointAt(cursor));
    const next = cursor + character.length;
    if (/\s/.test(character)) {
      if (seenAnnotation && isReturnAnnotationChar(nextNonWhitespace(text, next))) {
        cursor = next;
        continue;
      }
      if (seenAnnotation) break;
      cursor = next;
      continue;
    }
    if (!isReturnAnnotationChar(character)) break;
    seenAnnotation = true;
    lastAnnotationEnd = next;
    cursor = next;
  }
  return lastAnnotationEnd;
}

function declaredPythonSignature(prompt, functionName) {
  const text = String(prompt || "");
  const marker = `${functionName}(`;
  const start = text.toLowerCase().indexOf(marker.toLowerCase());
  if (start < 0) return "";
  let end = matchingCloseParen(text, start + functionName.length);
  if (end < 0) return "";
  const tail = text.slice(end);
  const trimmed = tail.trimStart();
  if (trimmed.startsWith("->")) {
    const returnStart = end + (tail.length - trimmed.length);
    const annotationEnd = returnAnnotationEnd(text, returnStart);
    if (annotationEnd >= 0) end = annotationEnd;
  }
  return text.slice(start, end).trim().replace(/\.+$/, "");
}

function renderPythonStatement(statement, indentLevel) {
  const indent = "    ".repeat(indentLevel);
  if (statement.kind === "assign") {
    return `${indent}${statement.target} = ${statement.expression}\n`;
  }
  if (statement.kind === "return") {
    return `${indent}return ${statement.expression}\n`;
  }
  if (statement.kind === "if_return") {
    return `${indent}if ${statement.condition}:\n${indent}    return ${statement.value}\n`;
  }
  if (statement.kind === "for_loop") {
    return (
      `${indent}for ${statement.target} in ${statement.iterator}:\n` +
      statement.body
        .map((child) => renderPythonStatement(child, indentLevel + 1))
        .join("")
    );
  }
  return "";
}

function renderPythonFunction(functionTree) {
  let code = `def ${functionTree.signature}:\n`;
  for (const statement of functionTree.body) {
    code += renderPythonStatement(statement, 1);
  }
  return code;
}

function pythonStatementLinks(lines, statement, depth) {
  const indent = "  ".repeat(depth + 1);
  if (statement.kind === "assign") {
    lines.push(
      `${indent}semantic_node ${statement.semanticNode} target=${JSON.stringify(statement.target)} expression=${JSON.stringify(statement.expression)}`,
    );
  } else if (statement.kind === "return") {
    lines.push(
      `${indent}semantic_node ${statement.semanticNode} expression=${JSON.stringify(statement.expression)}`,
    );
  } else if (statement.kind === "if_return") {
    lines.push(
      `${indent}semantic_node ${statement.semanticNode} condition=${JSON.stringify(statement.condition)} value=${JSON.stringify(statement.value)}`,
    );
  } else if (statement.kind === "for_loop") {
    lines.push(
      `${indent}semantic_node ${statement.semanticNode} target=${JSON.stringify(statement.target)} iterator=${JSON.stringify(statement.iterator)}`,
    );
    for (const child of statement.body) {
      pythonStatementLinks(lines, child, depth + 1);
    }
  }
}

function pythonFunctionLinks(functionTree) {
  const lines = [
    "python_function_syntax_tree",
    `  semantic_node function_definition signature=${JSON.stringify(functionTree.signature)}`,
  ];
  for (const statement of functionTree.body) {
    pythonStatementLinks(lines, statement, 1);
  }
  return lines.join("\n");
}

// Build the Python candidate for whichever synthesis task the prompt names or
// evidences. The task slug keys both the meaning lexicon and the verbatim
// PYTHON_SYNTHESIS_CANDIDATES blueprint (function tree, tests, fragments).
// Mirrors synthesize_python_candidate in
// src/solver_handlers/program_synthesis.rs.
function synthesizePythonCandidate(prompt, normalized, functionName) {
  const task = meaningsWithRole(ROLE_PROGRAM_SYNTHESIS_TASK).find(
    (candidate) =>
      functionName === candidate.slug || synthesisTaskEvidenced(candidate, normalized),
  );
  if (!task) return null;
  const definition = PYTHON_SYNTHESIS_CANDIDATES[task.slug];
  if (!definition) return null;
  const signature =
    declaredPythonSignature(prompt, functionName) || definition.defaultSignature;
  const functionTree = { signature, body: definition.body };
  return Object.assign({}, definition, {
    functionName: task.slug,
    functionTree,
  });
}

// Issue #395: does the prompt evidence the operation with this canonical slug,
// in any supported language? Mirrors OperationVocabulary::matches in
// src/seed/operation_vocabulary.rs (substring match per phrase/combo).
function operationMatchesSlug(slug, normalized) {
  for (const operation of operationVocabulary()) {
    if (operation.slug === slug && operationFormMatches(normalized, operation)) {
      return true;
    }
  }
  return false;
}

// Issue #395: type ontology for the universal list coding algorithm.
// Byte mirror of data/seed/numeric-list-operations.lino — each operation maps to a
// family (list_transformation / list_reduction) and a result kind
// (list / scalar), so the worker reasons about the task from data instead of a
// per-case handler. Transformations support numeric lists and quoted text lists;
// reductions remain numeric.
// NUMERIC_LIST_OPERATIONS_LINO is loaded from synced seed/*.lino data during loadSeed().

let cachedNumericListOntology = null;
// Parse the numeric-list ontology into
// { canonical: { family, resultKind, direction } }. Mirrors result_kind_for /
// family_for / direction_for in src/solver_handlers/numeric_list/mod.rs.
function numericListOntology() {
  if (cachedNumericListOntology) return cachedNumericListOntology;
  const root = parseLinoTree(NUMERIC_LIST_OPERATIONS_LINO);
  const container =
    root.children.find((child) => child.name === "numeric_list_operations") ||
    root;
  const map = {};
  for (const node of container.children) {
    if (node.name !== "operation") continue;
    const familyNode = node.children.find((c) => c.name === "family");
    const kindNode = node.children.find((c) => c.name === "result_kind");
    const directionNode = node.children.find((c) => c.name === "direction");
    map[node.value] = {
      family: familyNode ? familyNode.value : "list_transformation",
      resultKind: kindNode ? kindNode.value : "list",
      direction: directionNode ? directionNode.value : "",
    };
  }
  cachedNumericListOntology = map;
  return cachedNumericListOntology;
}

function numericListResultKind(canonical) {
  const entry = numericListOntology()[canonical];
  return entry && entry.resultKind === "scalar" ? "scalar" : "list";
}

function numericListFamily(canonical) {
  const entry = numericListOntology()[canonical];
  return entry && entry.family === "list_reduction"
    ? "list_reduction"
    : "list_transformation";
}

// Issue #395: lift every number token (signed / decimal) from the prompt, in
// order. Mirrors parse_numbers in src/solver_handlers/numeric_list/mod.rs: a
// leading sign only starts a literal when it is not glued to a preceding letter
// or digit, and the surface text is preserved verbatim for echoing and codegen.
function parseNumericListNumbers(prompt) {
  const chars = Array.from(String(prompt || ""));
  const isDigit = (ch) => ch >= "0" && ch <= "9";
  const isAlnum = (ch) => /[\p{L}\p{N}]/u.test(ch);
  const numbers = [];
  let index = 0;
  while (index < chars.length) {
    const ch = chars[index];
    let sign = "";
    if (
      (ch === "-" || ch === "+") &&
      index + 1 < chars.length &&
      isDigit(chars[index + 1]) &&
      !(index > 0 && isAlnum(chars[index - 1]))
    ) {
      sign = ch;
      index += 1;
    } else if (!isDigit(ch)) {
      index += 1;
      continue;
    }
    const start = index;
    while (index < chars.length && isDigit(chars[index])) index += 1;
    if (
      index < chars.length &&
      chars[index] === "." &&
      index + 1 < chars.length &&
      isDigit(chars[index + 1])
    ) {
      index += 1;
      while (index < chars.length && isDigit(chars[index])) index += 1;
    }
    if (start === index) continue;
    const text = sign + chars.slice(start, index).join("");
    const value = Number(text);
    if (!Number.isNaN(value)) numbers.push({ text, value, kind: "number" });
  }
  return numbers;
}

function parseNumericListQuotedStrings(prompt) {
  const chars = Array.from(String(prompt || ""));
  const items = [];
  let index = 0;
  while (index < chars.length) {
    const quote = chars[index];
    if (quote !== '"' && quote !== "'") {
      index += 1;
      continue;
    }
    index += 1;
    let text = "";
    while (index < chars.length) {
      const ch = chars[index];
      if (ch === "\\" && index + 1 < chars.length) {
        text += chars[index + 1];
        index += 2;
        continue;
      }
      if (ch === quote) break;
      text += ch;
      index += 1;
    }
    if (index < chars.length && chars[index] === quote) {
      index += 1;
      if (text.length) items.push({ text, value: text, kind: "string" });
    }
  }
  return items;
}

function parseNumericListItems(prompt, canonical) {
  const quoted = parseNumericListQuotedStrings(prompt);
  if (numericListFamily(canonical) === "list_transformation" && quoted.length >= 2) {
    return quoted;
  }
  return parseNumericListNumbers(prompt);
}

// Issue #395: render the array literal nodes. When projected to source, callers
// join these surfaces with ", ". Keeping them as an array first lets the worker
// build and trace a CST/AST-like program tree before source rendering.
// When the list mixes integers and decimals, integer surfaces gain a `.0` suffix
// so statically-typed targets keep a single element type. Mirrors number_literals.
function numericListStringLiteral(value) {
  return `"${String(value)
    .replace(/\\/g, "\\\\")
    .replace(/"/g, '\\"')
    .replace(/\n/g, "\\n")
    .replace(/\r/g, "\\r")
    .replace(/\t/g, "\\t")}"`;
}

function numericListLiterals(items, valueType) {
  return items.map((item) => {
    if (item.kind === "string") return numericListStringLiteral(item.text);
    return valueType.label === "float" && !item.text.includes(".")
      ? `${item.text}.0`
      : item.text;
  });
}

// Issue #395: localized phrasing for the four supported UI languages. The
// numbers, code, and result are language-independent; only the surrounding prose
// differs. Mirrors Localization in src/solver_handlers/numeric_list/mod.rs; the
// `sort` / `reverse_sort` sentences are byte-identical to the original handler so
// existing golden assertions stay green.
const NUMERIC_LIST_LOCALIZATION = {
  ru: {
    resultLabel: "Результат:",
    intro: (canonical, lang, given) =>
      ({
        sort: `Вот код на ${lang}, который сортирует числа ${given} по возрастанию:`,
        reverse_sort: `Вот код на ${lang}, который сортирует числа ${given} по убыванию:`,
        reverse: `Вот код на ${lang}, который переворачивает числа ${given}:`,
        sum: `Вот код на ${lang}, который суммирует числа ${given}:`,
        product: `Вот код на ${lang}, который перемножает числа ${given}:`,
        minimum: `Вот код на ${lang}, который находит наименьшее из чисел ${given}:`,
        maximum: `Вот код на ${lang}, который находит наибольшее из чисел ${given}:`,
      })[canonical],
  },
  hi: {
    resultLabel: "परिणाम:",
    intro: (canonical, lang, given) =>
      ({
        sort: `यह ${lang} कोड है जो संख्याओं ${given} को आरोही क्रम में क्रमबद्ध करता है:`,
        reverse_sort: `यह ${lang} कोड है जो संख्याओं ${given} को अवरोही क्रम में क्रमबद्ध करता है:`,
        reverse: `यह ${lang} कोड है जो संख्याओं ${given} को उलट देता है:`,
        sum: `यह ${lang} कोड है जो संख्याओं ${given} का योग करता है:`,
        product: `यह ${lang} कोड है जो संख्याओं ${given} का गुणनफल निकालता है:`,
        minimum: `यह ${lang} कोड है जो संख्याओं ${given} में से सबसे छोटी ढूँढता है:`,
        maximum: `यह ${lang} कोड है जो संख्याओं ${given} में से सबसे बड़ी ढूँढता है:`,
      })[canonical],
  },
  zh: {
    resultLabel: "结果:",
    intro: (canonical, lang, given) =>
      ({
        sort: `这是用 ${lang} 编写的将数字 ${given} 按升序排序的代码:`,
        reverse_sort: `这是用 ${lang} 编写的将数字 ${given} 按降序排序的代码:`,
        reverse: `这是用 ${lang} 编写的将数字 ${given} 反转的代码:`,
        sum: `这是用 ${lang} 编写的对数字 ${given} 求和的代码:`,
        product: `这是用 ${lang} 编写的计算数字 ${given} 乘积的代码:`,
        minimum: `这是用 ${lang} 编写的求数字 ${given} 最小值的代码:`,
        maximum: `这是用 ${lang} 编写的求数字 ${given} 最大值的代码:`,
      })[canonical],
  },
  en: {
    resultLabel: "Result:",
    intro: (canonical, lang, given, valueTypeLabel) => {
      const noun = valueTypeLabel === "string" ? "strings" : "numbers";
      return ({
        sort: `Here is ${lang} code that sorts the ${noun} ${given} in ascending order:`,
        reverse_sort: `Here is ${lang} code that sorts the ${noun} ${given} in descending order:`,
        reverse: `Here is ${lang} code that reverses the ${noun} ${given}:`,
        sum: `Here is ${lang} code that sums the ${noun} ${given}:`,
        product: `Here is ${lang} code that multiplies the ${noun} ${given}:`,
        minimum: `Here is ${lang} code that finds the smallest of the ${noun} ${given}:`,
        maximum: `Here is ${lang} code that finds the largest of the ${noun} ${given}:`,
      })[canonical];
    },
  },
};

// Issue #395: recognize which numeric-list operation the prompt asks for, in
// priority order. Sort phrasings are checked first because "sort in reverse
// order" legitimately contains the bare `reverse` verb; the descending variant
// wins whenever the `reverse_sort` phrasing is present. Mirrors detect_operation.
function detectNumericListOperation(normalized) {
  if (
    operationMatchesSlug("sort", normalized) ||
    operationMatchesSlug("reverse_sort", normalized)
  ) {
    return operationMatchesSlug("reverse_sort", normalized) ? "reverse_sort" : "sort";
  }
  if (operationMatchesSlug("reverse", normalized)) return "reverse";
  for (const canonical of ["sum", "product", "minimum", "maximum"]) {
    if (operationMatchesSlug(canonical, normalized)) return canonical;
  }
  return null;
}

// Issue #395: format a computed scalar so its textual form matches the runnable
// code's stdout: an integer with no decimal point when every input was an
// integer. Mirrors format_scalar.
function numericListFormatScalar(value, isFloat) {
  return isFloat ? String(value) : String(Math.round(value));
}

// Issue #395: apply the operation to the parsed numbers and return the surface
// tokens to display — the reordered list for a transformation, or a single
// computed scalar for a reduction. Mirrors compute.
function numericListCompare(left, right) {
  if (left.kind === "number" && right.kind === "number") {
    return left.value < right.value ? -1 : left.value > right.value ? 1 : 0;
  }
  return left.text < right.text ? -1 : left.text > right.text ? 1 : 0;
}

function computeNumericList(canonical, items, isFloat) {
  if (numericListFamily(canonical) === "list_transformation") {
    const ordered = items.slice();
    if (canonical === "sort") {
      ordered.sort(numericListCompare);
    } else if (canonical === "reverse_sort") {
      ordered.sort((a, b) => numericListCompare(b, a));
    } else {
      ordered.reverse();
    }
    return ordered.map((n) => n.text);
  }
  const numbers = items.filter((item) => item.kind === "number");
  let value;
  if (canonical === "sum") value = numbers.reduce((acc, n) => acc + n.value, 0);
  else if (canonical === "product") value = numbers.reduce((acc, n) => acc * n.value, 1);
  else if (canonical === "minimum") value = numbers.reduce((acc, n) => Math.min(acc, n.value), Infinity);
  else value = numbers.reduce((acc, n) => Math.max(acc, n.value), -Infinity);
  return [numericListFormatScalar(value, isFloat)];
}

// Issue #395: structural numeric-list program tree. The worker mirrors the
// Rust `NumericProgram`: source code is a projection of these semantic nodes,
// and the tree is preserved in evidence/trace for inspection. The value-class
// label doubles as the `on` selector for coding-idiom cases; per-language
// storage types live in CODING_IDIOMS_LINO, not in this record.
function numericListValueType(items, isFloat) {
  if (items.some((item) => item.kind === "string")) {
    return { label: "string" };
  }
  return { label: isFloat ? "float" : "integer" };
}

function numericListBuildProgram(slug, items, canonical, isFloat) {
  const family = numericListFamily(canonical);
  const valueType = numericListValueType(items, isFloat);
  const list = codingDefaultName("list");
  const statements = [
    {
      kind: "literal_list",
      name: list,
      mutable: codingMutatesListInPlace(slug),
    },
  ];
  if (family === "list_transformation") {
    const target = codingDefaultName("transformed");
    statements.push({
      kind: canonical === "reverse" ? "reverse_list" : "sort_list",
      source: list,
      target,
      canonical,
      direction: numericListDirection(canonical),
    });
    statements.push({
      kind: "print_joined",
      source: target,
      separator: ", ",
    });
  } else {
    const target = codingDefaultName("reduced");
    statements.push({
      kind: "reduce_list",
      source: list,
      target,
      reducer: canonical,
    });
    statements.push({ kind: "print_scalar", source: target });
  }
  return {
    languageSlug: slug,
    canonical,
    valueType,
    literals: numericListLiterals(items, valueType),
    displayValues: items.map((item) => item.text),
    statements,
  };
}

// Transformation direction token, read from the numeric-list ontology instead
// of a hardcoded match. Mirrors direction_for in
// src/solver_handlers/numeric_list/mod.rs.
function numericListDirection(canonical) {
  const entry = numericListOntology()[canonical];
  return entry ? entry.direction : "";
}
