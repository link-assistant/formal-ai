# Release observations, conservative shard capacity and targeted cache evidence

PR: https://github.com/link-assistant/formal-ai/pull/1581

The published db19866f batch exposes two independent failures. Layered CI jobs
137238538583 and 137238538645 fail the conservative worker-queue contract; its
retained estimates need about 381.8 seconds in eight shards. Nine complete-suite
shards fit every projected queue below the existing 360-second execution budget,
with all 27 focused controls passing. This is a planning result, not a new hosted
wall-time claim. Every test, receipt and earlier timing floor remains retained.

Native job 137238752857 fails at raw log lines 5322–5335 because the newly shared
`first_heading_offset` has no caller in the strict workflow-contract test target.
A real byte-offset contract now exercises that function with Unicode, CRLF,
fenced examples, invalid headings and missing headings. It preserves `-D warnings`;
no local native compiler or test executable ran.

The release waiter now validates registry response identity through the existing
bounded helper. Unknown success bodies and transport errors cannot confirm crate
availability. The change detector requires a complete relevant changelog inventory
before emitting job decisions, including fallible directory entries and filenames.
The native harness now builds ten production CLIs and declares 25 test methods.
Its eight additional methods still require hosted execution. Earlier 17-method
success remains bound to db19866f; it is not credited to the new cases.

The shared template waiter defect is reported in
[upstream issue 264](https://github.com/link-foundation/rust-ai-driven-development-pipeline-template/issues/264).
That report follows complete current-source inspection. It clearly separates
source-derived outcomes from execution and does not attribute the local detector
defect to the template.

The actual opt-in cache trace also exposes an observation gap. Contract-build job
137238752864 retains only six tail records at raw lines 3196–3201. One compiler
argument record is 57,146 bytes; the bounded tail contains no backend failure.
The pinned sccache v0.16.0 producer increments `cache_write_errors` immediately
after logging `Error executing cache write:`. Its logger uses env_logger's
documented level/message-regex filter. The shared setup now selects that message
before server startup; the native job retains the bounded, URL-redacted report
even after another step fails. Defaults, cache policy and compiler outcomes are
preserved. The startup regression fails before the repair; 14 cache tests and
the native workflow binding pass afterward. This establishes local configuration
behavior and source compatibility; the real backend response remains unknown.

Five initial focused JavaScript files pass across original/corrected commands,
along with seven gates, changed-file rustfmt and Python syntax. The initial
NODE_OPTIONS wrapper failure and corrected successful command remain retained.
The cache draft additionally passes its focused workflow check and six gates.
Complete evidence, source hashes, primary-source logging excerpts and supplied
controllers are in the
[manifest](../../../experiments/issue-1580/evidence/checks/release-capacity-and-cache-draft/manifest.json).
No local Rust build, new-source hosted success, live publication, upstream
execution or independently authored Formal AI work is claimed. Current CI
collection, all broader original criteria and the draft PR remain unfinished.
