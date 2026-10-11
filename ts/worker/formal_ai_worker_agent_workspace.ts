// Bounded browser agent actions use a fresh in-memory filesystem.
// Grammar mirrors rust/src/agent.rs; unavailable processes fail visibly.
function browserWorkspacePlan(prompt) {
  const actions = [];
  const patterns = [
    ["create_file", /create file\s+(.+?)\s+with\s+\x60([^\x60]*)\x60/giu],
    ["modify_file", /modify\s+(?:file\s+)?(.+?)\s+to\s+\x60([^\x60]*)\x60/giu],
    ["delete_file", /delete\s+(?:file\s+)?([^,;]+?)(?=,|;|\s+and\s+|\s+then\s+|$)/giu],
    ["run_command", /run\s+(?:terminal command\s+|command\s+)?\x60([^\x60]+)\x60/giu],
  ];
  for (const [kind, pattern] of patterns) {
    for (const match of String(prompt || "").matchAll(pattern)) {
      actions.push({ index: match.index, kind, target: match[1].trim(), content: match[2] ?? "" });
    }
  }
  return actions.sort((left, right) => left.index - right.index);
}
function browserWorkspacePath(value) {
  const path = String(value).replaceAll("\\", "/");
  if (!path || path.startsWith("/") || /^[A-Za-z]:/u.test(path) || path.split("/").includes("..")) return null;
  return path.split("/").filter((part) => part && part !== ".").join("/") || null;
}
function runBrowserWorkspaceCommand(command, files) {
  const [name, ...operands] = String(command).trim().split(/\s+/u);
  if (name === "cat" && operands.length > 0) {
    let output = "";
    for (const operand of operands) {
      const path = browserWorkspacePath(operand);
      if (!path || !files.has(path)) return { code: 1, output: "", error: "file unavailable in isolated workspace: " + operand };
      output += files.get(path);
    }
    return { code: 0, output, error: "" };
  }
  if (name === "ls" && operands.length === 0) return { code: 0, output: Array.from(files.keys()).sort().join("\n"), error: "" };
  if (name === "pwd" && operands.length === 0) return { code: 0, output: "/sandbox", error: "" };
  return { code: 127, output: "", error: "command unavailable in browser workspace: " + name };
}
function tryBrowserAgentWorkspace(prompt) {
  if (!isAgentModeRequest(prompt)) return null;
  const plan = browserWorkspacePlan(prompt);
  if (plan.length === 0) return null;
  const files = new Map(), commands = [], summaries = [], events = [
    { kind: "agent_mode:opted_in", payload: prompt },
    { kind: "agent_mode:active", payload: prompt },
    { kind: "execution_environment", payload: "isolated in-memory workspace; no host filesystem or process environment" },
  ];
  let failed = false;
  for (const action of plan) {
    let error = "", label = "";
    if (action.kind === "run_command") {
      const result = runBrowserWorkspaceCommand(action.target, files);
      commands.push({ command: action.target, ...result });
      error = result.error;
      label = "ran command " + action.target;
    } else {
      const path = browserWorkspacePath(action.target);
      if (!path) error = "path escapes the isolated workspace";
      else if (action.kind === "create_file") {
        if (files.has(path)) error = "file already exists";
        else { files.set(path, action.content); label = "created " + action.target; }
      } else if (action.kind === "modify_file") {
        if (!files.has(path)) error = "file does not exist";
        else { files.set(path, action.content); label = "modified " + action.target; }
      } else {
        if (!files.delete(path)) error = "file does not exist";
        else label = "deleted " + action.target;
      }
    }
    const status = error ? "failed" : "completed";
    failed ||= Boolean(error);
    events.push({ kind: "action_log:" + action.kind, payload: action.target + ":" + status });
    summaries.push(error ? "failed to " + action.kind.replaceAll("_", " ") + " " + action.target + ": " + error : label);
  }
  const status = failed ? "failed" : "completed", quote = String.fromCharCode(96);
  events.push({ kind: "execution_status", payload: "agent:" + status });
  if (failed) events.push({ kind: "trace:execution_failure", payload: "agent workspace action failed" });
  let content = "Execution status: " + status + " in isolated sandbox.\nWorkspace isolation: fresh in-memory filesystem (no host files or process environment).\n\nAction log:";
  content += summaries.map((summary) => "\n- " + summary).join("");
  for (const result of commands) {
    content += "\n\nCommand: " + quote + result.command + quote + "\nExit: " + result.code + "\nOutput:\n" + quote.repeat(3) + "text\n" + (result.output || "(no output)") + "\n" + quote.repeat(3);
    if (result.error) content += "\nStderr:\n" + quote.repeat(3) + "text\n" + result.error + "\n" + quote.repeat(3);
  }
  return { intent: failed ? "agent_workspace_task_failed" : "agent_workspace_task", content, confidence: failed ? 0.4 : 0.9, evidence: [], solverEvents: events };
}
