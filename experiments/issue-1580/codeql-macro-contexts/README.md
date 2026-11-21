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
query execution requests two threads, 4 GiB RAM and a 512 MiB disk cache.
The hosted runs on `39196d1f`, `f4588ce4` and `1d7ba479` each retain 122,534
rows. Their 2, 3 and 4 GiB evaluator requests leave 914, 1698 and 1800 MiB
loading heaps respectively; all emit the query-loading stage-cache warning.
A later hosted probe established that the launcher JVM's `-J-Xmx3072m`
does not set the query child's loading heap. The observer now supplies the shared
`--ram=4096 --off-heap-ram=0` policy to both the completed-database resolver and
the actual query. On published `9fd9360d`, the resolver selected 2,607 MiB;
compilation and evaluation finished in 20.3 and 53.5 seconds without runtime
diagnostics. All 126 retained original producer payloads match their recorded
identities. The actual results are retained in
`../evidence/ci/codeql-producer-9fd9360d/`. This establishes the observed memory
and warning outcome, while per-child macro semantic qualification remains open.
Local adapter controls alone establish neither allocation nor coverage.

Columns are the macro entity, relative source path, start/end line and column,
whether the call has an expansion, and whether it is inside another expansion.
Compare the actual rows to the original SARIF locations and compiled source
contexts before drawing a coverage conclusion. Neither a nonempty table nor
zero security findings proves complete macro semantics.

Local adapter controls use a fake CLI to exercise transport and failure
handling. They do not compile QL or establish Rust extraction coverage. Actual
hosted compilation and execution remain required. All supplied query, script
and conclusions receive zero independent Formal AI authorship credit.


Pinned primary-library inspection at CodeQL CLI tag `codeql-cli/v2.27.5`
establishes an ownership limitation: `LocatableImpl.getLocationDefault` uses the
enclosing macro invocation's location when an expanded node has no own database
location. Nested generated macros can therefore all name their parent include
site. The 631 unexpanded outside-expansion calls belong to 68 files. Eight
production files account for 282 of those calls and have literal include parents;
their macro-label counts have expanded counterparts at those parent locations.
This count/location correspondence establishes neither per-child identity nor
security/dataflow semantics. Test fragments can also have compiled include
parents; being under fixtures does not make a source irrelevant. These primary
sources, exact pinned parent bytes and the limited crosswalk are retained under
`../evidence/checks/source-warning-secondary-receipts/`. No additional local QL
compilation, database extraction or query execution is claimed.

The subsequent published `f8b372ad` observation independently verifies 126
producer files, 122,542 contexts over 2,221 files, a 2,607 MiB resolved loading
heap and no query runtime diagnostics. Original SARIF findings and warning/error
notifications remain zero; the same 631 informational macro positions remain
unexpanded. See `../evidence/ci/codeql-producer-f8b372ad/`.

The subsequent published `3e705675` observation verifies 126 producer files,
122,553 contexts over 2,228 files, the resolved 2,607 MiB heap and no runtime
diagnostics. All 631 original informational positions still match unexpanded
nodes; this establishes no semantic coverage. The additional default-off
`included-string-payloads` profile retains complete literal text and explicit
missing-expansion/CFG state. Its local adapter controls pass; actual hosted
query compilation/evaluation and semantic qualification remain pending.

The opt-in Rust investigation requests retention of available intermediate
results from the standard analysis and both observers. Run 44848465533 compiled
the payload query but timed out during whole-database name binding before its
control-flow result was available. Cache eviction as the cause remains an
inference. Run 44855577716 rejected the attempted inverse of the action's
`--expect-discarded-cache`: positive and negative spellings are one CLI option,
and appending the inverse is a duplicate rather than an override. No standard
SARIF or payload observation was produced by that failed Rust analysis.

The investigation now appends only `--keep-full-cache --max-disk-cache=4096`.
The action still supplies its planning expectation; retaining available cache
does not prove that the producer computes every relation needed by an observer.
Both observers retain a 4096 MiB disk cache. The four-minute step bounds, memory
policy, nine output columns and security gate still apply. Hosted execution
must qualify this repair; fixture regressions do not execute CodeQL or establish
macro semantics.

Cache option contracts: [query run](https://docs.github.com/en/code-security/reference/code-scanning/codeql/codeql-cli-manual/query-run),
[producer action](https://github.com/github/codeql-action/blob/1bd86f2fdb4b86187fa4059f16f31a3202acc9ea/src/analyze.ts#L461),
[action argument order](https://github.com/github/codeql-action/blob/1bd86f2fdb4b86187fa4059f16f31a3202acc9ea/src/codeql.ts#L774).
