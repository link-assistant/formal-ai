bump: patch

### Fixed

- A line a search *quoted* is no longer read as the search's own diagnosis. The
  failure lexicon is now asked only about a result's own words — the part before
  it starts naming the places it is quoting — so a `grep` that matched fifty
  lines no longer reports itself as the command that failed because one of the
  files it matched is an installer that prints *not found* when a program is
  missing. The cut is made wherever a line number stands as its own word before a
  colon, so it holds for a plain `<path>:<line>:<text>` listing and equally for a
  search that announces `Found 100 matches` and then quotes each hit under a
  path heading. A step that really did fail still says so: one citation is not a
  quotation, and a harness announcing its own refusal cites no place at all
  (#1066).
