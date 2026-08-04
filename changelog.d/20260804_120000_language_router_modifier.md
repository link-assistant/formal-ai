---
bump: patch
---

### Fixed
- The implementation-language router now validates what fills the `in <language>` position instead of taking whatever word follows `in` (issue #906):
  - a closed-class word is no longer read as a language name, so "Create a file named `hello.txt` in the current directory…" is no longer routed as a request in language `the` (an unknown *name* such as `elvish` is still read, so the engine can report what it was asked for);
  - a request that names no language gets its own answer — "I will not guess an implementation language… name the language" — with its own intent, event and response link, so the internal `missing` sentinel never reaches the reply;
  - the modifier modifies the request instead of replacing it: "Fix the failing CI job in Rust." is answered about the CI job rather than with an encyclopedia definition of Rust;
  - a request whose artefact is a *file* is no longer normalised into a `write_program` request, so the path and the content survive into the change plan instead of being replaced by `console.log('Hello, world!')`.
