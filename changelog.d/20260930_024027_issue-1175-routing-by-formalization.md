---
bump: patch
---

### Fixed

- Prompts whose first word merely happens to be a shell-token word ("Find the
  bug: …", "Make a 3-day itinerary …") are no longer answered as terminal
  commands: the leading-token detection path now requires the rest of the
  prompt to parse as command arguments (no question mark, no colon-space
  outside quotes, no sentence-final punctuation on bare tokens, no bare
  function words; quoted spans are masked first), while real commands such as
  `find . -name '*.log' -size +10M` and `git commit -m 'fix the parser bug'`
  stay detected. The software-project detector likewise no longer picks its
  artifact kind from an artifact word anywhere in the prompt — only from the
  head noun of the authoring verb's object phrase — so "Write a regular
  expression that matches a US ZIP code with an optional 4-digit extension."
  no longer becomes an implementation plan for an extension, while
  artifact-headed requests in English, Russian, Hindi and Chinese keep their
  frame (issue #1175).
