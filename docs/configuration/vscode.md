# VS Code setup

## Install

```bash
curl -fsSL https://raw.githubusercontent.com/link-assistant/formal-ai/main/scripts/install.sh | sh -s -- vscode
```

Open the **Formal AI** activity view or run **Formal AI: Open Chat**. The same
extension has two hosts:

- the desktop/remote **Node desktop host** can opt into `formal-ai serve`, local
  files, specialized tools, Docker, and native shared memory;
- the `vscode.dev`/`github.dev` **web host** stays in-process because a Web
  Worker cannot spawn processes, open local sockets, or read host files.

Enable `formal-ai.server.enabled` only on the Node host to use the local API.
Settings for the server binary/port, Docker image, default agent mode, and tool
permissions map directly to the status shown by the Webview.

The Node host reconciles Webview IndexedDB with the same shared memory at
`~/.formal-ai/memory.lino` (or `%APPDATA%\formal-ai\memory.lino`). The web host
cannot access that file; use **Export memory** and **Import memory** there.

Verify that the status says `VS Code - in-process` without a server and changes
to the ready local API path after enabling it. Package checks use
`npm run vscode:test`, `npm run vscode:smoke`, and `npm run vscode:package`.
