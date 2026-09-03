---
bump: patch
---

### Fixed

- Discover structural member insertions from the target file's own bytes
  instead of one hardcoded request shape. The route previously required the
  word "array", exactly one quoted value, and a `snake_case` identifier, then
  inserted at the first `[` after it — so a `matches!` alternation produced no
  tool calls at all, and `const NAMES: &[&str] = &["a"];` was corrupted into
  `&[&str, "b"]` by writing into the type instead of the value. The anchor is
  now the delimiter pair that already holds the quoted members (or, when none
  are present yet, the literal-bearing pair inside the named declaration), and
  the separator and spacing are copied from the members already there (#1069).
