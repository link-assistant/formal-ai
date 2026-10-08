## Summary

`formal-ai with <tool> --global` writes environment variables to `~/.profile` and
nothing else. For `gemini` and `qwen` that is not enough to start the CLI headlessly:
both refuse to run non-interactively because the setting that selects an auth type is
never written. `--global` reports success, so the gap only surfaces later as an
apparently unrelated startup error.

```
gemini → Invalid auth method selected.
qwen   → No auth type is selected. Please configure an auth type
         (e.g. via settings or `--auth-type`) before running in non-interactive mode.
```

## Reproduction

Script: [`repro-headless-config-gaps.sh`](https://github.com/link-assistant/hive-mind/blob/main/experiments/issue-2130/repro-headless-config-gaps.sh) ·
stored output: [`formal-ai-0.317.0-headless-config-gaps.log`](https://github.com/link-assistant/hive-mind/blob/main/docs/case-studies/issue-2130/data/probes/formal-ai-0.317.0-headless-config-gaps.log)

For each tool it prints what `--global` writes, then runs the CLI twice against a live
`formal-ai serve --agent-mode`: **A** with exactly that environment, **B** with the same
environment plus the one missing piece.

```
########## gemini
  --- what `formal-ai with gemini --global` writes to ~/.profile
      export GEMINI_API_KEY="${FORMAL_AI_API_KEY:-formal-ai}"
      export GEMINI_DEFAULT_AUTH_TYPE="gemini-api-key"
      export GEMINI_CLI_TRUST_WORKSPACE="true"
      export GOOGLE_GEMINI_BASE_URL="http://127.0.0.1:8080/api/gemini"
  A: environment only (what --global writes)
      REFUSED TO START: Invalid auth method selected.
  B: same environment + settings.json security.auth.selectedType
      started: reached the model

########## qwen
  --- what `formal-ai with qwen --global` writes to ~/.profile
      export OPENAI_API_KEY="${FORMAL_AI_API_KEY:-formal-ai}"
      export OPENAI_BASE_URL="http://127.0.0.1:8080/api/openai/v1"
  A: environment only (what --global writes)
      REFUSED TO START: No auth type is selected. …
  B: same environment + OPENAI_MODEL
      started: reached the model
```

The script writes `--global` output into a throwaway `HOME`, so it will not modify the
operator's real profile.

### gemini

`GEMINI_DEFAULT_AUTH_TYPE` is written, but gemini-cli only treats an auth type as
*selected* when it is present in a settings file. The working configuration is

```json
{ "security": { "auth": { "selectedType": "gemini-api-key" } } }
```

pointed at by `GEMINI_CLI_SYSTEM_SETTINGS_PATH` (or placed at `~/.gemini/settings.json`).
With that one key added and nothing else changed, the same run starts.

### qwen

qwen-code selects the OpenAI-compatible auth path only when the OpenAI triple is
complete. `--global` writes `OPENAI_API_KEY` and `OPENAI_BASE_URL` but omits
`OPENAI_MODEL`, so no auth type is selected — even though `--model formal-ai` is passed
on the command line. Adding `OPENAI_MODEL=formal-ai` starts the run.

## Suggested fix

1. **`--global` should materialise files, not only exports, when the client needs
   them.** For gemini, write `~/.gemini/settings.json` with
   `security.auth.selectedType`. The client registry already knows each client's
   protocol; the config surface it requires belongs there too.
2. **Write `OPENAI_MODEL` for the OpenAI-compatible clients** (`qwen`, and any other
   client whose auth selection keys on the full triple).
3. **Verify after configuring.** `--global` currently reports `configured qwen at
   …/.profile` and exits 0 without checking that the tool can now start. A one-shot
   probe — start the CLI non-interactively and check it does not print an auth
   refusal — would turn a later mystery into an immediate, accurate error.

## Workaround

Write the missing settings yourself alongside the generated exports:

```bash
formal-ai with gemini --global
mkdir -p ~/.gemini
printf '{ "security": { "auth": { "selectedType": "gemini-api-key" } } }\n' > ~/.gemini/settings.json

formal-ai with qwen --global
echo 'export OPENAI_MODEL="formal-ai"' >> ~/.profile
```

Hive Mind takes this route programmatically rather than using `--global`: it builds a
per-run config root and injects both pieces itself
([`src/formal-ai-runtime.lib.mjs`](https://github.com/link-assistant/hive-mind/blob/main/src/formal-ai-runtime.lib.mjs)).

## Context

Found while diagnosing link-assistant/hive-mind#2130, where `--model formal-ai` had to
work across claude, codex, agent, gemini and qwen.

Environment: formal-ai 0.317.0, gemini-cli, qwen-code 0.21.2, Linux 6.8.0, Node 24.18.1.

