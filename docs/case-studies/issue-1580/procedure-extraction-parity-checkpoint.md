# Procedure coverage after fixture extraction

The full JavaScript gate run caught a regression in the earlier unpushed Rust
split: four native wrappers hid three previously carried specification cases
and made three documentation exceptions appear stale. The repair restores every
assertion to its original native test, moving only source-backed fixture text,
case arrays and their types into a child. Original test names and order remain;
the parent has 879 lines and the child 132 under unchanged scanner limits.

The specification gate again executes all 221 carried cases successfully. Both
documentation gates pass without editing their records or dropping exceptions.
All 13 headroom checks and the new extraction regression pass. The initial new
regression confused the many-topics documentation exception with the carried
prior-how-to case; its failed draft and corrected oracle are retained. No passing
credit is assigned to that draft. All 145 default-branch policy tests and the
final maintained observation suite pass. The complete original full-gate output
is retained with its three failures, alongside the successful affected retries.
No Rust compiler ran; hosted native qualification remains pending.

Supplied scripts and decisions receive zero independent authorship or reasoning
credit. Full issue acceptance remains unfinished.

Evidence: `experiments/issue-1580/evidence/checks/procedure-extraction-parity/`.
