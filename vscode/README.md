# formal-ai for VS Code

A symbolic, deterministic assistant inside VS Code — the **same chat UI as the [formal-ai](https://github.com/link-assistant/formal-ai) web app**, embedded in a Webview around the same HTTP/web boundary. No neural-network inference; every answer is derived symbolically and is fully traceable.

The extension ships **two hosts from one manifest**, so it runs both on the desktop and in the browser:

- **Desktop / remote** (VS Code desktop, Remote-SSH, Codespaces, dev containers) — reports `shell: "VS Code"`. With the opt-in `formal-ai.server.enabled` setting it starts a loopback `formal-ai serve` process and routes chat through `POST /v1/chat/completions`, and can drive Docker-sandboxed code execution.
- **Web** (`vscode.dev`, `github.dev`) — reports `shell: "VS Code Web"`. The browser sandbox cannot spawn a process, so it stays on the in-process WebAssembly symbolic engine while exposing the same chat, network, memory, and permission surfaces.

## Features

- Symbolic chat with markdown rendering and traceable reasoning.
- Optional **links network view** of the reasoning (`GET /v1/network`).
- Full-memory **export / import** of the self-contained `formal_ai_bundle` Links-Notation document.
- **Agent mode** off by default; tool calls (HTTP fetch, file read, host shell, and sandboxed code execution) are permission-gated until you opt in.
- Multilingual UI (English, Russian, Hindi, Chinese).

## Commands

| Command | Description |
|---|---|
| `formal-ai: Open Chat` | Focus the chat view. |
| `formal-ai: Toggle Local Server` | Start/stop the local `formal-ai serve` process (desktop host only). |
| `formal-ai: Sync Memory` | Reconcile browser memory with the native store (requires the local server). |
| `formal-ai: Open Network View` | Open the links network view. |

## Settings

| Setting | Default | Description |
|---|---|---|
| `formal-ai.server.enabled` | `false` | Start a local OpenAI-compatible server and route chat through it (desktop host only). |
| `formal-ai.server.host` | `127.0.0.1` | Loopback host the server binds to. |
| `formal-ai.server.port` | `18080` | Port the server binds to. |
| `formal-ai.docker.image` | `konard/box-dind:2.1.1` | Image used to sandbox code-execution tool calls. |
| `formal-ai.tools.allowByDefault` | `false` | Grant tool calls by default (off = default-deny). |
| `formal-ai.agent.defaultOn` | `false` | Open the chat with agent mode on. |

The opt-in local server and Docker code execution only run in trusted desktop windows; the in-process symbolic agent is safe everywhere, so the extension supports virtual and untrusted workspaces.

## Development

```bash
npm run vscode:test     # node:test unit suite + static smoke check (from the repo root)
npm run vscode:dev      # launch in a browser host via @vscode/test-web
npm run vscode:package  # produce a .vsix (runs prepare-resources first)
```

In VS Code desktop you can also open the `vscode/` folder and press **F5** (Run Extension).

See [docs/vscode/extension.md](../docs/vscode/extension.md) for the full architecture — the dual-host design, the Webview sandbox reconciliation (CSP nonce, same-origin Worker bootstrap, seed rebasing), packaging, and the honest list of what is and isn't verified in CI.

## License

[Unlicense](https://github.com/link-assistant/formal-ai/blob/main/LICENSE) — the same as the parent project.
