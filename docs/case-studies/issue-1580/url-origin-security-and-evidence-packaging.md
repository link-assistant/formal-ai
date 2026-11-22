# URL origin audit and evidence packaging

At published `79a024bffdcee2dd94149d929e81b4b65496ed2d`,
[Security run 45011615337](https://github.com/link-assistant/formal-ai/actions/runs/45011615337)
fails on RUSTSEC-2026-0592 in both audited `url 2.5.8` lockfile entries.
The complete failed log retains the primary diagnostic at lines 536–545 and
the quadratic experiment diagnostic at lines 548–558. The audit correctly
rejects the dependency; its strictness and warning policies are unchanged.

The RustSec advisory and servo/rust-url pull request #1186 describe recursive
origin computation for nested `blob:` schemes. The patched `2.5.9` package
parses one inner URL and treats a further blob scheme as opaque. Cargo updates
both audited locks to that release and its required `form_urlencoded 1.2.3`.
Complete package-object comparison permits only those two version/checksum
changes. The root manifest requires `2.5.9`; the maintained witness generator
refreshes only four manifest hashes. All 1,189 governed test identities,
221 supported cases and 968 unsupported cases remain unchanged.

Reproduce the pre-repair audit failure with `bash scripts/check-rust-dependencies.sh`
at the published source. The new `url_origins_keep_nested_blob_schemes_opaque`
native interface case preserves ordinary HTTPS and a single blob HTTPS origin,
requires opaque nested-blob and blob-WebSocket origins, and bounds its longest
input to 128 prefixes. It joins the existing unfiltered interface contract target.
No local Rust compilation or execution occurred; the new native behavior and
patched online audit still require hosted execution of the repaired commit.

The latest Rust template observation, pinned at
`260ae6f826d6bff0cbfe2280098c287a4c09b3f4`, contains no `url` package in either
lockfile. It does not reproduce this advisory, so no upstream issue was filed
for it. This observation does not qualify every template practice.

[Pipeline 45011615539](https://github.com/link-assistant/formal-ai/actions/runs/45011615539)
also rejects a 2,024-line evidence manifest at complete job-log line 349.
The subsequent local 142-entry inventory passes 85 gates, skips 53 explicitly
and fails four packaging checks. Thirteen archive names contain abbreviations;
the secret scan selects old names from committed HEAD after local renames.
An atomic local packaging commit makes its inventory match the renamed tree.
All four focused checks then pass. The manifest now has 477 lines, and complete
object comparison permits only those 13 names. All 221 original and compressed
payload hashes are unchanged; no external archive links remain unresolved.

The first Cargo helper incorrectly expected only `url` to change, the first
format command omitted the Rust edition, and a documentation helper named an
absent abbreviated script. Their failed observations remain retained. Corrected
commands use actual package metadata, Rust 2024 formatting and the maintained
`scripts/check-required-documentation.mjs`. Cargo inventory, native admission,
both governed witness controls, required documentation and formatting pass.

Evidence is under
`experiments/issue-1580/evidence/checks/url-origin-and-evidence-packaging-repair/`.
Its manifest binds each complete stream to original and compressed SHA256 hashes.
All supplied procedures and patches earn zero independent-authorship credit.
The wider issue remains unfinished and
[PR #1581](https://github.com/link-assistant/formal-ai/pull/1581) remains draft.
