# Issue 781: evidence-backed compatibility research

Issue [#781](https://github.com/link-assistant/formal-ai/issues/781) asks Formal AI to find an Amazon India charger for an Acer Aspire 3 A325-45, ingest the supplied ChatGPT and Google AI Mode shares, capture the real pages, and improve its research method instead of encoding one product answer.

## Outcome

The implementation fixes two reusable boundaries:

1. Agentic web research now captures up to three distinct search results in one bounded round, keeps every successful fetch associated with the URL that produced it, and emits a separately cited extract for each source. The previous planner selected and fetched only one URL.
2. `formal-ai shared-dialog convert` now accepts the normalized JSON contract emitted by `web-capture shared-dialog`. This connects the browser-backed provider adapter to Formal AI's existing `demo_memory` Links Notation export without copying provider scraping logic into Rust.

The supplied ChatGPT page was captured and converted successfully: both direct HTML parsing and normalized adapter JSON produce the same 35-event, 48,281-byte Links Notation file. The supplied Google page still exposes no transcript to a real Chromium capture; its upstream `google_ai_mode / no_transcript_in_captured_dom` diagnostic is preserved. All three Amazon pages attempted—including the strongest new candidate found by project-native search—return Amazon's automated-access page. The study therefore distinguishes indexed specifications from live listing evidence and does not invent polarity or current availability.

## Reproduction and proof

Before the fix:

```text
assertion failed: research must capture multiple independent sources
left: 1
right: 3
```

The regression in `tests/unit/issue_781.rs` replays search results for an official specification, independent connector evidence, and a candidate listing. It requires three `webfetch` calls and an answer that retains all three facts and URLs. The real Agent CLI E2E drives the Russian compatibility question over the release server and proves that the client executes all three live fetches.

Adapter parity was checked with:

```bash
formal-ai shared-dialog convert --input raw-data/chatgpt-share.html \
  --format chatgpt-share-html --output raw-data/chatgpt-direct.demo-memory.lino
formal-ai shared-dialog convert --input raw-data/chatgpt-web-capture.json \
  --format web-capture-json --output raw-data/chatgpt-adapter.demo-memory.lino
cmp raw-data/chatgpt-direct.demo-memory.lino raw-data/chatgpt-adapter.demo-memory.lino
```

See [requirements.md](requirements.md), [investigation.md](investigation.md), [recommendation.md](recommendation.md), and [raw-data/README.md](raw-data/README.md).
