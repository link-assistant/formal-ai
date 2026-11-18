---
bump: patch
---

### Fixed
- An output literal is printed once (#1156): "print `Hello, World!` and the program outputs it" is one line, not two. A count ("twice", "3 times") or "again" still repeats it, and "outputs" now counts as an output verb.
