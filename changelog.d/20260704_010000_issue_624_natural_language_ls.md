### Fixed
- Routed natural-language current-directory listing prompts, such as "what files are in this folder?", to agent-mode shell tool calls with `{"command":"ls"}` instead of falling through to the unknown-answer response.
