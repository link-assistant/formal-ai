### Fixed

- Read Rust release identity from the parsed Cargo package table. Reject missing
  or inherited versions, select workspace members from their actual tables, and
  update only the package version token while preserving unrelated manifest text.
