# Requirements trace

| ID | Requirement | Evidence |
|---|---|---|
| R1 | Route the exact reported desktop-folder request locally. | `reported_desktop_request_uses_find_instead_of_the_web` asserts a Bash `find` rooted at Desktop with `-type d`. |
| R2 | Locate the real `hive-control-center` spelling despite the longer spoken name. | `fuzzy_find_command_locates_the_reported_folder_name` executes the generated command against the isolated `Archive/hive-control-center` fixture. |
| R3 | Do not consult unrelated or dead web pages for a local path. | The planner prioritizes explicit local path intent; unit and E2E checks reject every web tool call. |
| R4 | Support both folder and file discovery. | Seeded kind associations map folder/directory terms to `-type d` and file/document terms to `-type f`. |
| R5 | Support natural-language variations of `find`. | `every_seeded_local_path_phrase_routes_to_find` walks every declared action, scope, and kind cue; the seed contains English, Russian, Hindi, and Chinese phrases. |
| R6 | Solve the class rather than encode this folder name. | Production code contains no hive-specific term. It composes bounded `-iname` patterns from arbitrary request words and maps arbitrary seeded scopes/kinds. |
| R7 | Preserve a clear local-search/web-search boundary. | A local scope is mandatory for path lowering. `open_web_find_requests_still_use_web_search` pins two open-world requests to `websearch`. |
| R8 | Cover OpenCode and the other CLI surfaces with E2E tests. | `run_issue_819.sh` runs real Agent, OpenCode, Claude, and Codex installations against the release server. |
| R9 | Use `link-foundation/command-stream` to capture TUI rendering frames. | `tui-transcript.mjs` streams a PTY with `command-stream@0.14.1` and renders chunks with `@xterm/headless@6.0.0`. |
| R10 | Deduplicate frames and verify the entire TUI dialog, not only CLI text. | The TUI unit fixture pins frame deduplication; the real OpenCode run retains every distinct rendered frame and separately validates user → call → result → final server exchanges. |
| R11 | Include unit, integration, CLI, and TUI layers. | `tests/unit/issue_819.rs`, `tests/integration/issue_819_local_path_search.rs`, the four-client harness, and its TUI fixture/capture cover all four layers. |
| R12 | Prevent protocol drift across Agent/OpenCode, Claude, and Codex. | Native Chat Completions, Anthropic Messages, and Responses integration regressions assert the correct protocol-specific tool name and common `find` semantics. |
| R13 | Reproduce the bug before implementing the fix. | The first atomic test commit failed with the exact prompt routing to `websearch`; the raw red output is retained under `raw-data/tests/`. |
| R14 | Exercise the repository's self-hosting workflow. | A release Formal AI server drove Agent CLI through plan, Write, read-back Run, and Final steps; all raw evidence is under `raw-data/self-authoring/`. |
| R15 | Keep the change releasable and auditable. | A patch changelog fragment, CI E2E step, issue/PR JSON, raw native dialogs, focused logs, and this case study ship with the fix. |
| R16 | Generalize beyond hand-picked examples. | A Formal-AI-authored 56-case benchmark crosses en/ru/hi/zh, Desktop/home/current scopes, and file/directory kinds; its executable regression also pins the scope-kind ambiguity it uncovered. |
