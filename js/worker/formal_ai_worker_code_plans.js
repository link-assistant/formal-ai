// Browser twins of the remaining native-only coding/policy rows of the handler
// precedence seed: `algorithm`, `execution_failure`,
// `document_generation_plan`, `source_conflict` and `shell_refusal`. Each
// `handle*` function mirrors its Rust counterpart (named in the JSDoc); the
// `try*` bindings pass the lowercased prompt as `normalized`, exactly as the
// native dispatcher does (rust/src/meta_method_dispatch.rs).
//
// `document_generation_plan` reads its cue and format tables from
// data/seed/handler-rules.lino and renders the seeded plan responses, as its
// Rust counterpart does; `algorithm`, `execution_failure`, `source_conflict`
// and `shell_refusal` are seed rules (data/seed/handler-rules.lino) run by the
// shared rule interpreter, so their cues, tables and responses stay data.

// ---------------------------------------------------------------------------
// algorithm (data/seed/handler-rules.lino, rules algorithm*)
// ---------------------------------------------------------------------------

/**
 * Browser binding for the `algorithm` precedence row: a sorting or algorithm
 * request gets the seeded reviewable snippet of the language the
 * `algorithm_language` table finds, through the shared rule interpreter over
 * the lowercased prompt, as the native dispatcher runs it.
 * @param {string} prompt raw prompt
 * @returns {object|null} the worker answer, or null
 */
function tryAlgorithm(prompt) {
  // Rust's `web_search` row runs before `algorithm`; here it is an async-phase
  // row, so a search request ("search for ... algorithms") is left to it.
  if (extractWebSearchRequest(prompt, normalizePrompt(prompt))) return null;
  return runHandlerRuleSet("algorithm", prompt, prompt.toLowerCase(), []);
}

// ---------------------------------------------------------------------------
// execution_failure (data/seed/handler-rules.lino, rules execution_failure*)
// ---------------------------------------------------------------------------

/**
 * Browser binding for the `execution_failure` precedence row: an explicit
 * failing-call prompt surfaces the seeded failure trace (an agent opt-in also
 * records the action log). The seed rules run through the shared rule
 * interpreter over the lowercased prompt, as the native dispatcher runs them.
 * @param {string} prompt raw prompt
 * @returns {object|null} the worker answer, or null
 */
function tryExecutionFailure(prompt) {
  return runHandlerRuleSet("execution_failure", prompt, prompt.toLowerCase(), []);
}

// ---------------------------------------------------------------------------
// write_script (rust/src/solver_handlers/mod.rs try_write_script)
// ---------------------------------------------------------------------------

/**
 * The catalogued language whose minimal script the request asks for, or null
 * when the route cannot render everything the prompt names. Mirrors
 * `is_write_script_request` and `names_no_task_beyond_the_minimal_script`.
 * @param {string} prompt raw prompt
 * @param {string} normalized normalized prompt
 * @returns {string|null} the language slug
 */
function writeScriptLanguage(prompt, normalized) {
  if (lexiconMentionsRole("program_genus", normalized) || lexiconMentionsRole("hello_world_reference", normalized)) return null;
  if (!lexiconMentionsRole("script_authoring_verb", normalized) || !lexiconMentionsRole("script_or_code_artifact", normalized)) return null;
  if (looksLikePythonFunctionSynthesis(prompt, canonicalizedPrompt(normalized))) return null;
  if (metaNamesFileNoun(prompt, detectLanguage(prompt))) return null; // a script over files is derived, not templated
  const program = normalizeProgramPrompt(prompt);
  const language = programLanguageFromPrompt(program);
  if (!language || !writeProgramTemplate("hello_world", language)) return null;
  const task = programTaskFromPrompt(program);
  if (task && task !== "hello_world") return null;
  return tryNumericList(prompt, []) ? null : language;
}

/**
 * Browser binding for the `write_script` precedence row: the hello-world
 * template of the one catalogued language the request names.
 * @param {string} prompt raw prompt
 * @param {string} responseLanguage reply language
 * @returns {object|null} the worker answer, or null
 */
function tryWriteScript(prompt, responseLanguage) {
  const language = writeScriptLanguage(prompt, normalizePrompt(prompt));
  if (!language) return null;
  const template = writeProgramTemplate("hello_world", language);
  const languageInfo = writeProgramLanguageInfo("hello_world", language);
  const taskInfo = WRITE_PROGRAM_TASKS.hello_world;
  const i18n = writeProgramStrings(responseLanguage);
  const output = writeProgramExpectedOutput("hello_world", languageInfo, taskInfo);
  const execution = writeProgramExecutionLines(language, "hello_world", template, output, i18n, responseLanguage);
  return {
    intent: `write_script_${language}`,
    content: [i18n.intro(languageInfo.name, taskInfo.label), "", "```" + languageInfo.fence, template, "```", "", ...execution].join("\n"),
    confidence: 1.0,
    evidence: [`response:write_program:hello_world:${language}`, `program_parameter:language:${language}`],
  };
}

// ---------------------------------------------------------------------------
// document_generation_plan (rust/src/solver_handlers/document_request.rs,
// the document_* tables of data/seed/handler-rules.lino)
// ---------------------------------------------------------------------------

/**
 * The requested document container label: "PDF", "DOCX", …, "" for a generic
 * document, or null when no document artifact is named. The formats, the
 * ebook markers and the generic nouns are the `document_*` tables of
 * data/seed/handler-rules.lino, in priority order (Rust
 * `detect_document_format`).
 * @param {string} lowercased lowercased prompt
 * @returns {string|null} format label
 */
function documentPlanFormat(lowercased) {
  const explicit = handlerRulesTableRow("document_format", lowercased);
  if (explicit !== null) return explicit;
  const ebook = handlerRulesTableRow("document_ebook_marker", lowercased);
  if (ebook !== null && handlerRulesTableRow("document_book_noun", lowercased) !== null) return ebook;
  return handlerRulesTableRow("document_noun", lowercased) === null ? null : "";
}

/**
 * Render the seeded document-generation plan (Rust `render_document_plan`):
 * the `document_generation_plan` response with the
 * `document_generation_plan_format` suffix when a format is named.
 * @param {string} language language slug
 * @param {string} label format label, "" for a generic document
 * @returns {string} the plan
 */
function documentPlanRender(language, label) {
  const suffix = label === "" ? "" : answerFor("document_generation_plan_format", language).split("{label}").join(label);
  return answerFor("document_generation_plan", language).split("{format_suffix}").join(suffix);
}

/**
 * Answer a document-generation request with the universal-algorithm plan.
 * Mirrors `try_document_request` without its meta-language conversion branch:
 * an agent request or a named software artifact (the build task of the
 * software-project row) is left alone, and the request must carry a seeded
 * authoring action and name a document.
 * @param {string} prompt raw prompt
 * @param {string} normalized normalized prompt
 * @returns {object|null} the worker answer, or null
 */
function handleDocumentGenerationPlan(prompt, normalized) {
  const lowercased = normalized.toLowerCase();
  if (isAgentTextRequest(lowercased)) return null;
  if (handlerRulesTableRow("document_software_artifact", lowercased) !== null) return null;
  if (handlerRulesTableRow("document_authoring_action", lowercased) === null) return null;
  const label = documentPlanFormat(lowercased);
  if (label === null) return null;
  const log = codeTaskLog();
  codeTaskLogAppend(log, "document_request:format", label === "" ? "document" : label);
  return codeTaskAnswer(log, "document_generation_plan", "response:document_generation_plan",
    documentPlanRender(detectLanguage(prompt), label), 0.6);
}

/**
 * Browser binding for the `document_generation_plan` precedence row.
 * @param {string} prompt raw prompt
 * @returns {object|null} the worker answer, or null
 */
function tryDocumentGenerationPlan(prompt) {
  return handleDocumentGenerationPlan(prompt, prompt.toLowerCase());
}

// ---------------------------------------------------------------------------
// source_conflict (data/seed/handler-rules.lino, rule source_conflict)
// ---------------------------------------------------------------------------

/**
 * Browser binding for the `source_conflict` precedence row: a source
 * disagreement is recorded instead of resolved silently. The seed rule runs
 * through the shared rule interpreter over the lowercased prompt, the subject
 * the native dispatcher hands every rule.
 * @param {string} prompt raw prompt
 * @returns {object|null} the worker answer, or null
 */
function trySourceConflict(prompt) {
  return runHandlerRuleSet("source_conflict", prompt, prompt.toLowerCase(), []);
}

// ---------------------------------------------------------------------------
// shell_refusal (data/seed/handler-rules.lino, rule shell_refusal)
// ---------------------------------------------------------------------------

/**
 * Browser binding for the `shell_refusal` precedence row: the seed rule runs
 * through the shared rule interpreter (formal_ai_worker_handler_rules.js) over
 * the lowercased prompt, which keeps the backticks its `run \`` cue needs —
 * the same subject the native dispatcher hands the rule.
 * @param {string} prompt raw prompt
 * @returns {object|null} the worker answer, or null
 */
function tryShellRefusal(prompt) {
  return runHandlerRuleSet("shell_refusal", prompt, prompt.toLowerCase(), []);
}
