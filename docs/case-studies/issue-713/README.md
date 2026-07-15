# Issue 713: valid interactive CLI invocations

## Reproduction and root cause

The wrapper's seed declared Codex's `exec` subcommand as mode-specific but kept
`--skip-git-repo-check` in the unconditional argument list. Bare interactive
Codex therefore received a flag accepted only by `codex exec` and exited during
argument parsing.

Gemini and Qwen declared `--prompt-interactive` as an unconditional interactive
mode flag. Both upstream CLIs define that option as a string taking an initial
prompt, so the wrapper emitted an incomplete option when interactive mode had no
prompt. The previous regression test only asserted that each interactive argv
omitted its headless flag; it did not compare the complete argv or launch under
a PTY.

The failing pre-fix test output is preserved in the solution-draft log and was
independently reproduced by the exact-argv matrix before implementation.

## Implemented behavior

- Codex's `--skip-git-repo-check` is now emitted only beside the non-interactive
  `exec` subcommand. Bare Codex retains the shared sandbox and provider config.
- The seed schema can mark interactive flags as requiring a prompt. Gemini and
  Qwen omit `--prompt-interactive` for an empty TUI launch and retain
  `--prompt-interactive <prompt>` when `--interactive` seeds a session.
- Mode-injected subcommands keep their model option beside the subcommand, so
  OpenCode receives `run -m <model> <prompt>`.
- The integration suite snapshots the exact interactive and non-interactive
  argv for all eight advertised tools. A PTY smoke test launches every default
  interactive path, sends input, and requires a rendered response without an
  argument-parser error.
- Existing global/undo and persistent-config isolation tests continue to cover
  the configuration paths.

## Agent CLI evidence

The documented `examples/self-coding/run.sh --live` entry point was attempted
first. `solve` 2.6.0 rejected `formal-ai` as an Agent CLI model before launching;
the captured output is in `agent-cli-self-coding.log`.

The lower-level real round-trip then booted `formal-ai serve --agent-mode` and
drove `@link-assistant/agent` 0.25.0 through the local OpenAI-compatible
endpoint. Formal AI received the task and issued a tool call, but selected the
generic `make` command; this repository has no Makefile, so that authoring
attempt ended without edits. The unmodified transcript and server trace are in
`agent-cli-e2e-run.log` and `formal-ai-server.log`.
