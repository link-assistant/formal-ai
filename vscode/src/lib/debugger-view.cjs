"use strict";
const { createChatViewProvider } = require("./chat-view.cjs");

// Reuse the existing resource roots, CSP, bridge and exact app projection.
function registerDebugger({ vscode, context, host }) {
  return vscode.commands.registerCommand("formal-ai.openDebugger", async () => {
    const panel = vscode.window.createWebviewPanel("formal-ai.debugger", "Formal AI Debugger", vscode.ViewColumn.Beside, {
      enableScripts: true, retainContextWhenHidden: true,
    });
    const localSubscriptions = [];
    const localContext = { extensionUri: context.extensionUri, subscriptions: localSubscriptions };
    const provider = createChatViewProvider({ vscode, context: localContext, host: { ...host, debuggerView: true } });
    try { await provider.resolveWebviewView({ webview: panel.webview }); }
    catch (error) { panel.dispose(); throw error; }
    panel.onDidDispose(() => { for (const subscription of localSubscriptions) subscription.dispose(); });
    context.subscriptions.push(panel);
  });
}
module.exports = { registerDebugger };
