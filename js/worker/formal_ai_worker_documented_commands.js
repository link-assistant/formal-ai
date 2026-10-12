// Issue #1165 R1165-6: no catalog language row states a check or run command
// that a captured documentation page states for it. The policy seed's
// `command_procedure` rows (data/seed/program-cache-policy.lino) name, per
// language and role, the captured page and the command_verb its command line
// starts with; installDocumentedLanguageCommands fills WRITE_PROGRAM_LANGUAGES from
// those pages whenever the seed bundle is installed, so the browser worker and
// the JavaScript server read the same data the Rust catalog does. The Rust
// twins are `documented_language_commands` in
// rust/src/discovery_production_contract.rs and `program_languages` in
// rust/src/coding/catalog/mod.rs.

const DOCUMENTED_COMMANDS_POLICY_FILE = "program-cache-policy.lino";
const DOCUMENTED_COMMANDS_CAPTURES_FILE = "coding-documentation-captures.lino";

/**
 * Every `command` row of the policy seed's `command_procedure` record, in
 * seed order. Mirrors `command_procedures`.
 * @param {object} raw the seed bundle's raw files
 * @returns {Array<{language: string, role: string, page: string, commandVerb: string}>}
 */
function commandProcedures(raw) {
  const root = parseLinoTree(seedRawText(raw, DOCUMENTED_COMMANDS_POLICY_FILE)).children[0];
  const record = (root ? root.children : []).find((node) => node.name === "command_procedure");
  return (record ? record.children : []).filter((node) => node.name === "command").map((node) => ({
    language: childValue(node, "language"),
    role: childValue(node, "role"),
    page: childValue(node, "page"),
    commandVerb: childValue(node, "command_verb"),
  }));
}

/**
 * The captures of one page for one language, shaped as
 * documentationCommandLines reads them.
 * @param {object} raw
 * @param {string} language
 * @param {string} page
 * @returns {Array<{url: string, blocks: Array<{language: string, text: string}>}>}
 */
function documentedCommandPage(raw, language, page) {
  const root = parseLinoTree(seedRawText(raw, DOCUMENTED_COMMANDS_CAPTURES_FILE)).children
    .find((node) => node.name === "coding_documentation_captures");
  return (root ? root.children : [])
    .filter((node) => node.name === "capture" && childValue(node, "language") === language
      && childValue(node, "url") === page)
    .map((node) => ({
      url: page,
      blocks: node.children.filter((child) => child.name === "block")
        .map((block) => ({ language: childValue(block, "language"), text: childValue(block, "code") })),
    }));
}

/**
 * `word` with the documented file name `from` bound to `to`, when the word is
 * that name or that name followed by a dot. Mirrors `bind_documented_name`.
 */
function bindDocumentedName(word, from, to) {
  if (!from || from === to) return word;
  if (word === from) return to;
  return word.startsWith(from + ".") ? to + word.slice(from.length) : word;
}

/**
 * The check and run commands a captured page states for a catalog row, each
 * with the page as its source (R1165-6). Each `command_procedure` row of the
 * language takes the first line of its page's code blocks, the shell prompt
 * removed, whose first word is the row's command_verb; the name the page gives its
 * source file (the stem of the first word across those lines that ends with
 * the extension of `saveAs`) is bound to the stem of `saveAs`. Mirrors
 * `documented_language_commands`.
 * @param {string} language
 * @param {string} saveAs
 * @param {object} [raw] the seed bundle's raw files (SEED_RAW by default)
 * @returns {Array<{role: string, command: string, source: string}>}
 */
function documentedLanguageCommands(language, saveAs, raw) {
  const seeds = raw || SEED_RAW;
  const stated = [];
  for (const row of commandProcedures(seeds)) {
    if (row.language !== language || (row.role !== "check" && row.role !== "run")) continue;
    const found = documentationCommandLines(documentedCommandPage(seeds, language, row.page))
      .find((entry) => entry.line.split(/\s+/)[0] === row.commandVerb);
    if (found) stated.push({ role: row.role, line: found.line, url: found.url });
  }
  const file = String(saveAs || "").split("/").pop();
  const extension = file.includes(".") ? file.slice(file.lastIndexOf(".")) : "";
  const named = stated.flatMap((entry) => entry.line.split(/\s+/))
    .find((word) => extension && word.endsWith(extension) && word.length > extension.length);
  const documented = named ? named.slice(0, named.length - extension.length) : "";
  const stem = codeExampleFileStem(saveAs);
  return stated.map((entry) => ({
    role: entry.role,
    command: entry.line.split(/\s+/).map((word) => bindDocumentedName(word, documented, stem)).join(" "),
    source: entry.url,
  }));
}

/**
 * Fill every WRITE_PROGRAM_LANGUAGES row's documented commands from the seed
 * bundle (R1165-6), as `program_languages` resolves the Rust rows. Idempotent.
 * @param {object} raw the seed bundle's raw files
 * @returns {Array<string>} `<language>:<role>` for every command filled
 */
function installDocumentedLanguageCommands(raw) {
  if (typeof WRITE_PROGRAM_LANGUAGES !== "object") return [];
  const filled = [];
  for (const [slug, row] of Object.entries(WRITE_PROGRAM_LANGUAGES)) {
    for (const command of documentedLanguageCommands(slug, row.saveAs, raw)) {
      if (command.role === "check") row.checkCommand = command.command;
      else row.runCommand = command.command;
      filled.push(slug + ":" + command.role);
    }
  }
  return filled;
}
