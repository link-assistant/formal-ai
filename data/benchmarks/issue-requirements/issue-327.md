## Vision pillar / gap

Issue #244 feedback (PR #245, 2026-05-29): *"Make sure all Rust and JavaScript logic are in sync."*

Vision pillar 18 ("Rust-to-WebAssembly parity with JavaScript reserved for UI/glue") was advanced by E19 #282, but the **synthesis batch E28-E31** (#313-#316) added new reasoning capabilities to the Rust core that the JavaScript browser worker (`src/web/formal_ai_worker.js`) has **not** yet absorbed:

- E28 #313 — general link-native synthesis substrate (`record_candidates` composing decomposed sub-results).
- E29 #314 — deterministic arithmetic / word-problem and counting answers (GSM8K, MATH, BIG-bench).
- E30 #315 — program synthesis from spec + tests, verified in the bounded workspace.
- E31 #316 — generalized text manipulation over arbitrary user input.

So a prompt that the Rust core now solves by derivation may fall back to a weaker path in the browser. This breaks the "all Rust and JavaScript logic are in sync" requirement.

## Acceptance criteria

- [ ] The browser worker derives synthesis / numeric / program / text answers using the same algorithm shape as the Rust core (link-native composition, not a separate seeded path).
- [ ] Shared parity tests: a fixture set of prompts asserted to produce equivalent answers from both the Rust solver and the JS worker (extend the existing parity harness from E19 #282 if present).
- [ ] The anti-memorization rule and benchmark ratchet hold on the JS side too.
- [ ] WebAssembly remains the bridge for shared primitives; JavaScript stays UI/glue per pillar 18.

Tracked under the issue #244 vision roadmap (`ROADMAP.md` → Next Planning Batch, E34). Mirrors the E19 #282 precedent of a dedicated browser-worker parity epic.

