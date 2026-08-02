"use strict";

// Maps agent-commander/OpenCode-style NDJSON events onto the answer contract
// already rendered by the web chat UI.

function parseNdjsonEvents(text) {
  const events = [];
  for (const line of String(text || "").split(/\r?\n/)) {
    const trimmed = line.trim();
    if (!trimmed) {
      continue;
    }
    try {
      events.push(JSON.parse(trimmed));
    } catch (_error) {
      events.push({ type: "text", text: trimmed });
    }
  }
  return events;
}

function compactObject(value = {}) {
  const result = {};
  for (const [key, entry] of Object.entries(value || {})) {
    if (entry === undefined || entry === null || entry === "") {
      continue;
    }
    result[key] = entry;
  }
  return result;
}

function eventType(event = {}) {
  return String(event.type || event.event || event.kind || "")
    .trim()
    .toLowerCase()
    .replace(/[-.]/g, "_");
}

function stringValue(value) {
  if (typeof value === "string") {
    return value;
  }
  if (typeof value === "number" || typeof value === "boolean") {
    return String(value);
  }
  return "";
}

function nestedString(value, keys) {
  if (!value || typeof value !== "object") {
    return "";
  }
  for (const key of keys) {
    const direct = stringValue(value[key]);
    if (direct) {
      return direct;
    }
  }
  return "";
}

function contentText(event = {}) {
  const direct = nestedString(event, ["text", "content", "delta", "body", "message"]);
  if (direct) {
    return direct;
  }
  if (event.message && typeof event.message === "object") {
    const messageText = nestedString(event.message, ["text", "content", "delta"]);
    if (messageText) {
      return messageText;
    }
  }
  if (event.part && typeof event.part === "object") {
    const partText = nestedString(event.part, ["text", "content", "delta"]);
    if (partText) {
      return partText;
    }
  }
  if (Array.isArray(event.content)) {
    return event.content
      .map((part) => stringValue(part) || nestedString(part, ["text", "content"]))
      .join("");
  }
  if (event.data && typeof event.data === "object") {
    return contentText(event.data);
  }
  return "";
}

function parseMaybeJsonObject(value) {
  if (!value) {
    return null;
  }
  if (typeof value === "object" && !Array.isArray(value)) {
    return value;
  }
  if (typeof value !== "string") {
    return null;
  }
  try {
    const parsed = JSON.parse(value);
    return parsed && typeof parsed === "object" && !Array.isArray(parsed) ? parsed : null;
  } catch (_error) {
    return null;
  }
}

function eventPayload(event = {}, keys) {
  for (const key of keys) {
    if (event[key] !== undefined) {
      return event[key];
    }
  }
  if (event.data && typeof event.data === "object") {
    for (const key of keys) {
      if (event.data[key] !== undefined) {
        return event.data[key];
      }
    }
  }
  return undefined;
}

function toolId(event = {}) {
  return nestedString(event, [
    "id",
    "tool_call_id",
    "toolCallId",
    "toolUseId",
    "call_id",
    "request_id",
  ]);
}

function normalizeToolName(name) {
  const tool = String(name || "").trim();
  if (tool === "bash" || tool === "terminal") {
    return "shell";
  }
  return tool || "agent";
}

function toolName(event = {}) {
  const direct = nestedString(event, ["tool", "name", "tool_name", "toolName"]);
  if (direct) {
    return normalizeToolName(direct);
  }
  for (const key of ["tool", "call", "function"]) {
    if (event[key] && typeof event[key] === "object") {
      const nested = nestedString(event[key], ["name", "tool", "toolName"]);
      if (nested) {
        return normalizeToolName(nested);
      }
    }
  }
  if (event.data && typeof event.data === "object") {
    return toolName(event.data);
  }
  return "agent";
}

function toolInputs(event = {}) {
  const raw = eventPayload(event, [
    "input",
    "inputs",
    "parameters",
    "params",
    "args",
    "arguments",
    "tool_input",
    "toolInput",
  ]);
  const parsed = parseMaybeJsonObject(raw);
  const input = parsed
    ? { ...parsed }
    : typeof raw === "string"
      ? { value: raw }
      : {};
  return compactObject({
    ...input,
    command: input.command || input.cmd || event.command || event.cmd,
    pattern: input.pattern || event.pattern,
    title: input.title || event.title,
    scope: input.scope || event.scope,
  });
}

function toolOutputs(event = {}) {
  const raw = eventPayload(event, ["output", "outputs", "result", "results", "response"]);
  const parsed = parseMaybeJsonObject(raw);
  const output = parsed
    ? { ...parsed }
    : typeof raw === "string"
      ? { body: raw }
      : {};
  const text = contentText(event);
  return compactObject({
    ...output,
    body: output.body || output.output || event.body || (text && !output.stdout ? text : ""),
    stdout: output.stdout || event.stdout,
    stderr: output.stderr || event.stderr,
    exitCode: output.exitCode ?? output.exit_code ?? event.exitCode ?? event.exit_code,
    status: output.status || event.status,
    error: output.error || event.error,
    message: output.message || (typeof event.message === "string" ? event.message : ""),
  });
}

function commandFromEvent(event = {}, inputs = {}) {
  return String(
    event.command ||
      event.cmd ||
      inputs.command ||
      inputs.cmd ||
      inputs.pattern ||
      event.pattern ||
      event.title ||
      inputs.title ||
      "",
  ).trim();
}

function errorText(event = {}) {
  const direct = nestedString(event, ["error", "message", "reason", "stderr"]);
  if (direct) {
    return direct;
  }
  if (event.error && typeof event.error === "object") {
    return nestedString(event.error, ["message", "reason", "text"]);
  }
  return contentText(event);
}

function detailForTool(tool, command, suffix) {
  const prefix = command ? `${tool}: ${command}` : tool;
  return suffix ? `${prefix} (${suffix})` : prefix;
}

function pushStep(steps, step, detail, level = "high") {
  const text = String(detail || "").trim();
  if (!text) {
    return;
  }
  steps.push({ step, detail: text, level });
}

function upsertToolCall(toolCalls, event = {}, patch = {}) {
  const id = toolId(event);
  let call = null;
  if (id) {
    call = toolCalls.find((entry) => entry.id === id);
  }
  if (!call) {
    const name = patch.tool || toolName(event);
    call = [...toolCalls]
      .reverse()
      .find(
        (entry) =>
          entry.tool === name && (!entry.outputs || Object.keys(entry.outputs).length === 0),
      );
  }
  if (!call) {
    call = {
      tool: patch.tool || toolName(event),
      inputs: {},
      outputs: {},
    };
    if (id) {
      call.id = id;
    }
    toolCalls.push(call);
  }
  if (id && !call.id) {
    call.id = id;
  }
  if (patch.tool) {
    call.tool = patch.tool;
  }
  if (patch.inputs) {
    call.inputs = compactObject({ ...call.inputs, ...patch.inputs });
  }
  if (patch.outputs) {
    call.outputs = compactObject({ ...call.outputs, ...patch.outputs });
  }
  return call;
}

function outputSummary(outputs = {}) {
  if (outputs.status) {
    return String(outputs.status);
  }
  if (outputs.exitCode !== undefined) {
    return `exit ${outputs.exitCode}`;
  }
  if (outputs.error || outputs.stderr) {
    return "error";
  }
  return "completed";
}

function fallbackContentFromToolCalls(toolCalls) {
  for (const call of [...toolCalls].reverse()) {
    const outputs = call.outputs || {};
    const text =
      outputs.stdout || outputs.body || outputs.output || outputs.message || outputs.stderr;
    if (text) {
      return String(text).trim();
    }
  }
  return "";
}

function agentEventsToChatAnswer(events = [], options = {}) {
  const source = String(options.source || "agent_provider");
  const steps = [];
  const toolCalls = [];
  const textParts = [];
  const errors = [];
  const normalizedEvents = Array.isArray(events) ? events : [];

  for (const event of normalizedEvents) {
    const entry =
      event && typeof event === "object" ? event : { type: "text", text: String(event || "") };
    const type = eventType(entry);
    if (
      type === "text" ||
      type === "assistant" ||
      type === "assistant_message" ||
      type === "message" ||
      type === "message_delta" ||
      type === "content_delta" ||
      type === "assistant_delta"
    ) {
      const text = contentText(entry);
      if (text) {
        textParts.push(text);
        pushStep(steps, "agent_text", text.replace(/\s+/g, " ").slice(0, 160));
      }
      continue;
    }

    if (
      type === "tool_use" ||
      type === "tool_start" ||
      type === "tool_call" ||
      type === "function_call" ||
      type === "permission_request"
    ) {
      const inputs = toolInputs(entry);
      const tool = toolName(entry);
      const command = commandFromEvent(entry, inputs);
      upsertToolCall(toolCalls, entry, {
        tool,
        inputs: command ? { ...inputs, command } : inputs,
      });
      pushStep(
        steps,
        type === "permission_request" ? "agent_permission_request" : "agent_tool_start",
        detailForTool(tool, command, type === "permission_request" ? "approval requested" : ""),
      );
      continue;
    }

    if (
      type === "tool_result" ||
      type === "tool_output" ||
      type === "tool_finish" ||
      type === "function_result" ||
      type === "permission_response"
    ) {
      const outputs = toolOutputs(entry);
      const inputs = toolInputs(entry);
      const tool = toolName(entry);
      const command = commandFromEvent(entry, inputs);
      upsertToolCall(toolCalls, entry, {
        tool,
        inputs: command ? { ...inputs, command } : inputs,
        outputs,
      });
      pushStep(
        steps,
        type === "permission_response" ? "agent_permission_response" : "agent_tool_result",
        detailForTool(tool, command, outputSummary(outputs)),
      );
      continue;
    }

    if (type === "error" || type === "agent_error") {
      const text = errorText(entry) || "agent provider error";
      errors.push(text);
      pushStep(steps, "agent_error", text, "high");
      const tool = toolName(entry);
      if (tool !== "agent") {
        upsertToolCall(toolCalls, entry, {
          tool,
          outputs: compactObject({ error: text, status: "error" }),
        });
      }
      continue;
    }

    if (type.endsWith("_start") || type.endsWith("_finish") || type === "step") {
      pushStep(steps, "agent_step", contentText(entry) || type, "medium");
      continue;
    }

    // Lifecycle/config/log events may have a generic `message` field, but that
    // is diagnostics rather than assistant output. Preserve the event in the
    // trace without leaking it into the final chat answer.
    pushStep(steps, "agent_event", type || "event", "low");
  }

  const text = textParts.join("").trim();
  const errorTextBlock = errors.join("\n").trim();
  const content = [
    text || String(options.fallbackContent || "").trim() || fallbackContentFromToolCalls(toolCalls),
    errorTextBlock,
  ]
    .filter(Boolean)
    .join("\n\n");
  return {
    intent: errors.length > 0 ? "agent_cli_error" : "agent_cli_turn",
    source,
    confidence: errors.length > 0 ? "low" : "medium",
    content,
    evidence: [
      `agent_events:${normalizedEvents.length}`,
      "agent_provider:ndjson",
      options.provider ? `provider:${options.provider}` : "",
      options.status ? `status:${options.status}` : "",
    ].filter(Boolean),
    steps,
    toolCalls,
  };
}

function agentProviderResultToChatAnswer(result = {}, request = {}) {
  if (result.answer && typeof result.answer === "object") {
    return result.answer;
  }
  const events = Array.isArray(result.events)
    ? result.events
    : parseNdjsonEvents(result.body || result.stdout || "");
  const fallbackContent = result.ok === false
    ? String(result.stderr || result.reason || "").trim()
    : String(result.finalAnswer || result.answerText || "").trim();
  const answer = agentEventsToChatAnswer(events, {
    provider: result.provider,
    source: "agent_provider",
    status: result.status,
    fallbackContent,
  });
  const command = commandFromRequest(request);
  if (command) {
    answer.evidence = [
      ...(Array.isArray(answer.evidence) ? answer.evidence : []),
      `command:${command}`,
    ];
  }
  if (result.ok === false) {
    answer.intent = "agent_cli_error";
    answer.confidence = "low";
    if (!answer.content) {
      answer.content = String(
        result.reason || result.stderr || result.status || "agent provider failed",
      );
    }
    if (!answer.steps.some((step) => step.step === "agent_error")) {
      pushStep(
        answer.steps,
        "agent_error",
        String(result.reason || result.stderr || result.status || "agent provider failed"),
      );
    }
  }
  return answer;
}

function commandFromRequest(request = {}) {
  return String(
    request.command ||
      (request.input && request.input.command) ||
      (request.tool === "shell" && request.prompt) ||
      "",
  ).trim();
}

module.exports = {
  parseNdjsonEvents,
  agentEventsToChatAnswer,
  agentProviderResultToChatAnswer,
};
