// Semantic failure classification shared by chat and Agent-plan rendering.
// Deliberately avoid matching prose: only solver intents and structured tool
// results may trigger the proactive issue-report invitation.

const FAILURE_INTENTS = new Set([
  "unknown",
  "agent_cli_error",
  "tool_result_failed",
]);

const EXPECTED_STOP_STATUSES = new Set([
  "refused",
  "denied",
  "cancelled",
  "canceled",
  "aborted",
  "pending",
  "awaiting_approval",
  "not_granted",
]);

const SUCCESS_STATUSES = new Set([
  "ok",
  "success",
  "succeeded",
  "completed",
  "passed",
]);

function normalizedStatus(value) {
  return typeof value === "string" ? value.trim().toLowerCase() : value;
}

function statusIsFailure(value) {
  const status = normalizedStatus(value);
  if (typeof status === "number") {
    return status < 0 || (status > 0 && status < 100) || status >= 400;
  }
  if (!status || EXPECTED_STOP_STATUSES.has(status)) return false;
  if (SUCCESS_STATUSES.has(status)) return false;
  const numeric = Number(status);
  if (Number.isFinite(numeric)) return statusIsFailure(numeric);
  return true;
}

function structuredResultHasFailure(value) {
  if (typeof value === "string") {
    const trimmed = value.trim();
    if (!trimmed.startsWith("{") && !trimmed.startsWith("[")) return false;
    try {
      return structuredResultHasFailure(JSON.parse(trimmed));
    } catch (_error) {
      return false;
    }
  }
  if (Array.isArray(value)) return value.some(structuredResultHasFailure);
  if (!value || typeof value !== "object") return false;

  const status = value.status ?? value.state ?? value.outcome;
  const normalized = normalizedStatus(status);
  if (EXPECTED_STOP_STATUSES.has(normalized)) return false;

  if (value.ok === false || value.success === false) return true;
  if (value.exit_code !== undefined && Number(value.exit_code) !== 0) return true;
  if (value.exitCode !== undefined && Number(value.exitCode) !== 0) return true;
  if (value.status_code !== undefined && Number(value.status_code) >= 400) return true;
  if (value.statusCode !== undefined && Number(value.statusCode) >= 400) return true;
  if (statusIsFailure(status)) return true;
  return [value.error, value.failure].some(
    (entry) => entry !== undefined && entry !== null && String(entry).trim() !== "",
  );
}

export function answerHasDetectedFailure(answer) {
  if (!answer || typeof answer !== "object") return false;
  if (answer.detectedFailure === true) return true;
  if (FAILURE_INTENTS.has(String(answer.intent || "").toLowerCase())) return true;
  if (structuredResultHasFailure(answer)) return true;

  const calls = [answer.toolCalls, answer.diagnosticsToolCalls]
    .filter(Array.isArray)
    .flat();
  return calls.some((call) => structuredResultHasFailure(call?.outputs));
}
