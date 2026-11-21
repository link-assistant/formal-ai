from pathlib import Path
import json
root=Path('/tmp/gh-issue-solver-1794900398048');scratch=Path('/tmp/issue-1580-research');publication=scratch/'query-import-php-publication';publication.mkdir(exist_ok=False)
lead='''Latest fully collected published head `d48b09883ac91cd1d2319bea2afbaf0df2df8f86`
has 27 terminal workflows: 23 success, one skip and three failures. All 200
retained command streams verify by original/compressed hash, bytes and lines.
CI/CD Pipeline 44843761970 has nine compile-blocked native jobs: seven remaining
seed parent call sites lack split_pipe_list (first full-log error line 42913),
and the protocol parent retains unused ThinkingStep (line 42990). Commit
db4f64b41 restores/removes those private imports. Exact moved-text inverses did
not prove remaining parent name bindings; new hosted compilation is pending.
The separate registry helper job passes 48 Rust and eight actual CLI cases.

Layered CI 44843761720 fails the retained producer fingerprint at full-log line
8379. Commit 5792163c5 updates only the new included-string observer fingerprint
after verifying an exact inverse to the original producer. The maintained case
fails before and passes after repair. Its production admission is unchanged.

Security 44843748460 fails the additional included-string observation at full-log
line 4819: compilation takes 25.1 seconds, evaluation exceeds the unchanged
four-minute deadline, and no result or final manifest is produced. The local
query change narrows helper domains and enables opt-in evaluator tracing. Its
performance hypothesis and result equivalence still need hosted verification;
the exact expensive relation has not yet been established. No timeout increase
or weakening of the standard security gates is made.

Both original standard SARIF reports have zero findings and warning/error
notifications. The separate completed macro profile has 122,555 contexts across
2,229 files, a 2,607 MiB resolved heap and no runtime diagnostics; all 136
original producer payloads verify. The 631 unexpanded original positions still
lack per-child security/dataflow qualification. Clean standard reports do not
prove the failed new observation, or unresolved macro semantics.

The complete maintained template adapter passes all 93 cases, including 15 new
PHP original/candidate input-integrity cases and the seven admitted process
lifetime cases. Nine selected version/base-admission cases pass. Rust formatting,
projection currency, the query adapter, retained producer and strict security
tests pass locally. No local Rust compilation or execution occurs. The full
141-entry gate replay initially catches new receipt filename abbreviations;
payload-preserving names are repaired without changing the ratchet. Its final
outcome and the focused repaired check are recorded with the batch seal.

Pinned PHP review now adds 98 fully read source files and partial composer.lock
ranges to the prior 17. Twenty-two exact bounded pages are retained. Seven
bounded original PHP ownership cases leave a live worker at wrapper exit;
all seven current Formal AI counterpart controls pass. PHP version observation
accepts missing/invalid evidence; release-note truncation produces invalid
UTF-8 even at its default 120000-byte cap. Fifteen maintained tests retain
original failures, disposable candidates and positive controls. Fresh upstream
b61e334703d93a15e08bf1a278e0df0a9d6e98a8 has identical relevant source blobs.
No upstream repair or full template/runtime acceptance is claimed.

Twenty-one extraction groups leave 97 source-size warnings and zero ordinary
or embedded-data violations under the original limits. Published annotations
include those 97 warnings and token-scope uncertainty; experimental unsafe-libyaml
maintenance remains open. One Workflows error is the deliberate canonical
registry outage control followed by actual mirror recovery. Whole aggregate
literal review, all-warning resolution, dependency migration, complete PR review,
all eight full template comparisons/practices and original/live acceptance remain
unfinished. Rust literal review remains 352/373 identities and 49,166 lines.
Strict dated whole-file original acceptance remains 31/53 plus 22 separate scoped
assertion reviews; original budgets and live criteria remain open. Independent
Formal AI reasoning remains 0/20. Supplied work receives zero independent credit.
PR #1581 remains draft and unmerged; full issue completion remains false.
'''
checkpoint=root/'docs/case-studies/issue-1580/source-warning-extraction-checkpoint.md';source=checkpoint.read_text();begin=source.index('## Current repair checkpoint\n');end=source.index('## Earlier dated observations\n');checkpoint.write_text(source[:begin]+'## Current repair checkpoint\n\n'+lead+'\n'+source[end:])
plan=root/'experiments/issue-1580/work-plan.md';source=plan.read_text();begin=source.index('## Current verification checkpoint\n');end=source.index('Earlier chronology:',begin);plan.write_text(source[:begin]+'## Current verification checkpoint\n\n'+lead+'\n'+source[end:])
body=(scratch/'protocol-observation-publication/pull-request-body.md').read_text();begin=body.index('Verification:\n');end=body.index('- The Pages regression',begin)
verification='''Verification:

- The latest fully collected head `d48b09883ac91cd1d2319bea2afbaf0df2df8f86` has 27 terminal workflows: 23 success, one skip and three failures. All 200 original/compressed command streams verify. [Pipeline 44843761970](https://github.com/link-assistant/formal-ai/actions/runs/44843761970) exposes seven missing seed-parent imports and one stale protocol-parent import (full-log lines 42913 and 42990); commit db4f64b41 corrects those bindings. Nine native jobs are compile-blocked; the registry job separately passes 48 Rust and eight actual CLI cases. New hosted compilation remains pending. [Layered CI 44843761720](https://github.com/link-assistant/formal-ai/actions/runs/44843761720) exposes the stale producer fingerprint at line 8379; commit 5792163c5 retains the failing-before/passing-after maintained case and exact producer inverse.
- [Security 44843748460](https://github.com/link-assistant/formal-ai/actions/runs/44843748460) fails the new included-string observation at line 4819: the query compiles in 25.1 seconds but evaluation exceeds the unchanged four-minute deadline. No final table/manifest is produced. Narrow helper domains and opt-in evaluator tracing are a performance hypothesis pending hosted verification; the exact expensive relation and result equivalence remain unproven. Both original standard SARIF reports remain clean. The completed macro profile has 122,555 contexts across 2,229 files, a 2,607 MiB heap and no runtime diagnostics; 136 original payloads verify. All 631 unexpanded positions still require semantic qualification. The local query adapter, strict security and retained producer suites pass; payload/CFG presence alone will not establish coverage.
- The complete maintained template adapter passes 93 cases, including 15 new PHP original/candidate input-integrity cases and all seven lifetime cases. Nine selected version/base-admission cases, Rust formatting and projection currency pass. The complete 141-entry gate replay catches new receipt filename abbreviations; the payload-preserving repairs leave the ratchet unchanged. Final replay/focused outcomes are retained in the batch seal. No local Rust compilation or execution occurs.
- Twenty-one source extraction groups leave 97 size warnings and zero ordinary/embedded-data violations under unchanged limits. Published annotations retain those 97 source warnings and publish-token uncertainty; unsafe-libyaml maintenance remains open. The Workflows canonical-registry outage is a deliberate negative control with actual mirror recovery. All aggregate lines have not been literally reviewed; no all-warnings-resolved claim is made. [Source checkpoint](https://github.com/link-assistant/formal-ai/blob/issue-1580-8d61a32a61e1/docs/case-studies/issue-1580/source-warning-extraction-checkpoint.md).
- PHP source review adds 98 fully read files and partial composer.lock ranges to 17 previous files, with 22 pinned bounded pages retained. Seven bounded pinned PHP process-group cases leave live workers; all seven Formal AI counterparts pass. Version observation admits missing/invalid evidence, and release-note truncation emits invalid UTF-8 at its default cap. The 15 maintained tests preserve original failures, disposable candidates and positive controls. Fresh upstream b61e3347 has identical relevant source blobs. No upstream repair or full template/runtime qualification is claimed. The existing package-identity and checksum scoped practice evidence remains separate from complete template acceptance.
'''
body=body[:begin]+verification+body[end:]
body=body.replace('Focused reproduction (run each test separately):','Focused reproduction (run each test separately):').replace('```sh\n','```sh\nnode --test --test-concurrency=1 experiments/issue-1580/template-php-input-integrity.test.mjs\n',1)
assert body.count('Fixes #1580')==1
(publication/'pull-request-body.md').write_text(body)
(publication/'README.txt').write_text('Publication is not yet performed. Preserve draft/unmerged state; require terminal previous-head workflows, current main ancestry, complete actionable-failure batch, passing local checks and a clean tree. New native and CodeQL runtime qualification remains pending.\n')
print('Updated source checkpoint, requirement work plan and concrete draft PR body; publication remains pending.')
