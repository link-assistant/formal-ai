---
bump: patch
---

Resume recipe execution after an observed, correctly bound successful retry,
without discarding the failed attempt or accepting unrelated setup as proof.
Reject unchanged seeded artifacts as self-authored changes before publishing
them, and clarify that the authoring helper's no-commit mode does not stage files.
Reuse the shared source digest helper in the recurrence-cache generator.
