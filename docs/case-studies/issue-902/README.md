# Issue 902: preserve the Codex provider when callers add overrides

This case study records the reproduction, root cause, implementation, and
verification for the Codex 0.146 configuration-ordering regression reported in
issue 902.

## Reproduction and root cause

The wrapper previously emitted the Formal AI provider, model, and model catalog
as global `-c` arguments before Codex's `exec` subcommand. A caller such as Hive
Mind then appended ordinary reasoning overrides after `exec`:

```text
codex <Formal AI -c arguments> exec <wrapper arguments> \
  -c model_reasoning_effort=none -c model_reasoning_summary=auto hi
```

Codex CLI 0.146.0 discards the global override group when a subcommand override
group is also present. It consequently selects the default OpenAI provider and
sends the model request to `api.openai.com`, which returns 401.

The focused test in `tests/issue_902.rs` runs the exact caller argument shape
against a fake Codex executable. Before the implementation change it failed as
expected: `red-test.log` shows all seven wrapper-owned `-c` pairs mixed into the
argv before the two caller-owned pairs, with no isolated Codex config file.

## Fix

The Codex integration now writes the seven wrapper-owned settings into
`.codex/config.toml` inside the temporary home that the wrapper already creates.
Only caller-owned `-c` arguments remain on the command line. This avoids
depending on Codex's global/subcommand precedence while retaining the existing
temporary home, model catalog, dummy API key, and cleanup behavior.

No new renderer was required. The shared integration runner already supports
`temp_home_config_path` and `temp_home_toml_set` for other clients, so the
production change is confined to the canonical integration seed. There is no
JavaScript mirror of this Rust/data-driven CLI path.

The regression test verifies the exact resulting argv, every provider setting,
the generated model catalog, and that the user's original home is untouched.
The existing Codex model-metadata test now reads the generated TOML and catalog
from the isolated home as a real Codex process does.

## Automated verification

- `focused-test.log`: the issue-specific regression passed.
- `wrapper-integration-test.log`: all 17 wrapper integration tests passed.
- The complete `cargo test` run passed, including 2,407 unit tests (3 ignored),
  300 integration tests (1 ignored), the remaining integration targets, and
  doctests.
- `cargo fmt --check`, all-features Clippy, all-features example compilation,
  file-size policy, hardcoded-language policy, and total seed closure passed.

## Real Codex 0.146 verification

The installed `codex-cli 0.146.0` was launched through the fixed debug wrapper
against a local Formal AI server with the reported post-`exec` overrides:

```bash
formal-ai with --no-start-server --non-interactive \
  --base-url http://127.0.0.1:8793 codex \
  -c model_reasoning_effort=none \
  -c model_reasoning_summary=auto \
  'Create no files. Reply with: issue 902 provider route verified.'
```

`codex-e2e/wrapper.log` records Codex configuring `model=formal-ai`, provider
`formal-ai server`, and base URL
`http://127.0.0.1:8793/api/openai/v1`. It completed the turn on the first
attempt. `codex-e2e/server.log` independently records the POST to
`/api/openai/v1/responses`. Codex also made an unrelated best-effort request to
ChatGPT's featured-plugin endpoint, which returned 401; the model request did
not use `api.openai.com` and completed through Formal AI.

## Agent-authored artifact

Formal AI session `ses_038c815d3ffeEPJr9o0qLes425` authored the changelog
fragment and its general change plan. The successful raw stream and stderr are
preserved in `self-coding-changelog/`. Earlier attempts either did not edit the
workspace or produced an incorrect canned test; the available failed streams
are retained in `self-coding-red/` and `self-coding-changelog/` rather than
being omitted. The regression test and production fix were then written and
reviewed manually.

## Preserved evidence

- `raw-data/issue.json` and `raw-data/issue-comments.json`: issue definition and
  the empty comment set captured before implementation.
- `raw-data/pr-before.json`: the prepared draft PR metadata before finalization.
- `red-test.log`: focused regression failing against the old seed.
- `codex-e2e/`: real Codex and Formal AI logs.
- `self-coding-red/` and `self-coding-changelog/`: self-coding attempts and the
  successful agent-authored release note.
