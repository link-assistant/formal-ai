# Bounded APT repair: terminal hosted baseline

The original cohort for source bc08222b5a5195dc9992c0459bde2b6369377b9f
finished with 32 successful workflows and one skipped workflow. This is the
baseline for the whole-producer rerun, not hosted qualification of the later
command-receipt, credential, sparse-index or first-mirror changes.

[CI/CD Pipeline 45162493977, attempt 1](https://github.com/link-assistant/formal-ai/actions/runs/45162493977/attempts/1)
passes its producer and all ten native groups: 441 selected tests pass.
All ten downloads report the original artifact's digest
f580f63d8571f695071e278d718257726f573240a55d23d9c17659ce8ad083b1.
Artifact 14812694215 belongs to run/attempt 45162493977/1 and checkout
a11b118a119341e7c8df168ea33dd1cf8a0741c1, the pull-request merge checkout
for this source. Its 1,088,007,636 bytes were not downloaded locally.
The hosted registry CLI suite still has 33 methods; the eleven newly drafted
credential/sparse methods and the sparse parser test have not executed.

[Layered CI 45162493244](https://github.com/link-assistant/formal-ai/actions/runs/45162493244)
also passes, while retaining two real first-attempt APT deadline warnings:

- Line 9012 follows regional HTTP download stalls. Lines 9013-9014 record
  pending-package recovery and the official HTTPS alternate; installation
  completes at lines 9056-9059.
- Line 25085 interrupts dpkg database reading. Lines 25087-25088 record
  recovery and the HTTPS alternate; line 25142 confirms the second attempt
  succeeds inside the existing limit.

The Pipeline's line 196 is a third actual warning: crates.io publication
authority is unknown. Token presence and successful image-registry preflight
cannot establish permission to publish a crate. This remains unfinished.
Of fourteen warning candidates across all 33 complete captured log streams,
nine are quoted shell commands and two are passing test names.
The complete capture has 116,543 lines. This classification does not claim
a literal read of every log line or that all original CI criteria are met.

The guarded whole-producer rerun was requested once at
2026-11-23T13:18:37.130171+00:00. Its second attempt remains in progress at this
checkpoint. The observer keeps exact remote source/run identity, the original
artifact and the single-request marker. Local descendant commits are allowed;
remote source mutation is refused. A terminal rerun result is still required.

## Evidence

The [sealed baseline receipt](../../../experiments/issue-1580/evidence/checks/apt-repair-terminal-baseline-20261123/manifest.json)
contains all 33 workflow logs and command results, the terminal cohort, selected
warning contexts and native summaries, artifact identity, dated feedback refresh,
the successful local secret scan and the observer guard refinement. Every archive
member was read back and hash checked: 638 members, 3,753,161 compressed bytes,
SHA-256 f61fc9b8cfa712871a1a4db9eb327ddbab48286f4d9c5f4d4e0c0812af339ce7.

Skipped production lanes earn no execution credit. The original nine workflow
families, remaining template practices, strict style, live publication/delivery,
identity and semantic requirements remain open. Keep
[PR #1581](https://github.com/link-assistant/formal-ai/pull/1581) draft and unmerged.
Supplied scripts, edits and conclusions earn zero independent authorship credit.
