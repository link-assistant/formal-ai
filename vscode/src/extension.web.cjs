"use strict";

// Web extension host — vscode.dev, github.dev, and other browser-based VS Code.
//
// Issue #353 (R6 / ROADMAP V5): the same committed `src/web/` chat UI runs in a
// browser-based VS Code, where the extension executes inside a Web Worker. A
// browser cannot spawn a process or talk to Docker, so this host is *always*
// in-process: it advertises a status with no `apiBase`, and the web app falls
// back to its in-browser symbolic engine (the same WASM worker the public demo
// uses). Tool routing and memory sync are reported as unavailable rather than
// silently executing in the browser.
//
// To stay loadable in a Web Worker this module imports ONLY the pure shared
// libraries — no `node:*` builtins, no `vscode` at module scope beyond the host
// API itself, and none of the desktop tool-router / server-process code.

const vscode = require("vscode");

const { statusFromConfig } = require("./lib/config.cjs");
const { createBridge } = require("./lib/bridge.cjs");
const { createChatViewProvider } = require("./lib/chat-view.cjs");

const SHELL = "VS Code Web";
const VIEW_ID = "formal-ai.chatView";

function activate(context) {
  // `serverCapable: false` pins the surface to in-process regardless of the
  // `formal-ai.server.enabled` setting — there is no process to start here.
  const status = statusFromConfig(vscode.workspace.getConfiguration("formal-ai"), {
    shell: SHELL,
    serverCapable: false,
  });

  const appVersion = (context.extension && context.extension.packageJSON
    && context.extension.packageJSON.version) || "";

  // No tool router, no memory sync, server permanently off: `invokeTool` is
  // refused and `syncMemory` is unavailable, but `getStatus`, `setToolGrants`
  // (records intent) and `openExternal` still work.
  const bridge = createBridge({
    getStatus: () => status,
    serverEnabled: false,
    openExternal: (url) => vscode.env.openExternal(vscode.Uri.parse(url)),
  });

  const host = { appVersion, getStatus: () => status, bridge };

  const provider = createChatViewProvider({ vscode, context, host });
  context.subscriptions.push(
    vscode.window.registerWebviewViewProvider(VIEW_ID, provider, {
      webviewOptions: { retainContextWhenHidden: true },
    }),
  );

  context.subscriptions.push(
    vscode.commands.registerCommand("formal-ai.openChat", async () => {
      try {
        await vscode.commands.executeCommand(`${VIEW_ID}.focus`);
      } catch (_error) {
        await vscode.commands.executeCommand("workbench.view.extension.formal-ai");
      }
    }),
    vscode.commands.registerCommand("formal-ai.toggleServer", () => {
      vscode.window.showInformationMessage(
        "formal-ai: the local server is desktop-only. The web extension always runs the in-process symbolic agent.",
      );
    }),
    vscode.commands.registerCommand("formal-ai.syncMemory", () => {
      vscode.window.showInformationMessage(
        "formal-ai: memory sync requires the desktop local server; the web extension keeps memory in the browser.",
      );
    }),
    vscode.commands.registerCommand("formal-ai.openNetworkView", async () => {
      try {
        await vscode.commands.executeCommand(`${VIEW_ID}.focus`);
      } catch (_error) {
        /* view may not be registered yet */
      }
      vscode.window.showInformationMessage(
        "formal-ai: the network view renders in the chat panel from the in-process engine.",
      );
    }),
  );

  return { getStatus: () => status, isServerRunning: () => false };
}

function deactivate() {}

module.exports = { activate, deactivate };
