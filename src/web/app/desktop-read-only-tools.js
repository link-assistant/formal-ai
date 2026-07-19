const NATIVE_READ_ONLY_TOOLS = new Set(["web_search", "web_fetch", "http_fetch", "url_navigate"]);

function nativeRequest(answer) {
  const calls = Array.isArray(answer && answer.toolCalls) ? answer.toolCalls : [];
  const call = calls.find((candidate) => NATIVE_READ_ONLY_TOOLS.has(candidate && candidate.tool));
  if (!call) return null;
  const inputs = call.inputs && typeof call.inputs === "object" ? call.inputs : {};
  if (call.tool === "web_search") {
    const query = String(inputs.query || "").trim();
    return query ? { call, tool: "web_search", input: { ...inputs, query } } : null;
  }
  const url = String(inputs.url || "").trim();
  return url ? { call, tool: "web_fetch", input: { ...inputs, url } } : null;
}

export async function enhanceWithDesktopReadOnlyTool(answer, invoke) {
  if (!answer || typeof invoke !== "function") return answer;
  const request = nativeRequest(answer);
  if (!request) return answer;
  let result;
  try {
    result = await invoke(request.tool, request.input);
  } catch (_error) {
    return answer;
  }
  if (!result || result.ok !== true || result.executed !== true || !result.body) return answer;
  const evidence = Array.isArray(answer.evidence) ? answer.evidence : [];
  const toolCalls = Array.isArray(answer.toolCalls)
    ? answer.toolCalls.map((call) =>
        call === request.call ? { ...call, tool: request.tool, outputs: result } : call,
      )
    : [];
  return {
    ...answer,
    content: String(result.body),
    evidence: [...evidence, `desktop_tool:${request.tool}`, "desktop_tool:permission_free"],
    toolCalls,
  };
}

export { nativeRequest };
