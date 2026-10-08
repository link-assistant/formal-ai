// Browser twin of the issue #1177 shell-command composer. Mirrors
// `handle_shell_command_compose` in
// rust/src/solver_handlers/shell_command_compose.rs: action-verb and
// file-context cues come from data/seed/code-task-cues.lino, and every emitted
// flag is explained from the manual-page table in data/seed/manual-pages.lino.
// Nothing is executed (the browser worker has no shell).

/**
 * Other code-task intents: when one of their cues matches, the shell composer
 * steps aside (Rust `OTHER_INTENTS`).
 * @returns {Array<string>} intent slugs
 */
function shellComposeOtherIntents() {
  return [
    "code_debugging",
    "regex_synthesis",
    "sql_synthesis",
    "code_explanation",
    "code_review",
    "test_generation",
    "code_refactoring",
    "format_conversion",
  ];
}

/**
 * The words the `shell_cue` map of data/seed/code-task-cues.lino reads in one
 * role, in seed order (Rust `role_words`).
 * @param {string} role cue role
 * @returns {Array<string>} words
 */
function shellComposeRoleWords(role) {
  const out = [];
  for (const entry of codeTaskWordEntries("shell_cue")) {
    if (codeTaskChildValue(entry, "value") === role) out.push(codeTaskChildValue(entry, "word"));
  }
  return out;
}

/**
 * The first raw word after the first seeded lead phrase of a role, or null.
 * @param {string} prompt raw prompt
 * @param {string} role lead role
 * @returns {string|null} the raw word (case kept)
 */
function shellComposeWordAfterLead(prompt, role) {
  const lower = prompt.toLowerCase();
  for (const lead of shellComposeRoleWords(role)) {
    const at = lower.indexOf(lead + " ");
    if (at === -1) continue;
    const words = codeTaskWords(prompt.slice(at + lead.length + 1));
    return words.length > 0 ? words[0] : "";
  }
  return null;
}

/**
 * The `command` records of data/seed/manual-pages.lino.
 * @returns {Array<object>} {name, packageName, url, flags} records
 */
function shellComposeManualPages() {
  const out = [];
  for (const record of codeTaskSeedRecords("manual-pages.lino")) {
    if (record.name !== "command") continue;
    const name = codeTaskChildValue(record, "name");
    if (name === "") continue;
    const body = record.children.length > 0 && record.children[0].name === "name" ? record.children[0] : null;
    if (body === null) continue;
    const flags = [];
    for (const flag of codeTaskChildren(body, "flag")) {
      flags.push({ spelling: codeTaskChildValue(flag, "spelling"), meaning: codeTaskChildValue(flag, "meaning") });
    }
    out.push({
      name: name,
      packageName: codeTaskChildValue(body, "package"),
      url: codeTaskChildValue(body, "url"),
      flags: flags,
    });
  }
  return out;
}

/**
 * The manual record for one command, or null.
 * @param {string} name command name
 * @returns {object|null} the manual record
 */
function shellComposeManual(name) {
  for (const command of shellComposeManualPages()) {
    if (command.name === name) return command;
  }
  return null;
}

/**
 * The manual meaning rows for the flags a composition used.
 * @param {object} manual manual record
 * @param {Array<string>} used flag spellings
 * @returns {Array<Array<string>>} [flag, meaning] rows
 */
function shellComposeExplainedFlags(manual, used) {
  const out = [];
  for (const flag of manual.flags) {
    if (used.includes(flag.spelling)) out.push([flag.spelling, flag.meaning]);
  }
  return out;
}

/**
 * Index of the first token in the list, or -1.
 * @param {Array<string>} tokens request tokens
 * @param {Array<string>} candidates accepted tokens
 * @returns {number} index
 */
function shellComposeFind(tokens, candidates) {
  for (let index = 0; index < tokens.length; index += 1) {
    if (candidates.includes(tokens[index])) return index;
  }
  return -1;
}

/**
 * The search root: the first path-like word, defaulting to ".".
 * @param {string} prompt raw prompt
 * @returns {string} path
 */
function shellComposeRootPath(prompt) {
  for (const word of codeTaskWords(prompt)) {
    const trimmed = codeTaskTrimMatches(word, function (c) {
      return !codeTaskIsAlphanumeric(c) && c !== "/" && c !== "~";
    });
    if (trimmed.startsWith("/") || trimmed.startsWith("~/")) return trimmed;
    // A relative path names its segments around an inner slash ("data/meta").
    const segments = trimmed.split("/");
    if (segments.length > 1 && segments.every((segment) => segment !== "")) return trimmed;
  }
  return ".";
}

/**
 * A file argument: a word with an inner dot ("app.log").
 * @param {string} prompt raw prompt
 * @returns {string|null} file name
 */
function shellComposeFileArgument(prompt) {
  for (const word of codeTaskWords(prompt)) {
    const trimmed = codeTaskTrimMatches(word, function (c) {
      return !codeTaskIsAlphanumeric(c) && c !== "." && c !== "-" && c !== "_";
    });
    if (trimmed.indexOf(".") > 0) return trimmed;
  }
  return null;
}

/**
 * The `-name` pattern: a raw ".ext" mention, or "<ext word> files".
 * @param {string} prompt raw prompt
 * @param {Array<string>} tokens request tokens
 * @returns {string|null} shell pattern
 */
function shellComposeNamePattern(prompt, tokens) {
  // "ending in .lino" / "ending with _test.go": the suffix itself.
  const suffix = codeTaskTrimMatches(shellComposeWordAfterLead(prompt, "suffix_lead") || "", function (c) {
    return !codeTaskIsAlphanumeric(c) && c !== "." && c !== "_" && c !== "-";
  });
  if (suffix !== "") return "*" + suffix;
  for (const word of codeTaskWords(prompt)) {
    const trimmed = codeTaskTrimMatches(word, function (c) { return !codeTaskIsAlphanumeric(c) && c !== "."; });
    if (!trimmed.startsWith(".")) continue;
    const ext = trimmed.slice(1);
    if (ext !== "" && Array.from(ext).every(codeTaskIsAlphanumeric)) return "*." + ext;
  }
  const extensions = codeTaskWordEntries("extension");
  for (let index = 1; index < tokens.length; index += 1) {
    if (!shellComposeRoleWords("files_noun").includes(tokens[index])) continue;
    const entry = codeTaskEntryFor(extensions, tokens[index - 1]);
    if (entry !== null) return codeTaskChildValue(entry, "value");
  }
  return null;
}

/**
 * The size test: a number with a unit word, qualified by a larger-than word.
 * @param {Array<string>} tokens request tokens
 * @returns {object|null} {count, unit, from, to}
 */
function shellComposeSizeTest(tokens) {
  const qualifierAt = shellComposeFind(tokens, shellComposeRoleWords("size_qualifier"));
  if (qualifierAt === -1) return null;
  const units = codeTaskWordEntries("size_unit");
  const numbers = codeTaskWordEntries("number");
  for (let index = 0; index < tokens.length; index += 1) {
    const token = tokens[index];
    const value = codeTaskNumberValue(token, numbers);
    if (value !== null && index + 1 < tokens.length) {
      const unit = codeTaskEntryFor(units, tokens[index + 1]);
      if (unit !== null) {
        return { count: value, unit: codeTaskChildValue(unit, "value"), from: qualifierAt, to: index + 1 };
      }
    }
    const digits = /^[0-9]*/.exec(token)[0];
    if (digits !== "") {
      const unit = codeTaskEntryFor(units, token.slice(digits.length));
      const joined = codeTaskParseU32(digits);
      if (unit !== null && joined !== null) {
        return { count: joined, unit: codeTaskChildValue(unit, "value"), from: qualifierAt, to: index };
      }
    }
  }
  return null;
}

/**
 * The mtime test: "older than 5 days" → +5, "in the last 3 days" → -3.
 * @param {Array<string>} tokens request tokens
 * @returns {string|null} signed day count
 */
function shellComposeMtimeTest(tokens) {
  const daysAt = shellComposeFind(tokens, shellComposeRoleWords("day"));
  if (daysAt < 1) return null;
  const count = codeTaskNumberValue(tokens[daysAt - 1], codeTaskWordEntries("number"));
  if (count === null) return null;
  const older = shellComposeFind(tokens, shellComposeRoleWords("older")) !== -1;
  return (older ? "+" : "-") + count;
}

/**
 * Build one composed command record.
 * @param {string} command composed command
 * @param {object} manual manual record
 * @param {Array<Array<string>>} rows mapping rows
 * @param {Array<Array<string>>} explained flag rows
 * @returns {object} the composition
 */
function shellComposeComposed(command, manual, rows, explained) {
  return { command: command, packageName: manual.packageName, url: manual.url, rows: rows, explained: explained, pipe: null };
}

/**
 * Escape the characters a basic regular expression (or, with `&`, a sed
 * replacement) reads specially (Rust `sed_literal`).
 * @param {string} word the literal word
 * @param {Array<string>} special characters to escape
 * @returns {string} escaped word
 */
function shellComposeSedLiteral(word, special) {
  let out = "";
  for (const c of word) out += (special.includes(c) ? "\\" : "") + c;
  return out;
}

/**
 * "replace foo with bar in config.txt" → `sed -i 's/foo/bar/g' config.txt`
 * from the seeded `substitute` / `substitute_with` role words (Rust
 * `sed_substitution`).
 * @param {string} prompt raw prompt
 * @returns {object|null} the composition
 */
function shellComposeSedSubstitution(prompt) {
  const raw = prompt.split(/\s+/u).filter((word) => word !== "");
  const bare = (word) => word.replace(/^['"`,?!]+|['"`,?!]+$/gu, "");
  const lower = raw.map((word) => bare(word).toLowerCase());
  const cueAt = shellComposeFind(lower, shellComposeRoleWords("substitute"));
  if (cueAt === -1) return null;
  const withWords = shellComposeRoleWords("substitute_with");
  let withAt = -1;
  for (let index = cueAt + 2; index < lower.length && withAt === -1; index += 1) {
    if (withWords.includes(lower[index])) withAt = index;
  }
  if (withAt === -1 || withAt + 1 >= raw.length) return null;
  const pattern = bare(raw[withAt - 1]);
  const replacement = bare(raw[withAt + 1]);
  const file = shellComposeFileArgument(raw.slice(withAt + 2).join(" "));
  if (file === null || pattern === "" || replacement === "" || /[/']/u.test(pattern + replacement)) return null;
  const manual = shellComposeManual("sed");
  if (manual === null) return null;
  const expression = "s/" + shellComposeSedLiteral(pattern, [".", "*", "[", "]", "^", "$", "\\"]) + "/" +
    shellComposeSedLiteral(replacement, ["&", "\\"]) + "/g";
  return shellComposeComposed("sed -i '" + expression + "' " + file, manual,
    [[raw.slice(cueAt, withAt + 2).join(" "), expression], [file, file]],
    shellComposeExplainedFlags(manual, ["-i", "s/regexp/replacement/g"]));
}

/**
 * A counting request ("count .lino files") pipes the composition into
 * `wc -l`, explained from wc's manual record (Rust `counted`).
 * @param {Array<string>} tokens request tokens
 * @param {object|null} composed the composition
 * @returns {object|null} the composition, piped when counted
 */
function shellComposeCounted(tokens, composed) {
  if (composed === null) return null;
  const countAt = shellComposeFind(tokens, shellComposeRoleWords("count"));
  const manual = shellComposeManual("wc");
  if (countAt === -1 || manual === null) return composed;
  composed.command += " | wc -l";
  composed.rows.push([codeTaskEcho(tokens, countAt, countAt), "| wc -l"]);
  composed.pipe = { tool: "wc", manual: manual, explained: shellComposeExplainedFlags(manual, ["-l"]) };
  return composed;
}

/**
 * Render explained flag rows through the shared `flag_line` template.
 * @param {Array<Array<string>>} explained [flag, meaning] rows
 * @returns {string} the concatenated lines
 */
function shellComposeFlagLines(explained) {
  let out = "";
  for (const row of explained) {
    out += codeTaskTemplate("flag_line", [["flag", row[0]], ["meaning", row[1]]]);
  }
  return out;
}

/**
 * "last/first N lines of FILE" → head/tail.
 * @param {Array<string>} tokens request tokens
 * @param {string} prompt raw prompt
 * @returns {object|null} the composition
 */
function shellComposeLineSlice(tokens, prompt) {
  const lastWords = shellComposeRoleWords("last");
  const firstWords = shellComposeRoleWords("first");
  const last = shellComposeFind(tokens, lastWords) !== -1;
  const first = shellComposeFind(tokens, firstWords) !== -1;
  if (!last && !first) return null;
  const linesAt = shellComposeFind(tokens, shellComposeRoleWords("line"));
  if (linesAt < 1) return null;
  const count = codeTaskNumberValue(tokens[linesAt - 1], codeTaskWordEntries("number"));
  if (count === null) return null;
  const directionAt = shellComposeFind(tokens, last ? lastWords : firstWords);
  const tool = last ? "tail" : "head";
  const file = shellComposeFileArgument(prompt);
  if (file === null) return null;
  const manual = shellComposeManual(tool);
  if (manual === null) return null;
  let meaning = "";
  for (const flag of manual.flags) {
    if (flag.spelling === "-n N") {
      meaning = flag.meaning;
      break;
    }
  }
  return shellComposeComposed(tool + " -n " + count + " " + file, manual, [
    [codeTaskEcho(tokens, directionAt, linesAt), tool + " -n " + count],
    [file, file],
  ], [["-n N", meaning]]);
}

/**
 * "search for X" / "files containing X" → grep.
 * @param {Array<string>} tokens request tokens
 * @param {string} prompt raw prompt
 * @returns {object|null} the composition
 */
function shellComposeGrepSearch(tokens, prompt) {
  let cueAt = shellComposeFind(tokens, shellComposeRoleWords("containment"));
  if (cueAt === -1 && shellComposeFind(tokens, shellComposeRoleWords("search_object")) !== -1) {
    cueAt = shellComposeFind(tokens, shellComposeRoleWords("search"));
  }
  if (cueAt === -1) return null;
  const pattern = codeTaskTrimMatches(shellComposeWordAfterLead(prompt, "pattern_lead") || "", function (c) {
    return !codeTaskIsAlphanumeric(c) && c !== "_" && c !== "-";
  });
  if (pattern === "") return null;
  const path = shellComposeRootPath(prompt);
  const flags = ["-r"];
  if (shellComposeFind(tokens, shellComposeRoleWords("ignore_case")) !== -1) flags.push("-i");
  if (shellComposeFind(tokens, shellComposeRoleWords("names_only")) !== -1) flags.push("-l");
  const manual = shellComposeManual("grep");
  if (manual === null) return null;
  return shellComposeComposed("grep " + flags.join(" ") + " '" + pattern + "' " + path, manual, [
    [codeTaskEcho(tokens, cueAt, cueAt), "grep(1)"],
    [pattern, "'" + pattern + "'"],
    [path, path],
  ], shellComposeExplainedFlags(manual, flags));
}

/**
 * The find composition: path + -name + -size + -mtime + -type.
 * @param {Array<string>} tokens request tokens
 * @param {string} prompt raw prompt
 * @returns {object|null} the composition
 */
function shellComposeFindFiles(tokens, prompt) {
  const pattern = shellComposeNamePattern(prompt, tokens);
  const size = shellComposeSizeTest(tokens);
  const mtime = shellComposeMtimeTest(tokens);
  const directoriesAt = shellComposeFind(tokens, shellComposeRoleWords("directory"));
  if (pattern === null && size === null && mtime === null && directoriesAt === -1) return null;
  const path = shellComposeRootPath(prompt);
  const parts = ["find " + path];
  const rows = [[path, "find " + path]];
  const used = [];
  if (pattern !== null) {
    parts.push("-name '" + pattern + "'");
    rows.push([pattern, "-name '" + pattern + "'"]);
    used.push("-name pattern");
  }
  if (size !== null) {
    const test = "-size +" + size.count + size.unit;
    parts.push(test);
    rows.push([codeTaskEcho(tokens, size.from, size.to), test]);
    used.push("-size +N[kMG]");
  }
  if (mtime !== null) {
    parts.push("-mtime " + mtime);
    rows.push([mtime, "-mtime " + mtime]);
    used.push("-mtime N");
  }
  if (directoriesAt !== -1) {
    parts.push("-type d");
    rows.push([codeTaskEcho(tokens, directoriesAt, directoriesAt), "-type d"]);
    used.push("-type f");
  }
  const manual = shellComposeManual("find");
  if (manual === null) return null;
  return shellComposeComposed(parts.join(" "), manual, rows, shellComposeExplainedFlags(manual, used));
}

/**
 * The ls composition: long/hidden/recursive flags.
 * @param {Array<string>} tokens request tokens
 * @param {string} prompt raw prompt
 * @returns {object|null} the composition
 */
function shellComposeLsListing(tokens, prompt) {
  const flags = [];
  const rows = [];
  const longAt = shellComposeFind(tokens, shellComposeRoleWords("long"));
  if (longAt !== -1) {
    flags.push("-l");
    rows.push([codeTaskEcho(tokens, longAt, longAt), "-l"]);
  }
  let hiddenAt = shellComposeFind(tokens, shellComposeRoleWords("hidden"));
  if (hiddenAt === -1) hiddenAt = shellComposeFind(tokens, shellComposeRoleWords("all"));
  if (hiddenAt !== -1) {
    flags.push("-a");
    rows.push([codeTaskEcho(tokens, hiddenAt, hiddenAt), "-a"]);
  }
  const recursiveAt = shellComposeFind(tokens, shellComposeRoleWords("recursive"));
  if (recursiveAt !== -1) {
    flags.push("-R");
    rows.push([codeTaskEcho(tokens, recursiveAt, recursiveAt), "-R"]);
  }
  const path = shellComposeRootPath(prompt);
  const manual = shellComposeManual("ls");
  if (manual === null) return null;
  rows.push([path, path]);
  const flagList = flags.length === 0 ? "" : flags.join(" ") + " ";
  return shellComposeComposed("ls " + flagList + path, manual, rows, shellComposeExplainedFlags(manual, flags));
}

/**
 * Compose a shell command from the manual-page flag table.
 * Mirrors `handle_shell_command_compose`.
 * @param {string} prompt raw prompt
 * @param {string} normalized normalized prompt
 * @returns {object|null} the worker answer, or null
 */
function handleShellCommandCompose(prompt, normalized) {
  const tokens = codeTaskTokens(normalized);
  const actions = codeTaskCuePhrases("shell_command_compose", "action");
  if (isAgentTextRequest(prompt.toLowerCase())) return null; // an agent opt-in is the agent flow's
  // A complete substitution request is its own cue (Rust `sed_substitution`).
  const substitution = shellComposeSedSubstitution(prompt);
  if (substitution === null && !tokens.some(function (token) { return actions.includes(token); })) return null;
  const fileContext = codeTaskMapWords("file_context");
  if (substitution === null && !tokens.some(function (token) { return fileContext.includes(token); })) return null;
  // A requested function/program is program synthesis, not a shell command.
  if (shellComposeFind(tokens, shellComposeRoleWords("program_artifact")) !== -1) return null;
  for (const intent of shellComposeOtherIntents()) {
    if (codeTaskAnyCueMatches(intent, prompt, normalized)) return null;
  }
  // An inline text payload under a seeded text operation is the text
  // handler's, not a filesystem command (Rust `names_text_operation`).
  if (parseTextManipulationRequest(normalized, normalized) !== null) return null;
  const log = codeTaskLog();
  codeTaskLogAppend(log, "shell_command_compose:request", "action + file context");
  let composed = substitution === null ? shellComposeLineSlice(tokens, prompt) : substitution;
  if (composed === null) composed = shellComposeGrepSearch(tokens, prompt);
  if (composed === null) composed = shellComposeCounted(tokens, shellComposeFindFiles(tokens, prompt));
  if (composed === null) composed = shellComposeLsListing(tokens, prompt);
  if (composed === null) {
    codeTaskLogAppend(log, "shell_command_compose:refusal", "no manual-page entry covers the request");
    return codeTaskAnswer(log, "shell_command_compose", "response:shell_command_compose",
      codeTaskTemplate("shell_compose_refusal", []), 0.4);
  }
  codeTaskLogAppend(log, "shell_command_compose:command", composed.command);
  const pipe = composed.pipe;
  const body = codeTaskTemplate(pipe === null ? "shell_command_composed" : "shell_command_piped", [
    ["command", composed.command],
    ["package", composed.packageName],
    ["flags", shellComposeFlagLines(composed.explained)],
    ["manual", composed.url],
    ["pipe_tool", pipe === null ? "" : pipe.tool],
    ["pipe_package", pipe === null ? "" : pipe.manual.packageName],
    ["pipe_flags", pipe === null ? "" : shellComposeFlagLines(pipe.explained)],
    ["pipe_manual", pipe === null ? "" : pipe.manual.url],
    ["derivation", codeTaskMappingRows(composed.rows)],
  ]);
  return codeTaskAnswer(log, "shell_command_compose", "response:shell_command_compose", body, 0.7);
}

/**
 * Browser binding for the `shell_command_compose` precedence row.
 * @param {string} prompt raw prompt
 * @returns {object|null} the worker answer, or null
 */
function tryShellCommandCompose(prompt) {
  return handleShellCommandCompose(prompt, prompt.toLowerCase());
}
