// Browser twins of native precedence rows the worker used to leave to the
// Rust engine (PR #1188, JavaScript-first parity, issue #1175 routing probes):
// agentic_continuation, how_it_works, meta_explanation, shell_command_transform,
// source_refresh, and the response-language demonstration. Every recogniser
// reads seed roles or seed cue records; every answer renders a seeded response
// template, so the code keeps only the reasoning.
//
// The native dispatcher hands these handlers `prompt.to_lowercase()` as their
// normalized text (rust/src/meta_method_dispatch.rs), not the punctuation-
// collapsed normalizePrompt form, so each twin re-derives that lowercase form
// from the raw prompt before matching.

const ROLE_ANSWER_RATIONALE_LEAD = "answer_rationale_lead";
const ROLE_CAUSAL_INTERROGATIVE = "causal_interrogative";
const ROLE_PRIOR_ANSWER_REFERENCE = "prior_answer_reference";
const ROLE_ASSISTANT_MECHANISM_INQUIRY = "assistant_mechanism_inquiry";
const ROLE_OPERATING_PRINCIPLE = "operating_principle";

/**
 * Record an inline handler hit on the turn's trace and finalize its answer.
 * @returns {object}
 */
function finalizeInlineHandler(events, steps, toolCalls, hit, name, formalizationContext) {
  events.push(`handler:${hit.intent}`);
  steps.push({ step: "dispatch_handler", detail: name });
  return finalize(events, steps, toolCalls, hit, formalizationContext);
}

/**
 * The native handlers' normalized text: the raw prompt lowercased.
 * @param {string} prompt
 * @returns {string}
 */
function nativeLaneLowercase(prompt) {
  return String(prompt || "").toLowerCase();
}

/**
 * Fill `{name}` slots of a seeded response template.
 * @param {string} template
 * @param {object} values
 * @returns {string}
 */
function nativeLaneRender(template, values) {
  let out = String(template || "");
  for (const [name, value] of Object.entries(values)) {
    out = out.split(`{${name}}`).join(String(value));
  }
  return out;
}

/**
 * `agentic_continuation` precedence row: a turn that is only a continuation
 * cue, with nothing in progress to resume, is answered from the seeded rule
 * (data/seed/handler-rules.lino) instead of being searched for (issue #1095).
 * @param {string} prompt
 * @param {string} normalized
 * @returns {object|null}
 */
function tryAgenticContinuation(prompt, normalized, history = []) {
  return runHandlerRuleSet("agentic_continuation", prompt, normalized, history);
}

/**
 * True when the prompt asks the assistant to justify its previous answer.
 * English and Russian rationale leads are matched directly (a prefix form
 * against the start, a bare form anywhere); the head-final Hindi and Chinese
 * questions need a same-language causal interrogative and prior-answer
 * reference. Mirrors is_why_question in rust/src/solver_handlers/meta_explanation.rs.
 * @param {string} lower
 * @returns {boolean}
 */
function metaExplanationLanguages(key) {
  return String(handlerRulesPolicy("meta_explanation", key) || "").split(/\s+/u).filter(Boolean);
}

function isWhyQuestion(lower) {
  const leadLanguages = metaExplanationLanguages("rationale-lead-languages");
  for (const meaning of meaningsWithRole(ROLE_ANSWER_RATIONALE_LEAD)) {
    for (const lexeme of meaning.lexemes) {
      if (!leadLanguages.includes(lexeme.language)) continue;
      for (const text of lexeme.words) {
        const form = makeWordForm(text, "", "");
        const matched = form.slot === "prefix"
          ? lower.startsWith(form.before) && whyQuestionAddressesAssistant(lower.slice(form.before.length))
          : lower.includes(form.text);
        if (matched) return true;
      }
    }
  }
  return metaExplanationLanguages("compositional-why-languages").some((language) => {
    const namesCause = wordsForRoleInLanguages(ROLE_CAUSAL_INTERROGATIVE, [language]).some((word) => lower.includes(word));
    const namesPrior = wordsForRoleInLanguages(ROLE_PRIOR_ANSWER_REFERENCE, [language]).some((word) => lower.includes(word));
    return namesCause && namesPrior;
  });
}

/**
 * True when the rest of a fronted why-question addresses the assistant: it
 * carries an assistant_self_reference surface ("you", "ты", …) as a whole
 * word, so "Why does this fail: …" about the user's code is not claimed.
 * Mirrors addresses_assistant in rust/src/solver_handlers/meta_explanation.rs.
 * @param {string} rest
 * @returns {boolean}
 */
function whyQuestionAddressesAssistant(rest) {
  const tokens = rest.split(/[^\p{Alphabetic}\p{N}]+/u).filter(Boolean).join(" ");
  return lexiconMentionsRole(ROLE_ASSISTANT_SELF_REFERENCE, tokens);
}

/**
 * True when the prompt asks the assistant how it works: a seeded mechanism
 * inquiry clause, or the compositional Russian operating-principle phrasing.
 * Mirrors is_how_you_work in rust/src/solver_handlers/meta_explanation.rs.
 * @param {string} lower
 * @returns {boolean}
 */
function isHowYouWork(lower) {
  if (lexiconMentionsRoleSubstring(ROLE_ASSISTANT_MECHANISM_INQUIRY, lower)) return true;
  const languages = metaExplanationLanguages("operating-principle-languages");
  const namesPrinciple = wordsForRoleInLanguages(ROLE_OPERATING_PRINCIPLE, languages).some((word) => lower.includes(word));
  const addressesAssistant = wordsForRoleInLanguages(ROLE_ASSISTANT_SELF_REFERENCE, languages).some((word) => lower.includes(word));
  return namesPrinciple && addressesAssistant;
}

/**
 * `meta_explanation` precedence row: why-questions, how-you-work questions and
 * architecture questions about the assistant itself. Mirrors
 * try_meta_explanation_with_runtime (the single-turn branches).
 * @param {string} prompt
 * @returns {object|null}
 */
function tryMetaExplanation(prompt) {
  const lower = nativeLaneLowercase(prompt);
  const why = isWhyQuestion(lower);
  const howYouWork = isHowYouWork(lower);
  const architecture = isArchitectureQuestion(lower);
  if (!why && !howYouWork && !architecture) return null;
  const language = selfAwarenessLanguage(prompt, lower);
  let content = answerFor("meta_explanation", language);
  if (why) content = answerFor("meta_explanation_why", language);
  else if (architecture) content = architectureExplanationContent(language);
  return {
    intent: "meta_explanation",
    content,
    confidence: 1.0,
    evidence: ["response:meta_explanation", `language:${language}`],
  };
}

/**
 * `how_it_works` precedence row: a "how does X work?" question whose subject
 * the local concept lookup (an earlier row) did not answer gets the
 * source-backed mechanism discovery plan — offline, the plan is the answer,
 * exactly as the native handler renders it. A bare "how does it work?" reads
 * its topic from the previous assistant reply, and with no reply to read
 * explains how to ask. Mirrors try_how_it_works in rust/src/solver_handler_how.rs.
 * @param {string} prompt
 * @param {object[]} history
 * @returns {object|null}
 */
function tryHowItWorks(prompt, history) {
  if (isBareHowItWorks(cleanMechanismFragment(prompt).toLowerCase())) return howItWorksFromHistory(history);
  const subject = extractHowItWorksSubject(prompt, nativeLaneLowercase(prompt));
  if (!subject) return null;
  const language = detectLanguage(prompt);
  const providers = WEB_SEARCH_PROVIDERS.map((provider) => provider.id).join(", ");
  return {
    intent: "how_it_works",
    content: nativeLaneRender(answerFor("how_it_works", language), { subject, providers }),
    confidence: 0.68,
    evidence: [
      `followup:subject:inline:${subject.toLowerCase()}`,
      `mechanism_query:request:${subject}`,
      "mechanism_query:source_gate:source_backed_mechanism_only",
      "response:how_it_works",
    ],
  };
}

/**
 * True when `lower` is a bare mechanism_inquiry phrase (a Bare word form, no
 * subject slot), alone or followed by a space. Mirrors is_bare_how_it_works.
 * @param {string} lower
 * @returns {boolean}
 */
function isBareHowItWorks(lower) {
  return lower !== "" && roleWordForms(ROLE_MECHANISM_INQUIRY)
    .some((form) => form.slot === "bare" && (lower === form.text || lower.startsWith(`${form.text} `)));
}

/**
 * The bare question's answer: the prior reply's topic through the concept
 * lookup, the seeded "no record yet" answer naming that topic, or — with no
 * prior reply — the seeded explanation of how to ask.
 * @param {object[]} history
 * @returns {object}
 */
function howItWorksFromHistory(history) {
  const prior = lastHistoryTurn(history, "assistant");
  const term = prior ? howItWorksPriorTopic(prior) : null;
  if (term) {
    const concept = tryConceptLookup(howPolicy("how_it_works", "concept-query", { term }));
    if (concept) {
      concept.evidence = [`followup:subject:prior_reply:${term}`, ...concept.evidence];
      return concept;
    }
    return {
      intent: "concept_elaboration_missing",
      content: nativeLaneRender(answerFor("how_it_works_prior_topic", "en"), { term }),
      confidence: 0.3,
      evidence: ["followup:prior_turn:assistant", `followup:subject:prior_reply_no_record:${term}`, "response:concept_elaboration_missing"],
    };
  }
  return {
    intent: "meta_explanation",
    content: answerFor("how_it_works_no_context", "en"),
    confidence: 0.5,
    evidence: ["how_it_works:refusal:no subject and no prior reply", "response:meta_explanation"],
  };
}

/**
 * The topic of a prior assistant reply: the term before "(" on its first line
 * ("Term (category): …"), else its first capitalised token of two or more
 * bytes that is not a topic_scan_stop_word. Mirrors extract_topic_from_prior_reply.
 * @param {string} reply
 * @returns {string|null}
 */
function howItWorksPriorTopic(reply) {
  const firstLine = String(reply).split(/\r?\n/u)[0].trim();
  const paren = firstLine.indexOf("(");
  if (paren >= 0 && firstLine.slice(0, paren).trim()) return firstLine.slice(0, paren).trim().toLowerCase();
  const stopWords = roleWordForms("topic_scan_stop_word").map((form) => form.text);
  for (const word of String(reply).split(/\s+/u).filter(Boolean)) {
    const clean = word.replace(/^[^\p{Alphabetic}\p{N}]+|[^\p{Alphabetic}\p{N}]+$/gu, "");
    if (new TextEncoder().encode(clean).length < 2 || !/^\p{Uppercase}/u.test(clean)) continue;
    if (!stopWords.includes(clean.toLowerCase())) return clean.toLowerCase();
  }
  return null;
}

/**
 * `source_refresh` precedence row: a refresh request about a cached page or
 * cache queues that source for refresh. The cue words, the refusal lane (no
 * source named, read by the same `source_reference` claim evidence the
 * capability table admits the row on) and the wording are the
 * `source_refresh` rules of data/seed/handler-rules.lino, walked by the shared
 * rule interpreter as the native dispatcher walks them.
 * @param {string} prompt
 * @returns {object|null}
 */
function trySourceRefresh(prompt) {
  return runHandlerRuleSet("source_refresh", prompt, nativeLaneLowercase(prompt), []);
}

/**
 * Every shell text (`code`) of `word` in the `shell_syntax` map of
 * data/seed/code-task-cues.lino (loop and session templates, joiners, prompt
 * markers), in seed order. Mirrors `syntax` in
 * rust/src/solver_handlers/shell_command_transform.rs.
 * @param {string} word
 * @returns {string[]}
 */
function shellSyntax(word) {
  return codeTaskWordEntries("shell_syntax")
    .filter((entry) => codeTaskChildValue(entry, "word") === word)
    .map((entry) => codeTaskChildValue(entry, "code"));
}

/** Whether `line` opens with the session program: the first token of the seeded screen template. */
function shellTransformOpensSession(line) {
  const program = (shellSyntax("screen_template")[0] || "").split(/\s+/u).filter(Boolean)[0];
  return Boolean(program) && String(line).split(/\s+/u).filter(Boolean)[0] === program;
}

/**
 * Whether a candidate line reads as a shell command rather than prose.
 * Mirrors looks_like_shell_command in rust/src/solver_handlers/shell_command_transform.rs;
 * the command heads and prose leads are seed cue records.
 * @param {string} candidate
 * @returns {boolean}
 */
function shellTransformLooksLikeCommand(candidate) {
  const text = String(candidate || "").trim();
  if (!text || text.includes("\n") || text.endsWith("?")) return false;
  if (codeTaskCuePhrases("shell_command_transform", "prose_lead").some((lead) => text.startsWith(lead))) return false;
  if (shellSyntax("joiner").some((joiner) => text.includes(joiner))) return true;
  const first = text.split(/\s+/u)[0] || "";
  return codeTaskCuePhrases("shell_command_transform", "command_head").includes(first);
}

function shellTransformStripFence(line) {
  return String(line || "").replace(/^`+|`+$/gu, "").trim();
}

function shellTransformBackticked(text) {
  const start = text.indexOf("`");
  if (start < 0) return null;
  const end = text.indexOf("`", start + 1);
  return end < 0 ? null : text.slice(start + 1, end);
}

function shellTransformAfterPrompt(line) {
  for (const marker of shellSyntax("prompt_marker")) {
    const index = line.lastIndexOf(marker);
    if (index < 0) continue;
    const command = line.slice(index + marker.length).trim();
    if (shellTransformLooksLikeCommand(command)) return command;
  }
  return null;
}

function shellTransformIsLoop(candidate) {
  const [open, close] = String(shellSyntax("loop_template")[0] || "").split("{command}");
  return Boolean(open) && close !== undefined && candidate.startsWith(open) && candidate.endsWith(close);
}

function shellTransformLoopCommand(text) {
  const trimmed = shellTransformStripFence(String(text || "").trim());
  if (shellTransformIsLoop(trimmed)) return trimmed;
  return String(text || "").split("\n").map((line) => shellTransformStripFence(line.trim())).find(shellTransformIsLoop) || null;
}

function shellTransformCommand(prompt) {
  for (const line of prompt.split("\n")) {
    const command = shellTransformAfterPrompt(shellTransformStripFence(line.trim()));
    if (command) return command;
  }
  const backticked = shellTransformBackticked(prompt);
  if (backticked !== null) {
    const command = shellTransformStripFence(backticked.trim());
    if (shellTransformLooksLikeCommand(command) && !shellTransformOpensSession(command)) return command;
  }
  return prompt.split("\n").map((line) => shellTransformCommandSpan(shellTransformStripFence(line.trim())))
    .find((line) => shellTransformLooksLikeCommand(line) && !shellTransformOpensSession(line)) || null;
}

/**
 * The command span of a line: a plain-word prose lead ending at a colon
 * ("Make this a single line loop: sleep 5m && cleanup -f") is the request,
 * not part of the command, so the command is the span after the colon. A
 * colon inside a real command (URL, quoted string, host:path) never splits it
 * because its lead is not plain words. Mirrors command_span in
 * rust/src/solver_handlers/shell_command_transform.rs.
 * @param {string} line
 * @returns {string}
 */
function shellTransformCommandSpan(line) {
  const index = line.indexOf(": ");
  if (index < 0) return line;
  const lead = line.slice(0, index);
  const rest = shellTransformStripFence(line.slice(index + 2).trim());
  const proseLead = lead.trim() !== "" && /^[\p{Alphabetic}\p{N}\s'-]+$/u.test(lead);
  return proseLead && shellTransformLooksLikeCommand(rest) ? rest : line;
}

function shellTransformWrapLoop(command) {
  if (shellTransformLoopCommand(command)) return command.trim();
  return String(shellSyntax("loop_template")[0] || "").split("{command}").join(command.trim());
}

function shellTransformScreenSession(prompt) {
  const backticked = shellTransformBackticked(prompt);
  const command = backticked !== null && shellTransformOpensSession(backticked)
    ? backticked
    : prompt.split("\n").map((line) => line.trim()).find(shellTransformOpensSession);
  if (!command) return null;
  let session = null;
  for (const token of command.split(/\s+/u).filter(Boolean).slice(1)) {
    if (!token.startsWith("-")) session = token;
  }
  return session || null;
}

function shellTransformQuote(command) {
  return `'${command.split("'").join("'\\''")}'`;
}

function shellTransformScreen(prompt, lower, history) {
  const template = String(shellSyntax("screen_template")[0] || "");
  const program = template.split(/\s+/u).filter(Boolean)[0];
  if (!program || !lower.includes(program)) return null;
  if (!codeTaskCued("shell_command_transform", "screen_execution", prompt, lower)) return null;
  const session = shellTransformScreenSession(prompt);
  if (!session) return null;
  let loop = shellTransformLoopCommand(prompt);
  if (!loop) {
    const turns = Array.isArray(history) ? history.slice().reverse() : [];
    for (const turn of turns) {
      if (String((turn || {}).role || "") !== "assistant") continue;
      loop = shellTransformLoopCommand(String(turn.content || ""));
      if (loop) break;
    }
  }
  if (!loop) {
    const command = shellTransformCommand(prompt);
    if (command) loop = shellTransformWrapLoop(command);
  }
  if (!loop) return null;
  const command = template.split("{session}").join(session).split("{command}").join(shellTransformQuote(loop));
  return { operation: "screen_session", input: loop, command };
}

/**
 * `shell_command_transform` precedence row (issue #552): rewrite a shell
 * command as an infinite loop or a detached screen session, producing the
 * command text without executing it. Mirrors try_shell_command_transform_with_history.
 * @param {string} prompt
 * @param {object[]} history
 * @returns {object|null}
 */
function tryShellCommandTransform(prompt, history) {
  const source = String(prompt || "");
  const lower = nativeLaneLowercase(source);
  let rewrite = shellTransformScreen(source, lower, history);
  if (!rewrite && codeTaskCued("shell_command_transform", "infinite_loop", source, lower)) {
    const command = shellTransformCommand(source);
    if (command) rewrite = { operation: "infinite_loop", input: command, command: shellTransformWrapLoop(command) };
  }
  if (!rewrite) return null;
  return {
    intent: "shell_command_transform",
    content: rewrite.command,
    confidence: 0.92,
    evidence: [
      `shell_transform:${rewrite.operation}`,
      `shell_command:input:${rewrite.input}`,
      `shell_command:output:${rewrite.command}`,
      `shell_transform:operation:${rewrite.operation}`,
      "response:shell_command_transform",
    ],
  };
}

/**
 * The ledger name of a language code (data/seed/languages.lino), or the code.
 * @param {string} code
 * @returns {string}
 */
function nativeLaneLanguageName(code) {
  const ledger = seedRawText(SEED_RAW, "languages.lino");
  const match = new RegExp(`^  language ${code}\\r?\\n    name (.+)$`, "m").exec(ledger);
  return match ? match[1].trim() : code;
}

/**
 * The response language the conversation already established, read from the
 * user's own turns (issue #724): the latest user turn naming a language
 * decides, and one that forbids it establishes none. Mirrors
 * `established_response_language` in rust/src/meta_method_answers.rs.
 * @param {Array<{role: string, content: string}>} history
 * @returns {string|null}
 */
function establishedResponseLanguage(history) {
  const turns = Array.isArray(history) ? history : [];
  for (let index = turns.length - 1; index >= 0; index -= 1) {
    const turn = turns[index];
    if (!turn || turn.role !== "user") continue;
    const content = String(turn.content || "");
    const language = detectResponseLanguage(content.toLowerCase());
    if (language) return responseLanguageForbidden(content) ? null : language;
  }
  return null;
}

/**
 * The response-language demonstration: a terse "answer in <language>" with no
 * earlier turn to re-render is answered in that language. The canonical name
 * comes from the language ledger, the surface is the marker word the prompt
 * used, the demonstration is the seeded greeting of that language. Mirrors
 * response_language_demonstration in rust/src/meta_method_answers.rs.
 * @param {string} prompt
 * @returns {object|null}
 */
function tryResponseLanguageDemonstration(prompt) {
  // "Never answer in Russian" names Russian only to forbid it.
  if (responseLanguageForbidden(prompt)) return null;
  const lower = nativeLaneLowercase(prompt);
  const target = detectResponseLanguage(lower);
  if (!target || !isLanguageReanswerFollowup(lower)) return null;
  const canonical = nativeLaneLanguageName(target);
  const marker = meaningsWithRole(ROLE_RESPONSE_LANGUAGE_MARKER)
    .find((meaning) => meaningDefinedLanguageCode(meaning) === target);
  const surface = (marker && marker.words.find((word) => lower.includes(word))) || canonical;
  const demonstration = answerFor("greeting", target) || canonical;
  return {
    intent: "response_language_demonstration",
    content: nativeLaneRender(answerFor("response_language_demonstration", "en"), { canonical, surface, demonstration }),
    confidence: 1.0,
    evidence: [`language_to:${target}`, "response:response_language_demonstration"],
  };
}

/**
 * The learnable-source registry of data/seed/learning-sources.lino (issue
 * #499): each declared source (host, keywords, capability) and the shared
 * directive cues. Mirrors `learning_sources` in rust/src/seed.rs.
 * @returns {{sources: Array<object>, directiveCues: string[]}}
 */
function learningSourceRegistry() {
  const registry = { sources: [], directiveCues: [] };
  const text = seedRawText(SEED_RAW, "learning-sources.lino");
  const root = text ? parseLinoTree(text).children[0] : null;
  for (const child of root ? root.children : []) {
    if (child.name === "source") {
      registry.sources.push({
        id: child.value,
        capability: childValue(child, "capability"),
        host: childValue(child, "host"),
        keywords: child.children.filter((entry) => entry.name === "keyword").map((entry) => entry.value),
      });
    } else if (child.name === "directive") {
      for (const entry of child.children.filter((cue) => cue.name === "cue")) registry.directiveCues.push(entry.value);
    }
  }
  return registry;
}

/**
 * The declared source a learning directive points at: the prompt carries a
 * directive cue and the source's host or one of its keywords. Mirrors
 * `LearningSources::match_directive` in rust/src/seed.rs.
 * @param {string} lowercased
 * @returns {object|null}
 */
function matchLearningDirective(lowercased) {
  const registry = learningSourceRegistry();
  if (!registry.directiveCues.some((cue) => cue && lowercased.includes(cue))) return null;
  return registry.sources.find((source) => (source.host && lowercased.includes(source.host))
    || source.keywords.some((keyword) => keyword && lowercased.includes(keyword))) || null;
}

/**
 * `learn_from_source` precedence row (issue 1173 R3, browser twin of
 * `try_learn_from_source` in rust/src/retrieval_procedures.rs): a directive
 * that points the engine at a declared learnable source is acknowledged
 * with the seeded intro. The learning loop that ingests the source runs in
 * the native engine, so the browser states that from a seeded template
 * instead of a report it cannot compute.
 * @param {string} prompt
 * @returns {object|null}
 */
function tryLearnFromSource(prompt) {
  const source = matchLearningDirective(nativeLaneLowercase(prompt));
  if (source === null) return null;
  const language = detectLanguage(prompt);
  const intro = textTransformLocalizedResponse("learn_from_source", language) || "";
  const summary = nativeLaneRender(textTransformLocalizedResponse("learn_from_source_browser_summary", language) || "", {
    source: source.id.split("_").join(" "),
    capability: source.capability,
  });
  return {
    intent: "learn_from_source",
    content: intro ? `${intro}\n\n${summary}` : summary,
    confidence: 1,
    evidence: ["handler:learn_from_source", `learning_source:${source.id}`, `learning_capability:${source.capability}`, "response:learn_from_source"],
  };
}
/**
 * The `diagnostic` prelude row (rust/src/meta_method_dispatch.rs `try_diagnostic`): a prompt carrying a
 * `diagnostic_marker` row of data/seed/handler-rules.lino is solved without the marker and answered with
 * its evidence and trace under that marker. The browser keeps no links-notation projection of an answer,
 * so the native block's links lines are absent here; the evidence and trace lines are the native ones.
 * @param {string} prompt
 * @param {function(string): Promise<object>} rerun
 * @returns {Promise<object|null>}
 */
async function tryDiagnosticMarker(prompt, rerun) {
  const marker = handlerRulesTableKeys("diagnostic_marker").find((key) => key && prompt.includes(key));
  if (!marker) return null;
  const inner = await rerun(prompt.split(marker).join("").trim());
  const evidence = Array.isArray(inner.evidence) ? inner.evidence : [];
  const lines = [`${inner.content}\n\n${marker}`, ...evidence.map((link) => `evidence: ${link}`), `trace: ${inner.intent}`];
  return Object.assign({}, inner, { content: `${lines.join("\n")}\n`, evidence: ["diagnostic_mode:active", ...evidence] });
}
