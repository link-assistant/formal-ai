---
bump: patch
---

### Fixed
- Added real loopback HTTP regression coverage for OpenAI Chat Completions `stream:true` responses so the SSE stream must use `chat.completion.chunk` frames with `choices[].delta.content`, and documented a verified OpenCode `hi` setup.
