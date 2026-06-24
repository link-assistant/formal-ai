"use strict";

const { contextBridge, ipcRenderer } = require("electron");

contextBridge.exposeInMainWorld("FormalAiDesktop", {
  getStatus: () => ipcRenderer.invoke("formalAiDesktop:getStatus"),
  openExternal: (url) => ipcRenderer.invoke("formalAiDesktop:openExternal", url),
  checkForUpdates: () => ipcRenderer.invoke("formalAiDesktop:checkForUpdates"),
  installUpdate: () => ipcRenderer.invoke("formalAiDesktop:installUpdate"),
  // Issue #554 (R2): one-click install of the VS Code extension from the
  // already-installed Desktop app (downloads the latest release .vsix and runs
  // `code --install-extension`).
  installVsCodeExtension: () => ipcRenderer.invoke("formalAiDesktop:installVsCodeExtension"),
  onUpdateStatus: (callback) => {
    if (typeof callback !== "function") {
      return () => {};
    }
    const listener = (_event, status) => callback(status);
    ipcRenderer.on("formalAiDesktop:updateStatus", listener);
    return () => ipcRenderer.removeListener("formalAiDesktop:updateStatus", listener);
  },
  // Issue #515: Agent / Full Auto mode auto-starts the local
  // OpenAI-compatible server and exposes its apiBase for provider wiring.
  ensureAgentServer: () => ipcRenderer.invoke("formalAiDesktop:ensureAgentServer"),
  // R5d: drive the default-deny tool-call gate and route permitted tools through
  // the local process / Docker sandbox.
  setToolGrants: (grants) => ipcRenderer.invoke("formalAiDesktop:setToolGrants", grants),
  invokeTool: (request) => ipcRenderer.invoke("formalAiDesktop:invokeTool", request),
  // Issue #516: swappable agent execution provider (in-process by default,
  // agent-commander when explicitly selected).
  runAgentProvider: (request) => ipcRenderer.invoke("formalAiDesktop:runAgentProvider", request),
  // R5c: reconcile browser (IndexedDB) memory with the native store.
  syncMemory: (payload) => ipcRenderer.invoke("formalAiDesktop:syncMemory", payload),
  // Issue #438 (follow-up): one-click start/stop of the prepared Telegram bot and
  // OpenAI-compatible server Docker containers.
  serviceStatus: () => ipcRenderer.invoke("formalAiDesktop:serviceStatus"),
  startService: (request) => ipcRenderer.invoke("formalAiDesktop:startService", request),
  installAgentEnvironment: () => ipcRenderer.invoke("formalAiDesktop:installAgentEnvironment"),
  stopService: (request) => ipcRenderer.invoke("formalAiDesktop:stopService", request),
});
