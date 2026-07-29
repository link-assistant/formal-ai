Issue #848 coding-task ladder — can Formal AI code on its own open issues?
=========================================================================

WHAT THIS IS
------------
A decomposition ladder over REAL open Formal AI issues that have never had a
pull request. Each seed issue is split from L1 ("solve this issue and open a
PR") down to L4 (one atomic edit to one named file), each level splitting into
at least two smaller tasks.

Every task is work we actually want done. Nothing here is a synthetic /tmp
exercise: the point is to teach Formal AI to code on our own backlog, so a
task that starts passing is a real contribution rather than a throwaway drill.

Driven through `formal-ai with agent`, which is exactly the path Hive Mind
uses: Hive Mind -> Agent CLI -> `formal-ai serve --agent-mode`
(link-assistant/hive-mind#2059).

FILES
-----
  prompts.json   13 tasks across 4 levels, seeded from issues #846, #843,
                 #700, #710 plus two decomposition meta-tasks (#847).
  generate_prompts.py   Writes prompts.json. Edit this, never prompts.json.
  run_coding_ladder.sh  Runs each task, verifies by observed effect, reverts
                 the tree between tasks, writes results.json.
  new_branch_for.sh     The L1 verify: exits 0 only for a branch that appeared
                 DURING the task, measured against the ref snapshot the runner
                 takes before launching it.
  results.json   Output of the last run. Regenerate; do not hand-edit.

USAGE
-----
  cargo build --release
  experiments/issue_847_coding_ladder/run_coding_ladder.sh
  ONLY=846 experiments/issue_847_coding_ladder/run_coding_ladder.sh

The working tree must be clean: tasks edit the real repository and are
reverted with `git checkout -- .` between runs. Changes under experiments/
are ignored by that guard so the harness can write its own results.

BASELINE @ v0.304.0 (this branch), agent CLI 0.25.0, Linux, 2026-07-25
----------------------------------------------------------------------
Dataset expanded to 130 tasks across 16 families (see generate_prompts.py).
Measured with the fixed harness: L1 verified through new_branch_for.sh, the
agent scratch home reclaimed after every task, 0 tasks NOT MEASURED.

  TOTAL 45/130
    L1 0/16   L2 3/12   L3 4/28   L4 38/74

  read 12/12   atomic_edit 6/22   knowledge 6/8   verification 4/5
  create 3/6   multilingual 4/11  decomposition 6/6  error_recovery 2/4
  search 1/8   replace_delete 1/5
  issue_to_pr 0/16   test_authoring 0/8   targeted_edit 0/7
  deliverable 0/5    multifile 0/4       refactor 0/3

decomposition 6/6 is this PR's contribution: every split, atomicity and
first-step prompt now routes to the decomposition handler instead of the
unknown-prompt fallback. On the pre-PR binary the same six scored 2/6.

The v0.303.0 (main) baseline of this dataset read 38/130 with
decomposition 2/6 and L1 0/16. The three deltas -- decomposition 2->6,
knowledge 4->6, multilingual 3->4 -- are the decomposition surfaces this PR
adds; L1 held at 0/16 under the corrected verify (an intermediate Linux run
mismeasured it as 7/16 before new_branch_for.sh landed; see defect six below).

An earlier 13-task sample reported 2/13 and concluded "no level where
Formal AI can write code". That was too small a sample and partly wrong.

THE HEADLINE FINDING
--------------------
The boundary is not LEVEL, it is ARTIFACT KIND.

All 10 passing write tasks are "append a line to a prose or config file"
(.md, .yml, .toml, .lino, .sh, .gitignore) or a numeric constant swap.
ZERO writes produce valid code: test_authoring 0/8, targeted_edit 0/7,
refactor 0/3, multifile 0/4, deliverable 0/5.

Text insertion works. Code generation does not. That distinction is only
visible with breadth across artifact types, which is why the dataset was
expanded from 13 to 130.

read 12/12 shows navigation is not the bottleneck. search 1/8 is the
surprise: locating a symbol fails where reading a named file succeeds.

Root cause is intent routing, not tool wiring. The agent CLI advertises
`write`, `edit` and `bash` (confirmed in the server log), and Formal AI simply
never calls them. Direct API probe:

  "Create a file named /tmp/x.txt containing exactly the word hello"
    -> formalize: "arithmetic"          <- routed as an arithmetic task
    -> "I could not determine ... from local Links Notation memory"

  "Split this coding task into at least two smaller subtasks ..."
    -> "I can route write_program(language, task), but I do not have a
        template for language `rust` and task `missing`."

The second is a misroute rather than a refusal: a decomposition request is
formalized as program generation. Both are the same class as issue #840 --
surface-token routing rather than modeled intent.

MEASUREMENT DEFECTS FOUND AND FIXED (read before trusting any number)
----------------------------------------------------------------------
The first run of this harness reported 5/13 with L1 at 2/2. "Solve the issue
and open a pull request" cannot pass 2/2. Three defects, all now fixed:

1. L1 tasks used `verify: "true"`, which cannot fail. They now require a real
   issue branch to exist.
2. The refusal text is produced by the SERVER and never reaches the agent
   CLI's stdout, so refusals scored as successes. The runner now reads back
   the server log path the CLI prints on exit.
3. A refusal now fails a task outright, independent of `verify`.

A fourth was found after that: `meta.L2.decompose` still passed because the
model neither refused nor answered -- it misrouted to `write_program`. Misroute
detection was added.

A fifth, on the 130-task run, which first reported 49/130 with
test_authoring 7/8: asked for a Rust test file, the agent creates the file
whose entire content is the echoed prompt fragment --

  $ cat tests/unit/ladder_spotcheck.rs
  one Rust test named ladder_spec_probe asserting the crate version string is not empty.

-- and a grep for the test name passes on that. The runner now requires any
.rs file a task was asked to create to contain an `fn` item. Ten tasks were
reclassified pass -> fail and test_authoring went 7/8 -> 0/8.

A sixth, found when the 130-task run was first repeated on Linux: L1 jumped
from 0/16 to 7/16 with no engine change that could explain it. The verify was
`git branch -a | grep -q "issue-846"`, and `-a` includes remote-tracking refs.
The machine that produced 0/16 had never fetched them; the machine that
produced 7/16 had. Every one of those seven "passes" was a branch pushed by
someone else weeks earlier -- one of them the very branch the measurement was
running from. L1 tasks now verify through new_branch_for.sh, which requires a
ref that is absent from a snapshot taken immediately before the task starts.

A seventh, on the same run: `formal-ai with agent` copies its whole
configuration into $TMPDIR/formal-ai-agent-home-config-<pid>-<nanos>/ and never
removes it -- about 200 MB per task. At task 88 of 130 the filesystem tightened
and the temporary server stopped coming up, so the last 42 tasks measured
nothing at all while recording ordinary FAILs (and, where `verify` happened to
be satisfied by the tree's existing state, ordinary PASSes). The runner now
reclaims the scratch home after every task and reports a task whose server
never started as NOT MEASURED, counted separately from a failure. A run with a
non-zero NOT MEASURED count is incomplete, not bad news.

Generalisable lesson, and the same one behind #839 and #842: assert on the
OBSERVED EFFECT, never on narration. "I created the file" with no file is a
failure. Any harness in this repo that checks only that a command was emitted,
or that some string appears in mixed stdout, will report false greens --
the agent CLI interleaves verbose JSON logs on the same streams, which is how
a bare "1." and "yes" matched and scored two refusals as passes.

CI SAFETY
---------
`experiments/` is excluded from any-code-changed (scripts/detect-code-changes.rs)
and from shellcheck (.github/workflows/release.yml). Note issue #846: that
exclusion is bypassed on direct pushes to main. Keep this directory free of
*.md and *.mjs -- docs-changed and mjs-changed are computed by file extension
alone and ignore the folder exclusion, which is why this file is README.txt.
