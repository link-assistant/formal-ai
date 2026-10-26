# Fresh translation without duplicate work

[PR 1418's self-translation job](https://github.com/link-assistant/formal-ai/actions/runs/40261377337/job/121378763549)
reached its 630-second deadline after fresh verification of 151 modules.
All 154 committed projections had compiled and all 92 constant/call assertions
had passed. The original failure remains a failed complete-check attempt.

A replay using the recorded module timings estimated only about one percent
improvement from dependency-aware scheduling. That was insufficient, so no
dispatch-order change was introduced.

The [pinned upstream implementation](https://github.com/link-foundation/meta-language/blob/eb0574d5528babd3b4aadece81590d8e2230b99b/js/src/self-translation.js)
already computes translation and exported signatures together. Its public
`selfTranslate` and `selfTranslationSignatures` wrappers each invoke that full
pass. Formal AI called both wrappers for imported modules.

The verification adapter exposes that existing combined return. Its metadata
binds the upstream commit, exact source path and complete source SHA256. A
mismatch fails before the adapted source is imported. The adapter relocates
static relative imports to their original file URLs and adds an export for the
existing function; it changes no translation function or result. Upstream source
is loaded from the checked-out pin, not copied into this repository.

Every module still receives fresh translation. Imported signatures come from
that same pass. Worker limits, dependency waiting, cycle handling, carried-item
diagnostics, lowering, final projection bytes, inventory snapshots, complete
Rust compilation and all assertions remain required. Adapter metadata is also
included in the final input snapshot. No deadline or skip policy changes.

JavaScript tests use synthetic modules to check one-pass equivalence, preserved
relative imports and fail-closed source/pin validation. Actual upstream
translation and Rust execution remain CI-only. Replace the adapter with an
upstream public combined-result API when one is available and verified.

The [first hosted packet run](https://github.com/link-assistant/formal-ai/actions/runs/40267812823/job/121394111811)
exposed an omitted static import companion in the
independently captured release protocol: the translator imported the new adapter,
but its source-network capture recipe did not include that file. Include the
adapter in the immutable snapshot. The historical-checkout regression imports
the captured translator, then removes, changes and restores that companion to
verify both execution availability and the original source-byte checks.

Every captured source also has an explicit publication trigger operand. Update
that projection with `projectReleaseTriggerPaths` whenever the capture manifest
changes, then run `node scripts/generate-release-trigger-paths.mjs --check` and
the release-trigger regression alongside the historical-capture checks. Existing
anchored wildcard filters do not replace explicit ownership declarations. The
first coverage check correctly rejected the missing adapter operand; regenerating
it preserves every other workflow byte and leaves the assertion unchanged.
