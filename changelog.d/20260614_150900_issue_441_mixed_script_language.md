---
bump: patch
---

### Fixed

- Issue #441: Russian definition prompts that start in Cyrillic and ask about
  Latin technical terms, such as `Что такое vulkan layer`, now keep
  `language:ru` instead of being misclassified as English and falling through to
  the unknown-intent answer. The browser worker mirror now follows the same
  detection rule.
