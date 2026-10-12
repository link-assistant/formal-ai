# UI boundary migration (#951)

The inventory lists every top-level function in the current main.jsx snapshot,
with provisional DOM/core categorization and matching Rust function names.
Pure menu/sidebar glyph components are extracted into glyphs.jsx. An explicit
compiled-core transport contract names the first five migration operations and
refuses unsupported exports instead of claiming a Rust implementation exists.
The bridge is scaffolding only; current call sites have not been migrated.

The dedicated boundary workflow prevents new lowercase top-level function
declarations and growth beyond the recorded main.jsx baseline. Its `--strict`
mode enforces the requested <2500-line target, which the current source does
not yet meet. Existing arrow-function declarations are outside this provisional
lexical lint and require a real AST-based lint in the final migration.

Remaining work: expose issue_report.rs rendering, URL fitting, desktop-status
normalization, memory-bundle handling and evidence slugging from the compiled
Rust browser core; replace synchronous JS call sites with asynchronous worker
requests; delete issue-report.js after moving the byte-pinned contract to Rust.
Then extract JSX-only components, enforce the strict budget and perform the
manual report flow comparison. Moving logic between JS files would not satisfy
the Rust-only requirement. No build or test was run.
