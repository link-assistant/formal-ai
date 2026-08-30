bump: patch

### Fixed

- A line a search *quoted* is no longer read as the search's own diagnosis: a
  result whose first line carries the file and line number it was found at is
  other people's text, so the failure lexicon is not asked about it. A `grep`
  that matched fifty lines had been reporting itself as the command that failed,
  because one of the files it matched is an installer that prints *not found*
  when a program is missing (#1066).
