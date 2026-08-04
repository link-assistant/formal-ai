# Issue 909: `--global` wrote an incomplete headless configuration

This case study records the reproduction, root cause, implementation, and
verification for the gap reported in issue
[#909](https://github.com/link-assistant/formal-ai/issues/909): `formal-ai with
<tool> --global` wrote shell exports and nothing else, so gemini and qwen still
refused to start headlessly while `--global` reported success and exited 0.

## Timeline

- **2026-08-02** — konard filed #909 with a reproduction script and a stored
  probe log from `formal-ai 0.317.0` (both hosted in `link-assistant/hive-mind`,
  issue 2130). The report names the two refusals verbatim:
  `Invalid auth method selected.` (gemini) and `No auth type is selected.
  Please configure an auth type … before running in non-interactive mode.`
  (qwen), and proposes a three-part fix.
- **2026-08-04** — this branch implemented the fix. `raw-data/issue.json`,
  `raw-data/issue-comments.json` (empty — the issue carries no comments), and
  `raw-data/pr-before.json` were captured before finalization.

## Requirements

Enumerated as R909-1..R909-6 in
[`REQUIREMENTS.md`](../../../REQUIREMENTS.md) and pinned to their tests in
[`docs/requirements-traceability.md`](../../requirements-traceability.md):

1. **R909-1** — `--global` materialises every file a headless start needs, not
   only exports, driven by the client registry.
2. **R909-2** — OpenAI-compatible clients receive the complete auth triple,
   including `OPENAI_MODEL`.
3. **R909-3** — `--global` does not report success when what it wrote cannot
   start the client.
4. **R909-4** — the startup contract is declared as data, apart from the
   settings that write it.
5. **R909-5** — the gap and its closure reproduce without a live client.
6. **R909-6** — the composition of all of the above breaks the build, not only
   each requirement in isolation.

## Root cause

The per-run (ephemeral) path in `data/seed/client-integrations.lino` already
wrote both missing pieces — `temp_home_json_set
"security.auth.selectedType={google_auth_type}"` for gemini and `env
"OPENAI_MODEL={model}"` for qwen. Only the `global` blocks had drifted: they
listed shell exports and no companion file, and qwen's list stopped at
`OPENAI_API_KEY` and `OPENAI_BASE_URL`.

The two clients then behave exactly as the issue describes:

- gemini-cli treats an auth type as *selected* only when a settings file says
  so, so `GEMINI_DEFAULT_AUTH_TYPE` alone leaves it unselected;
- qwen-code enters its OpenAI-compatible auth path only when the triple is
  complete, so a missing `OPENAI_MODEL` means no auth type is selected even
  though `--model formal-ai` is on the command line.

Nothing re-read the result, so `--global` could only report what it intended to
write, never what a client would find.

## Fix

Expressed as data rather than per-client branching in `src/`:

- a nested `companion` global-config node, parsed recursively by
  `parse_global_config` and written, backed up, and undone by the same machinery
  as the primary node — gemini's is `.gemini/settings.json` with
  `security.auth.selectedType={google_auth_type}`;
- `shell_env "OPENAI_MODEL={model}"` in qwen's `global` block;
- a `headless_require "<kind>=<target>"` contract declared *apart* from the
  settings that write it, plus `auth_refusal "<text>"` entries in each client's
  `verification` block.

`src/client_integrations/global_verify.rs` consumes that data at two levels:

- **always on** — `verify_written_config` re-reads every declared requirement
  from the files on disk (shell profile within the tool's managed block, or a
  dotted JSON/TOML key) and fails the run when one is missing. No client is
  needed, so it is safe offline and in CI.
- **opt in** — `formal-ai with --global --verify <tool>` runs
  `probe_headless_start`: it starts the configured client once
  non-interactively, sourcing the shell profile for `shell_env` targets, and
  fails when the output matches a seeded auth refusal. A client that is not
  installed is skipped, not failed.

All user-facing text lives in
`data/seed/multilingual-responses-client-config.lino` under the four
`client_global_config_*` intents, per the R379 doctrine.

## Verification

- `red-test.log` — the seven issue tests run against the **pre-fix** seed
  (`data/seed/client-integrations.lino` at `fc788a26^`, with the new parser and
  verifier in place). Five fail: no `.gemini/settings.json` is written, an
  existing settings file gains no `selectedType`, `OPENAI_MODEL` is absent, a
  refusing client is reported as started, and the whole-task sweep finds the
  registry declares zero requirements.
- `green-test.log` — the same tests against the fixed seed: 7 passed.
- `unit-test.log` — the per-requirement traceability test.
- `manual-verification.log` — `experiments/issue-909-headless-config-gaps.sh`
  run against the debug binary. It configures each tool into a throwaway `HOME`
  (never the operator's profile) and reports every headless requirement as
  written.

Manually confirmed on 2026-08-04 against the debug binary: `--global gemini`
into a throwaway `HOME` wrote `.profile` plus `.gemini/settings.json` and its
`.formal-ai.bak`; `--undo gemini` removed both; `--global qwen` wrote the full
`OPENAI_API_KEY` / `OPENAI_BASE_URL` / `OPENAI_MODEL` triple.

## Preserved evidence

- `raw-data/issue.json` — the issue definition captured before implementation.
- `raw-data/issue-comments.json` — the (empty) comment set, kept rather than
  omitted so the record shows the issue was decided on its opening report.
- `raw-data/pr-before.json` — the prepared draft PR metadata before
  finalization.
- `red-test.log`, `green-test.log`, `unit-test.log`, `manual-verification.log` —
  the runs described above.

## Prior art

No new dependency was required. The write/backup/undo machinery, the
`{placeholder}` renderer, and the seed-driven client registry already existed
for the per-run path; this change reuses all three and adds only the recursive
companion node, the requirement contract, and the readback/probe surface. The
upstream behaviour is not a bug we can file against gemini-cli or qwen-code —
both document a selected auth type as a precondition for non-interactive runs —
so the fix belongs here.
