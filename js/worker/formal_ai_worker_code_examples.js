// Code examples as formal knowledge (issue #1164), browser twin.
//
// JavaScript-first twin of rust/src/code_example_knowledge.rs: decompose a
// retrieved example into meaningful parts (entry point, output operation,
// string literal, build and run commands from the page's prose), align two or
// more examples into a generalized procedure, and recompose a program for a
// new requirement from the aligned parts. The vocabulary is
// data/seed/code-example-parts.lino and the grammar registry is
// data/seed/program-cst-grammars.lino; no language fact lives here.
//
// The browser has no meta-language parse, so the twin reads identifier
// tokens lexically where the native decomposer reads CST tokens, and records
// the schema's default node kinds. Import and test-assertion parts need CST
// node kinds and are therefore native-only. Recomposition, like the native
// one, builds from the target language's own example body with the output
// literal slotted, after checking that body still makes the aligned output
// call; nothing is filled from a stored template.

const CODE_EXAMPLE_PARTS_FILE = "code-example-parts.lino";
const CODE_EXAMPLE_GRAMMARS_FILE = "program-cst-grammars.lino";
const CODE_EXAMPLE_CAPTURES_FILE = "coding-documentation-captures.lino";
const CODE_EXAMPLE_POLICY_FILE = "program-cache-policy.lino";
const CODE_EXAMPLE_LITERAL_SLOT = "{" + "literal}";
const CODE_EXAMPLE_QUOTES = ["\"", "'"];
const CODE_EXAMPLE_NO_ENTRY = "none";
const CODE_EXAMPLE_DEFAULT_KINDS = {
  entry_point: "function_declaration",
  output_operation: "call_expression",
  string_literal: "string_literal",
  prose: "prose_link",
};
/** The canonical order shared structure is reported in. */
const CODE_EXAMPLE_KIND_ORDER = [
  "import",
  "entry_point",
  "output_operation",
  "string_literal",
  "test_assertion",
  "build_command",
  "run_command",
];

/**
 * The part vocabulary, read in document order at every depth: a
 * `part_language` header sets the language its rows belong to.
 * @returns {{outputCalls: object, entryNames: object, proseRelations: Array<Array<string>>, outputCallRelations: Array<string>}}
 */
function codeExampleVocabulary() {
  const vocabulary = { outputCalls: {}, entryNames: {}, proseRelations: [], outputCallRelations: [] };
  const pending = parseLinoTree(seedRawText(SEED_RAW, CODE_EXAMPLE_PARTS_FILE)).children.slice().reverse();
  let language = "";
  while (pending.length > 0) {
    const record = pending.pop();
    for (let index = record.children.length - 1; index >= 0; index -= 1) pending.push(record.children[index]);
    if (record.name === "part_language") language = record.value;
    else if (record.name === "output_call" && language && record.value) {
      (vocabulary.outputCalls[language] = vocabulary.outputCalls[language] || []).push(record.value);
    } else if (record.name === "entry_name" && language) {
      vocabulary.entryNames[language] = record.value;
    } else if (record.name === "prose_relation") {
      const relation = childValue(record, "relation");
      const kind = childValue(record, "kind");
      if (relation && (kind === "build_command" || kind === "run_command")) {
        vocabulary.proseRelations.push([relation, kind]);
      } else if (relation && kind === "print_stdout") vocabulary.outputCallRelations.push(relation);
    }
  }
  return vocabulary;
}

/**
 * Whether the grammar registry carries the slug.
 * @param {string} slug
 * @returns {boolean}
 */
function codeExampleGrammarRegistered(slug) {
  const text = seedRawText(SEED_RAW, CODE_EXAMPLE_GRAMMARS_FILE);
  for (const top of parseLinoTree(text).children) {
    for (const record of top.children) {
      if (record.name !== "cst_grammar") continue;
      if (record.value === slug || childValue(record, "program_language") === slug) return true;
    }
  }
  return false;
}

/**
 * Every quoted string literal in the source, unquoted (escapes kept).
 * @param {string} source
 * @returns {Array<string>}
 */
function codeExampleQuotedLiterals(source) {
  const out = [];
  let index = 0;
  while (index < source.length) {
    const quote = source.charAt(index);
    if (!CODE_EXAMPLE_QUOTES.includes(quote)) {
      index += 1;
      continue;
    }
    let literal = "";
    let cursor = index + 1;
    let closed = false;
    while (cursor < source.length) {
      const character = source.charAt(cursor);
      if (character === "\\" && cursor + 1 < source.length) {
        literal += character + source.charAt(cursor + 1);
        cursor += 2;
        continue;
      }
      if (character === quote) {
        closed = true;
        cursor += 1;
        break;
      }
      literal += character;
      cursor += 1;
    }
    if (closed && literal !== "") out.push(literal);
    index = closed ? cursor : index + 1;
  }
  return out;
}

/**
 * The source with the first quoted literal on a line that makes the call
 * replaced by the slot (quotes kept); empty when no such line carries it.
 * @param {string} source
 * @param {string} call
 * @param {string} literal
 * @returns {string}
 */
function codeExampleSlotLiteral(source, call, literal) {
  let offset = 0;
  for (const line of source.split(/(?<=\n)/)) {
    if (line.includes(call)) {
      for (const quote of CODE_EXAMPLE_QUOTES) {
        const at = line.indexOf(quote + literal + quote);
        if (at !== -1) {
          const start = offset + at + quote.length;
          return source.slice(0, start) + CODE_EXAMPLE_LITERAL_SLOT + source.slice(start + literal.length);
        }
      }
    }
    offset += line.length;
  }
  return "";
}

/**
 * Decompose one example into meaningful parts (issue #1164 R1 to R4). An
 * unregistered slug is refused by name, never guessed.
 * @param {string} source
 * @param {string} languageSlug
 * @param {Array<{relation: string, text: string, sourceUrl: string}>} proseLinks
 * @param {string} [sourceUrl]
 * @returns {{ok: object}|{error: {kind: string, language: string}}}
 */
function decomposeCodeExample(source, languageSlug, proseLinks, sourceUrl) {
  if (!codeExampleGrammarRegistered(languageSlug)) {
    return { error: { kind: "unknown_grammar", language: languageSlug } };
  }
  const url = sourceUrl || "";
  const tokens = source.match(/[A-Za-z_][A-Za-z0-9_]*/g) || [];
  if (tokens.length === 0) return { error: { kind: "parse_failed", language: languageSlug } };
  const vocabulary = codeExampleVocabulary();
  const parts = [];
  const part = (kind, text, nodeKind, partUrl) =>
    parts.push({ kind: kind, sourceText: text, cstNodeKind: nodeKind, sourceUrl: partUrl });
  const entry = vocabulary.entryNames[languageSlug];
  if (entry && entry !== CODE_EXAMPLE_NO_ENTRY && tokens.includes(entry)) {
    part("entry_point", entry, CODE_EXAMPLE_DEFAULT_KINDS.entry_point, url);
  }
  const relationOf = (prose) => pageAsciiLower(prose.relation).replace(/\s/g, "_");
  // Seed calls first, then a call the page's prose names (kind print_stdout), cited at the prose URL (R1164-9).
  const candidates = (vocabulary.outputCalls[languageSlug] || []).map((candidate) => [candidate, url]).concat(
    (proseLinks || []).filter((prose) => vocabulary.outputCallRelations.includes(relationOf(prose)))
      .map((prose) => [prose.text.trim(), prose.sourceUrl]).filter((pair) => pair[0] !== ""));
  const matched = candidates.find((pair) => tokens.includes(pair[0]) || source.includes(pair[0]));
  const call = matched === undefined ? undefined : matched[0];
  if (call !== undefined) part("output_operation", call, CODE_EXAMPLE_DEFAULT_KINDS.output_operation, matched[1]);
  const callLines = call === undefined ? [] : pageLines(source).filter((line) => line.includes(call));
  let literals = codeExampleQuotedLiterals(callLines.length > 0 ? callLines.join("\n") : source);
  const programBody = call !== undefined && literals.length > 0 ? codeExampleSlotLiteral(source, call, literals[0]) : "";
  if (literals.length === 0) literals = codeExampleQuotedLiterals(source);
  for (const literal of literals) part("string_literal", literal, CODE_EXAMPLE_DEFAULT_KINDS.string_literal, url);
  for (const prose of proseLinks || []) {
    const relation = relationOf(prose);
    const found = vocabulary.proseRelations.find((pair) => pair[0] === relation);
    if (found) part(found[1], prose.text, CODE_EXAMPLE_DEFAULT_KINDS.prose, prose.sourceUrl);
  }
  return { ok: { languageSlug: languageSlug, parts: parts, programBody: programBody } };
}

/**
 * Align decomposed examples into a generalized procedure (issue #1164 R5):
 * shared part kinds are the body; the printed literal, the output call and
 * each language's program body are per-language values.
 * @param {Array<object>} examples
 * @returns {object}
 */
function generalizeCodeExamples(examples) {
  const languages = Array.from(new Set(examples.map((example) => example.languageSlug))).sort();
  const sharedStructure = CODE_EXAMPLE_KIND_ORDER.filter((kind) =>
    examples.every((example) => example.parts.some((candidate) => candidate.kind === kind)),
  );
  const outputLiteral = {};
  const outputCall = {};
  const programBodies = {};
  const sourceUrls = [];
  for (const example of examples) {
    for (const candidate of example.parts) {
      if (candidate.kind === "string_literal" && !(example.languageSlug in outputLiteral)) {
        outputLiteral[example.languageSlug] = candidate.sourceText;
      }
      if (candidate.kind === "output_operation" && !(example.languageSlug in outputCall)) {
        outputCall[example.languageSlug] = candidate.sourceText;
      }
      if (candidate.sourceUrl && !sourceUrls.includes(candidate.sourceUrl)) sourceUrls.push(candidate.sourceUrl);
    }
    if (example.programBody && !(example.languageSlug in programBodies)) {
      programBodies[example.languageSlug] = example.programBody;
    }
  }
  const parameters = examples.length === 0 ? [] : [
    { name: "output_literal", perLanguage: outputLiteral },
    { name: "output_call", perLanguage: outputCall },
  ];
  return {
    id: "generalized:" + languages.join("+"),
    sharedStructure: sharedStructure,
    parameters: parameters,
    sourceUrls: sourceUrls,
    programBodies: programBodies,
  };
}

/**
 * Recompose a program for one language from the aligned parts (issue #1164
 * R6): the language's own example body, checked to still make the aligned
 * output call, with the bound literal escaped for the quote that opens its
 * slot. A language no example contributed, or an unbound literal, is a
 * refusal naming the language.
 * @param {object} procedure
 * @param {object} bindings
 * @param {string} targetLanguage
 * @returns {{ok: object}|{error: {kind: string, language: string}}}
 */
function recomposeCodeExample(procedure, bindings, targetLanguage) {
  const refuse = (kind) => ({ error: { kind: kind, language: targetLanguage } });
  const bound = bindings || {};
  const perLanguage = (name) => {
    const parameter = procedure.parameters.find((candidate) => candidate.name === name);
    return parameter ? parameter.perLanguage[targetLanguage] : undefined;
  };
  const body = procedure.programBodies[targetLanguage];
  const call = bound.output_call !== undefined ? bound.output_call : perLanguage("output_call");
  if (body === undefined || call === undefined) return refuse("no_program_body");
  const slotAt = body.indexOf(CODE_EXAMPLE_LITERAL_SLOT);
  if (slotAt === -1 || !body.includes(call)) return refuse("no_program_body");
  const literal = bound.output_literal !== undefined ? bound.output_literal : perLanguage("output_literal");
  if (literal === undefined) return refuse("unbound_output_literal");
  const quote = body.charAt(slotAt - 1);
  let escaped = "";
  for (const character of literal) {
    if (character === "\\" || character === quote) escaped += "\\";
    escaped += character;
  }
  return {
    ok: {
      languageSlug: targetLanguage,
      source: body.slice(0, slotAt) + escaped + body.slice(slotAt + CODE_EXAMPLE_LITERAL_SLOT.length),
      partSourceUrls: procedure.sourceUrls.slice(),
    },
  };
}

/** The policy seed's `documentation_route`, when the seed keeps it active. */
function documentationRouteActive() {
  const root = parseLinoTree(seedRawText(SEED_RAW, CODE_EXAMPLE_POLICY_FILE)).children[0];
  const route = (root ? root.children : []).find((node) => node.name === "documentation_route");
  return Boolean(route) && childValue(route, "active") === "true";
}

/**
 * The documentation captures for one language and task, in seed order, each
 * with its URL, rediscovery query and the code blocks its page holds.
 * @param {string} language
 * @param {string} task
 * @returns {Array<{url: string, query: string, blocks: Array<{language: string, text: string}>}>}
 */
function documentationCaptures(language, task) {
  const root = parseLinoTree(seedRawText(SEED_RAW, CODE_EXAMPLE_CAPTURES_FILE)).children
    .find((node) => node.name === "coding_documentation_captures");
  return (root ? root.children : [])
    .filter((node) => node.name === "capture" && childValue(node, "language") === language && childValue(node, "task") === task)
    .map((node) => ({
      url: childValue(node, "url"),
      query: childValue(node, "rediscovery_query"),
      blocks: node.children.filter((child) => child.name === "block")
        .map((block) => ({ language: childValue(block, "language"), text: childValue(block, "text") })),
    }));
}

/**
 * Whether a captured block's own language tag names `language`: the slug
 * itself, the formalizer's untagged fallback, or one of the surfaces of the
 * `program_language_<slug>` meaning (a page tagging its example `js`, `py`
 * or `c++`). Mirrors `block_names_language` in
 * rust/src/discovery_production_documentation.rs.
 * @param {string} tag
 * @param {string} language
 * @returns {boolean}
 */
function documentationBlockNamesLanguage(tag, language) {
  const lower = pageAsciiLower(String(tag || ""));
  return lower === language || lower === "unknown" || wordsForMeaning("program_language_" + language).includes(lower);
}

/** FNV-1a (64-bit) of the UTF-8 bytes as `0x` and 16 hex digits (`fnv1a64`). */
function codeExampleContentId(text) {
  let hash = 0xcbf29ce484222325n;
  for (const byte of new TextEncoder().encode(text)) {
    hash ^= BigInt(byte);
    hash = (hash * 0x100000001b3n) & 0xffffffffffffffffn;
  }
  return "0x" + hash.toString(16).padStart(16, "0");
}

/**
 * The names a language's run contract invokes that a program must declare:
 * the stem of the file it is saved as, when a run or check command names it
 * bare (`scala Main` runs the object `Main`; `rustc main.rs -o main` only
 * names the binary its `fn main` builds).
 * @param {object|null} languageInfo a WRITE_PROGRAM_LANGUAGES row
 * @returns {Array<string>}
 */
function runContractNames(languageInfo) {
  if (!languageInfo) return [];
  const file = String(languageInfo.saveAs || "").split("/").pop();
  const stem = file.includes(".") ? file.slice(0, file.lastIndexOf(".")) : file;
  const words = [languageInfo.runCommand, languageInfo.checkCommand].filter(Boolean).join(" ").split(/\s+/);
  return stem && words.includes(stem) ? [stem] : [];
}

/**
 * Rediscover a write_program procedure from the documentation captures
 * (issue #1165 R1165-1, the documentation_route of the policy seed). Mirrors
 * `rediscover_from_documentation` in rust/src/discovery_production.rs.
 * Each page's first block in the language (or untagged) that decomposes into
 * an output call printing a literal is recomposed with the task's expected
 * output bound into that literal; the shortest program is kept and verified
 * by decomposing it again and against the run contract. `rejected` names why
 * captured pages yielded no program; `captured` says whether any existed.
 * @param {string} task
 * @param {string} language
 * @returns {{recipe: object|null, rejected: string|null, captured: boolean}}
 */
function rediscoverDocumentedProgram(task, language) {
  const captures = documentationRouteActive() ? documentationCaptures(language, task) : [];
  if (captures.length === 0) return { recipe: null, rejected: null, captured: false };
  const reject = (reason) => ({ recipe: null, rejected: reason, captured: true });
  const taskInfo = typeof WRITE_PROGRAM_TASKS === "object" ? WRITE_PROGRAM_TASKS[task] : null;
  const expected = taskInfo ? String(taskInfo.output) : "";
  if (expected === "" || expected.includes("\n")) return reject("no_single_line_output");
  let best = null;
  for (const capture of captures) {
    for (const block of capture.blocks) {
      if (!documentationBlockNamesLanguage(block.language, language)) continue;
      const node = decomposeCodeExample(block.text, language, [], capture.url).ok;
      if (!node || !node.programBody || !node.parts.some((part) => part.kind === "output_operation")) continue;
      if (!best || [...node.programBody].length < [...best.node.programBody].length) best = { capture, node };
      break;
    }
  }
  if (!best) return reject("no_output_example");
  const recomposed = recomposeCodeExample(generalizeCodeExamples([best.node]), { output_literal: expected }, language).ok;
  if (!recomposed) return reject("no_program_body");
  const call = best.node.parts.find((part) => part.kind === "output_operation").sourceText;
  const check = decomposeCodeExample(recomposed.source, language, [], best.capture.url).ok;
  const verified = Boolean(check) && check.parts.some((part) => part.kind === "output_operation" && part.sourceText === call)
    && check.parts.some((part) => part.kind === "string_literal" && part.sourceText === expected);
  if (!verified) return reject("verification");
  const tokens = recomposed.source.split(/[^A-Za-z0-9_]+/);
  const languageInfo = typeof WRITE_PROGRAM_LANGUAGES === "object" ? WRITE_PROGRAM_LANGUAGES[language] : null;
  const undeclared = runContractNames(languageInfo).filter((name) => !tokens.includes(name));
  if (undeclared.length > 0) return reject("run_contract:" + undeclared.join(","));
  return {
    recipe: {
      language: language,
      task: task,
      rediscovery_query: best.capture.query,
      rediscovery_source: best.capture.url,
      entry: recomposed.source,
      verified_output: expected,
      content_id: codeExampleContentId(recomposed.source),
    },
    rejected: null,
    captured: true,
  };
}

/** The shell prompts a documented command line may start with. */
const CODE_EXAMPLE_COMMAND_PROMPTS = ["$ ", "% ", "> "];

/**
 * Whether a documented command is a catalog command with the documented file
 * name bound to the catalog's (issue #1165 R1165-6): the same words, except
 * that where the catalog word carries the stem of the file it saves the
 * program as, the documented word carries one other name in its place,
 * consistently (`kotlinc hello.kt -d hello.jar` is `kotlinc Main.kt -d
 * Main.jar` for a program saved as `Main.kt`). Mirrors
 * `documented_command_matches` in rust/src/discovery_production_documentation.rs.
 * @param {string} documented
 * @param {string} catalog
 * @param {string} saveAs
 * @returns {boolean}
 */
function documentedCommandMatches(documented, catalog, saveAs) {
  const file = String(saveAs || "").split("/").pop();
  const stem = file.includes(".") ? file.slice(0, file.lastIndexOf(".")) : file;
  const words = documented.trim().split(/\s+/);
  const expected = catalog.trim().split(/\s+/);
  if (words.length !== expected.length) return false;
  let bound = null;
  return expected.every((word, index) => {
    const said = words[index];
    if (said === word) return true;
    const at = stem ? word.indexOf(stem) : -1;
    if (at === -1) return false;
    const prefix = word.slice(0, at);
    const suffix = word.slice(at + stem.length);
    if (!said.startsWith(prefix) || !said.endsWith(suffix) || said.length <= prefix.length + suffix.length) return false;
    const name = said.slice(prefix.length, said.length - suffix.length);
    if (!/^[A-Za-z0-9_]+$/.test(name) || (bound !== null && bound !== name)) return false;
    bound = name;
    return true;
  });
}

/**
 * The catalog's check and run commands for a pair, each with the command
 * line its documentation captures state for it (R1165-6): a line of a
 * captured block, its shell prompt removed, that is the catalog command
 * with the documented file name bound to the catalog's. `documented` is null
 * when the captured pages state no such command. Mirrors
 * `documented_run_commands` in rust/src/discovery_production_documentation.rs.
 * @param {string} task
 * @param {string} language
 * @returns {Array<{catalog: string, documented: string|null}>}
 */
function documentedRunCommands(task, language) {
  const languageInfo = typeof WRITE_PROGRAM_LANGUAGES === "object" ? WRITE_PROGRAM_LANGUAGES[language] : null;
  if (!languageInfo) return [];
  const lines = [];
  for (const capture of documentationCaptures(language, task)) {
    for (const block of capture.blocks) {
      for (const raw of pageLines(block.text)) {
        let line = raw.trim();
        const prompt = CODE_EXAMPLE_COMMAND_PROMPTS.find((candidate) => line.startsWith(candidate));
        if (prompt) line = line.slice(prompt.length).trim();
        if (line) lines.push(line);
      }
    }
  }
  return [languageInfo.checkCommand, languageInfo.runCommand].filter(Boolean).map((command) => ({
    catalog: command,
    documented: lines.find((line) => documentedCommandMatches(line, command, languageInfo.saveAs)) || null,
  }));
}

/** Rediscovery is deterministic over the seed, so a worker computes it once per pair. */
const CODE_EXAMPLE_DOCUMENTED = new Map();

/**
 * The memoized rediscovery for a pair (never memoized before the captures
 * seed is loaded).
 * @param {string} task
 * @param {string} language
 */
function documentedProgram(task, language) {
  const key = task + ":" + language;
  if (CODE_EXAMPLE_DOCUMENTED.has(key)) return CODE_EXAMPLE_DOCUMENTED.get(key);
  const result = rediscoverDocumentedProgram(task, language);
  if (seedRawText(SEED_RAW, CODE_EXAMPLE_CAPTURES_FILE) !== "") CODE_EXAMPLE_DOCUMENTED.set(key, result);
  return result;
}

/**
 * Whether the documentation route knows a language (issue #1165 R1165-4):
 * some task its captures cover rediscovers a verified program. Mirrors
 * `language_has_documented_procedure` in rust/src/discovery_production.rs.
 * @param {string} language
 * @returns {boolean}
 */
function documentationKnowsLanguage(language) {
  const needle = pageAsciiLower(String(language || "").trim());
  if (!needle || !documentationRouteActive()) return false;
  const root = parseLinoTree(seedRawText(SEED_RAW, CODE_EXAMPLE_CAPTURES_FILE)).children
    .find((node) => node.name === "coding_documentation_captures");
  const tasks = (root ? root.children : [])
    .filter((node) => node.name === "capture" && childValue(node, "language") === needle)
    .map((node) => childValue(node, "task"));
  return Array.from(new Set(tasks)).some((task) => documentedProgram(task, needle).recipe !== null);
}

/**
 * The program a write_program answer starts from: the one rediscovered from
 * documentation when the captures yield a verified one, else the stored
 * catalog template, else null.
 * @param {string} task
 * @param {string} language
 * @returns {string|null}
 */
function writeProgramTemplate(task, language) {
  if (!task || !language) return null;
  const documented = documentedProgram(task, language);
  if (documented.recipe) return documented.recipe.entry;
  const stored = typeof WRITE_PROGRAM_TEMPLATES === "object" ? WRITE_PROGRAM_TEMPLATES[task]?.[language] : null;
  return typeof stored === "string" ? stored : null;
}

/**
 * The step records the adoption gate consumes (issue #1164 R7): one step per
 * shared part kind, counted from one, with provenance and license left for
 * the gate to fill.
 * @param {object} procedure
 * @returns {Array<object>}
 */
function codeExampleProcedureSteps(procedure) {
  return procedure.sharedStructure.map((kind, index) => ({
    ordinal: index + 1,
    part: kind,
    sourceId: procedure.id,
    sourceUrl: "",
    licenseName: "",
    licenseUrl: "",
  }));
}

/**
 * A multi-line value as one Links Notation string.
 * @param {string} text
 * @returns {string}
 */
function codeExampleNotationString(text) {
  return "\"" + text.split("\\").join("\\\\").split("\"").join("\\\"").split("\n").join("\\n") + "\"";
}

/**
 * A decomposed example as a Links Notation document (issue #1164 R10).
 * @param {object} node
 * @returns {string}
 */
function decomposedCodeExampleNotation(node) {
  const lines = ["decomposed_code_node", "  language_slug " + node.languageSlug];
  for (const candidate of node.parts) {
    lines.push("  part", "    kind " + candidate.kind);
    if (candidate.sourceText) lines.push("    source_text " + codeExampleNotationString(candidate.sourceText));
    lines.push("    cst_node_kind " + candidate.cstNodeKind);
    if (candidate.sourceUrl) lines.push("    source_url " + codeExampleNotationString(candidate.sourceUrl));
  }
  return lines.join("\n") + "\n";
}

/**
 * A generalized procedure as a Links Notation document (issue #1164 R10).
 * @param {object} procedure
 * @returns {string}
 */
function generalizedCodeExampleNotation(procedure) {
  const lines = ["generalized_procedure", "  id " + codeExampleNotationString(procedure.id)];
  for (const kind of procedure.sharedStructure) lines.push("  shared_structure " + kind);
  for (const parameter of procedure.parameters) {
    lines.push("  parameter", "    name " + parameter.name);
    for (const language of Object.keys(parameter.perLanguage).sort()) {
      lines.push("    " + language + " " + codeExampleNotationString(parameter.perLanguage[language]));
    }
  }
  for (const url of procedure.sourceUrls) lines.push("  source_url " + codeExampleNotationString(url));
  for (const language of Object.keys(procedure.programBodies).sort()) {
    lines.push("  program_body", "    language " + language);
    lines.push("    source " + codeExampleNotationString(procedure.programBodies[language]));
  }
  return lines.join("\n") + "\n";
}

/**
 * The code examples of a page the prompt supplies, decomposed (R1164-11):
 * when the query line evidences the seed meaning
 * `code_example_decomposition_request`, every code block in a registered
 * grammar is decomposed and the answer lists each part as `kind text`.
 * Mirrors `code_example_page_answer` in
 * rust/src/solver_handlers/page_query_text.rs. Null when the query asks
 * something else or no block decomposes.
 * @param {{query: string, page: string}} split
 * @returns {object|null}
 */
function tryCodeExamplePageQuery(split) {
  const meaning = findMeaning("code_example_decomposition_request");
  if (meaning === null || !meaningEvidencedIn(meaning, normalizePrompt(split.query))) return null;
  const lines = [];
  const trace = [];
  for (const block of formalizePage(split.page, null, null).blocks) {
    if (block.kind !== "code_block") continue;
    const node = decomposeCodeExample(block.text, block.language || "", [], "").ok;
    if (!node) continue;
    trace.push(`code_example_decomposition:${node.languageSlug} parts=${node.parts.length}`);
    for (const part of node.parts) {
      lines.push(`${part.kind} ${part.sourceText}`);
      trace.push(`code_example_part:${node.languageSlug} ${part.kind} ${part.sourceText}`);
    }
  }
  if (lines.length === 0) return null;
  return {
    intent: "code_example_decomposition",
    content: lines.join("\n"),
    confidence: 0.85,
    evidence: ["handler:page_query_text", ...trace, "response:code_example_decomposition"],
    trace: trace,
  };
}
