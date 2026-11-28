# Maintained native acceptance helpers

Nine maintained Rust test owners exceeded the unchanged 900-line warning
threshold. Sixteen complete assertion bodies now live in smaller helper
modules. Every original test name, attribute and wrapper remains in its
original owner. The extraction records prove an exact inverse before
formatting; all eighteen Rust files pass Rust 2024 formatting checks.
Platform-specific and relative-include bodies were left in their original
locations. No native compilation or native execution is claimed locally.

The new regression fails on all nine original warning owners and passes
after extraction. An intermediate check incorrectly required rustfmt to
keep a long call on one line; that failure and its whitespace-independent
correction are retained. All 217 maintained issue observations pass. The
unchanged-limit source scanner reports 46 warnings and zero violations.

Evidence: `experiments/issue-1580/evidence/checks/maintained-acceptance-helpers/`.
Every compressed stream retains raw and archive hashes, byte counts and
line counts. Supplied procedures receive zero independent authorship credit.

Current-head hosted validation and the governed source-witness repair are
separate work. All eight complete template/practice comparisons, remaining
warnings and wider issue acceptance remain unfinished. PR #1581 stays draft
and unmerged: https://github.com/link-assistant/formal-ai/pull/1581.
