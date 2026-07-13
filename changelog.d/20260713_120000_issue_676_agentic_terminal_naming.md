---
bump: minor
---

### Added
- The assistant now honours being named in conversation. After "Now your name is
  Ineffa" (or "I'll call you Ada", "you are called …") it acknowledges the name and
  recalls it when later asked "what is your name", using dialog-local memory with no
  server state — mirrored in the browser worker (issue #676).
- Reasoning traces now open with a human, first-person narrative of what the
  assistant understood and decided ("You asked how I'm doing, so I told you and
  offered to help.") instead of an identical per-intent category template. The
  concrete steps remain beneath it as an expandable, recursive "robotic detail"
  layer. Applied to the API/CLI reasoning field (what agentic clients such as
  OpenCode render) and mirrored in the web thinking preview across en/ru/zh/hi
  (issue #676).

### Fixed
- Agentic planner now runs any seed shell token (`pwd`, `git`, `cargo`, …) named in a
  prompt, not just `ls`. `execute pwd`, `run git status`, and their many phrasings map
  to the real command (issue #676).
- Natural-language file-listing requests such as "give me a list of files in current
  folder" resolve to `ls` across many more phrasings (issue #676).
- Self-healing now triggers on natural self-directed repair requests such as "Can you
  fix it yourself?", "debug yourself", or "heal yourself", while ordinary "fix this
  file" requests still stay out of the repair loop (issue #676).
- "How are you?" small talk now gets its own warm wellbeing reply instead of the
  generic greeting. A dedicated `wellbeing` intent is matched before `greeting`
  (first-match-wins), so "how are you", "как дела", "आप कैसे हैं", and "你好吗" reply
  with an actual answer across en/ru/hi/zh — mirrored in the browser worker (issue
  #676).
