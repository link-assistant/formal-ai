# Native regression module headroom

Three maintained Rust test owners exceeded the unchanged 900-line warning
threshold. The actual scanner reproduces all three warnings before extraction.
Command-capture failures, standalone script-suite discovery and image policy
now live in ordinary child modules under their existing test module prefixes.
Every original test and helper body is conserved. An exact inverse reconstructs
the pre-extraction source; formatting removes only trailing blank space.

The scanner regression passes all three cases after extraction. The maintained
observation entry passes 136 cases; write-planner, planner-twin and staged-release
checks pass 23, three and 15 cases respectively. Formatting and all three touched
JavaScript gates pass. The initial reporter assertion, inherited Node options
and misspelled gate name failures remain retained as investigation harness
failures. The corrected invocation preserves the source compiler's existing
refusal of external loader options.

The source scanner reports 84 warnings and zero violations. Its 900/1400 warning
thresholds and 1000/1500 limits are unchanged. These local checks execute no Rust
compiler or native tests. Hosted qualification of these extractions, full
template/practice review and the wider issue requirements remain unfinished.
Supplied scripts earn zero independent authorship or reasoning credit.

Evidence: `experiments/issue-1580/evidence/checks/native-test-headroom/`.
