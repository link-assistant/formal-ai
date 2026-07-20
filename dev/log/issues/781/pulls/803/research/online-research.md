# Online research and related components

Research was refreshed on 2026-07-20. Primary vendor/protocol documentation is
preferred for technical claims; marketplace results are treated as leads, not
authoritative compatibility evidence.

## Protocol facts that shaped the fix

- OpenAI's function-calling guide models Responses output as an ordered array
  that can contain multiple kinds of items; applications must iterate the array,
  execute each `function_call`, and append a matching `function_call_output`.
  This supports Formal AI's decision to preserve narration as a message item
  immediately before the call rather than collapsing output to one string:
  https://developers.openai.com/api/docs/guides/function-calling
- The same OpenAI guide shows Chat Completions tool calls on the assistant
  message and describes streaming tool-call deltas, including using streaming
  to surface progress. Chat content and tool calls therefore need not be treated
  as mutually exclusive by an OpenAI-compatible provider:
  https://developers.openai.com/api/docs/guides/function-calling
- Anthropic's official tool documentation gives the exact desired native shape:
  an assistant `content` array with a `text` block followed by `tool_use`, and
  says this helps users understand the action. It also warns that forced
  `tool_choice: any/tool` suppresses natural-language preambles, so the harness
  must not force tool choice when narration is required:
  https://platform.claude.com/docs/en/agents-and-tools/tool-use/define-tools
- Google's official Gemini documentation says function calling can repeat over
  multiple turns and supports sequential/compositional calls. It requires the
  model response and matching function result to be returned for synthesis, and
  warns callers to iterate all response parts because calls need not be last:
  https://ai.google.dev/gemini-api/docs/generate-content/function-calling
- The MCP 2025-11-25 schema defines `readOnlyHint`, `destructiveHint`,
  `idempotentHint`, and `openWorldHint`, with pessimistic defaults. It explicitly
  calls all annotations untrusted hints. This explains Codex's approval behavior
  while preventing us from treating annotations as a security boundary:
  https://modelcontextprotocol.io/specification/2025-11-25/schema

## Client integration references

- OpenCode supports local and remote MCP servers and makes their tools available
  alongside built-ins. Current permission keys distinguish `websearch`,
  `webfetch`, and wildcard MCP tool names. These docs informed the real OpenCode
  configuration and the fix for grep taking precedence:
  https://opencode.ai/docs/tools/
  https://opencode.ai/docs/mcp-servers/
- Claude Code's CLI supports print mode, verbose turn-by-turn logging,
  `--max-turns`, and MCP configuration. The harness uses non-interactive output
  and an explicit local MCP file:
  https://docs.anthropic.com/en/docs/claude-code/cli-usage
- The MCP tool result specification permits structured or unstructured content
  items. Codex's timing/`Output` wrapper is a client presentation envelope, not
  the server's domain payload, which is why Formal AI retains raw transport and
  separately unwraps only known envelopes:
  https://modelcontextprotocol.io/specification/2025-06-18/server/tools

## Product facts and evidence quality

- Acer's official A325-45 manual is available from Acer's global download
  service, but its user-facing power section does not specify connector size or
  polarity. It is therefore not enough to certify a replacement adapter:
  https://global-download.acer.com/GDFiles/Document/User%20Manual/User%20Manual_Acer_1.0_A_A.pdf?BC=ACER&LC=en&OS=ALL&SC=AAP_6&Step3=ASPIRE+3+A325-45&acerid=638629189859980254
- Acer India's official store confirms A325-45 configurations and that an adapter
  ships in the package, but the indexed store page likewise does not establish
  the replacement plug or polarity:
  https://store.acer.com/en-in/laptops/aspire/aspire-3
- Search indexing exposes a model-specific marketplace listing naming 12 V, 2 A,
  and 3.5 × 1.35 mm, but that is seller evidence rather than an Acer electrical
  specification:
  https://shopee.co.id/Adaptor-Charger-Acer-Aspire-3-A325-45-RMN-N15JPJ-12V-2A-DC-3.5X1.35mm-i.501202229.55559857895
- Amazon's candidate and fallback pages were browser-captured locally and all
  returned automated-access content. No online snippet is used to claim current
  inventory or polarity. See the case-study raw HTML and `recommendation.md`.

## Existing components reused

- `@link-assistant/web-capture`: browser-backed shared-dialog/page capture and a
  normalized provider boundary. Formal AI consumes its contract instead of
  copying selectors/anti-bot behavior.
- `@link-assistant/web-search`: Google/Bing/DuckDuckGo adapters and reciprocal-rank
  fusion already used by the project.
- Formal AI `world_model::Context`, Links Notation, `OptionNetwork`, and
  `option_evidence`: existing associative representation, generic constraint
  solving, and evidence extraction.
- `serde_json`: protocol/tool argument parsing, conservative wrapper
  normalization, and exact JSONL dialog records.
- The reusable native-client harness and its small stdio MCP fixture avoid a new
  test framework while exercising the actual released CLIs.

## Known related defects

- Agent CLI issue #194 documents a step ending with `reason: unknown` after a
  tool action, followed by process completion instead of another model turn.
  Local parallel and one-fetch controls reproduce the same boundary; the issue
  and its 2,651-line authenticated Gist capture are retained in
  `upstream/agent-issue-194-*`:
  https://github.com/link-assistant/agent/issues/194
- OpenCode issue #20465 documents the same visible failure class—tool-enabled
  work followed by blank assistant output—from an AI SDK finish-reason mapping
  regression. Its reported fix was to continue the loop for the SDK's `other`
  result and normalize all current request shapes. That is corroborating client
  evidence, not the cause assigned to Formal AI's OpenCode run, which passes on
  the retained current client:
  https://github.com/anomalyco/opencode/issues/20465
- OpenAI Codex issue #14242 discusses tool-only MCP discovery and namespace
  routing. Our Codex 0.144.6 custom-provider reproduction, workaround, and
  suggested code behavior were posted here:
  https://github.com/openai/codex/issues/14242#issuecomment-5018114084
- Formal AI issue #800 is the observability counterpart: ordinary error output
  did not expose the full dialog. PR #803 addresses it generally with opt-in
  exact per-dialog records rather than a charger-specific debug path.
- `link-foundation/gh-upload-log` is useful for publishing an existing file, but
  it does not create the missing server-side correlated record. Formal AI now
  creates that record; upload can remain a separate explicit user action.

## Conclusions from the online comparison

Existing protocols already support ordered mixtures of user-visible text and
tool actions. The defect was Formal AI's translation/planning policy, not a need
for a new wire protocol. MCP namespaces and annotations do add interoperability
constraints, so recursive tool discovery, exact executable identity, and
separate raw/normalized result views are the reusable solution.
