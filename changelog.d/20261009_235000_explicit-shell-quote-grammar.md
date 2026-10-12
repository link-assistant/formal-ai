### Fixed

- Explicit shell requests pair POSIX outer quotes and escapes before prose literal guards, preserving adjacent apostrophe fragments as opaque command data in JavaScript and Rust. Malformed command quotes still decline.
- Retain real-shell regression checks for apostrophe, dollar and backtick data, and the unchanged original evidence-collection retry.

All repository mutations used Formal AI or checked generators. Native execution remains a CI check; standalone formatting and four closest JavaScript checks pass. Broad source-feature synthesis remains open.
