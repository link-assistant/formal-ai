# Case study - Issue #478: "Unknown prompt: что такое нейросетевой инференс?"

- **Issue:** [#478](https://github.com/link-assistant/formal-ai/issues/478)
- **Reported version:** 0.193.0 (WASM worker), GitHub Pages, Firefox
- **Reported prompt:** `что такое нейросетевой инференс?`
- **Reported result:** `intent: unknown`
- **Pull request:** [#579](https://github.com/link-assistant/formal-ai/pull/579)
- **Raw data:** [`raw-data/`](./raw-data/) contains the issue JSON, issue comments, PR metadata, PR comments, review comments, and reviews captured for this fix.

## Timeline

- 2026-06-14: the issue was opened from the web app. Diagnostics showed `language:ru`, formalization `(@USER OP:define ?нейросетевой инференс)`, `wikipedia_lookup out: no_match`, and final fallback `unknown`.
- 2026-06-28: maintainer feedback asked for an ambitious, generalized meta-algorithmic fix for this case and related cases.
- 2026-06-28: local source regression reproduced the failure: the solver returned `intent: "unknown"` for `что такое нейросетевой инференс?`.

## Requirements

- The exact Russian prompt must not return the unknown fallback.
- The fix must be data-driven rather than a prompt-specific branch.
- Rust solver and browser worker behavior must stay aligned.
- Supported-language seed coverage must remain complete for English, Russian, Hindi, and Chinese.
- Regression tests must cover the issue before and after the fix.

## Root Cause

The reported trace already proved that the prompt was formalized correctly as a definition request. The concept parser also already supports Russian `что такое` prompts. The miss happened later: `data/seed/concepts.lino` had no concept record or aliases for neural-network inference, so `try_concept_lookup` returned `None`. The browser then tried external/public-knowledge fallbacks, Wikipedia had no matching article, and the turn ended in `unknown`.

This is a knowledge-base coverage gap, not a routing bug. The generalized fix is to add a grounded concept record with multilingual aliases and context links, so any equivalent prompt such as "neural network inference", "AI inference", "инференс нейросети", "न्यूरल नेटवर्क इन्फरेंस", or "神经网络推理" uses the same concept lookup path.

## Existing Components Used

- `data/seed/prompt-patterns.lino` already contains multilingual definition prompts.
- `src/concepts.rs` and `src/web/formal_ai_worker.js` already rank concept records by term, alias, and optional context.
- `data/seed/concept-contexts.lino` already defines `context_machine_learning` and `context_neural_network`, so the new concept can link to existing context semantics instead of duplicating them.
- `scripts/sync-seed.sh` copies canonical seed data into the web seed mirror before local Playwright runs and deployment.
- Grounding source: Google Cloud's AI inference overview, used as the stable source URL for the concept record.

## Fix

Added `concept_neural_inference` to `data/seed/concepts.lino` with:

- English, Russian, Hindi, and Chinese localized terms, aliases, summaries, source, and source kind.
- Context aliases for machine learning, deep learning, and neural networks.
- `context_links` to `context_machine_learning` and `context_neural_network`.
- A definition that distinguishes inference from training and names the practical engineering dimensions: latency, throughput, memory, accuracy, and runtime/hardware.

No JavaScript control-flow branch was needed because the browser worker loads the same seed data through `seed_loader.js`.

## Verification

- Added source regression `issue_478_multilingual_neural_inference_prompts_use_concept_lookup`.
- Added Playwright regression `tests/e2e/tests/issue-478.spec.js` for the WASM/web path with diagnostics enabled and English, Russian, Hindi, and Chinese prompts.
- Pre-fix source run failed with `left: "unknown"` and the Russian unknown fallback.
- Post-fix source run passed and hit `concept_lookup:hit:concept_neural_inference`.
