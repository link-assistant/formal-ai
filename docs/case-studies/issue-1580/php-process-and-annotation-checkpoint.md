# PHP process observations and annotation fixture readiness

The current PHP template at `b61e3347` retains the same `Process.php` bytes as
the original pinned fixture. Its SHA256 is
`97a1b126035a71434739ecc962d73c74b44ea8ea802263b74031e04332b43bbf`.
The helper drains stdout completely before reading stderr. A finite child that
writes 131072 stderr bytes first cannot finish while that pipe is full, and the
parent cannot finish reading stdout. A second defect converts literal `"0"`
output into an empty string through PHP's `?:` truthiness.

The three maintained tests execute the actual PHP helper and a disposable
candidate that drains both nonblocking streams with `stream_select`. The
original is killed as a process group after two seconds in the deadlock case;
the candidate preserves all 262144 bytes. Literal zero, small outputs and exit
codes 0 and 7 have explicit controls. Each PHP process has a 32 MiB limit and
captured output has a 524288-byte bound. This candidate has not been applied to
the upstream repository. Its existing issues were checked for pipe/deadlock
duplicates; the upstream report is pending publication of this evidence.

The first 119-case maintained run passed 118 cases and exposed a race in the
annotation warning fixture. A whole-second warning threshold could be reached
before Node wrote its partial stderr. The fixture now waits for the stderr
write callback before the initial liveness probe. Production wrapper behavior,
its one-second budget and the existing test timeouts are unchanged. The revised
fixture fails against the old wrapper because the annotation follows partial
stderr without a newline. The complete current maintained entry passes all
119 cases in 87.8 seconds. Both the initial failure and the expected old-wrapper
failure are retained alongside the successful current run.

Evidence is in `experiments/issue-1580/evidence/checks/php-pipe-and-annotation-readiness/`.
Every retained compressed stream verifies against its raw and archive hashes.
These supplied probes receive zero independent Formal AI authorship credit.
All eight complete template comparisons and full issue acceptance remain open.
PR: https://github.com/link-assistant/formal-ai/pull/1581
