---
bump: patch
---

### Fixed
- Count each Unicode scalar as one token across OpenAI, Anthropic, and Gemini usage metadata, sum all visible input message content, and return real response timestamps without fake cache or cost fields.
