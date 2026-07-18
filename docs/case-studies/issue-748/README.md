# Issue 748: Codex model metadata

## Root cause

Codex does not populate its internal model registry from a custom provider's
OpenAI-compatible `/v1/models` response. Codex 0.144.x instead loads custom
model metadata from the top-level `model_catalog_json` configuration key. The
wrapper configured `model_providers.formalai` and selected `formal-ai`, but did
not provide that catalog, so every session warned that metadata was missing.

## Fix

The seed-defined Codex integration now writes a complete catalog sidecar and
passes its absolute path through `model_catalog_json`. The catalog advertises
the same model slug and context window as the server's `/models` endpoint, plus
Codex capabilities such as the shell and patch tool shapes.

One-shot runs write the catalog only inside the existing temporary Codex home.
Global setup writes `~/.codex/formal-ai-model-catalog.json`, backs up any
pre-existing file, and restores it on `--undo` together with `config.toml`.

## Evidence

The regression test was authored through a real external Agent CLI connected
to Formal AI. Its generated patch and raw client/server logs are in
[`red-test-agent-run/`](red-test-agent-run/). Before the implementation, the
focused test failed because the captured Codex arguments had no
`model_catalog_json`; after it, the fake Codex reads and validates the catalog.

The real-client experiment runs installed Codex against a live Formal AI server
and rejects the exact warning from the issue:

```sh
cargo build --bin formal-ai
experiments/codex_model_metadata_e2e/run.sh
```

Verified runs used both the issue's `codex-cli 0.144.1` and the locally current
`0.144.5`. Codex reported both `model: formal-ai` and `slug=formal-ai`, then
completed a streamed Responses API round trip without `Model metadata for
formal-ai not found`. The captured 0.144.1 client/server evidence is in
[`codex-0.144.1-e2e/`](codex-0.144.1-e2e/).
