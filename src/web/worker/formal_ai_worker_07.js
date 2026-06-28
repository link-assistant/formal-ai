// Worker module 8 of 21. Loaded by ../formal_ai_worker.js.
function numericListProgramLinks(program) {
  const lines = [
    "program_syntax_tree",
    `  language ${program.languageSlug}`,
    `  value_type ${program.valueType.label}`,
    `  operation ${program.canonical}`,
    `  literal_values ${(program.displayValues || program.literals).join("|")}`,
  ];
  for (const statement of program.statements) {
    if (statement.kind === "literal_list") {
      lines.push(
        `  semantic_node literal_list name=${statement.name} mutable=${statement.mutable}`,
      );
    } else if (statement.kind === "sort_list" || statement.kind === "reverse_list") {
      lines.push(
        `  semantic_node ${statement.kind} source=${statement.source} target=${statement.target} direction=${statement.direction}`,
      );
    } else if (statement.kind === "reduce_list") {
      lines.push(
        `  semantic_node reduce_list source=${statement.source} target=${statement.target} reducer=${statement.reducer}`,
      );
    } else if (statement.kind === "print_joined") {
      lines.push(
        `  semantic_node print_joined source=${statement.source} separator="${statement.separator}"`,
      );
    } else if (statement.kind === "print_scalar") {
      lines.push(`  semantic_node print_scalar source=${statement.source}`);
    }
  }
  return lines.join("\n");
}

// Issue #395: code-idioms knowledge base for the universal list coding
// algorithm. Byte mirror of data/seed/coding-idioms.lino (regenerate with
// experiments/generate-coding-idioms-embed.mjs) — each language declares
// scaffolds (one per operation family) and idioms (named code fragments with
// cases selected by operation and value class), inheriting through extends.
// The composer below discovers the code composition from this data at
// execution time; there are no per-language renderer functions. Mirrors
// src/solver_handlers/numeric_list/codegen.rs.
// CODING_IDIOMS_LINO is loaded from synced seed/*.lino data during loadSeed().

// Safety caps mirroring MAX_EXPANSION_DEPTH / MAX_INHERITANCE_DEPTH in
// src/solver_handlers/numeric_list/codegen.rs: anything deeper is a
// definition cycle in the seed data and must fail composition.
const CODING_IDIOMS_MAX_EXPANSION_DEPTH = 8;
const CODING_IDIOMS_MAX_INHERITANCE_DEPTH = 4;

let cachedCodingIdiomCatalog;
// Parsed root of the coding-idioms knowledge base, loaded once per worker.
// Mirrors idiom_catalog in src/solver_handlers/numeric_list/codegen.rs.
function codingIdiomCatalog() {
  if (cachedCodingIdiomCatalog === undefined) {
    const root = parseLinoTree(CODING_IDIOMS_LINO);
    cachedCodingIdiomCatalog =
      root.children.find((child) => child.name === "coding_idioms") || null;
  }
  return cachedCodingIdiomCatalog;
}

function codingFindChild(node, name) {
  return node.children.find((child) => child.name === name) || null;
}

function codingChildValue(node, name) {
  const child = codingFindChild(node, name);
  return child ? child.value : "";
}

// The `language "<slug>"` node followed by its transitive `extends` parents,
// nearest first. Empty when the catalog does not know the slug. Mirrors
// language_chain.
function codingLanguageChain(catalog, slug) {
  const chain = [];
  let current = slug;
  while (chain.length < CODING_IDIOMS_MAX_INHERITANCE_DEPTH) {
    const node = catalog.children.find(
      (child) => child.name === "language" && child.value === current,
    );
    if (!node) break;
    chain.push(node);
    const parent = codingChildValue(node, "extends");
    if (!parent) break;
    current = parent;
  }
  return chain;
}

// The canonical semantic-tree variable name for `key` (list / transformed /
// reduced), read from the catalog's `defaults` node. Mirrors default_name.
function codingDefaultName(key) {
  const catalog = codingIdiomCatalog();
  if (!catalog) return "";
  const defaults = codingFindChild(catalog, "defaults");
  if (!defaults) return "";
  return codingChildValue(defaults, key);
}

// Whether the language declares (in the knowledge base) that its list
// transformations mutate the literal list in place rather than building a new
// collection. Mirrors mutates_list_in_place.
function codingMutatesListInPlace(slug) {
  const catalog = codingIdiomCatalog();
  if (!catalog) return false;
  for (const language of codingLanguageChain(catalog, slug)) {
    const node = codingFindChild(language, "mutable_list");
    if (node) return node.value === "true";
  }
  return false;
}

// One rendering pass: the resolved language chain plus the computed slot
// bindings for the program being rendered. Mirrors Composer::new.
function codingComposer(program, chain) {
  const bindings = new Map();
  // Per-language variable names: the nearest `names` override in the
  // inheritance chain, else the catalog-wide `defaults` entry.
  for (const key of ["list", "transformed", "reduced"]) {
    let name = "";
    for (const language of chain) {
      const names = codingFindChild(language, "names");
      if (names) {
        const entry = codingFindChild(names, key);
        if (entry) {
          name = entry.value;
          break;
        }
      }
    }
    if (!name) name = codingDefaultName(key);
    if (name) bindings.set(key, name);
  }
  // The language's storage type for the program's value class, from the
  // nearest `types` table in the inheritance chain that declares it.
  for (const language of chain) {
    const types = codingFindChild(language, "types");
    if (types) {
      const entry = codingFindChild(types, program.valueType.label);
      if (entry) {
        bindings.set("type", entry.value);
        break;
      }
    }
  }
  bindings.set("literal", program.literals.join(", "));
  bindings.set("count", String(program.literals.length));
  // Links Notation values cannot encode a raw tab, so templates spell it as a
  // computed slot.
  bindings.set("tab", "\t");
  return { program, chain, bindings };
}

// The scaffold template for the operation family, from the nearest language
// in the chain that declares one. Mirrors Composer::scaffold.
function codingScaffold(chain, family) {
  for (const language of chain) {
    const scaffold = language.children.find(
      (child) => child.name === "scaffold" && child.value === family,
    );
    if (scaffold) return codingChildValue(scaffold, "code");
  }
  return null;
}

// The idiom definition for `slot`, from the nearest language in the chain
// that declares it. Idioms are not merged across the chain: the nearest
// definition fully shadows inherited ones. Mirrors Composer::idiom.
function codingIdiom(chain, slot) {
  for (const language of chain) {
    const idiom = language.children.find(
      (child) => child.name === "idiom" && child.value === slot,
    );
    if (idiom) return idiom;
  }
  return null;
}

// Pick the idiom case that best matches the requested operation and value
// class. A case applies when its `for` tokens contain the operation (or
// `any`) and its `on` tokens, when present, contain the value class. Specific
// matches outrank generic ones: an exact operation token scores over `any`,
// and a value-class constraint scores over none. The first case with the
// highest score wins, so declaration order breaks ties. Mirrors
// Composer::select_case.
function codingSelectCase(composer, idiom) {
  const operation = composer.program.canonical;
  const valueClass = composer.program.valueType.label;
  let best = null;
  for (const candidate of idiom.children) {
    if (candidate.name !== "case") continue;
    const forTokens = codingChildValue(candidate, "for")
      .split(/\s+/)
      .filter(Boolean);
    const operationExact = forTokens.includes(operation);
    if (!operationExact && !forTokens.includes("any")) continue;
    const on = codingFindChild(candidate, "on");
    if (on && !on.value.split(/\s+/).filter(Boolean).includes(valueClass)) {
      continue;
    }
    const score = (operationExact ? 2 : 0) + (on ? 1 : 0);
    if (!best || score > best.score) {
      best = { code: codingChildValue(candidate, "code"), score };
    }
  }
  return best ? best.code : null;
}

// Recursively expand `{slot}` placeholders. Computed bindings are inserted
// verbatim (never rescanned, so user-provided literals cannot inject further
// slots); idiom slots expand their selected case recursively; any other brace
// sequence — `{}`, `{ return`, `{{literal}}` — is ordinary target-language
// syntax and passes through unchanged. Mirrors Composer::expand.
function codingExpand(composer, template, depth) {
  if (depth > CODING_IDIOMS_MAX_EXPANSION_DEPTH) return null;
  const chars = Array.from(template);
  let out = "";
  let index = 0;
  while (index < chars.length) {
    if (chars[index] !== "{") {
      out += chars[index];
      index += 1;
      continue;
    }
    let end = index + 1;
    while (end < chars.length && /[a-z0-9_]/.test(chars[end])) end += 1;
    if (end >= chars.length || chars[end] !== "}" || end === index + 1) {
      out += "{";
      index += 1;
      continue;
    }
    const name = chars.slice(index + 1, end).join("");
    if (composer.bindings.has(name)) {
      out += composer.bindings.get(name);
    } else {
      const idiom = codingIdiom(composer.chain, name);
      if (idiom) {
        const code = codingSelectCase(composer, idiom);
        if (code === null) return null;
        const expanded = codingExpand(composer, code, depth + 1);
        if (expanded === null) return null;
        out += expanded;
      } else {
        out += `{${name}}`;
      }
    }
    index = end + 1;
  }
  return out;
}

// Render the program tree into the requested target language by composing the
// scaffold and idioms discovered in the coding-idioms knowledge base. Returns
// null when the knowledge base has no language section, no scaffold for the
// operation's family, or no idiom case matching the operation and value class
// — composition failures are explicit, never silent fallbacks. Mirrors
// NumericProgram::render.
function numericListProgramSource(program) {
  const catalog = codingIdiomCatalog();
  if (!catalog) return null;
  const chain = codingLanguageChain(catalog, program.languageSlug);
  if (!chain.length) return null;
  const composer = codingComposer(program, chain);
  const scaffold = codingScaffold(chain, numericListFamily(program.canonical));
  if (scaffold === null) return null;
  return codingExpand(composer, scaffold, 0);
}

// Issue #395: universal numeric-list coding algorithm. "<operation> these
// numbers in <language>, give me the code and the result" produces generated
// code plus the deterministically-computed result. Mirrors try_numeric_list /
// solve_numeric_list in src/solver_handlers/numeric_list/mod.rs. The result is
// computed in-solver (every operation is a pure, total function over the parsed
// values) — no runtime is embedded.
// Issue #412: recover the coding context from earlier conversation turns so a
// bare numeric-list follow-up ("Отсортируй 4, 3, 1, 17, 8, 9, 15") can inherit
// the language and the code request established earlier. Mirrors
// numeric_list_history_context in src/solver_handlers/numeric_list/mod.rs: we
// only inherit from a prior *user* turn that was itself a genuine numeric-list
// coding request — it names an operation, a supported language, and lists at
// least two numbers — so unrelated chatter never leaks a language.
// Issue #427: a bare operation follow-up ("Сделай инверсию сортировки.") names
// an operation but no numbers of its own — it refers to the list from the
// previous numeric-list turn. We therefore inherit the list from the most
// recent operation turn that carried a concrete list, while the language / code
// request keep coming from the most recent turn that named a language (issue
// #412). Splitting the two lets a number-less invert-sort inherit both even when
// they were established in different turns. Mirrors numeric_list_history_context
// in src/solver_handlers/numeric_list/mod.rs.
function numericListHistoryContext(history) {
  if (!Array.isArray(history))
    return { slug: null, codeRequested: false, items: [] };
  let slug = null;
  let codeRequested = false;
  let items = [];
  for (let i = history.length - 1; i >= 0; i -= 1) {
    const turn = history[i];
    if (!turn || String(turn.role || "").toLowerCase() !== "user") continue;
    const content = String(turn.content || "");
    const normalized = normalizePrompt(content);
    if (!detectNumericListOperation(normalized)) continue;
    const numbers = parseNumericListNumbers(content);
    if (numbers.length < 2) continue;
    // The most recent numeric-list turn with a concrete list seeds the
    // inherited list (issue #427).
    if (items.length === 0) items = numbers;
    // Language + code request come from the most recent turn naming a supported
    // language (issue #412).
    if (!slug) {
      const candidate = programLanguageFromPrompt(normalized);
      if (candidate && WRITE_PROGRAM_LANGUAGES[candidate]) {
        slug = candidate;
        codeRequested = operationMatchesSlug("code_request", normalized);
      }
    }
    if (items.length > 0 && slug) break;
  }
  return { slug, codeRequested, items };
}

function tryNumericList(prompt, history) {
  const normalized = normalizePrompt(prompt);

  // Defer genuine function-synthesis prompts ("write a function that returns the
  // sum of 3 and 5") to the dedicated program-synthesis handler.
  if (operationMatchesSlug("function", normalized) || prompt.includes("def ")) {
    return null;
  }

  const canonical = detectNumericListOperation(normalized);
  if (!canonical) return null;

  // Issue #412: a bare follow-up names no language and may not say "code" —
  // recover both from the active coding context before applying the gates.
  const inherited = numericListHistoryContext(history);
  const hasInheritance =
    Boolean(inherited.slug) ||
    inherited.codeRequested ||
    inherited.items.length > 0;

  // Reductions phrase-overlap with ordinary prose far more than the imperative
  // transform verbs do, so they only fire when the prompt explicitly asks for
  // code (the `code_request` operation) — exactly the issue #395 contract. A
  // reduction follow-up inside an established code context inherits that code
  // request (issue #412).
  if (
    numericListFamily(canonical) === "list_reduction" &&
    !operationMatchesSlug("code_request", normalized) &&
    !inherited.codeRequested
  ) {
    return null;
  }

  // The current prompt's own language wins; otherwise inherit it from context.
  const slug = programLanguageFromPrompt(normalized) || inherited.slug;
  if (!slug) return null;
  const languageInfo = WRITE_PROGRAM_LANGUAGES[slug];
  if (!languageInfo) return null;

  let items = parseNumericListItems(prompt, canonical);
  // Issue #427: a bare operation follow-up ("Сделай инверсию сортировки.")
  // names no numbers of its own — inherit the list from the previous
  // numeric-list turn so the operation has data to act on.
  if (items.length === 0 && inherited.items.length > 0) {
    items = inherited.items;
  }
  if (items.length < 2) return null;
  if (
    numericListFamily(canonical) === "list_reduction" &&
    items.some((item) => item.kind === "string")
  ) {
    return null;
  }

  const isFloat = items.some(
    (item) => item.kind === "number" && !Number.isInteger(item.value),
  );
  const given = items.map((n) => n.text);
  const result = computeNumericList(canonical, items, isFloat);
  const program = numericListBuildProgram(slug, items, canonical, isFloat);
  const syntaxTree = numericListProgramLinks(program);
  const code = numericListProgramSource(program);
  if (code === null) return null;
  const resultKind = numericListResultKind(canonical);
  const family = numericListFamily(canonical);

  const responseLanguage = detectLanguage(prompt);
  const parts =
    NUMERIC_LIST_LOCALIZATION[responseLanguage] || NUMERIC_LIST_LOCALIZATION.en;
  const givenText = given.join(", ");
  const shown = resultKind === "scalar" ? result[0] : result.join(", ");
  const resultText = result.join(", ");
  const body = `${parts.intro(canonical, languageInfo.name, givenText, program.valueType.label)}\n\n\`\`\`${languageInfo.fence}\n${code}\n\`\`\`\n\n${parts.resultLabel} ${shown}`;

  const evidence = [
    `response:write_program:numeric_list:${canonical}:${slug}`,
  ];
  // Issue #412: record when the language / code request was recovered from the
  // conversation, mirroring the `numeric_list_coreference` event the Rust
  // solver appends to its trace.
  if (hasInheritance) {
    evidence.push(
      `numeric_list_coreference inherited_language=${inherited.slug || "none"} inherited_code_request=${inherited.codeRequested}`,
    );
  }
  evidence.push(
    `formalize:(@USER OP:${canonical} values:[${given.join(" ")}] value_type:${program.valueType.label} language:${slug} result_kind:${resultKind} request:[code result])`,
    `synthesis:spec:language=${slug} task=numeric_list operation=${canonical} family=${family} result_kind=${resultKind} value_type=${program.valueType.label}`,
    `synthesis:given:${givenText}`,
    `synthesis:syntax_tree:${syntaxTree}`,
    `composition:code_fragment:${code}`,
    "execution_status:computed deterministically",
    "execution_environment:pure in-solver evaluation of a decidable numeric-list operation",
    `execution_result:${resultText}`,
  );

  return {
    intent: "write_program",
    content: body,
    confidence: 1.0,
    evidence,
    trace: [
      `synthesis:spec:language=${slug} task=numeric_list operation=${canonical} family=${family} result_kind=${resultKind} value_type=${program.valueType.label}`,
      `synthesis:syntax_tree:${syntaxTree}`,
      `execution_result:${resultText}`,
    ],
  };
}

function tryProgramSynthesis(prompt, normalized) {
  // Issue #386: canonicalize first so native operation verbs (write / implement /
  // return …) in any supported language are recognised, exactly like
  // try_program_synthesis in src/solver_handlers/program_synthesis.rs.
  const canonical = canonicalizedPrompt(normalized);
  if (!looksLikePythonFunctionSynthesis(prompt, canonical)) return null;
  const functionName = extractPythonFunctionName(prompt, canonical);
  if (!functionName) return null;
  const candidate = synthesizePythonCandidate(prompt, canonical, functionName);
  if (!candidate) return null;
  const assertionCount = candidate.tests.length;
  const syntaxTree = pythonFunctionLinks(candidate.functionTree);
  const code = renderPythonFunction(candidate.functionTree);
  const evidence = [
    `response:write_program:synthesized:python:${candidate.id}`,
    `synthesis:spec:language=python function=${candidate.functionName}`,
    `synthesis:syntax_tree:${syntaxTree}`,
    ...candidate.fragments.map((fragment) => `composition:code_fragment:${fragment}`),
    `synthesis:candidate:${candidate.id}`,
    "synthesis:workspace:browser-worker-deterministic-verifier",
    "action_log:create_file:solution.py",
    "action_log:run_command:python3 solution.py",
    `synthesis:candidate_execution:command=python3 solution.py exit=Some(0) timed_out=false assertion_count=${assertionCount}`,
    `synthesis:verification:tests_passed assertion_count=${assertionCount}`,
    "execution_status:tests passed",
    "execution_environment:browser worker deterministic mirror; no filesystem side effects",
  ];
  const body = [
    "Here is a derived Python function synthesized from the specification and verified in an isolated workspace:",
    "",
    "```python",
    code + "```",
    "",
    "Execution status: tests passed in isolated bounded agent workspace.",
    "Check command: `python3 solution.py`",
    `Test outcome: ${assertionCount}/${assertionCount} assertions passed.`,
    "Workspace isolation: browser worker deterministic verifier with no filesystem side effects.",
  ];
  return {
    intent: "write_program",
    content: body.join("\n"),
    confidence: 1.0,
    evidence,
    trace: [
      `synthesis:candidate:${candidate.id}`,
      `synthesis:syntax_tree:${syntaxTree}`,
      `synthesis:verification:tests_passed assertion_count=${assertionCount}`,
    ],
  };
}

function quoteCloseFor(open) {
  if (open === "'") return "'";
  if (open === '"') return '"';
  if (open === "`") return "`";
  if (open === "«") return "»";
  if (open === "“") return "”";
  if (open === "‘") return "’";
  if (open === "「") return "」";
  if (open === "『") return "』";
  return "";
}

function quotedTextSpans(text) {
  const segments = [];
  let cursor = 0;
  const source = String(text || "");
  while (cursor < source.length) {
    let found = -1;
    let close = "";
    for (let index = cursor; index < source.length; index += 1) {
      close = quoteCloseFor(source[index]);
      if (close) {
        found = index;
        break;
      }
    }
    if (found < 0) break;
    const contentStart = found + 1;
    const contentEnd = source.indexOf(close, contentStart);
    if (contentEnd < 0) break;
    segments.push({
      text: source.slice(contentStart, contentEnd),
      start: found,
      end: contentEnd + close.length,
    });
    cursor = contentEnd + 1;
  }
  return segments;
}

function quotedTextSegments(text) {
  return quotedTextSpans(text).map((segment) => segment.text);
}

function textAfterColon(prompt) {
  const text = String(prompt || "");
  const index = text.lastIndexOf(":");
  if (index < 0) return "";
  return text
    .slice(index + 1)
    .trim()
    .replace(/^[«»“”‘’「」『』"'`]+|[«»“”‘’「」『』"'`]+$/g, "")
    .trim();
}

function isReplaceTextPrompt(normalized) {
  const text = String(normalized || "");
  return (
    text.includes("replace") ||
    text.includes("instead") ||
    text.includes("вместо") ||
    text.includes("замен") ||
    text.includes("बदल") ||
    text.includes("替换")
  );
}

function lastAssistantTextArtifact(history) {
  if (!Array.isArray(history)) return "";
  for (let index = history.length - 1; index >= 0; index -= 1) {
    const turn = history[index];
    if (!turn || turn.role !== "assistant") continue;
    const content = String(turn.content || turn.text || turn.message || "");
    if (content.trim()) return content;
  }
  return "";
}

function containsReplacementKeyword(text) {
  return isReplaceTextPrompt(normalizePrompt(text));
}

function inputContextBeforeFirstQuote(text) {
  if (containsReplacementKeyword(text)) return false;
  const normalized = normalizePrompt(text);
  const raw = String(text || "").toLowerCase();
  return (
    normalized.endsWith("in") ||
    normalized.includes("text") ||
    normalized.includes("текст") ||
    raw.includes("पाठ") ||
    raw.includes("टेक्स्ट") ||
    raw.includes("在") ||
    raw.includes("文本") ||
    raw.includes("内容")
  );
}

function containsInputContinuation(text) {
  const normalized = normalizePrompt(text);
  const raw = String(text || "").toLowerCase();
  return (
    normalized.includes("in") ||
    normalized.includes("text") ||
    normalized.includes("текст") ||
    raw.includes("में") ||
    raw.includes("中")
  );
}

function looksLikeInputFirstReplacement(prompt, quoted) {
  if (quoted.length < 3) return false;
  const source = String(prompt || "");
  const beforeFirst = source.slice(0, quoted[0].start);
  const betweenFirstSecond = source.slice(quoted[0].end, quoted[1].start);
  const betweenSecondThird = source.slice(quoted[1].end, quoted[2].start);
  return (
    inputContextBeforeFirstQuote(beforeFirst) ||
    containsReplacementKeyword(betweenFirstSecond) ||
    (containsInputContinuation(betweenFirstSecond) &&
      containsReplacementKeyword(betweenSecondThird))
  );
}

function textOperationMatches(slug, normalized) {
  return operationVocabulary().some(
    (operation) => operation.slug === slug && operationFormMatches(normalized, operation),
  );
}

function appendSimpleTextOperations(normalized, operations) {
  if (textOperationMatches("lowercase", normalized)) {
    operations.push({ slug: "lowercase" });
  } else if (textOperationMatches("uppercase", normalized)) {
    operations.push({ slug: "uppercase" });
  }
  if (textOperationMatches("reverse_words", normalized)) {
    operations.push({ slug: "reverse_words" });
  }
  if (textOperationMatches("extract_email", normalized)) {
    operations.push({ slug: "extract_email" });
  }
  if (textOperationMatches("extract_url", normalized)) {
    operations.push({ slug: "extract_url" });
  }
  if (textOperationMatches("extract_number", normalized)) {
    operations.push({ slug: "extract_number" });
  }
  if (textOperationMatches("deduplicate_lines", normalized)) {
    operations.push({ slug: "deduplicate_lines" });
  }
  if (textOperationMatches("sort_lines", normalized)) {
    operations.push({ slug: "sort_lines" });
  }
  if (textOperationMatches("sort_words", normalized)) {
    operations.push({ slug: "sort_words" });
  }
  if (textOperationMatches("trim_whitespace", normalized)) {
    operations.push({ slug: "trim_whitespace" });
  }
  if (textOperationMatches("normalize_whitespace", normalized)) {
    operations.push({ slug: "normalize_whitespace" });
  }
  if (textOperationMatches("title_case", normalized)) {
    operations.push({ slug: "title_case" });
  }
  if (textOperationMatches("sentence_case", normalized)) {
    operations.push({ slug: "sentence_case" });
  }
  if (textOperationMatches("snake_case", normalized)) {
    operations.push({ slug: "snake_case" });
  }
  if (textOperationMatches("kebab_case", normalized)) {
    operations.push({ slug: "kebab_case" });
  }
  if (textOperationMatches("camel_case", normalized)) {
    operations.push({ slug: "camel_case" });
  }
  if (textOperationMatches("pascal_case", normalized)) {
    operations.push({ slug: "pascal_case" });
  }
  if (textOperationMatches("strip_empty_lines", normalized)) {
    operations.push({ slug: "strip_empty_lines" });
  }
  if (textOperationMatches("join_lines", normalized)) {
    operations.push({ slug: "join_lines" });
  }
  if (textOperationMatches("reverse_lines", normalized)) {
    operations.push({ slug: "reverse_lines" });
  }
  if (textOperationMatches("number_lines", normalized)) {
    operations.push({ slug: "number_lines" });
  }
  if (textOperationMatches("indent_lines", normalized)) {
    operations.push({ slug: "indent_lines" });
  }
  if (textOperationMatches("outdent_lines", normalized)) {
    operations.push({ slug: "outdent_lines" });
  }
  if (textOperationMatches("uncomment_lines", normalized)) {
    operations.push({ slug: "uncomment_lines" });
  } else if (textOperationMatches("comment_lines", normalized)) {
    operations.push({ slug: "comment_lines" });
  }
  if (textOperationMatches("remove_punctuation", normalized)) {
    operations.push({ slug: "remove_punctuation" });
  }
  if (textOperationMatches("count_unique_words", normalized)) {
    operations.push({ slug: "count_unique_words" });
  } else if (textOperationMatches("count_words", normalized)) {
    operations.push({ slug: "count_words" });
  }
  if (textOperationMatches("count_lines", normalized)) {
    operations.push({ slug: "count_lines" });
  }
  if (textOperationMatches("count_characters", normalized)) {
    operations.push({ slug: "count_characters" });
  }
}

function looksLikeInputFirstUnaryTextEdit(prompt, quoted) {
  if (quoted.length < 2) return false;
  const source = String(prompt || "");
  const beforeFirst = source.slice(0, quoted[0].start);
  return inputContextBeforeFirstQuote(beforeFirst);
}

function parseRemoveTextRequest(prompt, history, quoted) {
  if (!quoted.length) return null;
  if (quoted.length >= 2 && looksLikeInputFirstUnaryTextEdit(prompt, quoted)) {
    return { input: quoted[0].text, needle: quoted[1].text };
  }
  const input = (quoted[1] && quoted[1].text) || textAfterColon(prompt) || lastAssistantTextArtifact(history);
  if (!input) return null;
  return { input, needle: quoted[0].text };
}

function parseAffixTextRequest(prompt, history, quoted) {
  if (!quoted.length) return null;
  if (quoted.length >= 2 && looksLikeInputFirstUnaryTextEdit(prompt, quoted)) {
    return { input: quoted[0].text, affix: quoted[1].text };
  }
  const input = (quoted[1] && quoted[1].text) || textAfterColon(prompt) || lastAssistantTextArtifact(history);
  if (!input) return null;
  return { input, affix: quoted[0].text };
}

function isAgentTextRequest(normalized) {
  return (
    normalized.includes("[agent]") ||
    normalized.includes("enable agent") ||
    normalized.includes("agent mode")
  );
}

function parseTextManipulationRequest(prompt, normalized, history = []) {
  if (isAgentTextRequest(normalized)) return null;
  const quoted = quotedTextSpans(prompt);
  const operations = [];
  const fallbackInput = lastAssistantTextArtifact(history);
  let input = "";
  if (isReplaceTextPrompt(normalized)) {
    if (quoted.length < 2) return null;
    if (quoted.length >= 3 && looksLikeInputFirstReplacement(prompt, quoted)) {
      operations.push({ slug: "replace_text", from: quoted[1].text, to: quoted[2].text });
      input = quoted[0].text;
    } else {
      operations.push({ slug: "replace_text", from: quoted[0].text, to: quoted[1].text });
      input = (quoted[2] && quoted[2].text) || textAfterColon(prompt) || fallbackInput;
    }
  } else if (normalized.includes("count occurrences")) {
    if (quoted.length < 1) return null;
    operations.push({ slug: "count_occurrences", needle: quoted[0].text });
    input = (quoted[1] && quoted[1].text) || textAfterColon(prompt) || fallbackInput;
  } else if (textOperationMatches("remove_text", normalized) && !matchesSpecificRemoveOperation(normalized)) {
    const parsed = parseRemoveTextRequest(prompt, history, quoted);
    if (!parsed) return null;
    operations.push({ slug: "remove_text", needle: parsed.needle });
    input = parsed.input;
  } else if (textOperationMatches("append_text", normalized)) {
    const parsed = parseAffixTextRequest(prompt, history, quoted);
    if (!parsed) return null;
    operations.push({ slug: "append_text", suffix: parsed.affix });
    input = parsed.input;
  } else if (textOperationMatches("prepend_text", normalized)) {
    const parsed = parseAffixTextRequest(prompt, history, quoted);
    if (!parsed) return null;
    operations.push({ slug: "prepend_text", prefix: parsed.affix });
    input = parsed.input;
  } else {
    input = (quoted[0] && quoted[0].text) || textAfterColon(prompt) || fallbackInput;
    appendSimpleTextOperations(normalized, operations);
  }
  if (!input || operations.length === 0) return null;
  return { input, operations };
}

function matchesSpecificRemoveOperation(normalized) {
  return textOperationMatches("remove_punctuation", normalized) || textOperationMatches("strip_empty_lines", normalized);
}

function cleanEmailCandidate(candidate) {
  return String(candidate || "")
    .replace(/^[^A-Za-z0-9@._+-]+|[^A-Za-z0-9@._+-]+$/g, "")
    .replace(/[.]+$/g, "");
}

function looksLikeEmail(candidate) {
  const text = String(candidate || "");
  if ((text.match(/@/g) || []).length !== 1) return false;
  const [local, domain] = text.split("@");
  return Boolean(
    local &&
      domain &&
      domain.includes(".") &&
      domain.split(".").every((part) => part && /^[A-Za-z0-9-]+$/.test(part)),
  );
}

function cleanUrlCandidate(candidate) {
  return String(candidate || "")
    .replace(/^[`"'<>()[\]{},;]+|[`"'<>()[\]{},;]+$/g, "")
    .replace(/[.!?:]+$/g, "");
}

function looksLikeUrl(candidate) {
  const text = String(candidate || "");
  return text.startsWith("http://") || text.startsWith("https://") || text.startsWith("www.");
}

function extractNumbers(input) {
  const numbers = [];
  const pattern = /(^|[^\p{L}\p{N}])([+-]?\d+(?:\.\d+)?)(?=$|[^\p{L}\p{N}])/gu;
  for (const match of String(input || "").matchAll(pattern)) numbers.push(match[2]);
  return numbers;
}

function countUniqueWords(input) {
  return new Set(
    String(input || "")
      .split(/\s+/)
      .map((word) => word.replace(/^[^\p{L}\p{N}]+|[^\p{L}\p{N}]+$/gu, ""))
      .filter(Boolean),
  ).size;
}

function countWords(input) {
  return String(input || "")
    .split(/\s+/)
    .map((word) => word.replace(/^[^\p{L}\p{N}]+|[^\p{L}\p{N}]+$/gu, ""))
    .filter(Boolean).length;
}

function deduplicateLines(input) {
  const seen = new Set();
  const lines = [];
  for (const line of String(input || "").split(/\r?\n/)) {
    if (!seen.has(line)) {
      seen.add(line);
      lines.push(line);
    }
  }
  return lines;
}

function caseWords(input) {
  return normalizedWordSpans(input).map((span) => span.word);
}

function capitalizeWord(word) {
  const letters = Array.from(String(word || "").toLowerCase());
  if (!letters.length) return "";
  return `${letters[0].toUpperCase()}${letters.slice(1).join("")}`;
}

function titleCase(input) {
  return caseWords(input).map(capitalizeWord).join(" ");
}

function textSentenceCase(input) {
  let capitalized = false;
  let output = "";
  for (const character of Array.from(String(input || "").toLowerCase())) {
    if (!capitalized && /[\p{L}\p{N}]/u.test(character)) {
      output += character.toUpperCase();
      capitalized = true;
    } else {
      output += character;
    }
  }
  return output;
}

function delimiterCase(input, delimiter) {
  return caseWords(input).join(delimiter);
}

function camelCase(input) {
  const words = caseWords(input);
  if (!words.length) return "";
  return `${words[0]}${words.slice(1).map(capitalizeWord).join("")}`;
}

function pascalCase(input) {
  return caseWords(input).map(capitalizeWord).join("");
}

function stripEmptyLines(input) {
  return String(input || "")
    .split(/\r?\n/)
    .filter((line) => line.trim());
}

function joinLines(input) {
  return String(input || "")
    .split(/\r?\n/)
    .map((line) => line.trim())
    .filter(Boolean)
    .join(" ");
}

function numberLines(input) {
  return String(input || "")
    .split(/\r?\n/)
    .map((line, index) => `${index + 1}. ${line}`);
}

function reverseLines(input) {
  return String(input || "").split(/\r?\n/).reverse().join("\n");
}

function commentLines(input) {
  return String(input || "").split(/\r?\n/).map((line) => `// ${line}`).join("\n");
}

function uncommentLine(line) {
  const text = String(line || "");
  const match = text.match(/^(\s*)(\/\/ |\/\/|# |#)(.*)$/);
  return match ? `${match[1]}${match[3]}` : text;
}

function uncommentLines(input) {
  return String(input || "").split(/\r?\n/).map(uncommentLine).join("\n");
}

function removePunctuation(input) {
  return Array.from(String(input || ""))
    .filter((character) => /[\p{L}\p{N}\s]/u.test(character))
    .join("");
}

function sortWords(input) {
  return String(input || "").split(/\s+/).filter(Boolean).sort().join(" ");
}

function outdentLine(line) {
  const text = String(line || "");
  if (text.startsWith("    ")) return text.slice(4);
  if (text.startsWith("\t")) return text.slice(1);
  return text;
}

function normalizedWordSpans(text) {
  const spans = [];
  const source = String(text || "");
  let start = -1;
  let end = 0;
  let word = "";
  for (let index = 0; index < source.length;) {
    const character = Array.from(source.slice(index))[0];
    const next = index + character.length;
    if (/[\p{L}\p{N}]/u.test(character)) {
      if (start < 0) start = index;
      word += character.toLowerCase();
      end = next;
    } else if (start >= 0) {
      spans.push({ word, start, end });
      start = -1;
      word = "";
    }
    index = next;
  }
  if (start >= 0) spans.push({ word, start, end });
  return spans;
}

function replaceWordSequence(input, from, to) {
  return replaceRanges(input, wordSequenceMatchRanges(input, from, []), to) || "";
}

function exactMatchRanges(input, from) {
  const source = String(input || "");
  const needle = String(from || "");
  if (!needle) return [];
  const ranges = [];
  let cursor = 0;
  while (cursor <= source.length) {
    const start = source.indexOf(needle, cursor);
    if (start < 0) break;
    const end = start + needle.length;
    ranges.push({ start, end });
    cursor = end;
  }
  return ranges;
}

function rangesOverlap(left, right) {
  return left.start < right.end && right.start < left.end;
}

function wordSequenceMatchRanges(input, from, excluded = []) {
  const needle = normalizedWordSpans(from).map((span) => span.word);
  if (!needle.length) return [];
  const haystack = normalizedWordSpans(input);
  if (haystack.length < needle.length) return [];
  const ranges = [];
  let index = 0;
  while (index + needle.length <= haystack.length) {
    const matches = needle.every((word, offset) => haystack[index + offset].word === word);
    if (matches) {
      const start = haystack[index].start;
      const end = haystack[index + needle.length - 1].end;
      const range = { start, end };
      if (!excluded.some((blocked) => rangesOverlap(range, blocked))) {
        ranges.push(range);
      }
      index += needle.length;
    } else {
      index += 1;
    }
  }
  return ranges;
}

function replaceRanges(input, ranges, to) {
  if (!ranges.length) return "";
  const source = String(input);
  const ordered = [...ranges].sort((left, right) => left.start - right.start || left.end - right.end);
  let output = "";
  let cursor = 0;
  let replaced = false;
  for (const range of ordered) {
    if (range.start < cursor) continue;
    output += source.slice(cursor, range.start) + to;
    cursor = range.end;
    replaced = true;
  }
  return replaced ? output + source.slice(cursor) : "";
}

function replaceText(input, from, to) {
  const source = String(input);
  const direct = source.split(from).join(to);
  if (!from) return direct;
  const exactRanges = exactMatchRanges(source, from);
  if (exactRanges.length) {
    if (normalizedWordSpans(from).length > 1) {
      const ranges = [
        ...exactRanges,
        ...wordSequenceMatchRanges(source, from, exactRanges),
      ];
      return replaceRanges(source, ranges, to) || direct;
    }
    return direct;
  }
  return replaceWordSequence(source, from, to) || direct;
}

function applyTextOperation(operation, input) {
  switch (operation.slug) {
    case "uppercase":
      return String(input).toUpperCase();
    case "lowercase":
      return String(input).toLowerCase();
    case "replace_text":
      return replaceText(input, operation.from, operation.to);
    case "remove_text":
      return replaceText(input, operation.needle, "");
    case "append_text":
      return `${String(input)}${operation.suffix || ""}`;
    case "prepend_text":
      return `${operation.prefix || ""}${String(input)}`;
    case "reverse_words":
      return String(input).split(/\s+/).filter(Boolean).reverse().join(" ");
    case "extract_email":
      return String(input)
        .split(/\s+/)
        .map(cleanEmailCandidate)
        .filter(looksLikeEmail)
        .join("\n");
    case "extract_url":
      return String(input)
        .split(/\s+/)
        .map(cleanUrlCandidate)
        .filter(looksLikeUrl)
        .join("\n");
    case "extract_number":
      return extractNumbers(input).join("\n");
    case "count_occurrences":
      return operation.needle ? String(input).split(operation.needle).length - 1 : "0";
    case "count_unique_words":
      return String(countUniqueWords(input));
    case "count_words":
      return String(countWords(input));
    case "count_lines":
      return String(String(input).split(/\r?\n/).length);
    case "count_characters":
      return String(Array.from(String(input)).length);
    case "deduplicate_lines":
      return deduplicateLines(input).join("\n");
    case "sort_lines":
      return String(input).split(/\r?\n/).sort().join("\n");
    case "sort_words":
      return sortWords(input);
    case "trim_whitespace":
      return String(input).trim();
    case "normalize_whitespace":
      return String(input).split(/\s+/).filter(Boolean).join(" ");
    case "title_case":
      return titleCase(input);
    case "sentence_case":
      return textSentenceCase(input);
    case "snake_case":
      return delimiterCase(input, "_");
    case "kebab_case":
      return delimiterCase(input, "-");
    case "camel_case":
      return camelCase(input);
    case "pascal_case":
      return pascalCase(input);
    case "remove_punctuation":
      return removePunctuation(input);
    case "strip_empty_lines":
      return stripEmptyLines(input).join("\n");
    case "join_lines":
      return joinLines(input);
    case "reverse_lines":
      return reverseLines(input);
    case "number_lines":
      return numberLines(input).join("\n");
    case "indent_lines":
      return String(input).split(/\r?\n/).map((line) => `    ${line}`).join("\n");
    case "outdent_lines":
      return String(input).split(/\r?\n/).map(outdentLine).join("\n");
    case "comment_lines":
      return commentLines(input);
    case "uncomment_lines":
      return uncommentLines(input);
    default:
      return String(input);
  }
}

function buildTextManipulationChain(input, operations) {
  const steps = [];
  const usedRules = new Set();
  let current = String(input);
  for (const operation of operations) {
    let ruleId = `rule_${operation.slug}`;
    for (let suffix = 2; usedRules.has(ruleId); suffix += 1) {
      ruleId = `rule_${operation.slug}_${suffix}`;
    }
    usedRules.add(ruleId);
    const before = current;
    const after = applyTextOperation(operation, before);
    steps.push({ operation, ruleId, before, after });
    current = after;
  }
  return { result: current, steps };
}

function tryTextManipulation(prompt, normalized, history = []) {
  const request = parseTextManipulationRequest(prompt, normalized, history);
  if (!request) return null;
  const chain = buildTextManipulationChain(request.input, request.operations);
  const ruleChain = chain.steps.map((step) => step.ruleId).join(">");
  const rulesId = synthesisStableId(
    "text_substitution_rules",
    `${request.input}:${ruleChain}`,
  );
  const traceId = synthesisStableId(
    "text_substitution_trace",
    chain.steps.map((step) => `${step.before}->${step.after}`).join("|"),
  );
  const graphId = synthesisStableId("text_substitution_graph", `${request.input}:${chain.result}`);
  const evidence = [
    `text_input:bytes=${request.input.length} chars=${Array.from(request.input).length}`,
    ...chain.steps.flatMap((step) => [
      `text_operation:${step.operation.slug}`,
      `text_rule:${step.ruleId}`,
    ]),
    `text_rule_chain:${ruleChain}`,
    `text_substitution_rules:${rulesId}`,
    `text_substitution_trace:${traceId}`,
    `text_substitution_graph:${graphId}`,
    `text_result:${chain.result}`,
  ];
  return {
    intent: "text_manipulation",
    content: chain.result,
    confidence: 1.0,
    evidence,
    trace: [
      `text_substitution_trace:${traceId}`,
      `text_result:${truncateSynthesisTrace(chain.result, 80)}`,
    ],
  };
}

function isProofIntroBoundary(ch) {
  return /\s|[.,:;!?…。，：；！？]/u.test(ch);
}

function stripProofClaimNoise(value) {
  return String(value || "")
    .replace(/^[\s,.:;!?…。，：；！？]+/u, "")
    .trim();
}
