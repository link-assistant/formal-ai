# Issue 781: evidence-backed compatibility research

Issue [#781](https://github.com/link-assistant/formal-ai/issues/781) asks Formal AI to find an Amazon India charger for an Acer Aspire 3 A325-45, ingest the supplied ChatGPT and Google AI Mode shares, capture the real pages, and improve its research method instead of encoding one product answer.

## Outcome

The implementation now covers four reusable boundaries:

1. Agentic web research keeps every successful fetch associated with its URL,
   reasons over up to three independent sources, and can deepen toward one
   evidence gap. The original planner selected and retained only one source.
2. Research is externally observable: every search or fetch is its own turn with
   a localized, useful explanation before the tool call, followed by a final
   cited synthesis. Chat Completions, Responses, Anthropic Messages, and Gemini
   preserve the same ordering in their native envelopes.
3. Client-executable research tools are discovered inside Responses namespaces
   and returned as the exact `(namespace, name)` pair required by Codex. Known
   client wrappers are normalized for planning while raw transport remains
   available for exact audit logs.
4. `formal-ai shared-dialog convert` accepts the normalized JSON contract emitted
   by `web-capture shared-dialog`, connecting browser-backed ChatGPT/Google
   adapters to the existing `demo_memory` Links Notation export.

The supplied ChatGPT page was captured and converted successfully: direct HTML
and normalized adapter JSON produce the same 35-event, 48,281-byte Links file.
Google still exposes no transcript to real Chromium capture, and all three Amazon
pages returned automated-access content. The study therefore distinguishes
indexed leads from live evidence and does not invent polarity or availability.

## Reproduction and proof

Before the fix:

```text
assertion failed: research must capture multiple independent sources
left: 1
right: 3
```

The regressions in `tests/unit/issue_781.rs` replay an official specification,
independent connector evidence, and a candidate listing. They require a narrated
search, three separately narrated fetch turns, and an answer retaining all facts
and URLs. `experiments/agent_cli_e2e/run_issue_781.sh` drives the exact Russian
prompt through Agent, OpenCode, Claude, and Codex. The final preserved run records
one search, three fetches, and a synthesis in every client.

Set `FORMAL_AI_DIALOG_LOG_DIR` to record exact request/response exchanges in one
JSONL file per dialog. This is deliberately off by default because prompts and
tool results can contain private data.

Adapter parity was checked with:

```bash
formal-ai shared-dialog convert --input raw-data/chatgpt-share.html \
  --format chatgpt-share-html --output raw-data/chatgpt-direct.demo-memory.lino
formal-ai shared-dialog convert --input raw-data/chatgpt-web-capture.json \
  --format web-capture-json --output raw-data/chatgpt-adapter.demo-memory.lino
cmp raw-data/chatgpt-direct.demo-memory.lino raw-data/chatgpt-adapter.demo-memory.lino
```

See [requirements.md](requirements.md), [investigation.md](investigation.md),
[recommendation.md](recommendation.md), and [raw-data/README.md](raw-data/README.md).
The complete PR #803 evidence bundle, including all four client/server/dialog
logs and upstream reports, is under `dev/log/issues/781/pulls/803/`.
