# Issue 488 Online Research

Search performed on 2026-06-15.

## Sources

- OpenAI reasoning models guide: https://developers.openai.com/api/docs/guides/reasoning
  - Relevant finding: raw reasoning tokens are not exposed; products should use explicit reasoning summaries when they need user-visible reasoning status.
- Anthropic extended thinking documentation: https://platform.claude.com/docs/en/build-with-claude/extended-thinking
  - Relevant finding: visible thinking is commonly presented as summarized thinking blocks rather than raw internal tokens.
- Microsoft Semantic Kernel observability guide: https://learn.microsoft.com/en-us/semantic-kernel/concepts/enterprise-readiness/observability/
  - Relevant finding: agentic systems benefit from logs, metrics, and traces compatible with OpenTelemetry so operators can inspect behavior.
- Microsoft Semantic Kernel process framework: https://learn.microsoft.com/en-us/semantic-kernel/frameworks/process/process-framework
  - Relevant finding: process steps are reusable units that can trigger actions and transitions; formal-ai can reuse its existing structured `steps` events as the UI substrate.
- OpenTelemetry Generative AI blog: https://opentelemetry.io/blog/2024/otel-generative-ai/
  - Relevant finding: standardized telemetry attributes make GenAI behavior easier to monitor and compare; user-facing thinking should remain separate from raw debug payloads.

## Design Implications

- Show summaries, not hidden chain-of-thought or raw diagnostic identifiers.
- Reuse structured trace events already emitted by formal-ai instead of adding a second reasoning model.
- Keep raw diagnostics behind the diagnostics toggle for debugging and issue reports.
- Use the same visible surface for pending work and completed assistant messages so the experience is consistent.
