# Original Rust macro context observation

The original Rust SARIF retains 631 informational unresolved macro locations.
An unresolved source AST node alone does not establish whether its compiled
expansion is present elsewhere. This diagnostic records source macro AST nodes
and their expansion context in the database produced by the security job.

Dispatch `security.yml` on the issue branch with `codeql=true` and
`macro-contexts=true`. Both inputs default to false. Only the read-only Rust job
runs this observer, after its original analysis; the unchanged original SARIF
gate still runs after a diagnostic failure. The Actions security suite remains
unchanged, and no code scanning results are uploaded by the read-only job.

The observer uses the actual `init` binary and `analyze` database outputs. It
inherits the producer's bundled Rust query pack without resolving new libraries.
The temporary query directory is removed even when compilation fails. Retained
artifacts include the query, database metadata, pack metadata and lockfiles when
present, CLI version, command arguments, original streams/statuses, BQRS, decoded
eight-column table and hashes. The hosted step has a four-minute deadline;
query execution requests two threads, 3 GiB RAM and a 512 MiB disk cache.
The first hosted run on `39196d1f` compiled and executed this query, retaining
122,534 rows. Its earlier 2 GiB setting left a 914 MiB Java heap and emitted a
stage-cache warning. The bounded 3 GiB adjustment needs its own hosted
verification; local adapter controls cannot prove the actual heap allocation.

Columns are the macro entity, relative source path, start/end line and column,
whether the call has an expansion, and whether it is inside another expansion.
Compare the actual rows to the original SARIF locations and compiled source
contexts before drawing a coverage conclusion. Neither a nonempty table nor
zero security findings proves complete macro semantics.

Local adapter controls use a fake CLI to exercise transport and failure
handling. They do not compile QL or establish Rust extraction coverage. Actual
hosted compilation and execution remain required. All supplied query, script
and conclusions receive zero independent Formal AI authorship credit.
