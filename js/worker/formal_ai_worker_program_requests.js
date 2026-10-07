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
    WRITE_PROGRAM_TASKS[slug] = { label: label, output: childValue(record, "output") || "", input: childValue(record, "input") || "" };
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
