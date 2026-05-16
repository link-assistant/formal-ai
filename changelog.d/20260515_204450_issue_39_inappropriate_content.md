### Fixed

- Inappropriate or vulgar prompts (e.g. Russian mat) now receive a polite policy refusal (`intent: policy_inappropriate_content`) with a language-matched response instead of the generic "intent: unknown" fallback. Applies to Russian, Hindi, Chinese, and English content. Fixes issue #39.
