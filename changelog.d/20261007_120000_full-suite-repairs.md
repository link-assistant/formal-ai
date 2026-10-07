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
- Word problems are read as quantity relations: gain, loss, group and share words from the seed join the stated numbers ("Tom has 5 apples and buys 3 more" → 5 + 3 = 8), in Rust and the browser worker.
- The no-memorization gate also scans the nine code-task handlers' answer surfaces (issue #1177).

### Changed
- The coding-ladder gate accepts a lower floor only with a `coding_ladder_correction` note that says "corrected overcount" and names the previous floor.
- Derivation records under `data/cache/derivations/` are registered in the sources registry and git-ignored as run-time output.
- Unit conversion factors are checked against Wikidata: a cached P2370 table for the grounded units pins every SI factor of the length, mass and time units (#1176 R1).
- A stopped error-driven repair loop now says why it stopped (ladder spent, or no fetched source addressed the error) and attaches the attempt chain to the failure report, in both the Rust and JS roots (#1185 R4-R6).
- A no-memorization ratchet now counts the verbatim Hello World programs stored under data/ (14, the catalog templates) and fails if any more are added (#1165 R8).
- `formal-ai explain` now lists the rules a text-transform answer applied (register substitutions, grammar fixes, genre frames, summarization bounds, translation steps): the derivation record gains a schema-declared rule stage (#1174 R9).
- The SQL composer groups an aggregate by the column a seeded grouping cue names ("count users per country" → `GROUP BY country`), in Rust and the browser worker (#1177 R3).
- A manual dispatch of the CI/CD pipeline now runs the checks only unless a release mode is chosen, and every release, publish and tag job also requires the main branch (#1187 R3).
- The i18n catalog check registers the glass-opacity and material-skin settings keys added with the configurable skins.
- The JavaScript coding oracle reads the bootstrap gate of the cache policy correctly (it treated every language as unknown), and the Rust procedure cache reads its required fields from the policy seed, so both roots require `verified_output` (#1165 R10).
- The code debugger reports a second defect class: a loop bound that lets an index reach the length of a collection the code indexes ("range(len(xs) + 1)", "i <= arr.length"), with the bound that stops one position earlier (#1177 R1).
