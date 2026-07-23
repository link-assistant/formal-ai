---
bump: patch
---

### Fixed
- Agentic step narration now explains each action in natural language instead of
  echoing the shell command that OpenCode already prints, and drops the robotic
  "so I can verify the next step before continuing" tail (#819). Local-path finds,
  web searches, and report prompts are worded distinctly across all supported
  languages (en, ru, hi, zh), with the empty-result case explained in beginner
  terms.
