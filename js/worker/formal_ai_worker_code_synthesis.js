// Browser twins of the issue #1177 composers that build an artifact from a
// natural-language description: regular expressions and single-SELECT SQL.
// `handleRegexSynthesis` / `handleSqlSynthesis` mirror the Rust handlers in
// rust/src/solver_handlers/{regex,sql}_synthesis.rs; the `try*` bindings pass
// the lowercased prompt as `normalized`, as the native dispatcher does.
//
// Beyond Rust (issue #1177 R4): the browser has a regex engine, so the composed
// pattern is also compiled with `RegExp` and run against positive and negative
// example sets derived from the same constraints; the result is appended to the
// Rust template as an evidence line.

// ---------------------------------------------------------------------------
// regex_synthesis
// ---------------------------------------------------------------------------

/**
 * Scan the lowercased prompt for class counts and separators.
 * @param {string} lower lowercased prompt
 * @returns {object} {classes, separators}
 */
function regexSynthesisScanMentions(lower) {
  const numbers = codeTaskWordEntries("number");
  const separatorsByWord = codeTaskWordEntries("separator");
  const classesByWord = codeTaskWordEntries("class");
  const optionalMarkers = codeTaskMapWords("optional_marker");
  const words = [];
  let position = 0;
  for (const word of codeTaskWords(lower)) {
    words.push([position, codeTaskTrimMatches(word, function (c) { return !codeTaskIsAlphanumeric(c); })]);
    position += word.length + 1;
  }
  const classes = [];
  const separators = [];
  let pendingOptional = false;
  for (let index = 0; index < words.length; index += 1) {
    const at = words[index][0];
    const word = words[index][1];
    if (optionalMarkers.includes(word)) {
      pendingOptional = true;
      continue;
    }
    const separator = codeTaskEntryFor(separatorsByWord, word);
    if (separator !== null) {
      separators.push({ position: at, literal: codeTaskChildValue(separator, "value") });
      continue;
    }
    const count = codeTaskNumberValue(word, numbers);
    if (count === null) continue;
    let atLeast = false;
    let cursor = index + 1;
    while (cursor < words.length && (words[cursor][1] === "or" || words[cursor][1] === "more")) {
      atLeast = true;
      cursor += 1;
    }
    if (cursor >= words.length) continue;
    const entry = codeTaskEntryFor(classesByWord, words[cursor][1]);
    if (entry === null) continue;
    if (index > 0 && words[index - 1][1] === "least") atLeast = true;
    const optional = pendingOptional;
    pendingOptional = false;
    classes.push({
      position: at,
      cls: codeTaskChildValue(entry, "value"),
      classLabel: count + " " + codeTaskChildValue(entry, "label"),
      count: count,
      atLeast: atLeast,
      optional: optional,
    });
  }
  return { classes: classes, separators: separators };
}

/**
 * Render one class mention as `class{n}` or `class{n,}`.
 * @param {object} mention class mention
 * @returns {string} regex fragment
 */
function regexSynthesisRenderClass(mention) {
  return mention.cls + "{" + mention.count + (mention.atLeast ? ",}" : "}");
}

/**
 * The separator literal between two positions, or null.
 * @param {Array<object>} separators separator mentions
 * @param {number|null} from previous class position (null = none)
 * @param {number} to current class position
 * @returns {string|null} the separator literal
 */
function regexSynthesisSeparatorBetween(separators, from, to) {
  const lower = from === null ? Number.MAX_SAFE_INTEGER : from;
  for (const separator of separators) {
    if (separator.position > lower && separator.position < to) return separator.literal;
  }
  return null;
}

/**
 * The position of the last class mention before `mention`, or null.
 * @param {Array<object>} classes class mentions
 * @param {object} mention current mention
 * @returns {number|null} position
 */
function regexSynthesisPreviousPosition(classes, mention) {
  let previous = null;
  for (const other of classes) {
    if (other.position < mention.position) previous = other.position;
  }
  return previous;
}

/**
 * Compose the unanchored pattern body from the mentions.
 * @param {Array<object>} classes class mentions
 * @param {Array<object>} separators separator mentions
 * @returns {string|null} the pattern body, or null when nothing composes
 */
function regexSynthesisCompose(classes, separators) {
  if (classes.length === 0) return null;
  let pattern = regexSynthesisRenderClass(classes[0]);
  for (let index = 1; index < classes.length; index += 1) {
    const mention = classes[index];
    const separator = regexSynthesisSeparatorBetween(separators, regexSynthesisPreviousPosition(classes, mention), mention.position);
    const body = (separator === null ? "" : separator) + regexSynthesisRenderClass(mention);
    pattern += mention.optional ? "(" + body + ")?" : body;
  }
  return pattern;
}

/**
 * Structural verification problem codes (empty = verified).
 * @param {string} pattern composed pattern
 * @returns {Array<string>} problem codes
 */
function regexSynthesisVerifyStructurally(pattern) {
  const problems = [];
  let groupDepth = 0;
  let classDepth = 0;
  const chars = Array.from(pattern);
  let i = 0;
  while (i < chars.length) {
    switch (chars[i]) {
      case "\\":
        if (i + 1 >= chars.length) problems.push("dangling_escape");
        i += 2;
        continue;
      case "(":
        groupDepth += 1;
        break;
      case ")":
        groupDepth = groupDepth > 0 ? groupDepth - 1 : 0;
        break;
      case "[":
        classDepth += 1;
        break;
      case "]":
        classDepth = classDepth > 0 ? classDepth - 1 : 0;
        break;
      case "{": {
        let j = i + 1;
        while (j < chars.length && /^[0-9]$/.test(chars[j])) j += 1;
        if (j === i + 1) problems.push("bad_repetition");
        if (j < chars.length && chars[j] === ",") j += 1;
        if (j >= chars.length || chars[j] !== "}") problems.push("unclosed_repetition");
        i = j;
        break;
      }
      default:
        break;
    }
    i += 1;
  }
  if (groupDepth !== 0) problems.push("unbalanced_parens");
  if (classDepth !== 0) problems.push("unbalanced_class");
  return problems;
}

/**
 * The first probe character a class pattern accepts (wantMatch) or rejects.
 * @param {string} cls class pattern such as `\d`
 * @param {boolean} wantMatch whether the character must match
 * @returns {string|null} the character, or null
 */
function regexSynthesisProbeCharacter(cls, wantMatch) {
  const probe = new RegExp("^" + cls + "$");
  for (const ch of ["7", "a", "Q", "_", "-", " ", "\n"]) {
    if (probe.test(ch) === wantMatch) return ch;
  }
  return null;
}

/**
 * The literal text a separator pattern stands for (`\.` → `.`).
 * @param {string} literal separator pattern
 * @returns {string} the literal text
 */
function regexSynthesisSeparatorText(literal) {
  return literal.startsWith("\\") ? literal.slice(1) : literal;
}

/**
 * Derive positive and negative example strings from the class mentions.
 * @param {Array<object>} classes class mentions
 * @param {Array<object>} separators separator mentions
 * @param {boolean} anchored whether the pattern is anchored
 * @returns {object|null} {positives, negatives}, or null when underivable
 */
function regexSynthesisExamples(classes, separators, anchored) {
  const parts = [];
  for (let index = 0; index < classes.length; index += 1) {
    const mention = classes[index];
    const good = regexSynthesisProbeCharacter(mention.cls, true);
    const bad = regexSynthesisProbeCharacter(mention.cls, false);
    if (good === null || bad === null) return null;
    const separator = index === 0
      ? null
      : regexSynthesisSeparatorBetween(separators, regexSynthesisPreviousPosition(classes, mention), mention.position);
    parts.push({
      mention: mention,
      good: good,
      bad: bad,
      prefix: separator === null ? "" : regexSynthesisSeparatorText(separator),
    });
  }
  const render = function (overrides) {
    let text = "";
    for (let index = 0; index < parts.length; index += 1) {
      const part = parts[index];
      const override = overrides[index];
      if (override === "omit") continue;
      if (typeof override === "string") {
        text += part.prefix + override;
        continue;
      }
      text += part.prefix + part.good.repeat(part.mention.count);
    }
    return text;
  };
  const positives = [render({})];
  const withoutOptional = {};
  let anyOptional = false;
  for (let index = 1; index < parts.length; index += 1) {
    if (parts[index].mention.optional) {
      withoutOptional[index] = "omit";
      anyOptional = true;
    }
  }
  if (anyOptional) positives.push(render(withoutOptional));
  for (let index = 0; index < parts.length; index += 1) {
    if (parts[index].mention.atLeast) {
      const longer = {};
      longer[index] = parts[index].good.repeat(parts[index].mention.count + 2);
      positives.push(render(longer));
    }
  }
  const negatives = [];
  const limit = anchored ? parts.length : 1;
  for (let index = 0; index < limit; index += 1) {
    const part = parts[index];
    if (part.mention.count === 0) continue;
    const shorter = {};
    shorter[index] = part.good.repeat(part.mention.count - 1);
    negatives.push(render(shorter));
    const wrong = {};
    wrong[index] = part.bad + part.good.repeat(part.mention.count - 1);
    negatives.push(render(wrong));
  }
  if (anchored) {
    const last = parts[parts.length - 1];
    if (!last.mention.atLeast) negatives.push(render({}) + last.good);
  }
  const unique = function (list) {
    const out = [];
    for (const item of list) {
      if (!out.includes(item)) out.push(item);
    }
    return out;
  };
  const positiveSet = unique(positives);
  const negativeSet = [];
  for (const item of unique(negatives)) {
    if (!positiveSet.includes(item)) negativeSet.push(item);
  }
  return { positives: positiveSet, negatives: negativeSet };
}

/**
 * Compile the pattern with RegExp and run it on the derived examples.
 * @param {string} pattern the anchored or unanchored pattern
 * @param {object} examples {positives, negatives}
 * @returns {object} {status, failures}
 */
function regexSynthesisExecute(pattern, examples) {
  let compiled = null;
  try {
    compiled = new RegExp(pattern, "u");
  } catch (error) {
    return { status: "uncompilable", failures: [String(error && error.message)] };
  }
  const failures = [];
  for (const sample of examples.positives) {
    if (!compiled.test(sample)) failures.push("expected a match: " + JSON.stringify(sample));
  }
  for (const sample of examples.negatives) {
    if (compiled.test(sample)) failures.push("expected no match: " + JSON.stringify(sample));
  }
  return { status: failures.length === 0 ? "verified" : "failed", failures: failures };
}

/**
 * Quote a list of examples for the evidence line.
 * @param {Array<string>} samples example strings
 * @returns {string} the quoted, comma-separated list
 */
function regexSynthesisQuoteList(samples) {
  const quoted = [];
  for (const sample of samples) quoted.push(JSON.stringify(sample));
  return quoted.join(", ");
}

/**
 * Compose a regex from the stated constraints, then run it on examples.
 * Mirrors `handle_regex_synthesis` in rust/src/solver_handlers/regex_synthesis.rs.
 * @param {string} prompt raw prompt
 * @param {string} normalized normalized prompt
 * @returns {object|null} the worker answer, or null
 */
function handleRegexSynthesis(prompt, normalized) {
  if (!codeTaskCued("regex_synthesis", "", prompt, normalized)) return null;
  const lower = prompt.toLowerCase();
  const scan = regexSynthesisScanMentions(lower);
  const classes = scan.classes;
  const separators = scan.separators;
  const log = codeTaskLog();
  codeTaskLogAppend(log, "regex_synthesis:request", "class=" + classes.length);
  let anchored = true;
  for (const word of codeTaskMapWords("anchor_exception")) {
    if (normalized.includes(word)) anchored = false;
  }
  const composed = regexSynthesisCompose(classes, separators);
  if (composed === null) {
    codeTaskLogAppend(log, "regex_synthesis:refusal", "no composable constraints found");
    return codeTaskAnswer(log, "regex_synthesis", "response:regex_synthesis",
      codeTaskTemplate("regex_synthesis_refusal", []), 0.4);
  }
  const problems = regexSynthesisVerifyStructurally(composed);
  if (problems.length > 0) {
    const texts = [];
    for (const code of problems) texts.push(codeTaskTemplate("regex_problem_" + code, []));
    codeTaskLogAppend(log, "regex_synthesis:refusal", "problems: " + texts.join("; "));
    return codeTaskAnswer(log, "regex_synthesis", "response:regex_synthesis",
      codeTaskTemplate("regex_synthesis_broken", [["broken", composed], ["problems", texts.join("; ")]]), 0.4);
  }
  const pattern = anchored ? "^" + composed + "$" : composed;
  codeTaskLogAppend(log, "regex_synthesis:pattern", pattern);
  let mapping = "";
  for (let index = 0; index < classes.length; index += 1) {
    const mention = classes[index];
    let role = codeTaskTemplate("regex_role_required", []);
    if (index === 0) {
      role = codeTaskTemplate("regex_role_main", []);
    } else if (mention.optional) {
      role = codeTaskTemplate("regex_role_optional", []);
    }
    let separator = "";
    if (index > 0) {
      const literal = regexSynthesisSeparatorBetween(separators, classes[index - 1].position, mention.position);
      if (literal !== null) separator = codeTaskTemplate("regex_note_separator", [["literal", literal]]);
    }
    mapping += codeTaskTemplate("regex_mapping_line", [
      ["label", mention.classLabel],
      ["render", regexSynthesisRenderClass(mention)],
      ["role", role],
      ["separator", separator],
    ]);
  }
  const anchorNote = anchored
    ? codeTaskTemplate("regex_anchor_note_anchored", [])
    : codeTaskTemplate("regex_anchor_note_unanchored", []);
  let body = codeTaskTemplate("regex_synthesis_pattern", [
    ["pattern", pattern],
    ["mapping", mapping],
    ["anchor_note", anchorNote],
  ]);
  let confidence = 0.7;
  const examples = regexSynthesisExamples(classes, separators, anchored);
  if (examples !== null) {
    const run = regexSynthesisExecute(pattern, examples);
    codeTaskLogAppend(log, "regex_synthesis:execution", run.status);
    if (run.status === "verified") {
      body += "\n" + codeTaskTemplate("regex_synthesis_execution_verified", [
        ["positive_count", String(examples.positives.length)],
        ["positives", regexSynthesisQuoteList(examples.positives)],
        ["negative_count", String(examples.negatives.length)],
        ["negatives", regexSynthesisQuoteList(examples.negatives)],
      ]);
      confidence = 0.8;
    } else {
      body += "\n" + codeTaskTemplate("regex_synthesis_execution_failed", [["failures", run.failures.join("; ")]]);
      confidence = 0.4;
    }
  }
  return codeTaskAnswer(log, "regex_synthesis", "response:regex_synthesis", body, confidence);
}

/**
 * Browser binding for the `regex_synthesis` precedence row.
 * @param {string} prompt raw prompt
 * @returns {object|null} the worker answer, or null
 */
function tryRegexSynthesis(prompt) {
  return handleRegexSynthesis(prompt, prompt.toLowerCase());
}

// ---------------------------------------------------------------------------
// sql_synthesis
// ---------------------------------------------------------------------------

/**
 * Extract the table name from the token stream.
 * @param {Array<string>} tokens request tokens
 * @returns {string|null} table name
 */
function sqlSynthesisTableName(tokens) {
  for (let index = 0; index < tokens.length; index += 1) {
    switch (tokens[index]) {
      case "from":
      case "из": {
        let candidate = null;
        if (index + 1 < tokens.length && tokens[index + 1] !== "the" && tokens[index + 1] !== "таблицы") {
          candidate = tokens[index + 1];
        } else if (index + 2 < tokens.length) {
          candidate = tokens[index + 2];
        }
        if (candidate === null) return null;
        const name = codeTaskIdentifier(candidate);
        if (name !== "") return name;
        break;
      }
      case "select":
      case "selects":
      case "выбери":
      case "выберет": {
        const next = index + 1 < tokens.length ? tokens[index + 1] : null;
        if (next === "all" || next === "все" || next === "всех" || next === "every") {
          if (index + 2 >= tokens.length) return null;
          const name = codeTaskIdentifier(tokens[index + 2]);
          if (name !== "") return name;
        }
        break;
      }
      case "table":
      case "таблицы":
      case "таблицу": {
        if (index > 0) {
          const name = codeTaskIdentifier(tokens[index - 1]);
          if (name !== "" && name !== "the") return name;
        }
        if (index + 1 < tokens.length) {
          const name = codeTaskIdentifier(tokens[index + 1]);
          if (name !== "") return name;
        }
        break;
      }
      default:
        break;
    }
  }
  return null;
}

/**
 * Extract WHERE filters: seed comparatives, generic comparisons, equality.
 * @param {Array<string>} tokens request tokens
 * @returns {Array<object>} {clause, request} filters
 */
function sqlSynthesisFilters(tokens) {
  const numbers = codeTaskWordEntries("number");
  const comparatives = codeTaskWordEntries("sql_comparative");
  const at = function (index) { return index >= 0 && index < tokens.length ? tokens[index] : null; };
  const numberAt = function (index) {
    const word = at(index);
    return word === null ? null : codeTaskNumberValue(word, numbers);
  };
  const out = [];
  for (let index = 0; index < tokens.length; index += 1) {
    const token = tokens[index];
    const comparative = codeTaskEntryFor(comparatives, token);
    if (comparative !== null) {
      let value = numberAt(index + 2);
      if (value === null) value = numberAt(index + 1);
      if (value !== null) {
        out.push({
          clause: codeTaskChildValue(comparative, "column") + " " + codeTaskChildValue(comparative, "operator") + " " + value,
          request: codeTaskEcho(tokens, index, index + 2),
        });
      }
      continue;
    }
    const value = numberAt(index + 2);
    if (value !== null) {
      let column = null;
      let operator = "";
      let matched = true;
      if (token === "greater" || token === "more" || token === "больше") {
        column = at(index - 1);
        operator = ">";
      } else if (token === "less" || token === "fewer" || token === "меньше") {
        column = at(index - 1);
        operator = "<";
      } else if (token === "least" && index > 1 && tokens[index - 1] === "at") {
        column = at(index - 2);
        operator = ">=";
      } else if (token === "most" && index > 1 && tokens[index - 1] === "at") {
        column = at(index - 2);
        operator = "<=";
      } else {
        matched = false;
      }
      if (!matched) continue;
      if (column !== null) {
        const name = codeTaskIdentifier(column);
        if (name !== "") {
          out.push({ clause: name + " " + operator + " " + value, request: codeTaskEcho(tokens, index - 1, index + 2) });
        }
      }
    }
    if ((token === "named" || token === "имени") && index + 1 < tokens.length) {
      out.push({ clause: "name = '" + tokens[index + 1] + "'", request: codeTaskEcho(tokens, index, index + 1) });
    }
  }
  return out;
}

/**
 * The aggregate in the request, if any.
 * @param {Array<string>} tokens request tokens
 * @returns {Array<string>|null} [SQL expression, request echo]
 */
function sqlSynthesisAggregate(tokens) {
  for (let index = 0; index < tokens.length; index += 1) {
    let fn = "";
    switch (tokens[index]) {
      case "many":
      case "count":
      case "сколько":
        return ["COUNT(*)", codeTaskEcho(tokens, index, index)];
      case "average":
      case "mean":
      case "среднее":
        fn = "AVG";
        break;
      case "sum":
      case "total":
      case "сумма":
        fn = "SUM";
        break;
      case "maximum":
      case "highest":
      case "max":
        fn = "MAX";
        break;
      case "minimum":
      case "lowest":
      case "min":
        fn = "MIN";
        break;
      default:
        continue;
    }
    const column = index + 1 < tokens.length ? codeTaskIdentifier(tokens[index + 1]) : "";
    if (column === "") return null;
    return [fn + "(" + column + ")", codeTaskEcho(tokens, index, index + 1)];
  }
  return null;
}

/**
 * ORDER BY from "sorted by X" / "alphabetical order".
 * @param {Array<string>} tokens request tokens
 * @returns {Array<string>|null} [clause, request echo]
 */
function sqlSynthesisOrderClause(tokens) {
  for (let index = 0; index < tokens.length; index += 1) {
    const token = tokens[index];
    if ((token === "sorted" || token === "ordered" || token === "по") && tokens[index + 1] === "by") {
      const column = index + 2 < tokens.length ? codeTaskIdentifier(tokens[index + 2]) : "";
      if (column !== "") {
        const direction = tokens.includes("descending") || tokens.includes("reverse") ? " DESC" : "";
        return [" ORDER BY " + column + direction, codeTaskEcho(tokens, index, index + 2)];
      }
    }
    if (token === "alphabetical" || token === "алфавитном") {
      return [" ORDER BY name", codeTaskEcho(tokens, index, index)];
    }
  }
  return null;
}

/**
 * GROUP BY from the seed `sql_grouping` cues; mirrors `group_clause`.
 * @param {Array<string>} tokens request tokens
 * @returns {Array<string>|null} [column, request echo]
 */
function sqlSynthesisGroupClause(tokens) {
  const cues = codeTaskWordEntries("sql_grouping");
  for (let index = 0; index < tokens.length; index += 1) {
    const entry = cues.find((candidate) => codeTaskChildValue(candidate, "word") === tokens[index]);
    if (entry === undefined) continue;
    const at = index + 1 + (Number.parseInt(codeTaskChildValue(entry, "skip"), 10) || 0);
    const column = at < tokens.length ? codeTaskIdentifier(tokens[at]) : "";
    if (column !== "") return [column, codeTaskEcho(tokens, index, at)];
  }
  return null;
}

/**
 * LIMIT from "top N" / "first N" / "limit N".
 * @param {Array<string>} tokens request tokens
 * @param {Array<object>} numbers `number` word-map entries
 * @returns {Array<string>|null} [clause, request echo]
 */
function sqlSynthesisLimitClause(tokens, numbers) {
  for (let index = 0; index < tokens.length; index += 1) {
    if (["top", "first", "limit", "первые", "топ"].includes(tokens[index]) && index + 1 < tokens.length) {
      const value = codeTaskNumberValue(tokens[index + 1], numbers);
      if (value !== null) return [" LIMIT " + value, codeTaskEcho(tokens, index, index + 1)];
    }
  }
  return null;
}

/**
 * The SELECT column list and its request echo.
 * @param {Array<string>} tokens request tokens
 * @returns {Array<string>} [list, request echo]
 */
function sqlSynthesisSelectColumns(tokens) {
  let start = -1;
  for (let index = 0; index < tokens.length; index += 1) {
    if (["select", "selects", "выбери", "выберет"].includes(tokens[index])) {
      start = index;
      break;
    }
  }
  if (start !== -1) {
    let end = -1;
    for (let index = start + 1; index < tokens.length; index += 1) {
      if (tokens[index] === "from" || tokens[index] === "из") {
        end = index;
        break;
      }
    }
    if (end !== -1) {
      const columns = [];
      for (const token of tokens.slice(start + 1, end)) {
        if (["all", "the", "and", "все", "всех"].includes(token)) continue;
        const name = codeTaskIdentifier(token);
        if (name !== "") columns.push(name);
      }
      if (columns.length > 0) return [columns.join(", "), codeTaskEcho(tokens, start, end - 1)];
    }
  }
  for (let index = 0; index < tokens.length; index += 1) {
    if (["all", "все", "всех"].includes(tokens[index])) return ["*", codeTaskEcho(tokens, index, index)];
  }
  return ["*", ""];
}

/**
 * True when the prompt is a SQL request (cued, no file context, no code).
 * @param {string} prompt raw prompt
 * @param {string} normalized normalized prompt
 * @returns {boolean} whether this handler claims the prompt
 */
function sqlSynthesisIsRequest(prompt, normalized) {
  if (!codeTaskCued("sql_synthesis", "", prompt, normalized)) return false;
  for (const marker of ["def ", "function ", "fn ", "=>", "```"]) {
    if (prompt.includes(marker)) return false;
  }
  const fileContext = codeTaskMapWords("file_context");
  for (const token of codeTaskTokens(normalized)) {
    if (fileContext.includes(token)) return false;
  }
  return true;
}

/**
 * Compose a single SELECT from the stated constraints.
 * Mirrors `handle_sql_synthesis` in rust/src/solver_handlers/sql_synthesis.rs.
 * @param {string} prompt raw prompt
 * @param {string} normalized normalized prompt
 * @returns {object|null} the worker answer, or null
 */
function handleSqlSynthesis(prompt, normalized) {
  if (!sqlSynthesisIsRequest(prompt, normalized)) return null;
  const tokens = codeTaskTokens(normalized);
  const log = codeTaskLog();
  codeTaskLogAppend(log, "sql_synthesis:request", tokens.length + " token(s)");
  const table = sqlSynthesisTableName(tokens);
  if (table === null) {
    codeTaskLogAppend(log, "sql_synthesis:refusal", "no table identified");
    return codeTaskAnswer(log, "sql_synthesis", "response:sql_synthesis",
      codeTaskTemplate("sql_synthesis_refusal", []), 0.4);
  }
  codeTaskLogAppend(log, "sql_synthesis:table", table);
  const numbers = codeTaskWordEntries("number");
  const aggregate = sqlSynthesisAggregate(tokens);
  const selected = aggregate === null ? sqlSynthesisSelectColumns(tokens) : aggregate;
  const group = aggregate === null ? null : sqlSynthesisGroupClause(tokens);
  const columns = group === null ? selected[0] : group[0] + ", " + selected[0];
  const groupBy = group === null ? "" : " GROUP BY " + group[0];
  const filters = sqlSynthesisFilters(tokens);
  const clauses = [];
  for (const filter of filters) clauses.push(filter.clause);
  const whereClause = clauses.length === 0 ? "" : " WHERE " + clauses.join(" AND ");
  const order = sqlSynthesisOrderClause(tokens);
  const limit = sqlSynthesisLimitClause(tokens, numbers);
  const orderText = order === null ? "" : order[0];
  const limitText = limit === null ? "" : limit[0];
  const statement = "SELECT " + columns + " FROM " + table + whereClause + groupBy + orderText + limitText + ";";
  codeTaskLogAppend(log, "sql_synthesis:statement", statement);
  const rows = [[selected[1], "SELECT " + columns], [table, "FROM " + table]];
  for (const filter of filters) rows.push([filter.request, filter.clause]);
  if (group !== null) rows.push([group[1], groupBy.trim()]);
  if (orderText !== "") rows.push([order[1], orderText.trim()]);
  if (limitText !== "") rows.push([limit[1], limitText.trim()]);
  const body = codeTaskTemplate("sql_synthesis_statement", [
    ["statement", statement],
    ["mapping", codeTaskMappingRows(rows)],
  ]);
  return codeTaskAnswer(log, "sql_synthesis", "response:sql_synthesis", body, 0.7);
}

/**
 * Browser binding for the `sql_synthesis` precedence row.
 * @param {string} prompt raw prompt
 * @returns {object|null} the worker answer, or null
 */
function trySqlSynthesis(prompt) {
  return handleSqlSynthesis(prompt, prompt.toLowerCase());
}
