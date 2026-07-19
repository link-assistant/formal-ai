# Issue 747: native desktop web search and common tools

## Reproduction

On v0.297.1 the Electron header could report `agent permission off`, but “Search for X” stayed inside the browser worker. It queried only CORS-safe sources and produced the same DuckDuckGo/Wikipedia result as the web demo. The native router advertised neither `web_search` nor a browser-capture executor, denied every ungranted tool, and the VS Code bridge refused every tool while its optional server was off.

The original issue and folded scope-widening comment were captured through the GitHub API before implementation. The live issue-solving wrapper transcript is retained outside the repository at `/tmp/issue-747-agent-cli.log`; it stopped before execution because the wrapper no longer accepts its documented `formal-ai` model alias. The repository's local Agent CLI fixture is used for the required end-to-end proof instead.

## Root cause

The browser worker already recognizes multilingual search/fetch intent and emits structured tool calls, but the renderer returned that browser answer unchanged. Electron and VS Code shared a six-tool router where all tools—including reads—used one default-deny gate, and both native hosts required the optional local server before dispatch. There was no installed web-search/web-capture dependency or packaged Chromium runtime.

## Implementation

- `@link-assistant/web-search` browser providers visit Google, Bing, and DuckDuckGo concurrently through `@link-assistant/web-capture` Playwright pages; upstream RRF merges and deduplicates the results.
- `web_fetch` navigates a headless page and returns its rendered DOM, covering JavaScript and CORS-limited pages.
- The renderer promotes the symbolic worker's existing `web_search`, `http_fetch`, and `url_navigate` tool calls to native read-only calls. If the native path is unavailable, the existing browser answer remains a safe fallback.
- Electron and the VS Code Node host allow read-only web/file tools without agent permission or the local server. Writes, edits, shell, and code execution remain permission-gated.
- The common specialized tool vocabulary includes read/write/edit/multi-edit, grep/glob/list/read-many, shell/bash, web search/fetch, todo/plan, and subagent/task. Specialized operations are selected before the shell fallback and aliases normalize by capability.
- Desktop and VS Code packaging install Chromium during prebuild and bundle the executable outside ASAR/the extension archive so release artifacts work without user setup.

## Verification

The regression suite first failed because the modules and permission-free routes did not exist. It now covers:

- Google, Bing, and DuckDuckGo browser navigation and multi-source RRF;
- rendered JavaScript page fetch;
- agent-permission-off search/fetch in the shared router and server-off VS Code bridge;
- forty recognized-search variants (ten each in English, Russian, Hindi, and Chinese) handed from the symbolic answer to the native tool;
- an actual browser-level chat flow with agent permission off and no local API server;
- permission-free specialized inspection and explicitly granted write/edit/multi-edit operations; and
- release-browser cleanup and self-contained Electron/VSIX packaging.

The repository's real Agent CLI fixture also passed against a release build. Its
stream, formal-ai trace, replay session, general change plan, and resulting diff
are retained in [`agent-cli-evidence/`](agent-cli-evidence/).

Raw local test and self-coding evidence is stored in [`raw-data/`](raw-data/).
