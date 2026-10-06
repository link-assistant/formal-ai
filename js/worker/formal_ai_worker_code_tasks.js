// Browser twins of the issue #1177 code-task handlers that read pasted code:
// debugging, explanation, review, test generation and refactoring. Each
// `handle*` function mirrors the Rust `handle_*` function of the same name in
// `rust/src/solver_handlers/` (same cues, same seed tables, same templates);
// each `try*` function is the slug-named binding the browser dispatcher runs,
// and passes the lowercased prompt as `normalized` exactly as the native
// dispatcher does (`rust/src/meta_method_dispatch.rs`).
//
// Beyond Rust (issue #1177 R7): the promise-chain refactoring also RUNS both the
// original chain and its async/await rewrite in the worker against recording
// stubs, and reports whether the two call traces agree.

// ---------------------------------------------------------------------------
// code_debugging
// ---------------------------------------------------------------------------

/**
 * Offsets of the first top-level `/` and a top-level binary `+`/`-` after it.
 * A `+`/`-` is binary when the previous non-space character can end an
 * operand; comparisons and statement separators reset the scan.
 * @param {string} expr arithmetic expression
 * @returns {Array<number>|null} [divisionOffset, operatorOffset], or null
 */
function codeDebuggingQuotientShift(expr) {
  let depth = 0;
  let seenDivision = false;
  let prevEndsOperand = false;
  let divisionOffset = -1;
  let offset = 0;
  for (const ch of Array.from(expr)) {
    const at = offset;
    offset += ch.length;
    if (ch === "(" || ch === "[") {
      depth += 1;
      prevEndsOperand = false;
    } else if (ch === ")" || ch === "]") {
      depth = depth > 0 ? depth - 1 : 0;
      prevEndsOperand = true;
    } else if (ch === "/" && depth === 0) {
      if (divisionOffset === -1) divisionOffset = at;
      seenDivision = true;
      prevEndsOperand = false;
    } else if ((ch === "+" || ch === "-") && depth === 0 && seenDivision && prevEndsOperand) {
      return [divisionOffset, at];
    } else if ("=,;<>!&|?:".includes(ch) && depth === 0) {
      seenDivision = false;
      prevEndsOperand = false;
    } else if (/^\s$/u.test(ch)) {
      // Whitespace keeps the previous non-space character's verdict: the
      // documented contract is "the previous non-space character can end an
      // operand". The Rust scan currently resets here, so
      // `sum(xs) / len(xs) - 1` (its own unit-test prompt) is never flagged.
      continue;
    } else {
      prevEndsOperand = codeTaskIsAlphanumeric(ch) || ch === "_" || ch === "\"" || ch === "'" || ch === ".";
    }
  }
  return null;
}

/**
 * Scan `return` lines and assignment right-hand sides for a shifted quotient.
 * @param {string} code source text
 * @returns {object|null} {lineNumber, line, shift, fixed, parenthesized}, or null
 */
function codeDebuggingScanForDefect(code) {
  const lines = codeTaskLines(code);
  for (let index = 0; index < lines.length; index += 1) {
    const line = lines[index];
    const trimmed = line.trimStart();
    let expression = "";
    const assignAt = line.indexOf(" = ");
    const returnAt = trimmed.indexOf("return ");
    if (trimmed.startsWith("return ")) {
      expression = trimmed.slice(7).trim();
    } else if (assignAt !== -1) {
      expression = line.slice(assignAt + 3).trim();
    } else if (returnAt !== -1) {
      expression = trimmed.slice(returnAt + 7).trim();
    } else {
      continue;
    }
    if (!expression.includes("/")) continue;
    const shift = codeDebuggingQuotientShift(expression);
    if (shift === null) continue;
    const divAt = shift[0];
    const opAt = shift[1];
    const shiftText = expression.slice(opAt).trim();
    const fixed = expression.slice(0, opAt).trim();
    const divisor = expression.slice(divAt + 1, opAt).trim();
    return {
      lineNumber: index + 1,
      line: line.trim(),
      shift: shiftText,
      fixed: fixed,
      parenthesized: expression.slice(0, divAt).trim() + " / (" + divisor + " " + shiftText + ")",
    };
  }
  return null;
}

/**
 * Recognize a debugging request and locate a shifted-quotient defect.
 * Mirrors `handle_code_debugging` in rust/src/solver_handlers/code_debugging.rs.
 * @param {string} prompt raw prompt
 * @param {string} normalized normalized prompt
 * @returns {object|null} the worker answer, or null when not claimed
 */
function handleCodeDebugging(prompt, normalized) {
  if (!codeTaskCued("code_debugging", "", prompt, normalized)) return null;
  const code = codeTaskCodeBlock(prompt);
  if (code === null) return null;
  const log = codeTaskLog();
  codeTaskLogAppend(log, "code_debugging:request", "code extracted");
  const name = codeTaskFunctionName(code);
  if (name !== null) codeTaskLogAppend(log, "code_debugging:function_name", name);
  const intent = codeTaskIntentForName(name);
  if (intent !== null) codeTaskLogAppend(log, "code_debugging:intent_property", intent.property);
  const defect = codeDebuggingScanForDefect(code);
  let body = "";
  let confidence = 0.5;
  if (intent !== null && defect !== null) {
    codeTaskLogAppend(log, "code_debugging:defect", "line " + defect.lineNumber + ": `" + defect.shift + "`");
    body = codeTaskTemplate("code_debugging_defect", [
      ["name", name === null ? "" : name],
      ["property", intent.property],
      ["grounding", intent.grounding],
      ["line_no", String(defect.lineNumber)],
      ["line", defect.line],
      ["shift", defect.shift],
      ["fixed", defect.fixed],
      ["parenthesized", defect.parenthesized],
      ["correct_form", intent.correctForm],
    ]);
    confidence = 0.8;
  } else {
    codeTaskLogAppend(log, "code_debugging:defect", "no recognized defect pattern");
    const intentNote = intent === null
      ? codeTaskTemplate("code_debugging_no_intent", [["name", name === null ? "" : name]])
      : "";
    body = codeTaskTemplate("code_debugging_no_defect", [["intent_note", intentNote]]);
  }
  return codeTaskAnswer(log, "code_debugging", "response:code_debugging", body, confidence);
}

/**
 * Browser binding for the `code_debugging` precedence row.
 * @param {string} prompt raw prompt
 * @returns {object|null} the worker answer, or null
 */
function tryCodeDebugging(prompt) {
  return handleCodeDebugging(prompt, prompt.toLowerCase());
}

// ---------------------------------------------------------------------------
// code_explanation
// ---------------------------------------------------------------------------

/**
 * Built-in calls and the construct each one names (Rust `BUILTINS`).
 * @returns {Array<Array<string>>} [marker, construct] pairs
 */
function codeExplanationBuiltins() {
  return [["sum(", "builtin_sum"], ["len(", "builtin_len"], ["max(", "builtin_max"], ["min(", "builtin_min"]];
}

/**
 * The construct table: construct slug → explanation sentence.
 * @returns {Array<object>} {construct, meaning} records
 */
function codeExplanationStructures() {
  const out = [];
  for (const record of codeTaskSeedRecords("meanings-code-structure-explanations.lino")) {
    if (record.name !== "structure") continue;
    for (const body of codeTaskChildren(record, "construct")) {
      out.push({ construct: String(body.id || ""), meaning: codeTaskChildValue(body, "explanation") });
    }
  }
  return out;
}

/**
 * Build one explain-line result.
 * @param {string} construct construct slug
 * @param {Array<Array<string>>} values placeholder values
 * @param {boolean} isLoop whether the construct iterates over the input
 * @returns {object} {construct, values, isLoop}
 */
function codeExplanationMatch(construct, values, isLoop) {
  return { construct: construct, values: values, isLoop: isLoop };
}

/**
 * Match one code line against the construct table (Rust `explain_line`).
 * @param {string} line code line
 * @returns {object|null} the match, or null
 */
function codeExplanationExplainLine(line) {
  const trimmed = line.trim();
  if (trimmed === "") return null;
  const thenAt = trimmed.indexOf(".then(");
  if (thenAt !== -1) {
    return codeExplanationMatch("promise_then", [
      ["value", trimmed.slice(0, thenAt).trim()],
      ["handler", codeTaskInsideParens(trimmed, thenAt + 5)],
    ], false);
  }
  const arrowAt = trimmed.indexOf("=>");
  if (arrowAt !== -1) {
    const args = codeTaskTrimChar(codeTaskTrimChar(trimmed.slice(0, arrowAt).trim(), "("), ")");
    return codeExplanationMatch("arrow_function", [["args", args], ["expr", trimmed.slice(arrowAt + 2).trim()]], false);
  }
  if (trimmed.startsWith("const ")) {
    const rest = trimmed.slice(6);
    const eq = rest.indexOf(" = ");
    if (eq !== -1) {
      return codeExplanationMatch("const_declaration", [
        ["name", rest.slice(0, eq).trim()],
        ["value", rest.slice(eq + 3).trim()],
      ], false);
    }
  }
  for (const keyword of codeTaskDefKeywords()) {
    if (!trimmed.startsWith(keyword)) continue;
    const rest = trimmed.slice(keyword.length);
    const name = codeTaskIdentifierAt(rest);
    if (name === "") continue;
    const open = rest.indexOf("(");
    const args = open !== -1 ? codeTaskInsideParens(rest, open) : "";
    return codeExplanationMatch("function_definition", [["name", name], ["args", args]], false);
  }
  if (trimmed.startsWith("[") && trimmed.indexOf(" for ") !== -1) {
    const forAt = trimmed.indexOf(" for ");
    const expr = trimmed.slice(1, forAt).trim();
    const afterFor = trimmed.slice(forAt + 5);
    const inAt = afterFor.indexOf(" in ");
    if (inAt === -1) return null;
    return codeExplanationMatch("comprehension", [
      ["expr", expr],
      ["item", afterFor.slice(0, inAt).trim()],
      ["items", codeTaskTrimEndChar(afterFor.slice(inAt + 4).trim(), "]")],
    ], true);
  }
  if (trimmed.startsWith("for ")) {
    const rest = trimmed.slice(4);
    const inAt = rest.indexOf(" in ");
    if (inAt !== -1) {
      return codeExplanationMatch("for_loop", [
        ["item", rest.slice(0, inAt).trim()],
        ["items", codeTaskTrimEndChar(rest.slice(inAt + 4).trim(), ":")],
      ], true);
    }
  }
  if (trimmed.startsWith("if ")) {
    return codeExplanationMatch("if_statement", [["condition", codeTaskTrimEndChar(trimmed.slice(3).trim(), ":")]], false);
  }
  for (const builtin of codeExplanationBuiltins()) {
    const open = trimmed.indexOf(builtin[0]);
    if (open !== -1) {
      return codeExplanationMatch(builtin[1], [["items", codeTaskInsideParens(trimmed, open + builtin[0].length - 1)]], false);
    }
  }
  if (trimmed.startsWith("return ")) {
    return codeExplanationMatch("return_statement", [["value", trimmed.slice(7).trim()]], false);
  }
  if (trimmed.indexOf(" == ") !== -1 || trimmed.indexOf(" != ") !== -1) {
    return codeExplanationMatch("comparison", [], false);
  }
  const assignAt = trimmed.indexOf(" = ");
  if (assignAt !== -1) {
    return codeExplanationMatch("assignment", [
      ["name", trimmed.slice(0, assignAt).trim()],
      ["value", trimmed.slice(assignAt + 3).trim()],
    ], false);
  }
  if (trimmed.indexOf("/") !== -1) return codeExplanationMatch("division", [], false);
  const bracket = trimmed.indexOf("[");
  if (bracket !== -1) {
    const before = trimmed.slice(0, bracket).trimEnd();
    const beforeChars = Array.from(before);
    const last = beforeChars.length > 0 ? beforeChars[beforeChars.length - 1] : "";
    if (last !== "" && (codeTaskIsAlphanumeric(last) || last === "_")) {
      const close = trimmed.indexOf("]", bracket);
      if (close !== -1) {
        return codeExplanationMatch("indexing", [["items", before], ["index", trimmed.slice(bracket + 1, close)]], false);
      }
    }
  }
  const dot = trimmed.indexOf(".");
  if (dot !== -1) {
    const open = trimmed.indexOf("(", dot);
    if (open !== -1) {
      const value = trimmed.slice(0, dot).trim();
      const method = codeTaskIdentifierAt(trimmed.slice(dot + 1));
      const args = codeTaskInsideParens(trimmed, open);
      if (method !== "" && value !== "") {
        return codeExplanationMatch("method_call", [["value", value], ["method", method], ["args", args]], false);
      }
    }
  }
  return null;
}

/**
 * Explain code line by line against the construct table.
 * Mirrors `handle_code_explanation` in rust/src/solver_handlers/code_explanation.rs.
 * @param {string} prompt raw prompt
 * @param {string} normalized normalized prompt
 * @returns {object|null} the worker answer, or null
 */
function handleCodeExplanation(prompt, normalized) {
  if (!codeTaskCued("code_explanation", "", prompt, normalized)) return null;
  const code = codeTaskCodeBlock(prompt);
  if (code === null) return null;
  const table = codeExplanationStructures();
  const log = codeTaskLog();
  const lines = codeTaskLines(code);
  codeTaskLogAppend(log, "code_explanation:request", "line=" + lines.length);
  let linesBody = "";
  let matched = 0;
  let loopSeen = false;
  for (const line of lines) {
    const match = codeExplanationExplainLine(line);
    if (match === null) continue;
    matched += 1;
    loopSeen = loopSeen || match.isLoop;
    codeTaskLogAppend(log, "code_explanation:construct", match.construct);
    let structure = null;
    for (const candidate of table) {
      if (candidate.construct === match.construct) {
        structure = candidate;
        break;
      }
    }
    if (structure === null) continue;
    let meaning = structure.meaning;
    for (const pair of match.values) {
      meaning = meaning.split("{" + pair[0] + "}").join(pair[1]);
    }
    linesBody += codeTaskTemplate("code_explanation_line", [["line", line.trim()], ["meaning", meaning]]);
  }
  let body = "";
  let confidence = 0.4;
  if (matched === 0) {
    codeTaskLogAppend(log, "code_explanation:no_construct", "table=0");
    body = codeTaskTemplate("code_explanation_no_construct", []);
  } else {
    const name = codeTaskFunctionName(code);
    const intent = codeTaskIntentForName(name);
    const overall = intent !== null
      ? codeTaskTemplate("code_explanation_overall", [
        ["name", name === null ? "" : name],
        ["property", intent.property],
        ["correct_form", intent.correctForm],
        ["grounding", intent.grounding],
      ])
      : codeTaskTemplate("code_explanation_no_overall", []);
    const cost = loopSeen
      ? codeTaskTemplate("code_explanation_cost_linear", [])
      : codeTaskTemplate("code_explanation_cost_constant", []);
    body = codeTaskTemplate("code_explanation_lines", [["lines", linesBody], ["overall", overall], ["cost", cost]]);
    confidence = 0.7;
  }
  return codeTaskAnswer(log, "code_explanation", "response:code_explanation", body, confidence);
}

/**
 * Browser binding for the `code_explanation` precedence row.
 * @param {string} prompt raw prompt
 * @returns {object|null} the worker answer, or null
 */
function tryCodeExplanation(prompt) {
  return handleCodeExplanation(prompt, prompt.toLowerCase());
}

// ---------------------------------------------------------------------------
// code_review
// ---------------------------------------------------------------------------

/**
 * Markers that make the code read as JavaScript rather than Python.
 * @returns {Array<string>} markers (Rust `JS_MARKERS`)
 */
function codeReviewJavaScriptMarkers() {
  return ["=>", "const ", ".then(", "let ", "=== "];
}

/**
 * The review rule table from `data/seed/code-review-rules.lino`.
 * @returns {Array<object>} rule records
 */
function codeReviewRules() {
  const out = [];
  for (const record of codeTaskSeedRecords("code-review-rules.lino")) {
    if (record.name !== "rule") continue;
    const body = record.children.length > 0 && record.children[0].name === "id" ? record.children[0] : null;
    if (body === null) continue;
    const detects = [];
    for (const child of codeTaskChildren(body, "detect")) {
      if (child.id) detects.push(String(child.id));
    }
    const avoids = [];
    for (const child of codeTaskChildren(body, "avoid")) {
      if (child.id) avoids.push(String(child.id));
    }
    out.push({
      id: codeTaskChildValue(record, "id"),
      language: codeTaskChildValue(body, "language"),
      scope: codeTaskChildValue(body, "scope"),
      severity: codeTaskChildValue(body, "severity"),
      detects: detects,
      avoids: avoids,
      title: codeTaskChildValue(body, "title"),
      advice: codeTaskChildValue(body, "advice"),
      source: codeTaskChildValue(body, "source"),
    });
  }
  return out;
}

/**
 * The leading-indent width of a code line (spaces; tabs count as one).
 * @param {string} line code line
 * @returns {number} indent width
 */
function codeReviewIndentOf(line) {
  let count = 0;
  while (count < line.length && (line[count] === " " || line[count] === "\t")) count += 1;
  return count;
}

/**
 * The line indices inside loop bodies.
 * @param {Array<string>} lines code lines
 * @returns {Array<number>} indices
 */
function codeReviewLoopBodyLines(lines) {
  const out = [];
  let index = 0;
  while (index < lines.length) {
    const trimmed = lines[index].trimStart();
    if (trimmed.startsWith("for ") || trimmed.startsWith("while ")) {
      const headerIndent = codeReviewIndentOf(lines[index]);
      let body = index + 1;
      while (body < lines.length) {
        const line = lines[body];
        if (line.trim() !== "" && codeReviewIndentOf(line) <= headerIndent) break;
        out.push(body);
        body += 1;
      }
      index = body;
    } else {
      index += 1;
    }
  }
  return out;
}

/**
 * Apply every rule whose language matches and whose scope contains the line.
 * @param {string} code source text
 * @param {string} language "python" or "javascript"
 * @param {Array<object>} table review rules
 * @returns {Array<object>} findings {rule, lineNumber, line}
 */
function codeReviewFindings(code, language, table) {
  const lines = codeTaskLines(code);
  const loopBodies = codeReviewLoopBodyLines(lines);
  const out = [];
  for (const rule of table) {
    if (rule.language !== language) continue;
    for (let index = 0; index < lines.length; index += 1) {
      const line = lines[index];
      const trimmed = line.trim();
      if (trimmed === "") continue;
      let inScope = true;
      switch (rule.scope) {
        case "def_line": {
          const start = line.trimStart();
          inScope = codeTaskDefKeywords().some(function (keyword) { return start.startsWith(keyword); });
          break;
        }
        case "loop_body":
          inScope = loopBodies.includes(index);
          break;
        default:
          inScope = true;
      }
      if (!inScope) continue;
      const detected = rule.detects.some(function (detect) { return trimmed.includes(detect); });
      const suppressed = rule.avoids.some(function (avoid) { return trimmed.includes(avoid); });
      if (detected && !suppressed) out.push({ rule: rule, lineNumber: index + 1, line: trimmed });
    }
  }
  return out;
}

/**
 * Review code against the seed rule table.
 * Mirrors `handle_code_review` in rust/src/solver_handlers/code_review.rs.
 * @param {string} prompt raw prompt
 * @param {string} normalized normalized prompt
 * @returns {object|null} the worker answer, or null
 */
function handleCodeReview(prompt, normalized) {
  if (!codeTaskCued("code_review", "", prompt, normalized)) return null;
  const code = codeTaskCodeBlock(prompt);
  if (code === null) return null;
  const language = codeReviewJavaScriptMarkers().some(function (marker) { return code.includes(marker); })
    ? "javascript"
    : "python";
  const log = codeTaskLog();
  codeTaskLogAppend(log, "code_review:request", "lang=" + language);
  const findings = codeReviewFindings(code, language, codeReviewRules());
  codeTaskLogAppend(log, "code_review:findings", "n=" + findings.length);
  let body = "";
  let confidence = 0.4;
  if (findings.length === 0) {
    body = codeTaskTemplate("code_review_no_match", []);
  } else {
    let rendered = "";
    for (const finding of findings) {
      codeTaskLogAppend(log, "code_review:finding", finding.rule.id);
      rendered += codeTaskTemplate("code_review_finding", [
        ["severity", finding.rule.severity],
        ["title", finding.rule.title],
        ["line_no", String(finding.lineNumber)],
        ["line", finding.line],
        ["advice", finding.rule.advice],
        ["source", finding.rule.source],
      ]);
    }
    body = codeTaskTemplate("code_review_findings", [["findings", rendered]]);
    confidence = 0.7;
  }
  return codeTaskAnswer(log, "code_review", "response:code_review", body, confidence);
}

/**
 * Browser binding for the `code_review` precedence row.
 * @param {string} prompt raw prompt
 * @returns {object|null} the worker answer, or null
 */
function tryCodeReview(prompt) {
  return handleCodeReview(prompt, prompt.toLowerCase());
}

// ---------------------------------------------------------------------------
// test_generation
// ---------------------------------------------------------------------------

/**
 * The problem shapes from the seed `test_cases` records.
 * @returns {Array<object>} {name, triggers, cases} records
 */
function testGenerationShapes() {
  const out = [];
  for (const record of codeTaskSeedRecords("code-task-cues.lino")) {
    if (record.name !== "test_cases") continue;
    const name = codeTaskChildValue(record, "shape");
    if (name === "") continue;
    const body = record.children.length > 0 && record.children[0].name === "shape" ? record.children[0] : null;
    if (body === null) continue;
    const triggers = [];
    const cases = [];
    for (const child of body.children) {
      switch (child.name) {
        case "trigger":
          if (child.id) triggers.push(String(child.id));
          break;
        case "case": {
          const outcome = codeTaskChildValue(child, "outcome");
          const output = codeTaskChildValue(child, "output");
          cases.push({
            input: codeTaskChildValue(child, "input"),
            outcome: outcome === "" ? null : outcome,
            output: output === "" ? null : output,
          });
          break;
        }
        default:
          break;
      }
    }
    out.push({ name: name, triggers: triggers, cases: cases });
  }
  return out;
}

/**
 * The function under test: a backtick span, else the word after "for".
 * @param {string} prompt raw prompt
 * @returns {string|null} the function name, or null
 */
function testGenerationFunctionName(prompt) {
  const tick = prompt.indexOf("`");
  if (tick !== -1) {
    const close = prompt.indexOf("`", tick + 1);
    if (close !== -1) {
      const name = codeTaskIdentifierAt(prompt.slice(tick + 1, close));
      if (name !== "") return name;
    }
  }
  const words = codeTaskWords(prompt);
  for (let index = 0; index < words.length; index += 1) {
    if ((words[index] === "for" || words[index] === "для") && index + 1 < words.length) {
      const name = codeTaskIdentifierAt(words[index + 1]);
      if (name !== "") return name;
    }
  }
  return null;
}

/**
 * The Python literal for a sample input.
 * @param {string} input sample input
 * @returns {string} Python source literal
 */
function testGenerationPythonLiteral(input) {
  if (input.startsWith("[")) return input;
  return "\"" + input.split("\\").join("\\\\").split("\"").join("\\\"") + "\"";
}

/**
 * The test-function identifier for one sample input.
 * @param {string} shape shape name
 * @param {string} input sample input
 * @returns {string} identifier suffix
 */
function testGenerationCaseId(shape, input) {
  let cleaned = "";
  for (const ch of Array.from(input.toLowerCase())) {
    if (codeTaskIsAlphanumeric(ch)) cleaned += ch;
  }
  return shape + "_" + (cleaned === "" ? "empty" : cleaned);
}

/**
 * The normalize helper lines and call prefix for the requested properties.
 * @param {Array<string>} properties property slugs
 * @returns {Array} [helper lines, call prefix]
 */
function testGenerationNormalizeHelper(properties) {
  const lower = properties.includes("case");
  const filters = properties.includes("spaces") || properties.includes("punctuation");
  if (lower && filters) {
    return [["def normalize(value):", "    return \"\".join(ch for ch in value.lower() if ch.isalnum())"], "normalize"];
  }
  if (filters) {
    return [["def normalize(value):", "    return \"\".join(ch for ch in value if ch.isalnum())"], "normalize"];
  }
  if (lower) return [["def normalize(value):", "    return value.lower()"], "normalize"];
  return [[], ""];
}

/**
 * Generate a pytest suite for the function under test.
 * Mirrors `handle_test_generation` in rust/src/solver_handlers/test_generation.rs.
 * @param {string} prompt raw prompt
 * @param {string} normalized normalized prompt
 * @returns {object|null} the worker answer, or null
 */
function handleTestGeneration(prompt, normalized) {
  const lower = prompt.toLowerCase(); // an agent opt-in is the agent flow's (Rust `is_agent_opt_in`)
  if (isAgentTextRequest(lower) || !codeTaskCued("test_generation", "request", prompt, normalized)) return null;
  const log = codeTaskLog();
  codeTaskLogAppend(log, "test_generation:request", "cued");
  const fn = testGenerationFunctionName(prompt);
  if (fn === null) {
    codeTaskLogAppend(log, "test_generation:refusal", "function=none");
    return codeTaskAnswer(log, "test_generation", "response:test_generation",
      codeTaskTemplate("test_generation_refusal", []), 0.4);
  }
  codeTaskLogAppend(log, "test_generation:function", fn);
  const properties = [];
  const propertyWords = [];
  for (const entry of codeTaskWordEntries("test_property")) {
    const word = codeTaskChildValue(entry, "word");
    if (normalized.includes(word) || lower.includes(word)) {
      properties.push(codeTaskChildValue(entry, "value"));
      propertyWords.push(word);
    }
  }
  let shape = null;
  for (const candidate of testGenerationShapes()) {
    if (candidate.triggers.some(function (trigger) { return fn.includes(trigger); })) {
      shape = candidate;
      break;
    }
  }
  let normalizeApplied = false;
  let lines = [];
  let rows = [];
  if (shape !== null) {
    codeTaskLogAppend(log, "test_generation:shape", shape.name);
    const helper = testGenerationNormalizeHelper(properties);
    let callPrefix = helper[1];
    if (!shape.cases.some(function (sample) { return !sample.input.startsWith("["); })) callPrefix = "";
    normalizeApplied = callPrefix !== "";
    if (normalizeApplied) {
      lines = lines.concat(helper[0]);
      lines.push("");
    }
    for (const sample of shape.cases) {
      lines.push("def test_" + testGenerationCaseId(shape.name, sample.input) + "():");
      const argument = callPrefix === "" || sample.input.startsWith("[")
        ? testGenerationPythonLiteral(sample.input)
        : callPrefix + "(" + testGenerationPythonLiteral(sample.input) + ")";
      const call = fn + "(" + argument + ")";
      let assertion = "    assert " + call;
      if (sample.outcome !== null) {
        assertion = "    assert " + call + " is " + (sample.outcome === "true" ? "True" : "False");
      } else if (sample.output !== null) {
        assertion = "    assert " + call + " == " + sample.output;
      }
      lines.push(assertion);
      rows.push([sample.input, assertion]);
    }
  } else {
    codeTaskLogAppend(log, "test_generation:shape", "none");
    lines = ["def test_smoke():", "    assert " + fn + " is not None"];
    rows = [];
  }
  if (normalizeApplied) {
    for (const word of propertyWords) rows.push([word, "normalize()"]);
  }
  lines.push("");
  codeTaskLogAppend(log, "test_generation:suite", "case=" + rows.length);
  const body = codeTaskTemplate("test_generation_suite", [
    ["suite", lines.join("\n")],
    ["derivation", codeTaskMappingRows(rows)],
  ]);
  return codeTaskAnswer(log, "test_generation", "response:test_generation", body, 0.7);
}

/**
 * Browser binding for the `test_generation` precedence row.
 * @param {string} prompt raw prompt
 * @returns {object|null} the worker answer, or null
 */
function tryTestGeneration(prompt) {
  return handleTestGeneration(prompt, prompt.toLowerCase());
}

// ---------------------------------------------------------------------------
// code_refactoring
// ---------------------------------------------------------------------------

/**
 * Parse one handler expression: `x => expr`, `(x) => expr`, or
 * `function (x) { body }`.
 * @param {string} handler handler source
 * @returns {object|null} {param, body}, or null for any other shape
 */
function codeRefactoringParseHandler(handler) {
  const trimmed = handler.trim();
  const arrowAt = trimmed.indexOf("=>");
  if (arrowAt !== -1) {
    const param = codeTaskTrimChar(codeTaskTrimChar(trimmed.slice(0, arrowAt).trim(), "("), ")").trim();
    const body = trimmed.slice(arrowAt + 2).trim();
    if (param === "" || body === "") return null;
    return { param: param, body: body };
  }
  if (!trimmed.startsWith("function")) return null;
  const rest = trimmed.slice(8);
  const open = rest.indexOf("(");
  if (open === -1) return null;
  const span = codeTaskParenSpan(rest, open);
  const brace = rest.indexOf("{", span[1]);
  if (brace === -1) return null;
  let depth = 0;
  for (let index = brace; index < rest.length; index += 1) {
    if (rest[index] === "{") {
      depth += 1;
    } else if (rest[index] === "}") {
      depth -= 1;
      if (depth === 0) {
        const body = rest.slice(brace + 1, index).trim();
        if (span[0].trim() === "" || body === "") return null;
        return { param: span[0].trim(), body: body };
      }
    }
  }
  return null;
}

/**
 * Parse the promise chain out of whitespace-flattened code.
 * @param {string} flat code with whitespace runs collapsed to one space
 * @returns {object|null} {head, thens, catchHandler}, or null
 */
function codeRefactoringParseChain(flat) {
  const firstThen = flat.indexOf(".then(");
  if (firstThen === -1) return null;
  const head = codeTaskTrimEndChar(flat.slice(0, firstThen).trim(), ";").trim();
  if (head === "") return null;
  const thens = [];
  let catchHandler = null;
  let cursor = firstThen;
  for (;;) {
    const thenAt = flat.indexOf(".then(", cursor);
    const catchAt = flat.indexOf(".catch(", cursor);
    let at = -1;
    let isThen = true;
    if (thenAt !== -1 && catchAt !== -1 && thenAt < catchAt) {
      at = thenAt;
    } else if (catchAt !== -1) {
      at = catchAt;
      isThen = false;
    } else if (thenAt !== -1) {
      at = thenAt;
    } else {
      break;
    }
    const span = codeTaskParenSpan(flat, at + (isThen ? 6 : 7) - 1);
    const handler = codeRefactoringParseHandler(span[0]);
    if (handler === null) return null;
    if (isThen) {
      thens.push(handler);
    } else {
      catchHandler = handler;
    }
    cursor = span[1];
  }
  if (thens.length === 0) return null;
  return { head: head, thens: thens, catchHandler: catchHandler };
}

/**
 * Render the async/await rewrite; with `executable` the awaits are desugared to
 * `__await(…)` calls so the worker can run the twin synchronously.
 * @param {object} chain parsed chain
 * @param {boolean} executable whether to render the runnable twin
 * @returns {string} the rewritten source
 */
function codeRefactoringRenderAsync(chain, executable) {
  const lines = [executable ? "function run() {" : "async function run() {"];
  const hasCatch = chain.catchHandler !== null;
  if (hasCatch) lines.push("  try {");
  const indent = hasCatch ? "    " : "  ";
  let previous = chain.head;
  for (const handler of chain.thens) {
    const awaited = executable ? "__await(" + previous + ")" : "await " + previous;
    lines.push(indent + "const " + handler.param + " = " + awaited + ";");
    previous = handler.body;
  }
  lines.push(indent + (executable ? "__await(" + previous + ")" : "await " + previous) + ";");
  if (hasCatch) {
    lines.push("  } catch (" + chain.catchHandler.param + ") {");
    lines.push("    " + chain.catchHandler.body);
    lines.push("  }");
  }
  lines.push("}");
  return lines.join("\n");
}

/**
 * Run one program against recording stubs: every free identifier is a stub
 * whose calls are traced; promises are modelled synchronously (a stub is a
 * fulfilled thenable of itself); call number `rejectCall` (1-based, 0 = none)
 * returns a rejected thenable instead.
 * @param {string} source program text
 * @param {number} rejectCall which call rejects
 * @returns {Array<string>} the call trace
 */
function codeRefactoringTraceRun(source, rejectCall) {
  const trace = [];
  const state = { calls: 0 };
  const describe = function (value) {
    if (value !== null && (typeof value === "object" || typeof value === "function")) {
      if (value.__stubLabel) return value.__stubLabel;
      if (value.__plainWrapper) return describe(value.__value);
      if (value instanceof Error) return "Error(" + value.message + ")";
      return typeof value === "function" ? "<fn>" : "<object>";
    }
    return JSON.stringify(value) === undefined ? String(value) : JSON.stringify(value);
  };
  const isThenable = function (value) {
    return value !== null && (typeof value === "object" || typeof value === "function") &&
      (Boolean(value.__stubLabel) || Boolean(value.__rejected) || Boolean(value.__plainWrapper));
  };
  let makeStub = null;
  let makeRejected = null;
  const settle = function (value) {
    if (isThenable(value)) return value;
    const wrapper = {
      __plainWrapper: true,
      __value: value,
      then: function (onFulfilled) {
        if (typeof onFulfilled !== "function") return wrapper;
        try {
          return settle(onFulfilled(value));
        } catch (error) {
          return makeRejected(error);
        }
      },
      catch: function () { return wrapper; },
    };
    return wrapper;
  };
  makeRejected = function (error) {
    const rejected = {
      __rejected: true,
      __error: error,
      then: function (onFulfilled, onRejected) {
        if (typeof onRejected !== "function") return rejected;
        try {
          return settle(onRejected(error));
        } catch (inner) {
          return makeRejected(inner);
        }
      },
      catch: function (onRejected) {
        if (typeof onRejected !== "function") return rejected;
        try {
          return settle(onRejected(error));
        } catch (inner) {
          return makeRejected(inner);
        }
      },
    };
    return rejected;
  };
  makeStub = function (label) {
    const call = function (args, prefix) {
      const parts = [];
      for (const arg of args) parts.push(describe(arg));
      const callLabel = prefix + label + "(" + parts.join(", ") + ")";
      state.calls += 1;
      trace.push(callLabel);
      if (state.calls === rejectCall) return makeRejected(makeStub("error<" + callLabel + ">"));
      return makeStub(callLabel);
    };
    const target = function () {};
    let proxy = null;
    proxy = new Proxy(target, {
      get: function (unused, prop) {
        if (typeof prop === "symbol") {
          if (prop === Symbol.toPrimitive) return function () { return label; };
          return undefined;
        }
        switch (prop) {
          case "__stubLabel":
            return label;
          case "then":
            return function (onFulfilled) {
              if (typeof onFulfilled !== "function") return proxy;
              try {
                return settle(onFulfilled(proxy));
              } catch (error) {
                return makeRejected(error);
              }
            };
          case "catch":
            return function () { return proxy; };
          default:
            return makeStub(label + "." + prop);
        }
      },
      apply: function (unused, thisArg, args) { return call(args, ""); },
      construct: function (unused, args) { return call(args, "new "); },
    });
    return proxy;
  };
  const awaitValue = function (value) {
    if (value !== null && typeof value === "object" && value.__rejected) throw value.__error;
    if (value !== null && typeof value === "object" && value.__plainWrapper) return value.__value;
    return value;
  };
  const environment = new Proxy({}, {
    has: function (unused, prop) {
      return typeof prop === "string" && prop !== "__await";
    },
    get: function (unused, prop) {
      if (typeof prop === "symbol") return undefined;
      return makeStub(prop);
    },
  });
  try {
    const runner = new Function("__env", "__await", "with (__env) {\n" + source + "\n}");
    runner(environment, awaitValue);
  } catch (error) {
    trace.push("threw " + describe(error && error.__stubLabel ? error : error));
  }
  return trace;
}

/**
 * Run the original chain and its executable twin in every scenario (all calls
 * succeed, then each call in turn rejects) and compare the call traces.
 * @param {string} flat whitespace-flattened original code
 * @param {object} chain parsed chain
 * @returns {object} {status, scenarios, calls, detail}
 */
function codeRefactoringExecutionCheck(flat, chain) {
  const bodies = [chain.head];
  for (const handler of chain.thens) bodies.push(handler.body);
  if (chain.catchHandler !== null) bodies.push(chain.catchHandler.body);
  for (const body of bodies) {
    if (/\b(while|for|do)\b/.test(body)) {
      return { status: "skipped", scenarios: 0, calls: 0, detail: "a handler body contains a loop" };
    }
  }
  if (typeof Proxy !== "function") {
    return { status: "skipped", scenarios: 0, calls: 0, detail: "no Proxy in this runtime" };
  }
  const twin = codeRefactoringRenderAsync(chain, true) + "\ntry { run(); } catch (__unhandled) {}";
  const baseline = codeRefactoringTraceRun(flat, 0);
  const calls = baseline.length;
  const limit = Math.min(calls, 8);
  let scenarios = 0;
  for (let reject = 0; reject <= limit; reject += 1) {
    const original = codeRefactoringTraceRun(flat, reject);
    const rewritten = codeRefactoringTraceRun(twin, reject);
    scenarios += 1;
    if (original.join("\n") !== rewritten.join("\n")) {
      return {
        status: "mismatch",
        scenarios: scenarios,
        calls: calls,
        detail: (reject === 0 ? "all calls succeed" : "call " + reject + " rejects") +
          ": original [" + original.join("; ") + "] vs rewrite [" + rewritten.join("; ") + "]",
      };
    }
  }
  return { status: "matched", scenarios: scenarios, calls: calls, detail: baseline.join("; ") };
}

/**
 * Rewrite a promise chain with async/await, then run both versions.
 * Mirrors `handle_code_refactoring` in rust/src/solver_handlers/code_refactoring.rs.
 * @param {string} prompt raw prompt
 * @param {string} normalized normalized prompt
 * @returns {object|null} the worker answer, or null
 */
function handleCodeRefactoring(prompt, normalized) {
  if (!codeTaskCued("code_refactoring", "", prompt, normalized)) return null;
  const code = codeTaskCodeBlock(prompt);
  const log = codeTaskLog();
  codeTaskLogAppend(log, "code_refactoring:request", "cued");
  const flat = code === null ? "" : codeTaskWords(code).join(" ");
  const chain = code === null ? null : codeRefactoringParseChain(flat);
  if (chain === null) {
    codeTaskLogAppend(log, "code_refactoring:refusal", "chain=none");
    return codeTaskAnswer(log, "code_refactoring", "response:code_refactoring",
      codeTaskTemplate("code_refactoring_refusal", []), 0.4);
  }
  codeTaskLogAppend(log, "code_refactoring:chain", "then=" + chain.thens.length);
  let body = codeTaskTemplate("code_refactoring_async", [["code", codeRefactoringRenderAsync(chain, false)]]);
  const check = codeRefactoringExecutionCheck(flat, chain);
  codeTaskLogAppend(log, "code_refactoring:execution", check.status);
  switch (check.status) {
    case "matched":
      body += "\n\n" + codeTaskTemplate("code_refactoring_execution_matched", [
        ["scenarios", String(check.scenarios)],
        ["calls", String(check.calls)],
        ["trace", check.detail],
      ]);
      break;
    case "mismatch":
      body += "\n\n" + codeTaskTemplate("code_refactoring_execution_mismatch", [["detail", check.detail]]);
      break;
    default:
      body += "\n\n" + codeTaskTemplate("code_refactoring_execution_skipped", [["detail", check.detail]]);
  }
  return codeTaskAnswer(log, "code_refactoring", "response:code_refactoring", body,
    check.status === "mismatch" ? 0.4 : 0.7);
}

/**
 * Browser binding for the `code_refactoring` precedence row.
 * @param {string} prompt raw prompt
 * @returns {object|null} the worker answer, or null
 */
function tryCodeRefactoring(prompt) {
  return handleCodeRefactoring(prompt, prompt.toLowerCase());
}
