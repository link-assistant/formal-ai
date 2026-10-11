// Native nl_tool prelude: seeded calculator, search, shell and JavaScript permission gates.
// The browser runs its actual available executors and reports unavailable tools honestly.

const NL_TOOL_INVOCATION_CUE = "tool_invocation_cue";
const NL_TOOL_ARGUMENT_MARKER = "tool_argument_marker";
const NL_TOOL_LOCAL_SHELL_CUE = "local_shell_request_cue";
const NL_TOOL_NAMED = [
  ["calculator", "calculator_tool_name"],
  ["web_search", "web_search_tool_name"],
];
const NL_TOOL_LOCAL_SHELL = "local_shell";
// The punctuation `clean_argument` sheds at an argument's edges.
const NL_TOOL_EDGE_MARKS = new Set(["`", "\"", "'", ".", ",", ":", ";", "!", "?", "(", ")", "[", "]"]);

/** Mirrors `fn nl_tool_text`: a seeded `nl_tool_*` response with each slot filled once. */
function nlToolText(intent, language, values) {
  return handlerRulesFillOnce(answerFor(intent, language), values);
}

/** Mirrors `solver_helpers::extract_quoted_phrase` (its quote order). */
function nlToolQuotedPhrase(text) {
  for (const [open, close] of [["'", "'"], ["\"", "\""], ["`", "`"], ["«", "»"]]) {
    const start = text.indexOf(open);
    if (start < 0) continue;
    const end = text.indexOf(close, start + open.length);
    if (end >= 0) return text.slice(start + open.length, end);
  }
  return null;
}

/** Mirrors `fn after_argument_marker`: the text after the first English argument marker. */
function nlToolAfterMarker(lowered) {
  for (const marker of wordsForRoleInLanguages(NL_TOOL_ARGUMENT_MARKER, ["en"])) {
    const delimiter = ` ${marker} `;
    const index = lowered.indexOf(delimiter);
    if (index >= 0) return lowered.slice(index + delimiter.length);
  }
  return null;
}

/** Mirrors `fn extract_argument` and `fn clean_argument`. */
function nlToolArgument(prompt, lowered) {
  const parts = prompt.split("`");
  const raw = parts.length >= 3 ? parts[1] : nlToolQuotedPhrase(prompt) ?? nlToolAfterMarker(lowered);
  if (raw === null || raw === undefined) return null;
  const characters = Array.from(String(raw).trim());
  while (characters.length && NL_TOOL_EDGE_MARKS.has(characters[0])) characters.shift();
  while (characters.length && NL_TOOL_EDGE_MARKS.has(characters[characters.length - 1])) characters.pop();
  const cleaned = characters.join("").split(/\s+/u).filter(Boolean).join(" ");
  return cleaned || null;
}

function nlToolRefusal(content, evidence) {
  return { intent: "tool_call_refused", content, confidence: 1, evidence: [...evidence, "response:tool_call_refused"] };
}

/** The calculator the browser has, rendered through the native report records. */
function nlToolCalculator(expression) {
  const evidence = ["tool_call:calculator", `tool_parameter:expression=${expression}`];
  try {
    const result = evaluateSynthesisArithmetic(expression).formatted;
    return {
      intent: "natural_language_api_call",
      content: nlToolText("nl_tool_calculator_executed", "en", { expression, result }),
      confidence: 1,
      evidence: [...evidence, "execution_status:calculator:executed", "response:natural_language_api_call"],
    };
  } catch (error) {
    return {
      intent: "natural_language_api_call_failed",
      content: nlToolText("nl_tool_calculator_failed", "en", { expression, error: String(error && error.message || error) }),
      confidence: 0.4,
      evidence: [...evidence, "execution_status:calculator:error", "response:natural_language_api_call_failed"],
    };
  }
}

/** The answer for an allowed tool call: run what the browser can, else say it cannot run here. */
function nlToolAllowed(tool, argument, language) {
  if (tool === "calculator") return nlToolCalculator(argument);
  return nlToolRefusal(nlToolText("nl_tool_browser_unavailable", language, { tool }), [`execution_status:${tool}:unavailable`, "execution_environment:browser"]);
}

/**
 * Mirrors `try_natural_language_tool_request` for the calculator, web-search and
 * local-shell tool calls, or null when the prompt names none.
 */
function tryNaturalLanguageToolRequest(prompt, preferences) {
  const javascript = tryJavaScriptToolRequest(prompt, preferences);
  if (javascript) return javascript;
  const lowered = nativeLaneLowercase(prompt);
  const language = detectLanguage(prompt);
  const agentMode = Boolean(preferences && preferences.agentMode);
  const gate = (tool, argument) => {
    if (agentMode) return nlToolAllowed(tool, argument, language);
    const capability = `tool:${tool}`;
    return nlToolRefusal(nlToolText("nl_tool_agent_mode_required", language, { capability }), [`policy:agent_mode_required_for_tools:${capability}`]);
  };
  for (const [tool, nameRole] of NL_TOOL_NAMED) {
    if (!lexiconMentionsRole(nameRole, lowered) || !lexiconMentionsRole(NL_TOOL_INVOCATION_CUE, lowered)) continue;
    const argument = nlToolArgument(String(prompt || ""), lowered);
    if (argument !== null) return gate(tool, argument);
  }
  return lexiconMentionsRole(NL_TOOL_LOCAL_SHELL_CUE, lowered) ? gate(NL_TOOL_LOCAL_SHELL, "") : null;
}
