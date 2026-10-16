### Fixed

- Consolidate shared Electron updater metadata checksums from the actual published bytes, rejecting conflicting ordinary artifacts and normalizing Windows binary-mode checksum records. Extension transfer now checks its own unique checksum independently while preserving exact signed package, source and producer identity checks.
