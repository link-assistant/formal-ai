# Solutions, alternatives, and plans

## Implemented architecture

The selected design keeps one symbolic loop:

1. classify the request by capability, preferring research for open-world finds;
2. inspect all advertised tools, recursively including Responses namespaces;
3. plan one useful action and a localized narration naming its target;
4. serialize narration then the native protocol's tool-call representation;
5. preserve the exact raw result, normalize only recognized transport wrappers,
   and update attempted/successful evidence separately;
6. repeat for each independent source or one evidence-driven refinement;
7. terminate on completion, no refinement, no new source, or the round budget;
8. synthesize all successful source records with exact URLs.

This is shared across Chat, Responses, Anthropic, and Gemini. Protocol modules
translate envelopes; they do not decide what research to perform.

## Requirement-level alternatives

| Area | Alternatives considered | Selected plan and reason |
|---|---|---|
| Progress UX | Stream private chain-of-thought; emit a generic spinner; narrate only once before a batch. | Emit short user-facing action rationale before each single tool call. It is useful without exposing hidden reasoning and creates observable turn boundaries. |
| Multi-source work | Parallel batch; unbounded crawler; one fetch per whole answer. | One fetch per agent turn, bounded source set, optional evidence-driven deeper round. It is compatible with clients that stop on unusual multi-call finishes and makes progress visible. |
| Product reasoning | Charger-specific decision tree; model-generated score; generic constraints/Links. | Existing generic Links option network. It supports authentic/official/generic tiers and minimal composite options without domain vocabulary. |
| Tool selection | Always use hosted search; always use shell/grep; choose client-executable tool first. | Prefer explicit client-executable research functions/MCP children and retain hosted fallback for native hosted clients. |
| Responses MCP | Ignore namespaces; flatten permanently; flatten for planning and rehydrate output. | Flatten only the planning view and return exact `(namespace, name)` identity. This preserves Codex dispatch semantics. |
| Tool results | Store only normalized output; reason directly over raw wrappers; retain both views. | Durable messages/logs keep raw bytes; planner gets a conservative normalized view. This serves auditability and clean domain parsing. |
| Dialog logging | Always log full bodies; trace only summaries; explicit opt-in exact JSONL. | Opt-in JSONL at the common HTTP boundary. Default-off protects private prompts while enabling exact incident reconstruction. |
| Four-client test | Four unrelated scripts; one generic HTTP mock; native clients over deterministic MCP. | One reusable native-client harness with per-client protocol configuration and identical behavioral assertions. |
| Web evidence | Live internet in release CI; snippets only; deterministic CI plus preserved live captures. | Deterministic fixture proves orchestration; separate native captures prove real-world behavior without creating a flaky release gate. |

## Remaining solution plans

These are external or deliberate qualifications rather than unfinished defects in
the current PR.

### Current Amazon compatibility

1. Prefer an Acer India service-centre or seller listing that explicitly names
   A325-45 and all electrical/connector attributes.
2. Verify 12 V DC exactly, at least 2 A, plug dimensions, and center polarity
   against the laptop label/original adapter before purchase.
3. Use the one-piece Tonton candidate only after seller confirmation; use the
   two-item fallback only after confirming adapter direction and polarity.
4. Recheck stock and returnability in a human browser or authorized Amazon API.

### Google shared-dialog capture

1. Keep consuming `web-capture` normalized JSON so provider selector changes do
   not leak into Formal AI.
2. Preserve unsupported diagnostics today.
3. Add a real authenticated Google fixture when the upstream provider can
   legally and reliably expose the transcript; do not synthesize missing turns.

### Agent CLI early termination

1. Track upstream issue #249 and rerun the retained one-fetch and multi-fetch
   controls when a new Agent release is available.
2. Keep the retry bounded and visible; never accept a partial run as passing.
3. Remove the retry only after repeated full-session tests demonstrate the
   upstream transport is stable.

### Wall-clock feedback

1. Dialog timestamps now make request-to-response gaps measurable.
2. If field evidence still shows more than ten silent seconds inside a single
   server request, add protocol streaming for the narration item before the tool
   call completes and assert event timestamps in a dedicated latency harness.
3. Keep network/provider latency distinct from planner compute time in reports.

### Upstream namespace interoperability

1. Keep the Codex regression pinned to separate namespace/name output.
2. Monitor OpenAI Codex #14242 and linked namespace-tool issues.
3. If Codex introduces an explicit “flatten namespace tools” provider
   capability, prefer that negotiated contract over provider-side adaptation.
