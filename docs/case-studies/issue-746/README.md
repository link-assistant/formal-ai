# Issue 746: protocol-native hosted tool routing

Formal AI now normalizes hosted and protocol-native advertisements before the shared capability router runs. A type-only OpenAI Responses tool no longer collapses to an empty tool set and the `tool:*` permission refusal; Anthropic server tools, Gemini `functionDeclarations`, Google Search, and URL Context all reach real protocol-native tool-call output.

Evidence:

- `red-regression.log` captures the original public-API cases failing with the reported refusal or browser-demo prose.
- `native-shape-red.log` captures the real Codex contract regression: routing succeeded but a hosted search was incorrectly serialized as `function_call`.
- `green-regression.log` and the current nine-test suite capture the cases passing with real, protocol-native tool calls, native SSE lifecycle events, and one-shot deferred discovery.
- `agent-regression.jsonl` records the real Agent CLI asking Formal AI to apply the regression patch.
- `agent-fix.jsonl` records the real Agent CLI asking Formal AI to apply the production patch.
- `cli-matrix.log` records the installed real-CLI verification sweep; unavailable CLIs are reported explicitly rather than simulated.

The installed `solve` wrapper rejected `formal-ai` as a model name before starting its worker. That failed attempt is visible in the issue conversation. The successful self-coding evidence therefore follows the repository runbook's direct path: a local `formal-ai serve --agent-mode` process driven by the external `@link-assistant/agent` CLI.

Reproduce the automated API regression with:

```sh
cargo test --test integration issue_746 -- --nocapture
```
