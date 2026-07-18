"use strict";

// Host-agnostic implementation of the `FormalAiDesktop` contract.
//
// Issue #353 (ROADMAP D2/D3): the web chat UI talks to its host through a small
// bridge object — `window.FormalAiDesktop` — exposing `getStatus`,
// `openExternal`, `setToolGrants`, `invokeTool` and `syncMemory` (see
// `desktop/preload.cjs`). The Electron shell backs that contract with IPC
// handlers in `desktop/main.cjs`; the VS Code extension backs the *same*
// contract with this dispatcher, reached over a `postMessage` channel from the
// webview (see `webview-html.cjs`). Reusing the contract verbatim means the web
// app needs no shell-specific code.
//
// Every effectful dependency is injected, so the policy (default-deny tool
// routing, server-gated memory sync, http(s)-only external links) is unit
// testable without a live `vscode` host, Docker daemon, or network. The tool
// router and memory-sync client are the very modules the desktop shell uses
// (`desktop/lib/tool-router.cjs`, `desktop/lib/memory-sync.cjs`), reused as-is.

function refusedTool(tool, reason) {
  return {
    ok: false,
    tool: tool ? String(tool) : "",
    status: "refused",
    executed: false,
    reason,
  };
}

function unavailableTool(tool, reason) {
  return {
    ok: false,
    tool: tool ? String(tool) : "",
    status: "unavailable",
    executed: false,
    reason,
  };
}

function createBridge(options = {}) {
  const getStatus = typeof options.getStatus === "function" ? options.getStatus : () => ({});
  const toolRouter = options.toolRouter || null;
  const openExternalImpl =
    typeof options.openExternal === "function" ? options.openExternal : null;
  const getMemorySync =
    typeof options.getMemorySync === "function"
      ? options.getMemorySync
      : () => options.memorySync || null;
  // Server availability gate. On the Node host this tracks
  // `formal-ai.server.enabled` (and a healthy server); on the web host it is
  // always false, so tool routing and memory sync are refused — there is no
  // local process or Docker daemon in a browser.
  const isServerEnabled =
    typeof options.serverEnabled === "function"
      ? options.serverEnabled
      : () => Boolean(options.serverEnabled);

  async function status() {
    return getStatus();
  }

  // Drive the default-deny grant map from the renderer's permission toggles.
  // This always succeeds (it only records intent); whether a granted tool then
  // runs is decided by `invokeTool` + the router.
  async function setToolGrants(grants) {
    if (!toolRouter) {
      return { all: false };
    }
    return toolRouter.setGrants(grants);
  }

  // Server mode is required: tool routing only makes sense once the local app
  // is the execution surface. Read-only capabilities can use the Node host
  // without starting that optional server.
  async function invokeTool(request) {
    const tool = request && request.tool ? String(request.tool) : "";
    const readOnly = Boolean(toolRouter && toolRouter.isReadOnly && toolRouter.isReadOnly(tool));
    if (!isServerEnabled() && !readOnly) {
      return refusedTool(
        tool,
        "tool routing requires the local server (formal-ai.server.enabled)",
      );
    }
    if (!toolRouter) {
      return unavailableTool(tool, "no tool router is configured");
    }
    return toolRouter.invoke(request);
  }

  // Reconcile browser (IndexedDB) memory with the native store over the local
  // server's Links-Notation memory endpoints. Requires the server (for the
  // endpoints) and a known `apiBase`.
  async function syncMemory(payload) {
    const current = getStatus() || {};
    if (!isServerEnabled() || !current.apiBase) {
      return { ok: false, status: "unavailable", reason: "memory sync requires the local server" };
    }
    const memorySync = getMemorySync();
    if (!memorySync) {
      return { ok: false, status: "unavailable", reason: "memory sync is not configured" };
    }
    try {
      const outbound = payload && typeof payload.lino === "string" ? payload.lino : "";
      const pushed = outbound.trim() ? await memorySync.push(outbound) : { added: 0, total: 0 };
      const pulled = await memorySync.pull();
      return { ok: true, status: "ok", pushed, pulled };
    } catch (error) {
      return {
        ok: false,
        status: "error",
        reason: error && error.message ? error.message : String(error),
      };
    }
  }

  // Only ever hand http(s) links to the host opener.
  async function openExternal(url) {
    if (typeof url === "string" && /^https?:\/\//i.test(url) && openExternalImpl) {
      await openExternalImpl(url);
      return true;
    }
    return false;
  }

  const methods = {
    getStatus: status,
    setToolGrants,
    invokeTool,
    syncMemory,
    openExternal,
  };

  // Dispatch a single `{ method, payload }` RPC from the webview shim. Unknown
  // methods are reported rather than thrown, so a malformed message can never
  // crash the extension host.
  async function dispatch(method, payload) {
    const handler = methods[method];
    if (typeof handler !== "function") {
      throw new Error(`unknown bridge method: ${method || "(none)"}`);
    }
    return handler(payload);
  }

  return { ...methods, dispatch };
}

module.exports = { createBridge };
