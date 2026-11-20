# CodeQL child evaluator memory

PR: https://github.com/link-assistant/formal-ai/pull/1581

The read-only Security producer at published head
`90b5159008da68c5c73534a5b5a4963b81deb04a` completes in
[run 44804016891](https://github.com/link-assistant/formal-ai/actions/runs/44804016891).
Both original SARIF reports contain zero findings and zero warning/error
notifications. The separately retained query stderr still warns that its
1,800 MB heap prevents caching stages during loading. The explicit launcher
`-J-Xmx3072m` attempt therefore did not clear the observed warning.

All 124 original producer payloads pass compressed and raw hash, byte and line
verification. The decoded query has 122,534 rows from 2,216 files: 42,427 expanded
nodes outside expansions, 79,476 inside expansions and 631 unexpanded contexts.
All 631 original notification positions match unexpanded contexts. This remains
a positional observation; it does not qualify the generated children or their
security/dataflow semantics.

## Cause and bounded configuration probe

The exact pinned 2.27.5 release launcher has SHA-256
`63a1703830e5c322eac4df1f911cbb6e2bf43bebadb94193a40f71a01a4a2b98`.
Its `-J` arguments affect the initial JVM. Targeted inspection of the release's
query-run and memory-resolver bytecode shows that query run starts a child query
evaluator with newly resolved memory arguments. That sizing can replace the
initial heap. The warning branch checks the child's actual maximum heap against
2 GiB. This is targeted inspection, not a complete bytecode audit.

Bounded HTTP range requests retrieve only the launcher and the CLI jar entry;
the 418 MB distribution is not installed. The jar identity is SHA-256
`2b81236667592128cd532b23b2849965f2376793ec7491a3b75352e417669b0f`.
Three actual memory-resolver configuration calculations run sequentially with a
128 MiB Java heap and 512 KiB stack; no query, compiler or native evaluator runs
locally. The generic resolver returns 1,964 MiB for the original 3 GiB budget,
1,800 MiB plus 1,209 MiB off-heap for the 4 GiB default, and 2,874 MiB with a
4 GiB total budget and zero off-heap allocation. The generic calculation is not
an observation of the hosted database's allocation. Original command streams,
identities and exit statuses are retained; large executable distribution files
remain scratch data.

The official [query-run reference](https://docs.github.com/en/code-security/reference/code-scanning/codeql/codeql-cli-manual/query-run)
and [memory-resolution reference](https://docs.github.com/en/code-security/reference/code-scanning/codeql/codeql-cli-manual/resolve-ram)
describe the JVM argument and total/off-heap options. The release-specific child
behavior above is grounded in the exact observed release and configuration
output rather than an assumption that a launcher's heap survives chaining.

## Candidate and controls

The observer first resolves memory against the producer's actual completed
Rust dataset and records the original resolver stdout/stderr and arguments. It
requires one resolved heap between 2 and 4 GiB. The resolver and actual query
receive the same 4 GiB total/zero off-heap policy. Existing two-thread, disk,
warning and hosted time limits are preserved. No launcher heap override remains.
Any runtime WARNING, WARN or ERROR line in query stderr makes the observation
fail visibly, retaining the original diagnostics and BQRS before decoding.

The original observer fails the changed memory-policy control (18 groups pass,
one fails). The repaired suite passes all 19 groups, including memory-command
failure, malformed/missing/undersized/excessive allocation, nonzero off-heap,
runtime warning, query/decode failure, incomplete input and pack cleanup. These
fixtures establish adapter behavior, not hosted allocation or semantic coverage.
The profile suite passes 18 groups; seven touched gates, actual actionlint syntax
validation and whitespace pass. Preserve the initial command-transport failure
and the mistaken TAP-summary expectation separately from the actual failing
control and corrected passing run.

Actual new-head resolution, successful hosted query execution without runtime
diagnostics and every original semantic requirement remain unverified until a
new producer runs. Current native jobs are still being collected; the next push
must wait for their complete failures. Strict original source identity remains
31/53 with 22 separate scoped assertion reviews. Full issue completion is false;
independently accepted reasoning remains 0/20 and supplied scripts earn zero.
