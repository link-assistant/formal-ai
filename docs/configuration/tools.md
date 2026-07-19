# Tools reference

Formal AI routes by **capability**, not by a vendor's spelling of a tool name.
For example `read`, `read_file`, and schema-compatible variants map to the same
read capability. It chooses a matching specialized tool first; **bash** (or the
client's shell equivalent) is the fallback only when no advertised specialized
tool covers the operation. Read-only tools are permission-free on native Node
hosts; writes and shell require the active permission policy.

## Internal tools

These are symbolic engine operations and do not delegate to an agent harness:

| Capability | Complete internal tools registry |
| --- | --- |
| Reasoning/routing | `intent_routing`, `concept_lookup`, `fact_lookup`, `coreference`, `roleplay` |
| Generation | `write_program`, `brainstorm`, `summarize_conversation` |
| Calculation | `calculator` |
| Web/browser | `http_fetch`, `url_navigate`, `wikipedia_lookup`, browser `web_search` |
| Memory | `append_memory`, `conversation_recall`, `export_memory`, `import_memory` |
| Browser sandbox | `read_local_file`, `eval_js` |

The complete metadata—inputs, outputs, isolation, sources, and localized notes—
lives in `data/seed/tools.lino`.

## External tools

Agentic hosts advertise schema-bearing tools. The shared capability set covers:

- files: `read_file`, `read_many_files`, `write_file`, `edit_file`, `multi_edit`;
- discovery: `grep`, `glob`, `list_directory`;
- web: `web_search`, `web_fetch`, `http_fetch`, Playwright capture, and ranked
  Google/Bing/DuckDuckGo fusion where the host supplies them;
- planning: `todo`, `plan`, `task`, and `subagent`;
- execution: `shell`/`bash` and isolated `code_exec_box_dind`.

OpenAI hosted tool types such as `web_search`, `web_search_preview`, and
versioned `web_search_*` definitions normalize to the same web-search
capability. Anthropic/OpenAI function definitions are inspected the same way.
The advertising client or host executes hosted calls; the HTTP server emits the
protocol-native call and retains the returned bytes.

## Capability-to-tool mapping by environment

| Environment | Specialized capabilities | Execution/fallback |
| --- | --- | --- |
| Browser demo | seeded reasoning, calculator, Wikipedia, fetch/navigation, user-selected file, IndexedDB memory | worker-only `eval_js`; no host bash |
| CLI/library | symbolic chat, dataset, memory/bundle, server, Telegram | subcommands; external execution only through an agent client |
| HTTP server | protocol adapters, permission gate, hosted web search/fetch normalization | advertising harness executes calls; `--agent-mode` enables agent calls |
| Desktop | read/write/edit, grep/glob/list, web search/fetch/capture, todo/plan/task/subagent, memory sync | permission-gated shell/bash, isolated Docker execution |
| VS Code Node | Desktop-equivalent specialized tools and memory sync | permission-gated bash; Docker when configured |
| VS Code Web | in-process browser tool set | no process, filesystem, socket, or bash |
| Telegram | shared symbolic intents and HTML replies | no arbitrary host shell |
| Docker | Telegram/API/agent commands plus inner-Docker isolation | container shell and `start-command`, not the host socket |

`data/seed/environments.lino` is the authoritative full per-environment list.
When debugging selection, inspect the advertised JSON schema and server trace:
a specialized tool should win over bash, and the emitted arguments must use the
field names in that tool's schema.

## Complete environment inventory

This inventory deliberately uses the exact seed identifiers so the automated
documentation contract can detect drift:

- Browser: `intent_routing`, `write_program`, `concept_lookup`, `fact_lookup`,
  `summarize_conversation`, `brainstorm`, `coreference`, `roleplay`,
  `wikipedia_lookup`, `calculator`, `http_fetch`, `url_navigate`, `web_search`,
  `conversation_recall`, `read_local_file`, `eval_js`, `append_memory`,
  `export_memory`, `import_memory`.
- Rust library: the shared reasoning tools plus `recall`, `telegram_html`, and
  `http_chat`.
- CLI: `chat`, `dataset`, `memory`, `bundle`, `serve`, `telegram`.
- HTTP server: `v1_chat_completions`, `v1_responses`, `v1_graph`, `v1_bundle`,
  `telegram_webhook`, `agent_permission_gate`, `tool_router`, `web_search`,
  `web_fetch`, `http_fetch`.
- Desktop: `desktop_shell`, `formal-ai serve`, `v1_chat_completions`,
  `v1_graph`, `export_memory`, `import_memory`, `agent_permission_gate`,
  `tool_router`, `read_file`, `write_file`, `edit_file`, `multi_edit`, `shell`,
  `bash`, `grep`, `glob`, `list_directory`, `read_many_files`, `web_search`,
  `web_fetch`, `web_capture_playwright`, `rrf_google_bing_duckduckgo`, `todo`,
  `plan`, `subagent`, `task`, `code_exec_box_dind`, `memory_sync`.
- VS Code: `vscode_webview` followed by the same server, file, web, planning,
  execution, and memory-sync identifiers as Desktop.
- Telegram: shared reasoning plus `html_replies`.
- Docker: `telegram_polling`, `telegram_webhook`, `start_command`,
  `docker_isolation`, `inner_docker_daemon`, `bundle`, `memory`.
