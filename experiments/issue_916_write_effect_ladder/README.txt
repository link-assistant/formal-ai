ISSUE #916 -- WRITE-EFFECT LADDER (epic E69)
============================================

WHAT THIS MEASURES
------------------
Whether a request that must change the workspace actually changed it, and
whether the reported outcome matches what the workspace and the exit codes show.

The issue #840 task ladder judges what an answer says. This ladder judges what
the workspace holds. The distinction is the whole point of epic E69: the defects
recorded in issues #902-#909 were not wrong answers, they were right-sounding
answers over an unchanged workspace.

  "Assert on the OBSERVED EFFECT, never on narration. 'I created the file' with
   no file is a failure."                    -- experiments/issue_847_coding_ladder

HOW A RUNG IS JUDGED
--------------------
Each rung in rungs.json declares the effect its request must leave behind. The
harness boots a real `formal-ai serve --agent-mode`, runs the same tool loop an
OpenAI-compatible client runs, and EXECUTES the planned `write_file` and
`run_shell_command` calls for real inside a directory of that rung's own. Shell
results come back in the qwen-code envelope shape the #902-#909 corpus recorded:

  Command: python3 -m py_compile main.py
  Directory: (root)
  Output: (empty)
  Error: (none)
  Exit Code: 0
  Signal: 0
  Process Group PGID: 685377

After the loop ends, the judge reads the files off disk, re-runs every declared
verification command independently of the system's own run, and only then looks
at the answer. Three honesty rules then apply to every rung, not just the ones
that name them -- they are the acceptance criteria of issue #916 mechanized:

  1. No completion claim without an observed workspace effect.
  2. A non-zero exit code forbids a completion claim, and the code itself must
     appear in the reported outcome. Exit codes propagate to the outcome.
  3. A run whose declared effects all landed must not report failure.

A rung also fails on a refusal, a capability menu, a missing required tool, or a
forbidden command shape. Narration alone can never pass.

FAULT INJECTION
---------------
Some defects only appear when a tool misbehaves, so a rung may declare faults:

  "faults": [{"tool": "write_file", "match": "hello.txt", "skip_effect": true,
              "result": "Error: write_stdin failed: Unknown process id 0"}]

The named call then reports the fault and leaves the workspace untouched -- this
is the transport failure of issue #905, where the write never landed and the run
reported success anyway. A `run_shell_command` fault instead returns an envelope
with the declared `exit_code`, which is how R916-03 checks that a real non-zero
status reaches the report.

THE RUNGS
---------
  R916-01  #905  a lost write is not a completion
  R916-02  #908  a silent check command does not abandon the completed work
  R916-03  #908  the failure report names the exit code, not the harness
  R916-04  #905  an observed effect is what completion means
  R916-05  #905  an adverbial qualifier is not part of the content
  R916-06  #907  a caller context block does not hijack the request
  R916-07  #907  a declarative statement of fact is not a request
  R916-08a #909  --global writes a gemini configuration that starts headlessly
  R916-08b #909  --global writes the complete OpenAI triple for qwen

The same identifiers name the in-process regression tests in
tests/unit/issue_916.rs, so a defect, its fix, its unit test and its ladder rung
all carry one name. R916-08a/b are `kind: "cli"` rungs: they run the wrapper CLI
with a throwaway HOME and read the configuration files back, then run `--undo`
and check the workspace was restored. A configuration that cannot be taken back
is not one a user can safely accept.

THE #902-#909 DEFECT CLUSTER
----------------------------
Epic E69 requires every defect in #902-#909 to be fixed or explicitly closed with
a recorded reason, each fix tied to a named ladder rung. This is that record.

  #902  codex loses its provider block after `-c`
        CLOSED before this branch. Argv construction was fixed in the
        client-integrations wrapper; covered by the real-client matrix in
        tests/integration/with_formal_ai.rs, not by a write-effect rung -- it
        leaves no workspace effect to observe.
  #903  native CLI argv built incorrectly
        CLOSED before this branch, same wrapper fix, same reason for having no
        rung of its own.
  #904  agent mode reduces a coding task to writing a plan file
        CLOSED before this branch. The `planned_not_executed` terminal state is
        what R916-04 now builds on: a plan is not an effect, and completion is
        only claimed when the workspace is observed.
  #905  "Completed ... and verified it with `cat hello.txt`" after exit 1
        FIXED here.  R916-01, R916-04, R916-05.
  #906  language router takes the word after "in" as the target language
        CLOSED before this branch; a routing defect with no workspace effect, so
        it is pinned by tests/unit/issue_906_language_router.rs rather than by a
        rung.
  #907  caller framing hijacks intent routing
        FIXED here.  R916-06, R916-07.
  #908  step verification ignores the exit code
        CLOSED during this branch's work; the rungs that keep it closed are
        R916-02 (exit 0 with no output is success) and R916-03 (the report names
        the exit code).
  #909  `--global` writes an incomplete headless config
        FIXED here.  R916-08a, R916-08b.

RUNNING IT
----------
  cargo build --release --bin formal-ai
  experiments/issue_916_write_effect_ladder/run_write_effect_ladder.sh

  ONLY=R916-02 experiments/issue_916_write_effect_ladder/run_write_effect_ladder.sh
  SANDBOX_KEEP=1 ... run_write_effect_ladder.sh   # keep the sandbox to inspect

Judge unit tests (no server, no build required):

  cd experiments/issue_916_write_effect_ladder && python3 -m unittest -v test_ladder.py

THE RATCHET
-----------
results.json is both the record of the last measurement and the floor the next
one may not fall below. Setting BASELINE turns the run into a gate in the style
of the issue #408 ratchet:

  * every rung id present in the baseline must still be present,
  * no baseline-green rung may go red,
  * every rung -- including appended ones -- must be green before the baseline
    may move.

The throwaway sandbox path is redacted to `(sandbox)` in the record, so two runs
of an unchanged system write byte-identical results.json files and a diff of the
committed baseline shows a behavior change and nothing else.

.github/workflows/write-effect-ladder.yml runs exactly that on every pull request
that touches Rust sources, seed data, or this directory, so a regression is
visible on the pull request that causes it rather than months later in a bug
report.

WHY THIS DIRECTORY HAS NO .md FILE
----------------------------------
The release pipeline computes "docs changed" and "mjs changed" from file
extensions alone, so a *.md or *.mjs file here would misroute those jobs. Notes
in experiments/ are plain .txt for that reason.
