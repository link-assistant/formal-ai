# Hive commit expectation helper headroom

The maintained scanner reproduced the 1000-line Hive extended-test warning.
Three fixture helpers and their private template placeholders now live in a
137-line child module; the owner has 872 lines. All 30 original native test
names, order, assertions and timing records stay in the original module. Moved
helper bodies are unchanged except for visibility. An independent function
inventory verifies that exactly those three helpers leave the parent.

The focused regression fails before extraction and all 12 cases pass afterward.
Formatting, whitespace and three touched JavaScript gates pass. The scanner
reports 78 warnings and zero violations under unchanged limits. The preceding
batch's 144 maintained and 50 native-runner cases remain its own qualification;
this batch executes the focused regression only. Hosted native execution and
full issue acceptance remain unfinished. Supplied scripts receive zero
independent authorship or reasoning credit.

Evidence: `experiments/issue-1580/evidence/checks/hive-expectation-headroom/`.
