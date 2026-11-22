// Match command procedure declarations against explicitly acquired publisher pages.

const DOCUMENTED_COMMANDS_POLICY_FILE = "program-cache-policy.lino";

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
    role: childValue(node, "role") || childValue(record, "role"),
    page: childValue(node, "page"),
    commandVerb: childValue(node, "command_verb"),
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
 * @param {Array<object>} captures
 * @param {object} [raw] the seed bundle's raw files (SEED_RAW by default)
 * @returns {Array<{role: string, command: string, source: string}>}
 */
function documentedLanguageCommands(language, saveAs, captures, raw) {
  const seeds = raw || SEED_RAW;
  const stated = [];
  for (const row of commandProcedures(seeds)) {
    if (row.language !== language || (row.role !== "check" && row.role !== "run")) continue;
    const found = documentationCommandLines(captures.filter((capture) => capture.url === row.page))
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
