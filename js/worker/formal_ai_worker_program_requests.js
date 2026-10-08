// Browser twins of the issue #1021 write_program generalizations.
//
// JavaScript is the priority implementation (2026-10-06 directive); each
// function below mirrors the Rust code named in its comment, and stays inside
// link-foundation/meta-language's portable subset (plain top-level functions,
// `const`/`let`, no classes).
//
// Three rules live here, each read from seed data rather than written as a
// phrase list:
//   - every catalog task is installed, with its label, its templates, its
//     output and the input it was verified against, from
//     data/seed/hello-world-programs.lino (R1021-6/R1021-7, and the full
//     task x language set of the Rust catalog, R921-8);
//   - a task that reads standard input pipes that input into the run command a
//     reader is told to type (`ProgramTemplate::run_command_line`);
//   - a request that names code as its artefact and nothing else is a
//     write_program request with no parameters, whatever the asking verb
//     (R1021-31, `names_code_and_nothing_else`).

const PROGRAM_SEED_FILE = "hello-world-programs.lino";

/**
 * Install the catalog's tasks and programs from the seed: each task's label,
 * expected output and stdin fixture, and the program of every pair the seed
 * stores one for, in the seed's (the Rust table's) order. The seed bundle is
 * the one the Rust catalog tests hold equal to the compiled tables
 * (rust/tests/source/source_tests/coding/catalog/mod/lino_parity.rs), so the
 * worker answers every task in every language the Rust catalog
 * (rust/src/coding/catalog/{tasks,templates_*}.rs) does, and no table is
 * written per language here. A row whose `program_source` is the
 * documentation route stores no program; writeProgramTemplate rediscovers it.
 * A label the seed row omits falls back to the first surface of its
 * `program_task_<slug>` meaning. Idempotent.
 * @param {object} raw the seed bundle's raw files
 * @returns {Array<string>} the task slugs installed
 */
function installSeedProgramTasks(raw) {
  const text = seedRawText(raw, PROGRAM_SEED_FILE);
  if (!text || typeof WRITE_PROGRAM_TASKS !== "object") return [];
  const records = parseLinoTree(text).children;
  const installed = [];
  for (const record of records) {
    const slug = childValue(record, "task");
    if (!record.name.startsWith("task_") || !slug) continue;
    const label = childValue(record, "label") || wordsForMeaning(`program_task_${slug}`)[0] || slug.split("_").join(" ");
    WRITE_PROGRAM_TASKS[slug] = { label: label, output: childValue(record, "output") || "", input: childValue(record, "input") || "",
      procedure: childValue(record, "procedure") || "" };
    installed.push(slug);
  }
  for (const record of records) {
    const slug = childValue(record, "task");
    const code = childValue(record, "code");
    if (!record.name.startsWith("template_") || !installed.includes(slug) || !code) continue;
    WRITE_PROGRAM_TEMPLATES[slug] = WRITE_PROGRAM_TEMPLATES[slug] || {};
    WRITE_PROGRAM_TEMPLATES[slug][childValue(record, "language")] = code;
  }
  return installed;
}

/**
 * `input` with the characters a single-quoted `printf` format cannot carry
 * written as the escapes `printf` expands back into them. Mirrors
 * `shell_escaped` in rust/src/coding/catalog/types.rs.
 * @param {string} input
 * @returns {string}
 */
function programShellEscaped(input) {
  const escapes = { "\n": "\\n", "\t": "\\t", "\\": "\\\\", "%": "%%", "'": "'\\''" };
  return Array.from(String(input)).map((character) => escapes[character] || character).join("");
}

/**
 * The run command as a reader must actually type it: a task that reads
 * standard input has the fixture it was verified against piped in, so the
 * command is copy-pasteable instead of waiting on a terminal. Mirrors
 * `ProgramTemplate::run_command_line`.
 * @param {string|null} task
 * @param {string} runCommand
 * @returns {string}
 */
function programRunCommandLine(task, runCommand) {
  const taskInfo = task && typeof WRITE_PROGRAM_TASKS === "object" ? WRITE_PROGRAM_TASKS[task] : null;
  const input = taskInfo && taskInfo.input ? taskInfo.input : "";
  return input ? `printf '${programShellEscaped(input)}' | ${runCommand}` : runCommand;
}

/**
 * Does the request name code as its artefact and name nothing else? Every
 * surface the lexicon can account for (the asking verb, the artefact noun, the
 * closed-class words a request is built from) is subtracted, longest first,
 * and nothing may be left over. Mirrors `names_code_and_nothing_else` in
 * rust/src/intent_formalization/write_program_request.rs.
 * @param {string} normalized
 * @returns {boolean}
 */
function namesCodeAndNothingElse(normalized) {
  const namesArtefact =
    lexiconMentionsRole("script_or_code_artifact", normalized) || lexiconMentionsRole("program_genus", normalized);
  if (!lexiconMentionsRole("script_authoring_verb", normalized) || !namesArtefact) return false;
  const accounted = ["script_authoring_verb", "script_or_code_artifact", "program_genus", "request_function_word"]
    .flatMap((role) => wordsForRole(role))
    .filter((word) => word.trim() !== "")
    .sort((left, right) => Array.from(right).length - Array.from(left).length);
  let text = String(normalized);
  for (const phrase of accounted.filter((word) => word.includes(" "))) text = text.split(phrase).join(" ");
  const edge = /^[^\p{Alphabetic}\p{N}]+|[^\p{Alphabetic}\p{N}]+$/gu;
  return text
    .split(/\s+/)
    .map((token) => token.replace(edge, ""))
    .every((token) => token === "" || programTokenAccountedFor(token, accounted));
}

/**
 * Whether one token is accounted for. A Chinese token carries a whole clause,
 * so its surfaces are subtracted as substrings until nothing remains. Mirrors
 * `is_accounted_for`.
 * @param {string} token
 * @param {Array<string>} accounted
 * @returns {boolean}
 */
function programTokenAccountedFor(token, accounted) {
  if (accounted.includes(token)) return true;
  if (!containsCjk(token)) return false;
  let rest = token;
  let word = accounted.find((candidate) => containsCjk(candidate) && rest.includes(candidate));
  while (word && rest !== "") {
    rest = rest.split(word).join("");
    word = accounted.find((candidate) => containsCjk(candidate) && rest.includes(candidate));
  }
  return rest === "";
}

// Issue #1173 R1173-3: "a program that prints hello" names a catalog task whose
// output is the request's own operand. Its seed row (`print_text`) stores no
// program: it names the `procedure` it is composed from, and the program is
// that procedure rediscovered from its documentation with the operand bound
// into the output literal. Twins: rust/src/coding/operand_program.rs.

/** A word with its edge punctuation removed, lowercased. Mirrors `bare_word`. */
function programBareWord(word) {
  return String(word).replace(/^[^\p{Alphabetic}\p{N}]+|[^\p{Alphabetic}\p{N}]+$/gu, "").toLowerCase();
}

/**
 * The text a request asks its program to print: read around its first word
 * that evidences the `print_stdout` meaning. After it, a quoted literal that
 * opens right there, else the clause's words up to a seeded separator or a
 * sentence end, a trailing implementation-language span set aside, when they
 * are an utterance (programOperandUtterance). Before it (a verb-final request,
 * "जो नमस्ते प्रिंट करे"), a quoted literal or a greeting that ends right there.
 * Null when the request names no such text. Mirrors `program_task_operand`.
 * @param {string} prompt
 * @returns {string|null}
 */
function programTaskOperand(prompt) {
  const print = findMeaning("print_stdout");
  const words = String(prompt || "").split(/\s+/u).filter(Boolean);
  const at = print ? words.findIndex((word) => meaningEvidencedIn(print, programBareWord(word))) : -1;
  if (at < 0) return null;
  const word = words[at];
  const surface = containsCjk(word) ? print.words.find((candidate) => containsCjk(candidate) && word.includes(candidate)) : null;
  const cut = surface ? word.indexOf(surface) : word.length;
  const before = [...words.slice(0, at), surface ? word.slice(0, cut) : ""].join(" ").trim();
  const after = [surface ? word.slice(cut + surface.length) : "", ...words.slice(at + 1)].join(" ").trim();
  return programOperandAfter(after) ?? programOperandBefore(before);
}

/** A quoted literal is an operand when it is one non-empty line. */
function programOperandLiteral(text) {
  return text && !text.includes("\n") ? text : null;
}

/**
 * The operand that opens `after`, the text after the print word. Mirrors
 * `operand_after`.
 */
function programOperandAfter(after) {
  const quoted = quotedTextSpans(after)[0];
  // A second literal in the same sentence ("prints 'a' and 'b'") leaves the operand open.
  const sentence = quoted ? after.slice(quoted.end).split(/[\n.;\u3002]/u)[0] : "";
  if (quoted && quoted.start === 0) return quotedTextSpans(sentence).length === 0 ? programOperandLiteral(quoted.text) : null;
  const greetings = wordsForRole("social_greeting").slice().sort((left, right) => right.length - left.length);
  if (containsCjk(after.slice(0, 1))) return greetings.find((greeting) => containsCjk(greeting) && after.startsWith(greeting)) ?? null;
  const separators = wordsForRole("skill_procedure_clause_separator");
  const tokens = [];
  for (const word of after.split(/\s+/u).filter(Boolean)) {
    if (separators.includes(programBareWord(word))) break;
    tokens.push(word);
    if (/[.!?;:。！？।]$/u.test(word)) break;
  }
  const span = ["implementation_language_preposition", "implementation_language_noun", "program_language_alias"].flatMap(wordsForRole);
  while (tokens.length > 1 && span.includes(programBareWord(tokens[tokens.length - 1]))) tokens.pop();
  return programOperandUtterance(tokens.join(" "), greetings);
}

/**
 * The operand that ends `before`, the text before a verb-final print word.
 * Mirrors `operand_before`.
 */
function programOperandBefore(before) {
  const spans = quotedTextSpans(before);
  const last = spans[spans.length - 1];
  if (last && last.end === before.length) return programOperandLiteral(last.text);
  const tokens = before.split(/\s+/u).filter(Boolean);
  for (const greeting of wordsForRole("social_greeting").slice().sort((left, right) => right.length - left.length)) {
    if (containsCjk(greeting)) {
      if (before.endsWith(greeting)) return greeting;
      continue;
    }
    const size = greeting.split(" ").length;
    const tail = tokens.slice(-size);
    if (tail.length === size && tail.map(programBareWord).join(" ") === greeting) return tail.join(" ").replace(/^[^\p{Alphabetic}\p{N}]+|[^\p{Alphabetic}\p{N}]+$/gu, "");
  }
  return null;
}

/**
 * Whether `words` are an utterance to print rather than a description of a
 * value: an unquoted utterance (it opens with a capital and no word describes
 * a value, as `unquoted_utterance` reads a work obligation's output), else a
 * seeded greeting (`social_greeting`). Mirrors `operand_utterance`.
 */
function programOperandUtterance(words, greetings) {
  const output = words.replace(/[.,;:。।]+$/u, "");
  const lower = normalizePrompt(output.toLowerCase());
  const functionWords = wordsForRole("statement_function_word");
  const describes = output.split(/\s+/u).some((word) => functionWords.includes(programBareWord(word)))
    || lexiconMentionsRole("coding_structure", lower)
    || (lexiconMentionsRole("program_task_alias", lower) && !lexiconMentionsRole("social_greeting", lower));
  if (/^\p{Uppercase}/u.test(output) && !describes) return output;
  const bare = output.replace(/^[^\p{Alphabetic}\p{N}]+|[^\p{Alphabetic}\p{N}]+$/gu, "");
  return bare && greetings.includes(bare.toLowerCase()) ? bare : null;
}

/**
 * The catalog task a request names through its operand: the first task whose
 * seed row names a `procedure`, when the request names text to print. Mirrors
 * `operand_task`.
 * @param {string} prompt
 * @returns {string|null}
 */
function operandProgramTask(prompt) {
  if (programTaskOperand(prompt) === null) return null;
  return Object.keys(WRITE_PROGRAM_TASKS).find((slug) => WRITE_PROGRAM_TASKS[slug].procedure) || null;
}

/**
 * The write_program parameters of the request being answered: a request that
 * names no catalogued task but text to print names the operand task, unless
 * the minimal-script route answers it (writeScriptLanguage). Mirrors the
 * operand arm of `requested_write_program_parameters`.
 * @param {string} prompt
 * @returns {object|null}
 */
function requestedWriteProgramParameters(prompt) {
  const detected = writeProgramParameters(prompt);
  if (!detected || detected.task || writeScriptLanguage(prompt, normalizePrompt(prompt))) return detected;
  const task = operandProgramTask(prompt);
  return task ? { ...detected, task: task } : detected;
}

/**
 * The program for an operand task: its seed row's `procedure` rediscovered
 * from the documentation captures with the operand bound into the output
 * literal (rediscoverDocumentedProgram), or null when the task takes no
 * operand or no captured page verifies a program. Mirrors `operand_program`.
 * @param {string} task
 * @param {string} language
 * @param {string|null} operand
 */
function operandProgram(task, language, operand) {
  const taskInfo = task && typeof WRITE_PROGRAM_TASKS === "object" ? WRITE_PROGRAM_TASKS[task] : null;
  if (!taskInfo || !taskInfo.procedure || operand === null || !WRITE_PROGRAM_LANGUAGES[language]) return null;
  const documented = rediscoverDocumentedProgram(taskInfo.procedure, language, operand);
  return documented.recipe ? documented : null;
}
