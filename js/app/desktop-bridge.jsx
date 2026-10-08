// Desktop (Electron) bridge: status/updater normalization, service and VS Code
// installer labels, data-migration notice and desktop agent/tool requests.

import React from "react";
import { APP_VERSION } from "./app-constants.jsx";
import { desktopToolRouterGrants, normalizeSliderPreference } from "./preferences.jsx";

const { createElement: h } = React;

export function desktopBridge() {
  if (typeof window === "undefined" || !window.FormalAiDesktop) {
    return null;
  }
  return window.FormalAiDesktop;
}

// Issue #438 (follow-up): only the Electron desktop shell exposes the
// start/stop service handlers. The browser/VS Code surfaces lack them, so the
// Services panel is gated on the bridge actually carrying serviceStatus.
export function desktopServiceBridge() {
  const bridge = desktopBridge();
  if (!bridge || typeof bridge.serviceStatus !== "function") {
    return null;
  }
  return bridge;
}

function normalizeAppVersion(value) {
  const raw = String(value || "").trim();
  if (!raw || raw.startsWith("__") || raw.endsWith("__")) {
    return "";
  }
  return raw.replace(/^v/i, "");
}

export function desktopAppVersionLabel(status) {
  const desktopVersion = normalizeAppVersion(status && status.appVersion);
  const fallback = normalizeAppVersion(APP_VERSION) || APP_VERSION;
  const version = desktopVersion || fallback;
  return /^v/i.test(version) ? version : `v${version}`;
}

function normalizeDesktopUpdaterStatus(updater, currentVersion) {
  if (!updater || typeof updater !== "object") {
    return null;
  }
  const state = String(updater.state || (updater.updateAvailable ? "available" : "idle"));
  const progressPercent = Math.max(
    0,
    Math.min(100, Number(updater.progressPercent || 0) || 0),
  );
  return {
    supported: updater.supported !== false,
    enabled: updater.enabled !== false && updater.supported !== false,
    platform: String(updater.platform || ""),
    currentVersion: normalizeAppVersion(updater.currentVersion) || currentVersion || "",
    state,
    updateAvailable: Boolean(updater.updateAvailable),
    downloaded: Boolean(updater.downloaded),
    latestVersion: normalizeAppVersion(updater.latestVersion),
    progressPercent,
    checkedAt: String(updater.checkedAt || ""),
    error: String(updater.error || ""),
    message: String(updater.message || ""),
  };
}

export function mergeDesktopUpdateStatus(current, payload) {
  if (!payload || typeof payload !== "object") {
    return current;
  }
  if (payload.updater && typeof payload.updater === "object") {
    return normalizeDesktopStatus({ ...(current || {}), ...payload });
  }
  return normalizeDesktopStatus({
    ...(current || {}),
    appVersion:
      normalizeAppVersion(payload.currentVersion)
      || (current && current.appVersion)
      || "",
    updater: payload,
  });
}

export function desktopUpdaterStateLabel(updater, t) {
  const tr = typeof t === "function" ? t : (key) => key;
  if (!updater) {
    return "";
  }
  if (updater.error) {
    return `${tr("updates.state.error")}: ${updater.error}`;
  }
  if (updater.message && updater.state === "disabled") {
    return updater.message;
  }
  const key = {
    idle: "updates.state.idle",
    checking: "updates.state.checking",
    available: "updates.state.available",
    "not-available": "updates.state.notAvailable",
    downloading: "updates.state.downloading",
    downloaded: "updates.state.downloaded",
    installing: "updates.state.installing",
    disabled: "updates.state.disabled",
    error: "updates.state.error",
  }[updater.state] || "updates.state.idle";
  return tr(key, {
    version: updater.latestVersion || updater.currentVersion || "",
    percent: Math.round(updater.progressPercent || 0),
  });
}

export function desktopUpdaterBusy(updater) {
  return updater && ["checking", "downloading", "installing"].includes(updater.state);
}

// Human-readable summary for a single managed service state so the UI label and
// the indicator dot stay in lockstep.
export function serviceStateLabel(state, t) {
  const tr = typeof t === "function" ? t : (key) => key;
  const key = {
    running: "services.state.running",
    stopped: "services.state.stopped",
    absent: "services.state.stopped",
    "missing-config": "services.state.needsToken",
    "docker-unavailable": "services.state.dockerUnavailable",
    ready: "services.state.ready",
    error: "services.state.error",
  }[String(state || "")];
  if (key) {
    return tr(key);
  }
  return String(state || "") || tr("services.state.unknown");
}

// Issue #554 (R2): map the structured VS Code install result the main process
// returns to a localized status line shown under the one-click button.
export function vscodeInstallStateLabel(result, t) {
  const tr = typeof t === "function" ? t : (key) => key;
  if (!result || typeof result !== "object") {
    return "";
  }
  if (result.ok) {
    return tr("vscodeInstall.installed", { cli: String(result.cli || "code") });
  }
  const key = {
    "no-vscode-cli": "vscodeInstall.noCli",
    "no-release-asset": "vscodeInstall.noAsset",
    "release-lookup-failed": "vscodeInstall.lookupFailed",
    "download-failed": "vscodeInstall.downloadFailed",
    "install-failed": "vscodeInstall.installFailed",
    error: "vscodeInstall.error",
  }[String(result.state || "")];
  return key ? tr(key) : tr("vscodeInstall.error");
}

// Issue #672 (F2): the shape the desktop bridge reports for the profile
// migration. Normalising here means the notice never has to guard against a
// missing field, and an older desktop build that answers with a partial object
// degrades to "we know nothing" rather than rendering `undefined` at the user.
export function normalizeDataMigration(result, options = {}) {
  if (!result || typeof result !== "object") return null;
  const copied = Array.isArray(result.copied)
    ? result.copied.filter((item) => typeof item === "string" && item)
    : [];
  return {
    known: Boolean(result.known),
    migrated: Boolean(result.migrated),
    reason: typeof result.reason === "string" ? result.reason : "unknown",
    copied,
    migratedFrom:
      typeof result.migratedFrom === "string" && result.migratedFrom
        ? result.migratedFrom.split(/[\\/]/).filter(Boolean).pop()
        : null,
    error: typeof result.error === "string" && result.error ? result.error : null,
    replayed: Boolean(options.replayed),
  };
}

// The notice is worth the user's attention only when something actually
// happened (data moved, or a transfer failed) — or right after they asked for a
// replay, where "nothing left to copy" IS the answer they wanted.
function shouldShowDataMigrationNotice(migration) {
  if (!migration || !migration.known) return false;
  if (migration.replayed) return true;
  if (migration.reason === "failed") return true;
  return migration.migrated && migration.copied.length > 0;
}

export function DataMigrationNotice({ migration, busy, onReplay, onDismiss, t }) {
  if (!shouldShowDataMigrationNotice(migration)) return null;
  const failed = migration.reason === "failed";
  let body;
  if (failed) {
    body = t("dataMigration.failed", { error: migration.error || "" });
  } else if (migration.copied.length === 0) {
    body = t("dataMigration.nothing");
  } else if (migration.migratedFrom) {
    body = t("dataMigration.body", { source: migration.migratedFrom });
  } else {
    body = t("dataMigration.bodyUnknown");
  }
  return (
    <div
      className={`data-migration-notice${failed ? " is-failed" : ""}`}
      data-testid="data-migration-notice"
      data-reason={migration.reason}
      role="status"
    >
      <div className="data-migration-copy">
        <strong>{t("dataMigration.title")}</strong>
        <span data-testid="data-migration-body">{body}</span>
        {migration.copied.length > 0 ? (
          <span className="data-migration-items" data-testid="data-migration-items">
            {t("dataMigration.copied", { items: migration.copied.join(", ") })}
          </span>
        ) : null}
      </div>
      <div className="data-migration-actions">
        <button
          type="button"
          className="data-migration-replay"
          data-testid="data-migration-replay"
          disabled={busy}
          onClick={onReplay}
        >
          {busy ? t("dataMigration.replaying") : t("dataMigration.replay")}
        </button>
        <button
          type="button"
          className="data-migration-dismiss"
          data-testid="data-migration-dismiss"
          onClick={onDismiss}
        >
          {t("dataMigration.dismiss")}
        </button>
      </div>
    </div>
  );
}

export function normalizeDesktopStatus(status) {
  if (!status || typeof status !== "object") {
    return null;
  }
  const apiBase = String(status.apiBase || "").replace(/\/+$/, "");
  const appVersion = normalizeAppVersion(status.appVersion || status.version);
  const agentProvider =
    status.agentProvider && typeof status.agentProvider === "object"
      ? {
          type: String(status.agentProvider.type || "local-openai-compatible"),
          apiBase: String(status.agentProvider.apiBase || apiBase).replace(/\/+$/, ""),
          openAiBaseUrl: String(
            status.agentProvider.openAiBaseUrl || (apiBase ? `${apiBase}/v1` : ""),
          ).replace(/\/+$/, ""),
          model: String(status.agentProvider.model || "formal-ai"),
        }
      : null;
  const engineSelectionAvailable = Array.isArray(status.engines) && status.engines.length > 0;
  const engines = engineSelectionAvailable
    ? status.engines
        .filter((engine) => engine && engine.available !== false && engine.id)
        .map((engine) => ({
          id: String(engine.id),
          label: String(engine.label || engine.id),
          type: String(engine.type || (engine.id === "out-of-box" ? "native" : "passthrough")),
          available: true,
        }))
    : [{ id: "out-of-box", label: "Out of the box", type: "native", available: true }];
  const requestedEngine = String(status.activeEngine || "out-of-box");
  const activeEngine = engines.some((engine) => engine.id === requestedEngine)
    ? requestedEngine
    : "out-of-box";
  return {
    shell: String(status.shell || "Electron"),
    platform: String(status.platform || ""),
    mode: String(status.mode || (apiBase ? "server" : "in-process")),
    apiBase,
    staticBase: String(status.staticBase || ""),
    graphUrl: String(status.graphUrl || (apiBase ? `${apiBase}/v1/graph` : "")),
    traceUrl: String(status.traceUrl || (apiBase ? `${apiBase}/v1/graph?trace=answer_greeting_hi` : "")),
    memory: String(status.memory || "formal_ai_bundle"),
    appVersion,
    agentModeDefault: Boolean(status.agentModeDefault),
    toolCallPolicy: String(status.toolCallPolicy || "explicit-permission"),
    apiReady: status.apiReady !== false && Boolean(apiBase),
    apiError: String(status.apiError || ""),
    debugToken: String(status.debugToken || ""),
    agentProvider,
    engines,
    activeEngine,
    engineSelectionAvailable,
    updater: normalizeDesktopUpdaterStatus(status.updater, appVersion),
  };
}

export function compactUrl(value) {
  if (!value) {
    return "unavailable";
  }
  try {
    const parsed = new URL(value);
    const pathName = parsed.pathname === "/" ? "" : parsed.pathname;
    return `${parsed.host}${pathName}`;
  } catch (_error) {
    return String(value);
  }
}

// Issue #353: the same chat UI now also runs inside a VS Code Webview. Label the
// host surface from the bridge's shell string ("VS Code" / "VS Code Web" from the
// extension; "Electron" from the desktop shell) so the status line and sidebar
// read correctly in either embedder.
export function desktopSurfaceLabel(status) {
  return /code/i.test(String((status && status.shell) || "")) ? "VS Code" : "Desktop";
}

export function desktopStatusLabel(status, agentMode) {
  if (!status) {
    return "";
  }
  const api = status.apiReady
    ? "API local"
    : status.apiError
      ? "API unavailable"
      : "in-process";
  const agent = agentMode ? "agent opted in" : "agent permission off";
  const engine = status.engineSelectionAvailable
    ? ` - ${String(status.activeEngine || "out-of-box")}`
    : "";
  return `${desktopSurfaceLabel(status)}${engine} - ${api} - ${agent}`;
}

export function desktopAgentEventLabel(payload) {
  const event = payload && payload.event && typeof payload.event === "object"
    ? payload.event
    : payload;
  if (!event || typeof event !== "object") return String(event || "");
  return String(
    event.content || event.text || event.part && event.part.text || event.message || event.summary ||
    event.type || "agent event",
  );
}

export function desktopMessages(history, text) {
  const messages = [];
  for (const entry of Array.isArray(history) ? history : []) {
    if (!entry || !["user", "assistant"].includes(entry.role)) {
      continue;
    }
    const content = typeof entry.content === "string" ? entry.content : "";
    if (content.trim()) {
      messages.push({ role: entry.role, content });
    }
  }
  messages.push({ role: "user", content: String(text || "") });
  return messages;
}

// R5d / Issue #514: push the explicit per-tool grants to the local router. Chat
// mode always sends false grants; Agent and Full Auto activate only the tools
// the user has individually granted.
export function syncDesktopToolGrants(bridge, mode, grants) {
  if (!bridge || typeof bridge.setToolGrants !== "function") {
    return;
  }
  Promise.resolve(bridge.setToolGrants(desktopToolRouterGrants(mode, grants))).catch(() => {});
}

export async function ensureDesktopAgentServer(bridge) {
  if (!bridge || typeof bridge.ensureAgentServer !== "function") {
    return null;
  }
  return normalizeDesktopStatus(await bridge.ensureAgentServer());
}

// Route a single tool call through the desktop bridge to the local process /
// Docker sandbox. Returns a structured refusal when the bridge is unavailable so
// callers never silently fall back to executing in the browser.
export async function requestDesktopToolCall(bridge, tool, input = {}) {
  if (!bridge || typeof bridge.invokeTool !== "function") {
    return {
      ok: false,
      tool: String(tool || ""),
      status: "unavailable",
      executed: false,
      reason: "desktop tool router is unavailable",
    };
  }
  const readOnly = [
    "web_search", "web_fetch", "read_file", "read_local_file", "grep", "glob",
    "list_directory", "read_many_files",
  ].includes(tool);
  if (!readOnly && typeof bridge.ensureAgentServer === "function") {
    await bridge.ensureAgentServer();
  }
  return bridge.invokeTool({ tool: String(tool || ""), input: input || {} });
}

export async function requestDesktopAgentProvider(bridge, request = {}) {
  if (!bridge || typeof bridge.runAgentProvider !== "function") {
    return null;
  }
  try {
    return await bridge.runAgentProvider(request || {});
  } catch (error) {
    return {
      ok: false,
      provider: "desktop",
      status: "error",
      executed: false,
      reason: error && error.message ? error.message : String(error),
    };
  }
}

export function chatAnswerFromAgentProviderResult(result) {
  if (!result || !result.answer || typeof result.answer !== "object") {
    return null;
  }
  return result.answer;
}

export function terminalCommandFromAnswer(answer) {
  const evidence = Array.isArray(answer && answer.evidence) ? answer.evidence : [];
  for (const item of evidence) {
    const text = String(item || "");
    if (text.startsWith("terminal:command:")) {
      const command = text.slice("terminal:command:".length).trim();
      if (command) {
        return command;
      }
    }
  }
  return "";
}

export function desktopShellCommand(command, status) {
  const value = String(command || "").trim();
  if (value === "ps" && status && status.platform === "win32") {
    return "tasklist";
  }
  return value;
}

export function shellOutputMarkdown(body, t) {
  const noOutput = typeof t === "function"
    ? t("permissions.message.noOutput")
    : "(no output)";
  const text = String(body || "").trim() || noOutput;
  const safe = text.replace(/```/g, "` ` `");
  return `\`\`\`text\n${safe}\n\`\`\``;
}

export function desktopToolResultReason(result, t) {
  const translate = (key, fallback) =>
    typeof t === "function" ? t(key) : fallback;
  if (!result) {
    return translate(
      "permissions.message.reasonNoResult",
      "desktop tool router returned no result",
    );
  }
  return (
    result.reason ||
    result.error ||
    result.status ||
    translate(
      "permissions.message.reasonRefused",
      "desktop tool router refused the request",
    )
  );
}

// R5c: reconcile the browser (IndexedDB) memory log with the native store over
// the local server's Links-Notation memory endpoints. Best-effort: a failed sync
// never blocks the conversation.
export async function syncDesktopMemory(bridge, lino) {
  if (!bridge || typeof bridge.syncMemory !== "function") {
    return null;
  }
  try {
    return await bridge.syncMemory({ lino: String(lino || "") });
  } catch (_error) {
    return null;
  }
}

function normalizeApiThinkingStep(entry) {
  if (!entry || typeof entry !== "object") return null;
  const step = String(entry.step || entry.kind || entry.source_event || "fallback").trim();
  const detail = String(entry.detail || entry.payload || entry.source_event || "").trim();
  if (!step && !detail) return null;
  const normalized = {
    step: step || "fallback",
    detail,
  };
  if (entry.summary !== undefined) normalized.summary = String(entry.summary);
  if (entry.id !== undefined) normalized.id = String(entry.id);
  if (entry.order !== undefined) normalized.order = entry.order;
  if (entry.level !== undefined) normalized.level = String(entry.level);
  if (entry.source_event !== undefined) normalized.sourceEvent = String(entry.source_event);
  if (entry.parent_id !== undefined && entry.parent_id !== null) {
    normalized.parentId = String(entry.parent_id);
  }
  return normalized;
}

export async function requestDesktopAnswer(text, history, desktopStatus, preferences = {}) {
  const apiBase = desktopStatus && desktopStatus.apiBase;
  if (!apiBase) {
    throw new Error("desktop API is unavailable");
  }

  const endpoint = `${apiBase}/v1/chat/completions`;
  const response = await fetch(endpoint, {
    method: "POST",
    headers: { "content-type": "application/json" },
    body: JSON.stringify({
      model: "formal-ai",
      messages: desktopMessages(history, text),
      temperature: normalizeSliderPreference(preferences.temperature, 0),
      stream: false,
    }),
  });

  if (!response.ok) {
    throw new Error(`desktop API returned ${response.status}`);
  }

  const payload = await response.json();
  const message =
    payload &&
    payload.choices &&
    payload.choices[0] &&
    payload.choices[0].message
      ? payload.choices[0].message
      : {};
  const answerText =
    message && message.content !== undefined ? String(message.content || "") : "";
  if (!answerText) {
    // The caller asks the in-page worker instead of inventing an answer.
    throw new Error("desktop API returned no answer");
  }
  const apiThinkingSteps = Array.isArray(message.thinking_steps)
    ? message.thinking_steps.map(normalizeApiThinkingStep).filter(Boolean)
    : [];
  const fallbackDesktopSteps = [
    { step: "desktop_shell", detail: "Electron preload bridge supplied local API status" },
    { step: "http_chat", detail: "POST /v1/chat/completions on the local Rust server" },
    { step: "memory", detail: "UI import/export stays on formal_ai_bundle" },
  ];

  return {
    intent: "desktop_http_chat",
    content: answerText,
    source: "desktop_http",
    evidence: [
      "surface:desktop",
      "api:/v1/chat/completions",
      desktopStatus.graphUrl ? "network:/v1/graph" : "",
    ].filter(Boolean),
    steps: apiThinkingSteps.length > 0 ? apiThinkingSteps : fallbackDesktopSteps,
    diagnostics: {
      providers: [
        {
          id: "formal_ai_desktop_http",
          status: "ok",
          endpoint,
        },
      ],
      http: [
        {
          provider: "formal_ai_desktop_http",
          url: endpoint,
          method: "POST",
          status: response.status,
          ok: response.ok,
        },
      ],
    },
  };
}
