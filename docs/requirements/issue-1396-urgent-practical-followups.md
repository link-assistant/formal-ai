# Issue 1396: urgent practical requests and follow-ups

This requirement belongs to [PR 1437](https://github.com/link-assistant/formal-ai/pull/1437)
and [issue 1396](https://github.com/link-assistant/formal-ai/issues/1396).

Interpret an urgent follow-up together with its actual prior request. The reported toilet-urgency example must receive immediate relevant help rather than unrelated encyclopedic fragments or instructions for teaching the assistant a missing rule.

- Use a general intent/goal and trusted-knowledge strategy, with wording in seed data rather than prompt-specific branches in source code.
- Preserve urgency without inventing a diagnosis, a nearby facility, a completed physical action, or medical treatment. When evidence is unavailable, ask the smallest useful clarification rather than dumping implementation details.
- Distinguish a normal physical need from difficulty performing the action or other symptoms. Source any health guidance from an authoritative service.
- Keep English, Russian, Hindi, Chinese and Spanish behavior semantically aligned and test unseen paraphrases and context-bound follow-ups, including unrelated, quoted and negated counterexamples.
- Test minimal cached knowledge, cold discovery and unavailable-network paths through the real runtime with emulated services. Keep all Rust builds and execution in CI.
