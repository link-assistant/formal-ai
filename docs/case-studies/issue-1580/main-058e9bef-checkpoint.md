# Main 058e9bef checkpoint

This draft checkpoint belongs to [PR #1581](https://github.com/link-assistant/formal-ai/pull/1581)
for [issue #1580](https://github.com/link-assistant/formal-ai/issues/1580).
Local merge `8b2e929662530251ed301f8892da218b38837d40` incorporates default
head `058e9bef1214f2f1bc25971243cb787703ed64f1`, including the retention and
multilingual summarization changes in PRs #1600 and #1602.

The sole merge conflict is a generated translation-blocker table. The maintained
renderer regenerated it from the 161 retained module measurements; verification
passes without changing those measurements or the ledger. The complete governed
witness comparison visits 7,727 object pairs and finds zero changes, retaining
221 supported and 964 unsupported cases. Package mirroring verifies all 482
demands with no byte changes. Rust formatting passes; no native code executes.

The combined source passes all fourteen focused JavaScript files: **154 tests,
zero failures or skips**. The full gate inventory reports **86 passes, 52
explicit native/context exclusions, zero failures**, in 842.044
seconds with a 256 MiB Node heap cap. The aggregate and all 86 per-gate archives
are retained with hashes under
`experiments/issue-1580/evidence/checks/main-merge-058e9bef/`. Native/context
exclusions do not establish native behavior. The native startup-log and parser
repairs described in the preceding checkpoint remain candidates awaiting a
permitted runtime verification.

## Fresh CI evidence and remaining investigation

At the recorded observation time, main has seventeen push workflows: thirteen
successes, two failures and two still running. Complete failed-workflow logs and
their exact head/timestamps/hashes are retained under
`experiments/issue-1580/evidence/ci/main-058e9bef/`. These are targeted failure
reviews, not a claim to have read every raw log line.

- [Agentic CLI Matrix 44174503822](https://github.com/link-assistant/formal-ai/actions/runs/44174503822)
  still fails its twelve global-config/undo cases on empty diagnostic artifacts;
  the local native startup candidate has not executed.
- [Layered CI 44174503458](https://github.com/link-assistant/formal-ai/actions/runs/44174503458)
  includes previously repaired fixture/recipe/browser failures and additional
  open causes. Lines 15868–15880 show the Moon formalization unknown-term count
  falling from 196 to 195 after the new causal meaning is seeded. Both native
  and browser expectations still contain the earlier count. Lines 28964–28989
  show twenty unnecessary provider requests while a supplied-passage question
  ultimately receives its local answer. The recursive procedure drafts
  subrequests even when its root composer cannot replace a concrete answer;
  the complete trace and a behavior-preserving correction are still needed.

The branch's published head remains `83882c0ec7c26b28dc8ac274fada2b39d9dc3a67`
at preflight. Its twenty-four workflows have completed: twenty-three successes
and one Layered CI failure, solely the stale governed-source witness already
repaired locally. The fresh all-event inventory is authoritative; a push-only
query returns zero because these runs use `pull_request`. All feedback remains
unchanged: one issue comment, nine PR conversation comments, no inline comments
or reviews. No branch runs are queued or in progress at that observation.

The default-branch build policy explicitly prohibits feature-branch native
tests, including manual dispatch. No extra native test job is added and no
dispatch is represented as runtime evidence. The wider native, benchmark,
template, dependency, historical-failure and live-identity criteria in the work
plan remain unfinished. Independent Formal AI reasoning acceptance remains
0/20; externally supplied commands, scripts and conclusions receive zero
authorship credit. The PR stays draft and unmerged.
