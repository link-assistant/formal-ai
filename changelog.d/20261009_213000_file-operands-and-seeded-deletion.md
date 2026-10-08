### Fixed

- File-operation intent comes from the instruction rather than names in path operands. Copying files named for rename, word or line keeps the source and uses the same verified copy recipe for quoted, relative and absolute operands.
- Seeded bare deletion verbs resolve file operands in five languages, while requests to remove text within a file keep their existing edit guards.
- Both roots carry the same general instruction boundary and preserve scalar and list output separately.
