// Apply the native agent-mode gate before the browser's actual JavaScript executor.
// Requested source stays visible on refusal; execution events describe observed results.
function tryJavaScriptToolRequest(prompt, preferences) {
  const program = extractJavaScriptProgram(prompt);
  if (program === null) return null;
  if (preferences && preferences.agentMode) return tryJavaScriptExecution(prompt);
  const capability = "tool:javascript_execution";
  const denial = nlToolText("nl_tool_agent_mode_required", detectLanguage(prompt), { capability });
  return nlToolRefusal(nlToolText("nl_tool_requested_source", "en", { denial, program }), [
    "policy:agent_mode_required_for_tools:" + capability,
    "execution_status:javascript:refused",
    "execution_environment:agent-permission-gate",
  ]);
}
