---
bump: patch
---

### Fixed

- Recognize seed-defined source-first translation commands such as `<source> - translate to <target>` in both the native solver and browser worker.
- Route the reported formal-system proposition through one language-neutral meaning with English, Russian, Hindi, and Chinese renderings and round-trip coverage.
- Prevent long Unicode translation terms from panicking when cache filenames are truncated.
