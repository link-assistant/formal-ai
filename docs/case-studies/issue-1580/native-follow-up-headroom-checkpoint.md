# Follow-up and global configuration regression headroom

The maintained scanner reproduced two warnings in the procedural follow-up and
CLI wrapper test owners. Their assertions now live in ordinary child modules.
Five small parent wrappers preserve the original native test names and recorded
reasoning durations, using the same pattern as existing release-operation tests.
Every extracted body remains unchanged after removing only its test attribute
and adding the visibility required by the parent wrapper.

The two new scanner cases fail before repair. Afterward, all 11 focused cases,
50 native-runner cases, 144 maintained observations, three touched gates,
formatting and whitespace checks pass. The complete source scanner reports
79 warnings and zero violations under unchanged limits. No Rust compiler ran;
hosted native qualification of these local commits remains pending. The full
issue, complete template/practice review and independent authorship requirements
remain unfinished. Supplied scripts and edits earn zero independent credit.

Evidence: `experiments/issue-1580/evidence/checks/native-follow-up-headroom/`.
