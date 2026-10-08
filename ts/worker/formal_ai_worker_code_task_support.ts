// Shared support for the browser twins of the issue #1177 code-task handlers.
//
// JavaScript is the priority implementation (2026-10-06 directive): every
// code-task handler the Rust core runs has a browser twin built on these
// helpers, and the Rust side is later regenerated from this JavaScript through
// link-foundation/meta-language's portable subset. The helpers therefore stay
// inside that subset: plain top-level functions, `const`/`let`, no classes or
// generators, errors as `throw new Error('literal')`.
//
// Every vocabulary list and every sentence comes from the same seed records the
// Rust handlers read (`data/seed/code-task-cues.lino`,
// `data/seed/meanings-code-structure-explanations.lino`,
// `data/seed/multilingual-responses-code-tasks.lino`, …); nothing here
// hardcodes a cue Rust reads from a seed.

const CODE_TASK_SEED_CACHE = { raw: null, records: {} };

/**
 * The top-level records of one seed file, parsed once per loaded seed set.
 * @param {string} fileName base name such as "code-task-cues.lino"
 * @returns {Array<object>} the parsed top-level nodes (empty when absent)
 */
function codeTaskSeedRecords(fileName) {
  if (CODE_TASK_SEED_CACHE.raw !== SEED_RAW) {
    CODE_TASK_SEED_CACHE.raw = SEED_RAW;
    CODE_TASK_SEED_CACHE.records = {};
  }
  if (Object.prototype.hasOwnProperty.call(CODE_TASK_SEED_CACHE.records, fileName)) {
    return CODE_TASK_SEED_CACHE.records[fileName];
  }
  const text = seedRawText(SEED_RAW, fileName);
  let records = [];
  if (text && typeof self.FormalAiSeed === "object" && self.FormalAiSeed !== null) {
    const tree = self.FormalAiSeed.parse(text);
    records = tree && tree.name === "" ? tree.children : (tree ? [tree] : []);
  }
  CODE_TASK_SEED_CACHE.records[fileName] = records;
  return records;
}

/**
 * The children of a parsed node with the given name.
 * @param {object} node parsed Links Notation node
 * @param {string} name child name
 * @returns {Array<object>} matching children in order
 */
function codeTaskChildren(node, name) {
  const out = [];
  if (!node || !Array.isArray(node.children)) return out;
  for (const child of node.children) {
    if (child.name === name) out.push(child);
  }
  return out;
}

/**
 * The id of the first child with the given name, or "" (Rust `find_child_value`).
 * @param {object} node parsed Links Notation node
 * @param {string} name child name
 * @returns {string} the child's value
 */
function codeTaskChildValue(node, name) {
  const found = codeTaskChildren(node, name);
  return found.length > 0 ? String(found[0].id || "") : "";
}

/**
 * Trigger phrases for one intent (optionally one named role) from
 * `data/seed/code-task-cues.lino`.
 * @param {string} intent handler intent slug
 * @param {string} role role name, or "" for the role-less record
 * @returns {Array<string>} phrases in seed order
 */
function codeTaskCuePhrases(intent, role) {
  const out = [];
  for (const record of codeTaskSeedRecords("code-task-cues.lino")) {
    if (record.name !== "cues") continue;
    if (codeTaskChildValue(record, "intent") !== intent) continue;
    const intentNodes = codeTaskChildren(record, "intent");
    const recordRole = intentNodes.length > 0 ? codeTaskChildValue(intentNodes[0], "role") : "";
    if (recordRole !== role) continue;
    for (const child of record.children) {
      for (const phrase of child.children) {
        if (phrase.name === "phrase" && phrase.id) out.push(String(phrase.id));
      }
    }
  }
  return out;
}

/**
 * True when any cue phrase of the intent (any role) occurs in the normalized
 * prompt or the lowercased raw prompt.
 * @param {string} intent handler intent slug
 * @param {string} prompt raw prompt
 * @param {string} normalized normalized prompt
 * @returns {boolean} whether a cue matched
 */
function codeTaskAnyCueMatches(intent, prompt, normalized) {
  const lower = prompt.toLowerCase();
  for (const record of codeTaskSeedRecords("code-task-cues.lino")) {
    if (record.name !== "cues") continue;
    if (codeTaskChildValue(record, "intent") !== intent) continue;
    for (const child of record.children) {
      for (const phrase of child.children) {
        if (phrase.name !== "phrase") continue;
        const text = String(phrase.id || "");
        if (normalized.includes(text) || lower.includes(text)) return true;
      }
    }
  }
  return false;
}

/**
 * True when a role's cue phrase occurs in the normalized or lowercased prompt.
 * @param {string} intent handler intent slug
 * @param {string} role role name, or ""
 * @param {string} prompt raw prompt
 * @param {string} normalized normalized prompt
 * @returns {boolean} whether a cue matched
 */
function codeTaskCued(intent, role, prompt, normalized) {
  const lower = prompt.toLowerCase();
  for (const phrase of codeTaskCuePhrases(intent, role)) {
    if (normalized.includes(phrase) || lower.includes(phrase)) return true;
  }
  return false;
}

/**
 * The `entry` records of one word map from `data/seed/code-task-cues.lino`.
 * @param {string} map word-map name
 * @returns {Array<object>} entry nodes in seed order
 */
function codeTaskWordEntries(map) {
  const out = [];
  for (const record of codeTaskSeedRecords("code-task-cues.lino")) {
    if (record.name !== "map") continue;
    if (codeTaskChildValue(record, "name") !== map) continue;
    for (const child of record.children) {
      for (const entry of child.children) {
        if (entry.name === "entry") out.push(entry);
      }
    }
  }
  return out;
}

/**
 * The `word` values of one word map.
 * @param {string} map word-map name
 * @returns {Array<string>} words in seed order
 */
function codeTaskMapWords(map) {
  const out = [];
  for (const entry of codeTaskWordEntries(map)) out.push(codeTaskChildValue(entry, "word"));
  return out;
}

/**
 * The first entry of a word map whose `word` equals the given word.
 * @param {Array<object>} entries word-map entries
 * @param {string} word the word to look up
 * @returns {object|null} the entry, or null
 */
function codeTaskEntryFor(entries, word) {
  for (const entry of entries) {
    if (codeTaskChildValue(entry, "word") === word) return entry;
  }
  return null;
}

/**
 * Fill a localized response template's `{placeholder}` slots, in order, the
 * way the Rust `template` helper does (English record, "" when absent).
 * @param {string} intent response intent
 * @param {Array<Array<string>>} values [key, value] pairs
 * @returns {string} the filled template
 */
function codeTaskTemplate(intent, values) {
  const table = MULTILINGUAL_ANSWERS[intent];
  let out = table && table.en ? String(answerFor(intent, "en") || "") : "";
  for (const pair of values) {
    out = out.split("{" + pair[0] + "}").join(String(pair[1]));
  }
  return out;
}

/**
 * Render request→emission mapping rows through the shared `mapping_line`
 * template.
 * @param {Array<Array<string>>} rows [request, emission] pairs
 * @returns {string} the concatenated rows
 */
function codeTaskMappingRows(rows) {
  let out = "";
  for (const row of rows) {
    out += codeTaskTemplate("mapping_line", [["request", row[0]], ["emission", row[1]]]);
  }
  return out;
}

/**
 * True for a character Rust's `char::is_alphanumeric` accepts.
 * @param {string} ch one code point
 * @returns {boolean} whether it is alphabetic or numeric
 */
function codeTaskIsAlphanumeric(ch) {
  return /^[\p{Alphabetic}\p{N}]$/u.test(ch);
}

/**
 * Trim code points matching `shouldTrim` from both ends (Rust `trim_matches`).
 * @param {string} text input text
 * @param {function(string): boolean} shouldTrim predicate over one code point
 * @returns {string} the trimmed text
 */
function codeTaskTrimMatches(text, shouldTrim) {
  const chars = Array.from(text);
  let start = 0;
  let end = chars.length;
  while (start < end && shouldTrim(chars[start])) start += 1;
  while (end > start && shouldTrim(chars[end - 1])) end -= 1;
  return chars.slice(start, end).join("");
}

/**
 * Remove every leading and trailing occurrence of one character.
 * @param {string} text input text
 * @param {string} ch the character to strip
 * @returns {string} the stripped text
 */
function codeTaskTrimChar(text, ch) {
  return codeTaskTrimMatches(text, function (c) { return c === ch; });
}

/**
 * Remove every trailing occurrence of one character (Rust `trim_end_matches`).
 * @param {string} text input text
 * @param {string} ch the character to strip
 * @returns {string} the stripped text
 */
function codeTaskTrimEndChar(text, ch) {
  let end = text.length;
  while (end > 0 && text[end - 1] === ch) end -= 1;
  return text.slice(0, end);
}

/**
 * Split text into lines the way Rust `str::lines` does.
 * @param {string} text input text
 * @returns {Array<string>} the lines, without terminators
 */
function codeTaskLines(text) {
  if (text === "") return [];
  const parts = text.split("\n");
  if (parts[parts.length - 1] === "") parts.pop();
  const out = [];
  for (const part of parts) {
    out.push(part.endsWith("\r") ? part.slice(0, part.length - 1) : part);
  }
  return out;
}

/**
 * Split on runs of whitespace, dropping empties (Rust `split_whitespace`).
 * @param {string} text input text
 * @returns {Array<string>} the words
 */
function codeTaskWords(text) {
  const out = [];
  for (const word of text.split(/\s+/u)) {
    if (word !== "") out.push(word);
  }
  return out;
}

/**
 * The tokens of a normalized prompt split on single spaces, empties dropped.
 * @param {string} normalized normalized prompt
 * @returns {Array<string>} tokens
 */
function codeTaskTokens(normalized) {
  const out = [];
  for (const token of normalized.split(" ")) {
    if (token !== "") out.push(token);
  }
  return out;
}

/**
 * A contiguous echo of the request's own tokens (empty when out of range).
 * @param {Array<string>} tokens request tokens
 * @param {number} from first index
 * @param {number} to last index, inclusive
 * @returns {string} the joined tokens
 */
function codeTaskEcho(tokens, from, to) {
  if (from < 0 || to >= tokens.length || from > to + 1) return "";
  return tokens.slice(from, to + 1).join(" ");
}

/**
 * Parse an unsigned 32-bit decimal the way Rust `str::parse::<u32>` does.
 * @param {string} text candidate digits
 * @returns {number|null} the value, or null
 */
function codeTaskParseU32(text) {
  if (!/^\+?[0-9]+$/.test(text)) return null;
  const value = Number(text);
  if (value > 4294967295) return null;
  return value;
}

/**
 * The numeric value of a word: a digit string, or a word in the `number` map.
 * @param {string} word candidate word
 * @param {Array<object>} numbers `number` word-map entries
 * @returns {number|null} the value, or null
 */
function codeTaskNumberValue(word, numbers) {
  const direct = codeTaskParseU32(word);
  if (direct !== null) return direct;
  const entry = codeTaskEntryFor(numbers, word);
  if (!entry) return null;
  return codeTaskParseU32(codeTaskChildValue(entry, "value"));
}

/**
 * Keep only identifier characters (alphanumeric or `_`).
 * @param {string} word input word
 * @returns {string} the identifier characters
 */
function codeTaskIdentifier(word) {
  let out = "";
  for (const ch of Array.from(word)) {
    if (codeTaskIsAlphanumeric(ch) || ch === "_") out += ch;
  }
  return out;
}

/**
 * The run of identifier characters starting the text.
 * @param {string} text input text
 * @returns {string} the leading identifier
 */
function codeTaskIdentifierAt(text) {
  let out = "";
  for (const ch of Array.from(text)) {
    if (!(codeTaskIsAlphanumeric(ch) || ch === "_")) break;
    out += ch;
  }
  return out;
}

/**
 * The balanced-parenthesis span and the index just past its `)`; ["", openAt]
 * when unbalanced.
 * @param {string} text input text
 * @param {number} openAt index of the opening `(`
 * @returns {Array} [inner text, index after the close]
 */
function codeTaskParenSpan(text, openAt) {
  let depth = 0;
  for (let index = openAt; index < text.length; index += 1) {
    const ch = text[index];
    if (ch === "(") {
      depth += 1;
    } else if (ch === ")") {
      depth -= 1;
      if (depth === 0) return [text.slice(openAt + 1, index), index + 1];
      if (depth < 0) return ["", openAt];
    }
  }
  return ["", openAt];
}

/**
 * The balanced-parenthesis span starting at the `(` at `openAt`.
 * @param {string} text input text
 * @param {number} openAt index of the opening `(`
 * @returns {string} the inner text, or ""
 */
function codeTaskInsideParens(text, openAt) {
  return codeTaskParenSpan(text, openAt)[0];
}

/**
 * Extract the code under discussion: the first fenced block, else a backtick
 * span that looks like code, else the whole prompt when it carries code markers.
 * @param {string} prompt raw prompt
 * @returns {string|null} the code, or null
 */
function codeTaskCodeBlock(prompt) {
  const fence = prompt.indexOf("```");
  if (fence !== -1) {
    const rest = prompt.slice(fence + 3);
    const newline = rest.indexOf("\n");
    const afterOpen = newline !== -1 ? rest.slice(newline + 1) : rest;
    const end = afterOpen.indexOf("```");
    if (end !== -1) {
      const code = afterOpen.slice(0, end);
      if (code.trim() !== "") return code;
    }
  }
  const tick = prompt.indexOf("`");
  if (tick !== -1) {
    const close = prompt.indexOf("`", tick + 1);
    if (close !== -1) {
      const code = prompt.slice(tick + 1, close);
      for (const marker of ["(", "def ", "=>", "return "]) {
        if (code.includes(marker)) return code;
      }
    }
  }
  for (const marker of ["def ", "function ", "fn ", "=>", "return "]) {
    if (prompt.includes(marker)) return prompt;
  }
  // The colon after the request introduces the code itself
  // ("Explain this code: print(sum(range(10)))") when it carries a call,
  // an assignment or a subscript (Rust `code_debugging::code_block`).
  const colon = prompt.indexOf(": ");
  const payload = colon === -1 ? "" : prompt.slice(colon + 2).trim();
  return /[(=[]/u.test(payload) ? payload : null;
}

/**
 * The function-definition keywords the code-task family recognizes.
 * @returns {Array<string>} keywords with their trailing space
 */
function codeTaskDefKeywords() {
  return ["def ", "fn ", "function ", "async function "];
}

/**
 * The name of the first function defined in the code.
 * @param {string} code source text
 * @returns {string|null} the function name, or null
 */
function codeTaskFunctionName(code) {
  for (const line of codeTaskLines(code)) {
    const trimmed = line.trimStart();
    for (const keyword of codeTaskDefKeywords()) {
      if (!trimmed.startsWith(keyword)) continue;
      const name = codeTaskIdentifierAt(trimmed.slice(keyword.length));
      if (name !== "") return name;
    }
  }
  return null;
}

/**
 * The `function_intent` table: the property each function name promises.
 * @returns {Array<object>} {names, property, correctForm, grounding} records
 */
function codeTaskFunctionIntents() {
  const out = [];
  for (const record of codeTaskSeedRecords("meanings-code-structure-explanations.lino")) {
    if (record.name !== "function_intent") continue;
    const name = codeTaskChildValue(record, "name");
    if (name === "") continue;
    const body = record.children.length > 0 && record.children[0].name === "name" ? record.children[0] : null;
    if (!body) continue;
    const names = [name];
    for (const alias of codeTaskChildren(body, "alias")) {
      if (alias.id) names.push(String(alias.id));
    }
    out.push({
      names: names,
      property: codeTaskChildValue(body, "property"),
      correctForm: codeTaskChildValue(body, "correct_form"),
      grounding: codeTaskChildValue(body, "grounding"),
    });
  }
  return out;
}

/**
 * The intent record whose name or alias occurs in the function name.
 * @param {string|null} name function name
 * @returns {object|null} the matching intent, or null
 */
function codeTaskIntentForName(name) {
  if (name === null) return null;
  for (const intent of codeTaskFunctionIntents()) {
    for (const candidate of intent.names) {
      if (name.includes(candidate)) return intent;
    }
  }
  return null;
}

/**
 * A fresh event log (ordered [kind, payload] pairs, as the Rust EventLog).
 * @returns {Array<Array<string>>} an empty log
 */
function codeTaskLog() {
  return [];
}

/**
 * Append one event to a code-task log.
 * @param {Array<Array<string>>} log the log
 * @param {string} kind event kind
 * @param {string} payload event payload
 * @returns {void}
 */
function codeTaskLogAppend(log, kind, payload) {
  log.push([kind, String(payload)]);
}

/**
 * Build the worker answer the way Rust `finalize_simple` does: the log becomes
 * `kind:payload` evidence, followed by the response link and the trace.
 * @param {Array<Array<string>>} log the handler's event log
 * @param {string} intent answer intent
 * @param {string} responseLink response link such as "response:code_review"
 * @param {string} body answer body
 * @param {number} confidence answer confidence
 * @returns {object} the worker answer
 */
function codeTaskAnswer(log, intent, responseLink, body, confidence) {
  const evidence = [];
  for (const entry of log) evidence.push(entry[0] + ":" + entry[1]);
  evidence.push(responseLink);
  evidence.push("trace:" + intent);
  return {
    intent: intent,
    content: body,
    confidence: confidence,
    evidence: evidence,
  };
}
