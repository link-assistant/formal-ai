# Public release download deadline

Verify Public Release Downloads failed on main at source
`0e550f6d56528d6d26bf575e08ab35c13e4af234`. The `all-public-bytes` step
started at 00:18:30 UTC. The full log records the timeout and exit 1 at lines
4039–4040, about 11 minutes later. The resolver, browser job and both Mac jobs
succeeded. [Run 45295717630](https://github.com/link-assistant/formal-ai/actions/runs/45295717630)

The retained progress receipt records 26 of 74 assets and 5,272,103,454 of
6,311,618,402 bytes verified. In selection order the next asset was the
369,486,861-byte Windows ARM64 portable executable. The original error gives
no asset name or per-transfer timings, so the receipt establishes where the
serial verifier stopped. It cannot distinguish slow transfer from a stalled
connection. [Byte evidence](https://github.com/link-assistant/formal-ai/actions/runs/45295717630/artifacts/14875559860)

The all-assets verifier now runs at most three downloads under the same shared
deadline. It hashes archives in the response stream and writes only the small
evidence files it retains. The Mac path still writes both packages for the
renderer checks. An asset failure identifies the selected name and stops
starting new downloads. In-flight transfers can finish within the deadline and
contribute verified progress. Final evidence still requires exact size,
SHA-256, checksum coverage and anonymous public transport for every asset.

Run the regression with `node --test --test-concurrency=1 rust/tests/web/verify-public-release-downloads.test.mjs`.
The small fixture checks that public requests overlap within the worker bound,
then checks stable receipt ordering, retained evidence, complete byte coverage
and the refusal path for a corrupt asset. Before the change the overlap and
asset-name assertions both failed. The production workflow runs following a
successful Desktop Release on main. The local fixture establishes the control
flow and checksum behavior. The [published replay record](https://github.com/link-assistant/formal-ai/blob/459f298f42968dbfdcb284ade60d85fdf2d68631/experiments/issue-1580/work-plan.md) reports all 74 public assets verified in 343 seconds. Hosted throughput still needs a run of the new verifier. [PR #1581](https://github.com/link-assistant/formal-ai/pull/1581)

## Browser engine archive budget

The browser engine already limited publication receipts to 134,217,728 bytes.
The first workflow upload and the npm publisher could accept an archive that
the later receipt could not record. Check the actual compressed tarball after
`npm pack` and before smoke tests or uploads. The npm publisher also checks its
input before querying the registry. An oversized archive reports its real byte
count and the shared receipt limit. The check also reports the five largest
files by uncompressed size when the npm pack inventory matches the archive name
and measured compressed size. An unavailable or mismatched inventory cannot
override the size failure. The v0.356.11 engine archive was 5,349,120 bytes.
[Hive Mind release preflight](https://github.com/link-assistant/hive-mind/blob/18258198fe16ee19e80cb5586a48ae41134fc81e/docs/CI-CD-BEST-PRACTICES.md#16-prove-you-can-publish-before-you-build)
[Release assets](https://github.com/link-assistant/formal-ai/releases/tag/v0.356.11)

Run `node --test --test-concurrency=1 rust/tests/web/browser-engine-release-integration.test.mjs`.
Before the change the publisher queried npm for an empty archive and the
workflow lacked the pre-upload guard. The regression uses a sparse temporary
file to test exact-limit and one-byte-over archives. The corrected publisher
fails without an npm call for empty or oversized files. The historical
selected-source protocol test also verifies the captured publisher continues
to accept ordinary fixture archives.

The first hosted archive-gate revision failed the retained production-job
digest at downloaded job log lines 2503–2514. The repaired control verifies the
exact diagnostic capture and required size step before comparing the rest of the
package job to the original digest. The pinned production profile still keeps
this package job off ordinary PR runs. [Original job failure](https://github.com/link-assistant/formal-ai/actions/runs/45346137806/job/138281104429)
