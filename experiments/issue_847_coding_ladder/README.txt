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
  run_coding_ladder.sh  Runs each task, verifies by observed effect, reverts
                 the tree between tasks, writes results.json.
  results.json   Output of the last run. Regenerate; do not hand-edit.

USAGE
-----
  cargo build --release
  experiments/issue_847_coding_ladder/run_coding_ladder.sh
  ONLY=846 experiments/issue_847_coding_ladder/run_coding_ladder.sh

The working tree must be clean: tasks edit the real repository and are
reverted with `git checkout -- .` between runs. Changes under experiments/
are ignored by that guard so the harness can write its own results.

BASELINE @ v0.303.0 (main), agent CLI 0.25.0, 2026-07-25
--------------------------------------------------------
  TOTAL 2/13
    L1 0/2    L2 0/3    L3 1/5    L4 1/3

Both passes are READ-ONLY (846.L4.read_excluded, 710.L3.status_row).
Zero write operations succeeded at any level.

THE HEADLINE FINDING
--------------------
There is no level at which coding currently works. The ladder was built to
find the complexity at which tasks stop failing; that level does not exist
yet, because the floor operation -- "add this string to that array in this
named file" -- fails.

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
