# Testing Agentic CLI Tools

Use this guide when you need to prove that an external agentic CLI can drive a
local Formal AI server end to end. The goal is not just "the CLI exited 0"; the
goal is to verify provenance, tool routing, tool execution, and the final answer
from observable evidence.

This runbook documents the workflow that surfaced #624, #626, and #627, and it
feeds the CI e2e suite proposed in #625.

**Scope — what CI actually runs today.** Everything below is a *local* runbook.
The only agentic job in CI is `test-agent-cli-e2e`
(`.github/workflows/release.yml`), which drives our own Agent CLI against
`formal-ai serve` — with no recording proxy, no `with-formal-ai` wrapper, and
no other vendor CLI (codex, opencode, gemini, qwen, claude, grok, and aider
appear nowhere in the workflows). The multi-CLI × proxy matrix described in
*CI Shape* below is the target shape, tracked by
[#625](https://github.com/link-assistant/formal-ai/issues/625) and
[#671](https://github.com/link-assistant/formal-ai/issues/671); the OpenAI
Responses and Gemini `streamGenerateContent` paths mentioned later are covered
only by these manual procedures so far.

## Setup

Build the server and wrapper binaries:

```bash
cargo build --bin with-formal-ai --bin formal-ai
```

Install the client CLIs the same way users receive them. For CI, replace
`@latest` with pinned versions so a dependency release cannot silently change the
test result:

```bash
bun add --global @openai/codex@latest opencode-ai@latest @google/gemini-cli@latest @link-assistant/agent@latest t3@latest
```

Start Formal AI in agent mode on loopback:

```bash
formal-ai serve --agent-mode --host 127.0.0.1 --port 8080
```

Start the native logging proxy in another shell:

```bash
formal-ai proxy \
  --listen 127.0.0.1:8090 \
  --upstream http://127.0.0.1:8080 \
  --log proxy.jsonl
```

Use `--body` only when you need full request and response payloads for a local
debug run. The normal JSONL summary is enough for CI assertions.

## Fixture Workspace

Run each CLI from a temporary workspace with distinctive markers. A test passes
only when the final CLI answer contains the expected marker or file name.

```bash
WORKDIR="$(mktemp -d)"
cd "${WORKDIR}"

printf '%s\n' \
  'ALPHA_MARKER_11111' \
  'alpha second line' \
  'alpha third line' > alpha.txt

printf '%s\n' 'BETA_MARKER_22222' > beta.md
printf '%s\n' '{"gamma_marker":"GAMMA_33333","n":42}' > gamma.json
mkdir -p subdir
printf '%s\n' 'NESTED_MARKER_44444' > subdir/nested.log
```

Do not treat exit code 0 as success. Exit 0 means the CLI process ran; it does
not prove the server selected a tool, the CLI executed it, or the answer came
from the fixture workspace.

## Invocation Cheatsheet

For wrapper-based runs, point the CLI at the proxy, not directly at the server:

```bash
with-formal-ai --base-url http://127.0.0.1:<proxyport> opencode run "<prompt>" </dev/null
```

Use `http://127.0.0.1:8090` for the setup above. Redirect stdin from
`/dev/null` for every non-interactive run and strip ANSI escapes before scraping
output.

### OpenCode

OpenCode uses OpenAI Chat Completions through `@ai-sdk/openai-compatible`.

```bash
with-formal-ai --base-url http://127.0.0.1:8090 opencode run "list the files in the current directory" </dev/null
```

Expected protocol path: OpenAI `chat/completions`.

### OpenCode VS Code extension

The official `sst-dev.opencode` extension is a separate surface from the
OpenCode CLI/TUI and desktop app. Its extension command creates a VS Code
terminal and runs `opencode --port <port>`, so it consumes OpenCode's standard
provider config and the extension host's environment. Install the extension
and CLI, then start a fresh, isolated window through the wrapper:

```bash
code --install-extension sst-dev.opencode
with-formal-ai --base-url http://127.0.0.1:8090 opencode-vscode
```

Run **Open opencode** in that window and ask, using different wording from the
CLI row, to list the workspace files. The extension terminal must show model
`formalai/formal-ai`; the logging proxy must record a Chat Completions request
and at least one tool call/result round trip. Record `OPENCODE_CALLER=vscode`,
the extension version, request path, tool name, and result in the #671 matrix.
Use `opencode-code` as an equivalent wrapper alias. For persistent setup, use
`with-formal-ai --global opencode-vscode`; it manages the same
`~/.config/opencode/opencode.json` file as the CLI target and `--undo` restores
the backup.

The automated Linux harness installs the real Marketplace extension into an
isolated VS Code profile, invokes its command through a development-only test
driver, verifies the created terminal and caller environment, and exports the
exact provider config inherited by the extension host. It then replays that
config through OpenCode's non-interactive runner and asserts the proxy's
tool-call/result evidence. The replay avoids depending on TUI keystroke timing
while exercising the same OpenCode provider configuration:

```bash
experiments/opencode_vscode_e2e/run.sh
```

Expected protocol path: OpenAI `chat/completions`.

### Agent CLI

The wrapper injects an OpenCode-shaped provider JSON through
`LINK_ASSISTANT_AGENT_CONFIG_CONTENT`, so this is the normal one-shot command:

```bash
with-formal-ai --base-url http://127.0.0.1:8090 agent -p "read the file alpha.txt" </dev/null
```

When testing the Agent CLI without the wrapper, configure it with environment
only:

```bash
export FORMAL_AI_API_KEY="sk-local-demo"
export LINK_ASSISTANT_AGENT_CONFIG_CONTENT='{"provider":{"formalai":{"name":"formal-ai local server","npm":"@ai-sdk/openai-compatible","options":{"baseURL":"http://127.0.0.1:8090/api/openai/v1","apiKey":"{env:FORMAL_AI_API_KEY}"},"models":{"formal-ai":{"name":"formal-ai"}}}},"model":"formalai/formal-ai"}'
agent --model formalai/formal-ai --permission-mode auto -p "read the file alpha.txt" </dev/null
```

Expected protocol path: OpenAI `chat/completions`.

### Codex

Codex uses the OpenAI Responses wire API. Always include
`--skip-git-repo-check --sandbox read-only` for direct Codex invocations so
fixture directories do not have to be Git worktrees and the client uses the
same read-only tool sandbox as the wrapper.

```bash
with-formal-ai --base-url http://127.0.0.1:8090 codex "run ls" </dev/null
```

The wrapper adds `--skip-git-repo-check --sandbox read-only` automatically.

For direct Codex runs:

```bash
FORMAL_AI_API_KEY="sk-local-demo" codex exec \
  -c 'model_providers.formalai.name="formal-ai local server"' \
  -c 'model_providers.formalai.base_url="http://127.0.0.1:8090/api/openai/v1"' \
  -c 'model_providers.formalai.env_key="FORMAL_AI_API_KEY"' \
  -c 'model_providers.formalai.wire_api="responses"' \
  -c 'model_provider="formalai"' \
  -c 'model="formal-ai"' \
  --skip-git-repo-check --sandbox read-only \
  "run ls" </dev/null
```

Expected protocol path: OpenAI `responses`.

### T3 Code

T3 Code hosts Codex and Claude sessions in its local web interface. The wrapper
isolates Codex configuration from the user's normal home and launches the real
`t3` executable:

```bash
with-formal-ai --base-url http://127.0.0.1:8090 t3code
```

The aliases `t3code` and `t3` are equivalent. In the opened UI, create a Codex
thread and ask it to read `alpha.txt`; the final answer must contain
`ALPHA_MARKER_11111`. Provider settings should show provider `formalai`, model
`formal-ai`, base URL `http://127.0.0.1:8090/api/openai/v1`, and a non-empty API
key. For a Claude thread, launch `with-formal-ai --protocol anthropic ...` and
verify the configured Anthropic base URL is
`http://127.0.0.1:8090/api/anthropic`. Use `--non-interactive` when validating
startup without opening a browser; it maps to T3 Code's `--no-browser`.

Expected protocol paths: OpenAI `responses` for Codex sessions and Anthropic
`messages` for Claude sessions. Record both the T3 Code session output and the
proxy JSONL row for the multi-CLI matrix tracked by #671.

### Gemini CLI

Gemini CLI uses the native Gemini routes. Isolate it from cached OAuth state in
`~/.gemini`, select API-key auth, and trust the workspace for headless runs:

```bash
export GEMINI_CLI_HOME="$(mktemp -d)"
mkdir -p "${GEMINI_CLI_HOME}/.gemini"
printf '%s\n' '{"security":{"auth":{"selectedType":"gemini-api-key"}}}' \
  > "${GEMINI_CLI_HOME}/.gemini/settings.json"
export GEMINI_API_KEY="sk-local-demo"
export GEMINI_DEFAULT_AUTH_TYPE=gemini-api-key
export GEMINI_CLI_TRUST_WORKSPACE=true

with-formal-ai --base-url http://127.0.0.1:8090 gemini -p "list the files in the current directory" </dev/null
```

Expected protocol path: Gemini `streamGenerateContent` for streaming runs, or
Gemini `generateContent` for non-streaming clients.

## Proxy Assertions

Every passing run needs two evidence streams:

- CLI output: the final answer contains the expected marker or file name.
- Proxy log: the JSONL row proves which server answered and which tools were
  offered or returned.

Useful checks:

```bash
jq -c '. | {path, request_model, request_tools, response_model, response_tool_calls, status}' proxy.jsonl
```

For positive tool tests, assert:

- `status` is `200`.
- `request_model` or the protocol-specific model field is `formal-ai`.
- `response_model` is `formal-ai` when the protocol returns one.
- `request_tools` contains the expected tool names.
- `response_tool_calls` contains the expected tool call and arguments.
- The CLI answer contains `ALPHA_MARKER_11111`, `BETA_MARKER_22222`,
  `GAMMA_33333`, `NESTED_MARKER_44444`, or the expected file names.

Cover all protocol paths because bugs can hide in one surface while another
passes:

- OpenAI `chat/completions`: `opencode` and `agent`.
- OpenAI `responses`: `codex`.
- Gemini `streamGenerateContent`: `gemini`.

For negative tests, assert the opposite: no tool call is returned and the final
text is the intended non-tool answer.

## Phrasing Matrix

Use a phrasing matrix for each capability. Natural language routing is
wording-sensitive, so one direct prompt is not enough.

List-files prompts:

| Prompt | Expected evidence |
| --- | --- |
| `run ls` | Shell tool call, final answer lists `alpha.txt`, `beta.md`, `gamma.json`, `subdir`. |
| `list the files in the current directory` | Same as above. |
| `what files are in this folder?` | Same as above; this class is tracked by #624. |
| `show me the contents of this directory` | Same as above; this class is tracked by #624. |
| `print a directory listing of the current working directory` | Same as above. |

Read-file prompts:

| Prompt | Expected evidence |
| --- | --- |
| `read the file alpha.txt` | Read or shell tool call, final answer contains `ALPHA_MARKER_11111`. |
| `show me the contents of beta.md` | Final answer contains `BETA_MARKER_22222`, not a browser URL suggestion. |
| `cat gamma.json` | Shell tool call, final answer contains `GAMMA_33333`. |
| `open alpha.txt and tell me what's inside` | Final answer contains `ALPHA_MARKER_11111`. |
| `what is the value of gamma_marker in gamma.json?` | Final answer contains `GAMMA_33333`. |
| `print the first line of alpha.txt` | Final answer contains `ALPHA_MARKER_11111`. |
| `list the files then read the first one alphabetically` | Multi-step tool loop, final answer contains the marker from the selected file. |

Include negative cases in the same suite, for example a general knowledge
question or a request to explain what a directory listing is. Those should not
emit shell or read tool calls.

## Reading Failures

Classify each failure by observed evidence, not by guesswork:

- `tool_call` missing: the server routed to text instead of a tool.
- Tool call schema mismatch: the server returned a call, but the CLI could not
  parse or execute the arguments.
- Wrong protocol path: the CLI did not hit the expected endpoint.
- Missing provenance: the proxy log does not show `formal-ai`, so the answer may
  have come from client-side fallback behavior.
- Marker absent: the CLI ran, but it did not return fixture content.
- Filename treated as URL: local names like `alpha.txt` or `beta.md` were routed
  to browser or web-opening behavior instead of file access.

When filing an issue, include the prompt matrix, CLI versions, the proxy JSONL
summary, the final CLI output, and a minimal server-side `curl` repro that sends
the same `messages` or `input` plus `tools` directly to the server. The `curl`
repro lets maintainers separate a server routing bug from a CLI integration bug.

## CI Shape (proposed — not yet implemented)

The CI e2e suite should follow this sequence (tracked by #625 / #671; today CI
runs only step 1's `formal-ai` build, step 4, and a single-CLI variant of
step 6):

1. Build `formal-ai` and `with-formal-ai`.
2. Install pinned CLI versions.
3. Create the fixture workspace and marker files.
4. Start `formal-ai serve --agent-mode --host 127.0.0.1 --port 8080`.
5. Start `formal-ai proxy --listen 127.0.0.1:8090 --upstream http://127.0.0.1:8080 --log proxy.jsonl`.
6. Run the phrasing matrix for each CLI and protocol path.
7. Assert on both `proxy.jsonl` and stripped CLI output.
8. Fail on regressions in provenance, offered tools, returned tool calls, schema,
   or final marker content.

This is the bridge from manual investigation to the first-class e2e coverage
tracked by #625.

## References

- #624: natural-language file-listing requests that should route to shell.
- #625: built-in logging proxy and CI e2e suite.
- #626: Codex Responses schema and startup compatibility.
- #627: file-reading matrix and filename routing failures.
- [`docs/desktop/server-api.md`](../desktop/server-api.md): protocol and CLI
  configuration reference.
