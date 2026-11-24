# Contributor guide headroom

The maintained scanner reproduces the 1491-line contributor-guide warning.
Two complete sections now live in linked merge-layout and changelog guides.
Moved text is conserved after two relative-link adjustments and removing
the final empty line from each standalone guide, with
legacy changelog anchor IDs retained at the entry point. The entry now has
1399 lines; the unchanged scanner limits leave 77 repository warnings.

The focused regression fails before extraction and passes afterward. It checks
the real scanner, guide reachability, relative links and key instructions. All
existing push-gate documentation cases and three affected JavaScript gates pass.
Two intermediate drafts still exceeded the warning band; their failed checks
remain evidence, without passing credit. The first inventory probe exceeded
Node's default output buffer; the retry reads the Git inventory from a file.
These are development harness failures, not new production failures.

The preceding batch's 151 maintained cases retain their own source scope; this
batch executes its focused case and the existing push-gate suite. No Rust build
is run. Complete issue acceptance and independent Formal AI authorship remain
unfinished; supplied scripts and conclusions receive zero independent credit.

Evidence: `experiments/issue-1580/evidence/checks/contributing-headroom/`.

The final staged whitespace gate caught those two empty EOF lines; they were
removed and all focused checks were repeated before committing. The failed
staged gate and original extraction receipts remain in the evidence.
