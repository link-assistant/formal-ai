title:	docs: add an agentic CLI tools testing guide (fixtures, logging proxy, phrasing matrices, marker assertions)
state:	CLOSED
author:	konard (Konstantin Diachenko)
labels:	
comments:	1
assignees:	
projects:	
milestone:	
issue-type:	
parent:	
sub-issues:	
sub-issues-completed:	
blocked-by:	
blocking:	
number:	628
--
## Summary

Add a documented guide — e.g. `docs/testing/agentic-cli-tools.md` — describing how to test external agentic CLIs (codex, opencode, gemini, and our own `agent`) against a local Formal AI server. This session produced a repeatable methodology (fixtures, a logging proxy for provenance, phrasing matrices, marker-based assertions) that found a series of real bugs (#620, #621, #622, #624, #626, #627). Capturing it as a guide preserves that know-how and gives contributors a runbook for reproducing/extending the checks. It also seeds the concrete e2e suite proposed in #625.

## Why

Agent-mode behavior is easy to get wrong in ways plain unit tests miss: a request returns HTTP 200 but the model never emits the tool-call, or emits it in a schema the client can't parse, or a filename is misread as a URL. The only reliable way to catch these is to drive the real CLIs end-to-end and assert on observed behavior — with a way to *prove* the answer came from our server. That workflow should be written down.

## Proposed guide outline

### 1. Setup
- Build wrapper + server: `cargo build --bin with-formal-ai --bin formal-ai`.
- Install/update the client CLIs the same way they ship (bun global): `bun add --global @openai/codex@latest opencode-ai@latest @google/gemini-cli@latest @link-assistant/agent@latest`. Note pinning versions for reproducible CI.
- Start the server in agent mode: `formal-ai serve --agent-mode --host 127.0.0.1 --port 8080`.

### 2. Per-CLI invocation cheatsheet (what "just works" vs. what needs flags)
- **opencode**: `with-formal-ai opencode run "<prompt>"` (clean).
- **agent** (ours): OpenAI-compatible; configure via env only with `LINK_ASSISTANT_AGENT_CONFIG_CONTENT` (inline OpenCode-shaped provider JSON) + `FORMAL_AI_API_KEY`; run `agent --model formalai/formal-ai --permission-mode auto -p "<prompt>"`. (Wrapper support tracked in #621.)
- **codex**: uses the Responses wire API; needs `--skip-git-repo-check` to run outside a git repo; known tool-arg mismatch (#626).
- **gemini**: needs `GEMINI_DEFAULT_AUTH_TYPE=gemini-api-key`, `GEMINI_CLI_TRUST_WORKSPACE=true`, and isolation from `~/.gemini` cached OAuth (#620); headless `-p` sends no tool declarations.
- Redirect stdin from `/dev/null` for non-interactive runs; strip ANSI when scraping output.

### 3. Provenance via a logging proxy
- Put a logging reverse proxy between the CLI and the server and point the CLI at it with `with-formal-ai --base-url http://127.0.0.1:<proxyport>`.
- Assert every model call carried `model=formal-ai` (so no client-side model answered), and log per request: path, tools offered, tool-calls returned (name + arguments), status.
- Cover all three protocol paths: OpenAI `chat/completions` (opencode, agent), OpenAI `responses` (codex), Gemini `streamGenerateContent` (gemini) — including streamed SSE bodies. (A native Rust proxy is proposed in #625; until then a throwaway script works but shouldn't live in the repo.)

### 4. Fixtures and marker-based assertions
- Create a fixture workspace with known files and **distinctive content markers**, e.g. `alpha.txt` → `ALPHA_MARKER_11111`, `beta.md` → `BETA_MARKER_22222`, `gamma.json` → `{"gamma_marker":"GAMMA_33333"}`, `subdir/nested.log` → `NESTED_MARKER_44444`.
- A test passes only if the expected marker (or real filename, for listing) appears in the CLI's final answer — not merely exit code 0. Exit 0 means "the CLI ran," not "the task succeeded."

### 5. Phrasing matrices (test many wordings, expect natural conversation)
For each capability, run direct + natural-language + multi-step variations, because routing is wording-sensitive:
- **List files**: "run ls", "list the files in the current directory" (work today) vs. "what files are in this folder?", "show me the contents of this directory" (fail today — #624).
- **Read a file**: "read the file X", "cat X", "show me the contents of X", "what does X say?", "open X and tell me what's inside", "print the first line of X", "what is the value of Y in X?", plus multi-step "list the files then read the first one" (all failing today — #627).
- Include **negative cases** (prompts that should NOT trigger a tool call) to catch over-triggering.

### 6. Reading the results
- Classify each run: content/marker present? fell back to "could not determine" / "requires Agent mode" / URL-misparse? tool-call emitted in the proxy log?
- Compile failures into an issue with the full matrix and a minimal `curl` server-side repro (drop the CLI, send the same `messages` + `tools` array directly) so maintainers can reproduce without the CLI.

### 7. Wire it into CI
- Turn the matrices into an e2e job that runs on every PR (see #625): start server + proxy, install pinned CLIs, run matrices, assert on proxy-log + CLI output. Fail the build on regressions in intent-routing or tool-calling.

## Deliverable
A `docs/testing/agentic-cli-tools.md` (or under `docs/ci-cd/`) capturing the above, with copy-pasteable commands and the fixture/marker convention, cross-referenced from CONTRIBUTING. Should link the living examples: #624, #626, #627 (found with this method) and #625 (the CI suite it feeds).

