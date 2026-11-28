# Native release contracts and observed cache errors

[Pipeline run 45039209189](https://github.com/link-assistant/formal-ai/actions/runs/45039209189)
checks published `b8383d52b10fb8be79f8fe7e42944b88343a07dc`.
The completed type-and-lint job has 249 passing workflow contracts and two
failures. Its retained original job log has the exact failures at lines
2560–2566 and the summary at 2624–2632.

The first assertion still requires the initial Buildx build to publish the
version and `latest` tags. The implementation instead publishes a candidate,
pulls its immutable digest, verifies runtime and anonymous manifest bytes,
then promotes both release tags and checks their digests. The repaired native
contract checks that complete ordering. Mirror tag-before-push checks remain.

The second assertion requires the crates.io URL inside the waiter file, after
the actual lookup moved into `scripts/registry.rs`. The repaired contract
follows the waiter's module declaration and exact crate/version call to that
module's endpoint. The template-safe skip remains covered.

Local verification executes the existing 21 image behavior cases and eight
Docker stream cases successfully. The source-view case initially refuses an
inherited `NODE_OPTIONS`; removing that variable for its focused rerun yields
one pass. Both attempts remain in the archive. Six touched JavaScript gates
and Rust formatting pass. No local Rust compilation or execution occurred;
the two repaired native contracts still need hosted execution.

The completed hosted release-script job executes all 25 compiled CLI methods
successfully (job 137260657923, lines 725–727). Three subsequently added version
base-isolation cases are still pending publication. The focused native build
also succeeds, and coding group 2 passes all 32 selected cases.

The build log now identifies a cache failure cause: job 137260658030 lines
1819–1833 records typed cache-write failures with HTTP 429, `RateLimited`,
`resource_exhausted` and `retry-after` delays. Signed endpoint values are
redacted by the maintained producer. Its counters report 30 attempted writes,
15 errors and 15 stored entries. This establishes rate limiting for those
observations; reliable later writes and reuse remain unqualified.

[The lossless evidence manifest](../../../experiments/issue-1580/evidence/checks/native-release-source-contract-repair/manifest.json)
retains the complete original failed job, completed native job streams, exact
source diff, original local failure and corrected environment replay. Every
gzip payload is round-trip checked by its original byte hash. Supplied edits
receive zero authorship credit. The wider issue remains unfinished.
