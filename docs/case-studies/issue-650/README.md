# Issue 650: consistent `formal-ai with` behavior

## Reproduction and root causes

The issue report is preserved verbatim as API JSON in
`raw-data/issue.json`. The reported direct `/responses` request reproduced a
protocol asymmetry: `src/protocol/recording.rs::response_prompt` joined the
Responses `instructions` field to `input`, while chat completions selected the
latest user message and retained system/developer messages as history. Thus a
20 KB Codex policy became part of the task and displaced `hi` during routing.

The wrapper inconsistency was likewise structural. Codex's `exec` was an
unconditional `prepend_arg`, and the integration schema had no representation
for interactive versus one-shot invocation. With no prompt, several tools were
therefore forced down a headless path. The permanent-configuration spelling was
only declared as `--global`. Finally, conversation-summary prompts with inline
content were recognized as conversation operations, but the handler returned
`None` when there was no prior server-side event log.

## Implemented behavior

- Responses routing now selects the latest user item and never concatenates
  top-level instructions into the task text.
- The integration seed declares interactive and non-interactive native
  arguments for all eight tools. No message defaults to interactive; a message
  defaults to one-shot. `--interactive`, `--non-interactive`, `--print`, and
  `--one-shot` provide uniform overrides.
- Existing native mode arguments are de-duplicated.
- Inline summarize/compact requests use the existing
  formalize-summarize-deformalize dialog pipeline when no prior event-log turns
  exist.
- `--globally` aliases `--global`.
- Ephemeral modes continue to use isolated temporary homes/config paths; the
  all-tools persistent-config regression covers both mode paths.

## Verification

Per-requirement tests cover Responses instruction isolation, all eight mode
mappings, inline summarization, the alias, and persistent-config invariance.
The whole wrapper integration test exercises every registered tool through the
same seed-driven path. Agent-CLI evidence and command logs are captured beside
this document.
