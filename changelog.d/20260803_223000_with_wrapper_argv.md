---
bump: minor
---

### Fixed
- `formal-ai with <tool>` now parses the caller's arguments into a structured request (prompt plus options) and re-renders it in each client's own vocabulary instead of concatenating strings (issue #903):
  - an already-qualified `provider/model` selector is no longer given a second `formalai/` prefix;
  - everything after the tool name is forwarded to the client, so a flag the wrapper also defines (`--verbose`) is no longer swallowed;
  - interactive mode follows `isatty(stdin)`, so a piped, headless run is never given `--interactive`;
  - a piped prompt is rendered as that client's prompt argument, so `codex` keeps its `exec` subcommand and is never handed `-p`;
  - the completion ladder re-renders the caller's own option set with only the prompt substituted, so `--dangerously-skip-permissions`, `--mcp-config` and `--disallowedTools` survive a retry and the wrapper's overlay is no longer duplicated.
