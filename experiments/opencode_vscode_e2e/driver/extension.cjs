const fs = require("node:fs")
const path = require("node:path")
const vscode = require("vscode")

async function activate() {
  const signal = process.env.OPENCODE_VSCODE_E2E_SIGNAL
  const prompt = process.env.OPENCODE_VSCODE_E2E_PROMPT
  if (!signal || !prompt) return

  const result = { caller: "", config: "", error: "", terminal: "" }
  try {
    const extension = vscode.extensions.getExtension("sst-dev.opencode")
    if (!extension) throw new Error("sst-dev.opencode is not installed")
    await extension.activate()
    await vscode.commands.executeCommand("opencode.openNewTerminal")

    const terminal = vscode.window.terminals.find((candidate) => candidate.name === "opencode")
    if (!terminal) throw new Error("official extension did not create its opencode terminal")
    result.terminal = terminal.name
    result.caller = String(terminal.creationOptions.env?.OPENCODE_CALLER || "")
    result.config = String(process.env.OPENCODE_CONFIG || "")
    if (result.caller !== "vscode") {
      throw new Error(`expected OPENCODE_CALLER=vscode, got ${result.caller || "<empty>"}`)
    }

    fs.copyFileSync(result.config, path.join(path.dirname(signal), "opencode-config.json"))
    terminal.dispose()
  } catch (error) {
    result.error = error instanceof Error ? error.stack || error.message : String(error)
  } finally {
    fs.writeFileSync(signal, `${JSON.stringify(result)}\n`)
    await vscode.commands.executeCommand("workbench.action.quit")
  }
}

module.exports = { activate }
