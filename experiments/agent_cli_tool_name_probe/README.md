# `agent` CLI stream-json tool-name probe (issue #715)

Isolates why every `tool_use` event in the captured `agent` stream under
`docs/case-studies/issue-715/agent-cli-learning/` reads `"name": "unknown"` with
an empty `input`, while `opencode` — driven by the same `formal-ai` server, over
the same wire, on the same task — names `write` and `bash`.

Reported upstream as [agent#281](https://github.com/link-assistant/agent/issues/281).

## Why a mock server

The two-harness E2E (`experiments/agent_cli_e2e/run_issue_715_learning.sh`) shows
the *disagreement* but cannot localise it: a real `formal-ai` server, a real
7-round session and two CLIs are all in the frame at once. This probe removes
everything except the one claim under test. `mock-openai-server.mjs` streams a
single `write` call with its name and arguments in `delta.tool_calls[].function`
— plain OpenAI streaming, no model, no network, no API key, no `formal-ai`.

## Run

```bash
node experiments/agent_cli_tool_name_probe/mock-openai-server.mjs 8935 &

WORK=$(mktemp -d); git -C "$WORK" init -q
cfg='{"provider":{"formalai":{"name":"Mock","npm":"@ai-sdk/openai-compatible","options":{"baseURL":"http://127.0.0.1:8935/api/openai/v1","apiKey":"local"},"models":{"formal-ai":{"name":"Mock"}}}},"model":"formalai/formal-ai"}'

cd "$WORK" && LINK_ASSISTANT_AGENT_CONFIG_CONTENT="$cfg" agent \
  --model formalai/formal-ai --permission-mode auto \
  --output-format stream-json --compact-json --disable-stdin \
  --prompt 'Write hi to a file called hi.txt'
```

## What it establishes (agent 0.25.0)

* The event names nothing: `{"type":"tool_use","name":"unknown","input":{}, …}`.
* **The tool ran anyway.** `cat $WORK/hi.txt` prints `hi`. The call is dispatched
  with the correct name and arguments; only the reported event loses them. That
  is what makes the case study's reading of its own artifact safe — the report
  existing is the evidence the write executed.
* **It is not `--compact-json`.** Plain `--output-format stream-json` drops the
  name too. The case study said otherwise before this probe existed, because
  `--compact-json` was the visible suspect: it is what the E2E passes.
* Omitting `usage` from the final chunk sends the CLI into a retry-with-backoff
  loop (`provider returned invalid usage data; retrying as provider API error`),
  which is why the mock sends it — and which independently corroborates that the
  `warn` events in the captured stream are
  [agent#249](https://github.com/link-assistant/agent/issues/249) rather than a
  fault of ours.

The model id must be one the CLI's provider registry resolves; an invented
`mock/mock` fails before any request is made, which is why the probe reuses the
`formalai` provider id and points its `baseURL` at the mock.
