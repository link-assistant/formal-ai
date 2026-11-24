# Process fixture shell startup

The maintained observation entry initially passes 419 of 429 tests locally.
Nine process-lifetime cases fail before the fixture writes its PID, and one
JavaScript planner subprocess reaches its existing ten-second limit. Separate
focused runs had passed, so those results do not qualify the combined entry.

The finite diagnostic replay preserves the real Node-to-Python-to-Bash launch.
Its original case exits 124 without fixture output; disabling shell startup,
using an empty home, or closing stdin each reaches the same finite child.
This isolates host shell startup in the Node launch topology. A controlled
remote environment and a hostile temporary .bashrc reproduce both lifetime
failures; the complete planning regression also refuses the hostile startup.

The fixture shells now use --noprofile --norc, including the nested
TERM-resistant worker. Planning uses the same isolation with explicit empty
startup environment files and a controlled home. All real workflow commands,
fixture inputs, production deadlines and assertion thresholds remain intact.
A missing PID now retains the actual child status and output instead of
discarding those observations behind a file-not-found error.

Both hostile lifetime regressions fail before the repair; the diagnostic
control passes. The planning reproduction has one failure and its runner
continuation control passes. The final focused files pass three startup, two complete-profile, nine budget
and eight deadline cases. The complete maintained entry passes all 432 cases
in 195.974 seconds. These counts overlap; they are not independent totals.
The [complete repair receipt](../../../experiments/issue-1580/evidence/checks/dialogue-and-fixture-repairs-20261123/manifest.json)
retains both original failed runs and all final outputs.
This repairs local test isolation; exact hosted execution and the original
issue's remaining acceptance criteria are still required.
