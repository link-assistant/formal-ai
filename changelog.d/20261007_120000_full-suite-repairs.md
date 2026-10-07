---
bump: patch
---

### Fixed
- Page formalization and source trust read their seed records under the file's wrapper, so mime hints, sniff rules, code-fence language tags and trust weights apply again (issue #1163).
- `git log` history import passes its record format as `--format=`, not as a revision (issue #1180).
- A comma inside a date ("July 4, 1776") no longer splits the question into two sub-requests.
- A clarifying question about an unfilled slot (`*_unspecified`) and an arithmetic error naming what failed are kept rather than replaced by an open meta reply.
- The word-internal punctuation table no longer lists ASCII characters for spaced scripts, so names like `Node.js` stay whole inside an object phrase.
- The HTML page walker steps into container tags (`html`, `body`, `table`) instead of jumping past their close tag, skips comments and opaque tags, and no longer lets `</p` match `</pre>`, so captured pages yield their headings, rows and code blocks again (issues #1163, #1164).
- A folder named on a personal location ("on my desktop") stays the honest `list_dir` gap; only a file-type filter (`.lino files`) hands the request to the shell-command composer.
- "Make an add-on for ..." is read as an extension project: `add-on` is a seeded surface of the extension artifact, grounded in WordNet.
- The role registry is generated from exactly the files the meaning lexicon loads, so roles declared in `software-project-phrases.lino` are registered.
- Lean and Rocq renderings of a "No ..." statement keep the negation (`¬∃` in Lean, `~ (exists ...)` in Rocq).

### Changed
- The coding-ladder gate accepts a lower floor only with a `coding_ladder_correction` note that says "corrected overcount" and names the previous floor.
- Derivation records under `data/cache/derivations/` are registered in the sources registry and git-ignored as run-time output.
